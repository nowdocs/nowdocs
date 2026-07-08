//! Registry lifecycle: install / share / update / uninstall docsets.

use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::cache;
use crate::chunker::{Chunk, ChunkType};
use crate::errors::{archive_error, NowdocsError};
use crate::input;
use crate::manifest;
use crate::store::Store;

fn is_test_file_url(url: &str) -> bool {
    url.starts_with("file://")
}

/// Extract the host portion from a URL like "https://github.com/path".
fn url_host(url: &str) -> &str {
    let after_scheme = url.split("://").nth(1).unwrap_or(url);
    after_scheme.split('/').next().unwrap_or("")
}

fn is_allowed_registry_url(url: &str) -> bool {
    if is_test_file_url(url) {
        return true;
    }
    let host = url_host(url);
    match host {
        // github.com requires path prefix /nowdocs-registry/ to prevent
        // lookalike domains like github.com/nowdocs-registry.evil.com
        "github.com" => {
            let after_scheme = url.split("://").nth(1).unwrap_or(url);
            let path = after_scheme.strip_prefix(host).unwrap_or(after_scheme);
            path.starts_with("/nowdocs-registry/") || path == "/nowdocs-registry"
        }
        "registry.nowdocs.dev" => true,
        _ => false,
    }
}

fn download_to_temp(url: &str) -> Result<PathBuf> {
    if !is_allowed_registry_url(url) {
        anyhow::bail!(
            "registry URL not in allowed domains: {} (allowed: github.com/nowdocs-registry, registry.nowdocs.dev)",
            url
        );
    }
    let tmp = std::env::temp_dir().join(format!("nowdocs_dl_{}", std::process::id()));
    let status = std::process::Command::new("curl")
        .args(["-fsSL", "-o", tmp.to_str().unwrap(), url])
        .status()
        .context("failed to spawn curl")?;
    if !status.success() {
        let _ = std::fs::remove_file(&tmp);
        anyhow::bail!("curl failed for {}", url);
    }
    Ok(tmp)
}

/// Minimal ustar tar reader (no GNU extensions, no PAX).
///
/// Rejects unsafe entries and enforces size/count guardrails *before* allocating
/// content buffers, so a malicious header cannot cause unbounded memory use.
fn extract_tar<R: Read>(reader: &mut R) -> Result<Vec<(String, Vec<u8>)>> {
    let mut files = Vec::new();
    let mut header = [0u8; 512];
    let mut total_bytes: u64 = 0;

    loop {
        match reader.read_exact(&mut header) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }

        // All-zero block = end of archive.
        if header.iter().all(|&b| b == 0) {
            break;
        }

        let name = std::str::from_utf8(&header[0..100])
            .context("invalid tar filename utf8")?
            .trim_end_matches('\0')
            .to_string();

        let size = parse_octal(&header[124..136]).unwrap_or(0);
        let typeflag = header[156];

        if typeflag == 0 || typeflag == b'0' {
            // P1: reject per-entry size and total count BEFORE allocating.
            if size > MAX_ENTRY_BYTES {
                // Still need to skip the declared content to leave reader in a
                // consistent state for potential callers that ignore this error.
                skip_tar_content(reader, size)?;
                let err = archive_error(
                    "ARCHIVE_TOO_LARGE",
                    format!(
                        "entry '{}' is {} bytes, exceeds limit of {} bytes",
                        name, size, MAX_ENTRY_BYTES
                    ),
                    "use a smaller archive or report the broken registry release",
                );
                return Err(anyhow::anyhow!("{}", err));
            }
            if files.len() >= MAX_ENTRY_COUNT {
                skip_tar_content(reader, size)?;
                let err = archive_error(
                    "ARCHIVE_TOO_LARGE",
                    format!("archive exceeds entry count limit of {}", MAX_ENTRY_COUNT),
                    "use a smaller archive or report the broken registry release",
                );
                return Err(anyhow::anyhow!("{}", err));
            }

            let mut content = vec![0u8; size as usize];
            let mut read = 0usize;
            while read < size as usize {
                let n = reader.read(&mut content[read..]).context("tar read")?;
                if n == 0 {
                    break;
                }
                read += n;
            }

            // Skip padding (align to 512 bytes).
            let padded = (size as usize).div_ceil(512) * 512;
            if padded > size as usize {
                let mut skip = vec![0u8; padded - size as usize];
                let _ = reader.read_exact(&mut skip);
            }

            total_bytes += size;
            if total_bytes > MAX_ARCHIVE_BYTES {
                let err = archive_error(
                    "ARCHIVE_TOO_LARGE",
                    format!(
                        "archive is at least {} bytes after '{}', exceeds limit of {} bytes",
                        total_bytes, name, MAX_ARCHIVE_BYTES
                    ),
                    "use a smaller archive or report the broken registry release",
                );
                return Err(anyhow::anyhow!("{}", err));
            }

            files.push((name, content));
        } else if typeflag == b'5' {
            // Directory entry — skip silently (safe, needed for nested paths).
            let padded = (size as usize).div_ceil(512) * 512;
            let mut skip = vec![0u8; padded];
            let _ = reader.read_exact(&mut skip);
        } else {
            // Symlink (b'2'), hardlink (b'1'), device (b'3'/'4'), or other
            // unsafe non-regular entry — reject.
            let padded = (size as usize).div_ceil(512) * 512;
            let mut skip = vec![0u8; padded];
            let _ = reader.read_exact(&mut skip);

            let type_name = match typeflag {
                b'1' => "hardlink",
                b'2' => "symlink",
                b'3' => "character device",
                b'4' => "block device",
                b'6' => "fifo",
                b'7' => "contiguous file",
                _ => "unknown",
            };
            return Err(anyhow::anyhow!(
                "{}",
                archive_error(
                    "ARCHIVE_UNSUPPORTED_ENTRY",
                    format!(
                        "archive contains unsupported entry type '{}' ({}): {}",
                        typeflag as char, type_name, name
                    ),
                    "report the broken registry release",
                )
            ));
        }
    }
    Ok(files)
}

