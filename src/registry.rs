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

    let manifest_entry = entries
        .iter()
        .find(|(name, _)| name.ends_with("manifest.json"))
        .context("archive missing manifest.json")?;

    let manifest_json = std::str::from_utf8(&manifest_entry.1).context("manifest utf8")?;
    let m = manifest::parse_manifest(manifest_json)?;
    manifest::validate(&m)?;

    let db_dir = cache::db_path(&docset).parent().unwrap().to_path_buf();
    std::fs::create_dir_all(&db_dir)?;

    std::fs::write(cache::manifest_path(&docset), &manifest_entry.1)?;

    // Persist the upstream LICENSE bundled in the archive (if present) so a
    // later `share` of this docset carries the notice text forward. Without
    // this, docsets installed from a registry tar lose their LICENSE on
    // re-share even though the archive contained it — mirror what `ingest`
    // stashes at cache::license_text_path for locally-ingested docsets.
    let license_entry = entries.iter().find(|(name, _)| {
        std::path::Path::new(name)
            .file_name()
            .map(|f| f == std::ffi::OsStr::new("LICENSE"))
            .unwrap_or(false)
    });
    if let Some((_, data)) = license_entry {
        let license_path = cache::license_text_path(&docset);
        if let Some(parent) = license_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&license_path, data)?;
    }

    // Materialize chunks into the LanceDB store so retrieve::search works.
    // Uses zero vectors as placeholders; real vectors are rebuilt by CI (D10).
    let chunks_entry = entries
        .iter()
        .find(|(name, _)| name.ends_with("chunks.jsonl"));
    if let Some((_, data)) = chunks_entry {
        let jsonl = std::str::from_utf8(data).context("chunks.jsonl utf8")?;
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
        let store = Store::open(&docset)?;
        store.insert(&chunks, &zero_vectors)?;
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
