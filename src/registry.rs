//! Registry lifecycle: install / share / update / uninstall docsets.

use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::cache;
use crate::chunker::{Chunk, ChunkType};
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
fn extract_tar<R: Read>(reader: &mut R) -> Result<Vec<(String, Vec<u8>)>> {
    let mut files = Vec::new();
    let mut header = [0u8; 512];

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

            files.push((name, content));
        } else {
            // Skip non-regular entries.
            let padded = (size as usize).div_ceil(512) * 512;
            let mut skip = vec![0u8; padded];
            let _ = reader.read_exact(&mut skip);
        }
    }
    Ok(files)
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
    let active_db = cache::db_path(docset);
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