/// Skip `size` bytes + padding from a tar entry without allocating a buffer.
fn skip_tar_content<R: Read>(reader: &mut R, size: u64) -> Result<()> {
    let padded = (size as usize).div_ceil(512) * 512;
    let mut skip = vec![0u8; std::cmp::min(padded, 8192)];
    let mut remaining = padded;
    while remaining > 0 {
        let to_read = std::cmp::min(remaining, skip.len());
        let n = reader.read(&mut skip[..to_read]).context("tar skip")?;
        if n == 0 {
            break;
        }
        remaining -= n;
    }
    Ok(())
}

fn parse_octal(s: &[u8]) -> Option<u64> {
    let trimmed: Vec<u8> = s
        .iter()
        .copied()
        .skip_while(|&b| b == 0 || b == b' ')
        .take_while(|&b| (b'0'..=b'7').contains(&b))
        .collect();
    if trimmed.is_empty() {
        return Some(0);
    }
    std::str::from_utf8(&trimmed)
        .ok()
        .and_then(|s| u64::from_str_radix(s, 8).ok())
}

// --- R1: Archive validation guardrails ---

/// Max total archive bytes (512 MiB).
pub const MAX_ARCHIVE_BYTES: u64 = 512 * 1024 * 1024;
/// Max single entry bytes (256 MiB).
pub const MAX_ENTRY_BYTES: u64 = 256 * 1024 * 1024;
/// Max entry count.
pub const MAX_ENTRY_COUNT: usize = 100_000;

/// Basenames that must appear at most once in the archive.
const DUPLICATE_GUARD_BASENAMES: &[&str] = &["manifest.json", "chunks.jsonl", "LICENSE", "NOTICES"];

/// Validate archive entries before writing active cache.
///
/// Returns `Ok(())` if the archive passes all safety checks, or a structured
/// `NowdocsError` with a stable code and actionable hint.
pub fn validate_archive(entries: &[(String, Vec<u8>)]) -> Result<(), NowdocsError> {
    // Entry count guardrail.
    if entries.len() > MAX_ENTRY_COUNT {
        return Err(archive_error(
            "ARCHIVE_TOO_LARGE",
            format!(
                "archive has {} entries, exceeds limit of {}",
                entries.len(),
                MAX_ENTRY_COUNT
            ),
            "use a smaller archive or report the broken registry release",
        ));
    }

    let mut total_bytes: u64 = 0;
    let mut seen_duplicates: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut has_manifest = false;
    let mut has_chunks = false;

    for (name, data) in entries {
        total_bytes += data.len() as u64;

        // Per-entry size guardrail.
        if data.len() as u64 > MAX_ENTRY_BYTES {
            return Err(archive_error(
                "ARCHIVE_TOO_LARGE",
                format!(
                    "entry '{}' is {} bytes, exceeds limit of {} bytes",
                    name,
                    data.len(),
                    MAX_ENTRY_BYTES
                ),
                "use a smaller archive or report the broken registry release",
            ));
        }

        // Path safety: reject absolute paths.
        if name.starts_with('/') {
            return Err(archive_error(
                "ARCHIVE_UNSAFE_PATH",
                format!("archive contains absolute path: {}", name),
                "report the broken registry release",
            ));
        }

        // Path safety: reject .. components.
        let path = std::path::Path::new(name);
        for component in path.components() {
            if matches!(component, std::path::Component::ParentDir) {
                return Err(archive_error(
                    "ARCHIVE_UNSAFE_PATH",
                    format!("archive contains path traversal (..): {}", name),
                    "report the broken registry release",
                ));
            }
        }

        // Vector artifact detection — check every path component, not just the
        // full name suffix.  LanceDB stores are directories like
        // `index.lance/data.bin` where child files don't end in `.lance`.
        let basename = path
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_default();
        let has_vector_component = path.components().any(|c| {
            let s = c.as_os_str().to_string_lossy();
            s.ends_with(".lance")
                || s.ends_with(".faiss")
                || s.starts_with("vectors.")
                || s.starts_with("embeddings.")
        });
        if has_vector_component {
            return Err(archive_error(
                "ARCHIVE_VECTOR_ARTIFACT",
                format!("archive contains vector artifact: {}", name),
                "share must not include vector data; rebuild vectors with CI",
            ));
        }

        // Duplicate guard for security-sensitive basenames.
        if DUPLICATE_GUARD_BASENAMES.contains(&basename.as_str())
            && !seen_duplicates.insert(basename.clone())
        {
            return Err(archive_error(
                "ARCHIVE_DUPLICATE_ENTRY",
                format!("archive contains duplicate entry: {}", basename),
                "report the broken registry release",
            ));
        }

        // Track required entries.
        if name.ends_with("manifest.json") {
            has_manifest = true;
        }
        if name.ends_with("chunks.jsonl") {
            has_chunks = true;
        }
    }

    // Total archive size guardrail.
    if total_bytes > MAX_ARCHIVE_BYTES {
        return Err(archive_error(
            "ARCHIVE_TOO_LARGE",
            format!(
                "archive is {} bytes, exceeds limit of {} bytes",
                total_bytes, MAX_ARCHIVE_BYTES
            ),
            "use a smaller archive or report the broken registry release",
        ));
    }

    if !has_manifest {
        return Err(archive_error(
            "ARCHIVE_MISSING_MANIFEST",
            "registry archive is missing manifest.json",
            "retry install, or report the broken registry release",
        ));
    }

    if !has_chunks {
        return Err(archive_error(
            "ARCHIVE_MISSING_CHUNKS",
            "registry archive is missing chunks.jsonl",
            "retry install, or report the broken registry release",
        ));
    }

    Ok(())
}

/// Install a docset from an archive URL.
///
/// **Security**: production URLs must be on `nowdocs-registry` domains.
/// Test `file://` URLs are allowed (test fixture bypass).
pub fn install(docset: &str, url: &str) -> Result<()> {
    let docset = input::validate_docset(docset)?;
    cache::ensure_layout()?;

    // Check if there's an existing active docset for rollback
    let has_existing = cache::manifest_path(&docset).is_file();
    let existing_backup = if has_existing {
        Some(backup_existing(&docset)?)
    } else {
        None
    };

    // Install to staging first
    let staging_path = install_to_staging(&docset, url)?;

    // Promote staging to active
    match promote_staging(&docset, &staging_path) {
        Ok(()) => {
            // Success: cleanup staging and any rollback backup
            cleanup_staging(&staging_path)?;
            if let Some(backup) = existing_backup {
                cleanup_rollback(&backup)?;
            }
            Ok(())
        }
        Err(e) => {
            // Promotion failed: try to restore from backup if we had one
            if let Some(backup) = existing_backup {
                if let Err(restore_err) = restore_from_backup(&docset, &backup) {
                    eprintln!("warning: failed to restore from backup: {}", restore_err);
                }
            }
            // Leave staging for diagnostics
            Err(e)
        }
    }
}

/// Install a docset to a staging directory (not active paths).
fn install_to_staging(docset: &str, url: &str) -> Result<PathBuf> {
    let staging_path = cache::new_staging_path(docset);
    std::fs::create_dir_all(&staging_path)?;

    let (archive_path, is_temp) = if is_test_file_url(url) {
        let path = url.strip_prefix("file://").unwrap();
        (PathBuf::from(path), false)
    } else {
        (download_to_temp(url)?, true)
    };

    let mut file = std::fs::File::open(&archive_path).context("open archive")?;
    let entries = extract_tar(&mut file)?;
    drop(file);
    if is_temp {
        let _ = std::fs::remove_file(&archive_path);
    }

    // R1: validate archive before writing any active cache paths.
    validate_archive(&entries).map_err(|e| anyhow::anyhow!("{}", e))?;

    // Validate archive has manifest
    let manifest_entry = entries
        .iter()
        .find(|(name, _)| name.ends_with("manifest.json"))
        .context("archive missing manifest.json")?;

    let manifest_json = std::str::from_utf8(&manifest_entry.1).context("manifest utf8")?;
    let m = manifest::parse_manifest(manifest_json)?;
    manifest::validate(&m)?;

    // Write manifest to staging
    let staging_manifest = staging_path.join("manifest.json");
    std::fs::write(&staging_manifest, &manifest_entry.1)?;

    // Persist LICENSE to staging if present
    let license_entry = entries.iter().find(|(name, _)| {
        std::path::Path::new(name)
            .file_name()
            .map(|f| f == std::ffi::OsStr::new("LICENSE"))
            .unwrap_or(false)
    });
    if let Some((_, data)) = license_entry {
        let staging_license = staging_path.join("license.txt");
        std::fs::write(&staging_license, data)?;
    }

    // Save chunks to staging for later materialization
    let chunks_entry = entries
        .iter()
        .find(|(name, _)| name.ends_with("chunks.jsonl"));
    if let Some((_, data)) = chunks_entry {
        let staging_chunks = staging_path.join("chunks.jsonl");
        std::fs::write(&staging_chunks, data)?;
    }

    // Verify staged manifest
    verify_staging(&staging_path)?;

    Ok(staging_path)
}

/// Verify that staging contains valid manifest.
fn verify_staging(staging_path: &Path) -> Result<()> {
    let manifest_path = staging_path.join("manifest.json");
    if !manifest_path.is_file() {
        anyhow::bail!("staging missing manifest.json");
    }

    let raw = std::fs::read_to_string(&manifest_path)?;
    let m = manifest::parse_manifest(&raw)?;
    manifest::validate(&m)?;

    Ok(())
}

/// Promote staging to active cache.
fn promote_staging(docset: &str, staging_path: &Path) -> Result<()> {
    let active_manifest = cache::manifest_path(docset);
    let _active_db = cache::db_path(docset);
    let active_license = cache::license_text_path(docset);

    // Ensure parent directories exist
    if let Some(parent) = active_manifest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Copy manifest from staging to active
    let staging_manifest = staging_path.join("manifest.json");
    std::fs::copy(&staging_manifest, &active_manifest)?;

    // Copy license if present
    let staging_license = staging_path.join("license.txt");
    if staging_license.is_file() {
        if let Some(parent) = active_license.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(&staging_license, &active_license)?;
    }

    // Materialize chunks from staging to active store
    let staging_chunks = staging_path.join("chunks.jsonl");
    if staging_chunks.is_file() {
        let jsonl = std::fs::read_to_string(&staging_chunks)?;
        let parsed: Vec<JsonlChunk> = jsonl
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(serde_json::from_str::<JsonlChunk>)
            .collect::<Result<Vec<_>, _>>()
            .context("parse chunks.jsonl")?;

        let chunks: Vec<Chunk> = parsed
            .into_iter()
            .map(|c| Chunk {
                idx: c.idx,
                heading_path: c.heading_path,
                source_url: c.source_url,
                api_version: c.api_version,
                chunk_type: match c.chunk_type.as_deref() {
                    Some("Code") => ChunkType::Code,
                    _ => ChunkType::Info,
                },
                text: c.text,
            })
            .collect();

        let zero_vectors: Vec<Vec<f32>> = vec![vec![0.0f32; 512]; chunks.len()];
        let store = Store::open(docset)?;
        store.insert(&chunks, &zero_vectors)?;
    }

    Ok(())
}

/// Backup existing active docset for rollback.
fn backup_existing(docset: &str) -> Result<PathBuf> {
    let backup_path = cache::rollback_path(docset);
    std::fs::create_dir_all(&backup_path)?;

    let active_manifest = cache::manifest_path(docset);
    let active_db = cache::db_path(docset);
    let active_license = cache::license_text_path(docset);

    // Backup manifest
    if active_manifest.is_file() {
        std::fs::copy(&active_manifest, backup_path.join("manifest.json"))?;
    }

    // Backup license
    if active_license.is_file() {
        std::fs::copy(&active_license, backup_path.join("license.txt"))?;
    }

    // Backup store (db directory)
    if active_db.exists() {
        copy_dir_all(&active_db, &backup_path.join("store.lance"))?;
    }

    Ok(backup_path)
}

/// Restore from backup.
fn restore_from_backup(docset: &str, backup_path: &Path) -> Result<()> {
    let active_manifest = cache::manifest_path(docset);
    let active_db = cache::db_path(docset);
    let active_license = cache::license_text_path(docset);

    // Restore manifest
    let backup_manifest = backup_path.join("manifest.json");
    if backup_manifest.is_file() {
        std::fs::copy(&backup_manifest, &active_manifest)?;
    }

    // Restore license
    let backup_license = backup_path.join("license.txt");
    if backup_license.is_file() {
        if let Some(parent) = active_license.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(&backup_license, &active_license)?;
    }

    // Restore store (db directory)
    let backup_store = backup_path.join("store.lance");
    if backup_store.exists() {
        if active_db.exists() {
            std::fs::remove_dir_all(&active_db)?;
        }
        copy_dir_all(&backup_store, &active_db)?;
    }

    Ok(())
}

/// Cleanup staging directory.
fn cleanup_staging(staging_path: &Path) -> Result<()> {
    if staging_path.exists() {
        std::fs::remove_dir_all(staging_path)?;
    }
    Ok(())
}

/// Cleanup rollback directory.
fn cleanup_rollback(rollback_path: &Path) -> Result<()> {
    if rollback_path.exists() {
        std::fs::remove_dir_all(rollback_path)?;
    }
    Ok(())
}

/// Copy directory recursively.
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Share a docset: write manifest + text chunks (NO vectors, D10) to `out_dir`.
pub fn share(docset: &str, out_dir: &Path) -> Result<PathBuf> {
    let docset = input::validate_docset(docset)?;
    let mp = cache::manifest_path(&docset);
    if !mp.is_file() {
        anyhow::bail!("docset not installed: {}", &docset);
    }
    let raw = std::fs::read_to_string(&mp)?;
    let m = manifest::parse_manifest(&raw)?;
    manifest::validate(&m)?;

    let store = Store::open(&docset)?;
    let chunks = store.dump_chunks()?;

    let share_dir = out_dir.join(&docset);
    std::fs::create_dir_all(&share_dir)?;

    std::fs::write(share_dir.join("manifest.json"), &raw)?;

    let mut jsonl = String::new();
    for c in &chunks {
        let row = ChunkRow {
            idx: c.idx,
            heading_path: &c.heading_path,
            source_url: &c.source_url,
            api_version: c.api_version.as_deref(),
            chunk_type: match c.chunk_type {
                crate::chunker::ChunkType::Code => "Code",
                crate::chunker::ChunkType::Info => "Info",
            },
            text: &c.text,
        };
        jsonl.push_str(&serde_json::to_string(&row)?);
        jsonl.push('\n');
    }
    std::fs::write(share_dir.join("chunks.jsonl"), &jsonl)?;

    // Carry the upstream LICENSE text verbatim (stashed at ingest time) so
    // recipients can fulfill MIT/Apache notice retention and CC-BY-4.0
    // attribution. Omitted when the source had no license file.
    let license_path = cache::license_text_path(&docset);
    if license_path.is_file() {
        std::fs::write(share_dir.join("LICENSE"), std::fs::read(&license_path)?)?;
    }
    // Human-readable NOTICE synthesized from manifest legal + source fields.
    std::fs::write(share_dir.join("NOTICE"), build_notice(&m))?;

    Ok(share_dir)
}

/// Build a human-readable NOTICE for a share bundle from the manifest's
/// legal and source fields. Satisfies CC-BY-4.0's attribution requirement
/// and MIT/Apache's notice-retention requirement for downstream recipients.
fn build_notice(m: &manifest::Manifest) -> String {
    let mut s = String::new();
    s.push_str("nowdocs docset: ");
    s.push_str(&m.docset);
    s.push('\n');
    s.push_str("Source: ");
    s.push_str(&m.source.source_url);
    s.push('\n');
    s.push_str("Entry: ");
    s.push_str(&m.source.entry_url);
    s.push('\n');
    s.push_str("License: ");
    s.push_str(&m.legal.license);
    s.push('\n');
    if !m.legal.copyright_holder.trim().is_empty() {
        s.push_str("Copyright: ");
        s.push_str(&m.legal.copyright_holder);
        s.push('\n');
    }
    if !m.legal.attribution.trim().is_empty() {
        s.push_str("Attribution: ");
        s.push_str(&m.legal.attribution);
        s.push('\n');
    }
    s.push_str("\nThis bundle is a derived work produced by nowdocs (prep + chunk + embed)\n");
    s.push_str("from the upstream documentation source cited above.\n");
    s
}

#[derive(serde::Deserialize)]
struct JsonlChunk {
    idx: u32,
    heading_path: String,
    source_url: String,
    api_version: Option<String>,
    chunk_type: Option<String>,
    text: String,
}

#[derive(Serialize)]
struct ChunkRow<'a> {
    idx: u32,
    heading_path: &'a str,
    source_url: &'a str,
    api_version: Option<&'a str>,
    chunk_type: &'a str,
    text: &'a str,
}

/// Update a docset: re-download and replace.
///
/// In tests (file:// URL), `url` is passed directly.
/// In production, constructs the canonical registry URL.
pub fn update(docset: &str) -> Result<()> {
    let docset = input::validate_docset(docset)?;
    if is_test_file_url(&std::env::var("NOWDOCS_TEST_URL").unwrap_or_default()) {
        let url = std::env::var("NOWDOCS_TEST_URL")?;
        return install(&docset, &url);
    }

    let url = format!(
        "https://github.com/nowdocs-registry/{docset}/releases/latest/download/{docset}.tar"
    );
    install(&docset, &url)
}

/// Uninstall a docset: remove its db and manifest from the cache.
pub fn uninstall(docset: &str) -> Result<()> {
    let docset = input::validate_docset(docset)?;
    let db = cache::db_path(&docset);
    let mp = cache::manifest_path(&docset);
    if db.exists() {
        std::fs::remove_dir_all(&db).context("remove db")?;
    }
    if mp.is_file() {
        std::fs::remove_file(&mp).context("remove manifest")?;
    }
    Ok(())
}

// ===== Registry catalog discovery (U3: list / search) =====

/// A single docset entry in the registry catalog index.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegistryPackage {
    pub docset: String,
    pub version: String,
    pub license: String,
    pub chunk_count: u64,
    pub freshness: String,
    pub download_url: String,
    pub sha256: String,
    pub description: String,
}

/// The registry catalog index (`index.json`).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegistryIndex {
    pub schema_version: u64,
    pub generated_at: String,
    pub packages: Vec<RegistryPackage>,
}

/// Default catalog index URL: a dedicated repo under the `nowdocs-registry` GitHub org.
/// Override with `NOWDOCS_REGISTRY_INDEX_URL` (e.g. a `file://` path in tests).
///
/// NOTE: uses `github.com/.../raw/...` (not `raw.githubusercontent.com`) so the URL
/// passes `is_allowed_registry_url` (host allow-list), matching the existing
/// per-docset download convention.
pub fn index_url() -> String {
    if let Ok(url) = std::env::var("NOWDOCS_REGISTRY_INDEX_URL") {
        if !url.is_empty() {
            return url;
        }
    }
    "https://github.com/nowdocs-registry/registry-index/raw/main/index.json".to_string()
}

/// Fetch and parse the registry catalog index from an explicit URL.
pub fn fetch_index_from(url: &str) -> Result<RegistryIndex> {
    let tmp = if is_test_file_url(url) {
        PathBuf::from(url.strip_prefix("file://").unwrap_or(url))
    } else {
        download_to_temp(url)?
    };
    let text = std::fs::read_to_string(&tmp)
        .with_context(|| format!("reading registry index at {tmp:?}"))?;
    let idx: RegistryIndex = serde_json::from_str(&text).context("parsing registry index.json")?;
    // Security: every package's download_url must be on an allowed registry domain.
    for p in &idx.packages {
        if !is_allowed_registry_url(&p.download_url) {
            anyhow::bail!(
                "registry package {} has disallowed download_url: {}",
                p.docset,
                p.download_url
            );
        }
    }
    Ok(idx)
}

/// Fetch and parse the registry catalog index using the resolved/default URL.
pub fn fetch_index() -> Result<RegistryIndex> {
    fetch_index_from(&index_url())
}

/// Filter catalog packages by a case-insensitive substring match on name + description.
pub fn search_packages<'a>(idx: &'a RegistryIndex, query: &str) -> Vec<&'a RegistryPackage> {
    let q = query.to_lowercase();
    idx.packages
        .iter()
        .filter(|p| {
            p.docset.to_lowercase().contains(&q) || p.description.to_lowercase().contains(&q)
        })
        .collect()
}

/// `nowdocs registry list`: print the catalog (table or JSON).
pub fn list_index(json: bool) -> Result<()> {
    let idx = fetch_index()?;
    if json {
        println!("{}", serde_json::to_string_pretty(&idx)?);
        return Ok(());
    }
    println!(
        "{:<14} {:<10} {:<12} {:<12} {:<10}",
        "DOCSET", "VERSION", "LICENSE", "FRESHNESS", "INSTALLED"
    );
    println!("{}", "-".repeat(64));
    for p in &idx.packages {
        let installed = cache::db_path(&p.docset).exists();
        println!(
            "{:<14} {:<10} {:<12} {:<12} {:<10}",
            p.docset,
            p.version,
            p.license,
            p.freshness,
            if installed { "yes" } else { "no" }
        );
    }
    Ok(())
}

/// `nowdocs registry search <query>`: filter the catalog and print matches.
pub fn search_index(query: &str, json: bool) -> Result<()> {
    let idx = fetch_index()?;
    let matches = search_packages(&idx, query);
    if json {
        println!("{}", serde_json::to_string_pretty(&matches)?);
        return Ok(());
    }
    if matches.is_empty() {
        println!("No registry docsets match \"{query}\".");
        return Ok(());
    }
    println!(
        "{:<14} {:<10} {:<12} DESCRIPTION",
        "DOCSET", "VERSION", "LICENSE"
    );
    println!("{}", "-".repeat(64));
    for p in matches {
        println!(
            "{:<14} {:<10} {:<12} {}",
            p.docset, p.version, p.license, p.description
        );
    }
    Ok(())
}
