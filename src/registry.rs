//! Registry lifecycle: install / share / update / uninstall docsets.

use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::cache;
use crate::errors::{archive_error, NowdocsError};
use crate::input;
use crate::manifest::{self, Manifest};
use crate::sanitize;
use crate::store::Store;

/// Whether test-only code paths (file:// URL acceptance, NOWDOCS_TEST_URL
/// env var reads) are enabled.
///
/// This is a **compile-time** gate, not a spoofable runtime check. The
/// `test-fixture` Cargo feature is activated only during `cargo test` (via
/// the self dev-dependency in Cargo.toml). Production builds (`cargo build`,
/// `cargo build --release`) do not enable the feature, so `file://` URLs and
/// `NOWDOCS_TEST_URL` are rejected at compile time -- they do not exist in
/// the binary at all.
#[cfg(any(test, feature = "test-fixture"))]
fn is_test_mode() -> bool {
    true
}

#[cfg(not(any(test, feature = "test-fixture")))]
fn is_test_mode() -> bool {
    false
}

/// Returns true if `url` is a `file://` URL.
///
/// Used by `install_to_staging` and `fetch_index_from` to decide whether to
/// download or read locally. The security gate is `is_allowed_registry_url`,
/// which accepts `file://` for local test fixtures.
fn is_test_file_url(url: &str) -> bool {
    url.starts_with("file://")
}

/// Extract the host portion from a URL like "https://github.com/path".
fn url_host(url: &str) -> &str {
    let after_scheme = url.split("://").nth(1).unwrap_or(url);
    after_scheme.split('/').next().unwrap_or("")
}

fn is_allowed_registry_url(url: &str) -> bool {
    // `file://` URLs are accepted only in test mode for local test fixtures.
    // In production, `is_test_mode()` evaluates to false and file:// URLs are rejected.
    if is_test_file_url(url) {
        return is_test_mode();
    }
    let host = url_host(url);
    // Path after the host, e.g. "/nowdocs-registry/foo/releases/...".
    let after_scheme = url.split("://").nth(1).unwrap_or(url);
    let path = after_scheme.strip_prefix(host).unwrap_or(after_scheme);
    match host {
        // github.com requires path prefix /nowdocs-registry/ to prevent
        // lookalike domains like github.com/nowdocs-registry.evil.com
        "github.com" => path.starts_with("/nowdocs-registry/") || path == "/nowdocs-registry",
        // registry.nowdocs.dev mirrors the github.com strictness: release
        // artifacts must live under /releases/ so a bare-domain or unrelated
        // path on the CDN host cannot be served as a nowdocs artifact.
        "registry.nowdocs.dev" => path.starts_with("/releases/") || path == "/releases",
        _ => false,
    }
}

/// Stricter gate for PACKAGE download URLs (catalog `download_url` and the
/// install boundary). Unlike `is_allowed_registry_url` - which is also used to
/// fetch the catalog index itself from a `/raw/` repo path - package artifacts
/// must resolve to a GitHub Releases download (or `registry.nowdocs.dev`
/// release), so a catalog entry cannot point install at an arbitrary
/// raw/branch file and bypass the registry-release artifact contract.
///
/// `file://` URLs are NOT handled here; they are gated by `is_test_mode()`
/// directly in `install_to_staging` (S3) before this function is reached.
fn is_allowed_package_url(url: &str) -> bool {
    let host = url_host(url);
    let after_scheme = url.split("://").nth(1).unwrap_or(url);
    let path = after_scheme.strip_prefix(host).unwrap_or(after_scheme);
    match host {
        "github.com" => {
            // Anchor the release asset at the canonical GitHub Releases position
            // (exactly 6 path segments) so a raw/blob path with extra segments
            // before "releases" — e.g.
            // /nowdocs-registry/foo/raw/main/releases/download/foo.tar — cannot
            // pass merely because it contains "/releases/download/".
            //   /nowdocs-registry/<repo>/releases/download/<tag>/<asset>
            //   /nowdocs-registry/<repo>/releases/latest/download/<asset>
            let seg: Vec<&str> = path.trim_start_matches('/').split('/').collect();
            match seg.as_slice() {
                ["nowdocs-registry", repo, "releases", "download", tag, asset] => {
                    !repo.is_empty() && !tag.is_empty() && !asset.is_empty()
                }
                ["nowdocs-registry", repo, "releases", "latest", "download", asset] => {
                    !repo.is_empty() && !asset.is_empty()
                }
                _ => false,
            }
        }
        "registry.nowdocs.dev" => path.starts_with("/releases/") || path == "/releases",
        _ => false,
    }
}

/// Licenses permitted in the registry catalog index (per plan schema, §U3 line 283).
const ALLOWED_LICENSES: &[&str] = &["MIT", "Apache-2.0", "CC-BY-4.0"];

/// Validate a package `sha256` integrity value: exactly 64 ASCII hex characters.
fn is_valid_sha256(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Build the curl argument list for a registry artifact download.
///
/// `-f` fail on HTTP error, `-sS` silent-but-show-errors, `-L` follow the
/// single GitHub-Releases→CDN redirect, `--max-redirs 1` caps redirect
/// following at one hop (github.com → its own CDN) so an open-redirect or a
/// redirect chain to an attacker host is rejected. `-o` writes to `tmp`.
fn curl_args(tmp: &Path, url: &str) -> Vec<String> {
    vec![
        "-fsSL".to_string(),
        "--max-redirs".to_string(),
        "1".to_string(),
        "-o".to_string(),
        tmp.to_string_lossy().into_owned(),
        url.to_string(),
    ]
}

fn download_to_temp(url: &str, docset: &str) -> Result<PathBuf> {
    if !is_allowed_registry_url(url) {
        anyhow::bail!(
            "registry URL not in allowed domains: {} (allowed: github.com/nowdocs-registry, registry.nowdocs.dev/releases)",
            url
        );
    }
    // M11: include docset + pid + timestamp so concurrent installs (future MCP
    // server) never collide on the temp filename.
    let tmp = std::env::temp_dir().join(download_temp_name(docset));
    let status = std::process::Command::new("curl")
        .args(curl_args(&tmp, url))
        .status()
        .context("failed to spawn curl")?;
    if !status.success() {
        let _ = std::fs::remove_file(&tmp);
        anyhow::bail!("curl failed for {}", url);
    }
    Ok(tmp)
}

/// Lowercase hex encode a byte slice (no `hex` dependency).
fn hex_encode(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Lowercase hex sha256 of an in-memory byte slice.
///
/// Exported (hidden) so the integrity tests can compute an archive's expected
/// hash without adding `sha2` as a dev-dependency (Cargo.toml is a red-line
/// file). It is the in-memory counterpart to `sha256_file` and is also useful
/// to any caller that already holds the artifact bytes.
#[doc(hidden)]
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex_encode(&hasher.finalize())
}

/// Build the transient download filename for a docset install (M11).
///
/// Format: `nowdocs_dl_{docset}_{pid}_{timestamp_millis}`. The docset ties the
/// file to its install; pid + millisecond timestamp keep concurrent installs
/// (e.g. a future MCP server) from colliding on one path. Factored out so the
/// naming contract is unit-testable offline.
fn download_temp_name(docset: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("nowdocs_dl_{}_{}_{}", docset, std::process::id(), timestamp)
}

/// Streaming sha256 of a file (64 KiB buffer; never loads the whole file).
fn sha256_file(path: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    let mut f =
        std::fs::File::open(path).with_context(|| format!("open {} for sha256", path.display()))?;
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = f.read(&mut buf).context("read for sha256")?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex_encode(&hasher.finalize()))
}

/// Verify an archive's sha256 against the catalog-expected value. On mismatch,
/// remove the file when it is a transient download (`is_temp`) — never delete a
/// caller-supplied `file://` fixture — and bail with `ARCHIVE_SHA256_MISMATCH`.
fn verify_archive_integrity(path: &Path, expected: &str, is_temp: bool) -> Result<()> {
    let actual = sha256_file(path)?;
    // Catalog validation accepts uppercase hex (`is_ascii_hexdigit`) while
    // `sha256_file` always emits lowercase, so normalize before comparing —
    // otherwise an otherwise-valid uppercase catalog hash fails every install.
    let expected = expected.to_ascii_lowercase();
    if actual != expected {
        if is_temp {
            let _ = std::fs::remove_file(path);
        }
        let err = archive_error(
            "ARCHIVE_SHA256_MISMATCH",
            format!(
                "archive sha256 {actual} does not match expected {expected} ({})",
                path.display()
            ),
            "re-run install; if it persists, report the broken registry release",
        );
        return Err(anyhow::anyhow!("{}", err));
    }
    Ok(())
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

/// Archive artifact kind. The two kinds have distinct trust boundaries and
/// validation rules (architecture spec §3.2, OQ1 Method A):
///
/// - `ShareBundle`: contributor output (`nowdocs share`) — text-only
///   (`chunks.jsonl` + `manifest.json` + `LICENSE`); must NOT carry vectors.
/// - `RegistryRelease`: CI-built install artifact — a prebuilt `.lance` table
///   directory plus `manifest.json`; `chunks.jsonl` is optional and vectors are
///   trusted (CI rebuilt them with the pinned standard model).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArchiveType {
    ShareBundle,
    RegistryRelease,
}

/// Validate archive entries before writing active cache, using `ShareBundle`
/// rules (text-only; reject vectors; require `chunks.jsonl`). This is the
/// public, contributor-side contract. Install uses `validate_archive_with_mode`
/// with `RegistryRelease` instead.
///
/// Returns `Ok(())` if the archive passes all safety checks, or a structured
/// `NowdocsError` with a stable code and actionable hint.
pub fn validate_archive(entries: &[(String, Vec<u8>)]) -> Result<(), NowdocsError> {
    validate_archive_with_mode(entries, ArchiveType::ShareBundle)
}

fn has_drive_prefix(name: &str) -> bool {
    let mut chars = name.chars();
    if let (Some(c1), Some(':')) = (chars.next(), chars.next()) {
        c1.is_ascii_alphabetic()
    } else {
        false
    }
}

/// Validate archive entries under the given artifact contract.
fn validate_archive_with_mode(
    entries: &[(String, Vec<u8>)],
    mode: ArchiveType,
) -> Result<(), NowdocsError> {
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
    let mut has_lance = false;

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

        // Path safety: reject absolute and drive-prefixed paths (Windows safety).
        let path = std::path::Path::new(name);
        if path.is_absolute()
            || name.starts_with('/')
            || name.starts_with('\\')
            || has_drive_prefix(name)
        {
            return Err(archive_error(
                "ARCHIVE_UNSAFE_PATH",
                format!("archive contains absolute path: {}", name),
                "report the broken registry release",
            ));
        }

        // Path safety: reject .. and drive prefix components.
        for component in path.components() {
            let s = component.as_os_str().to_string_lossy();
            if matches!(
                component,
                std::path::Component::ParentDir | std::path::Component::Prefix(_)
            ) || has_drive_prefix(&s)
            {
                return Err(archive_error(
                    "ARCHIVE_UNSAFE_PATH",
                    format!(
                        "archive contains unsafe path component (.. or drive prefix): {}",
                        name
                    ),
                    "report the broken registry release",
                ));
            }
        }

        // Vector artifact detection — check every path component, not just the
        // full name suffix. LanceDB stores are directories like
        // `index.lance/data.bin` where child files don't end in `.lance`.
        let basename = path
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut component_is_lance = false;
        let mut component_is_other_vector = false;
        for c in path.components() {
            let s = c.as_os_str().to_string_lossy();
            if s.ends_with(".lance") {
                component_is_lance = true;
            }
            if s.ends_with(".faiss") || s.starts_with("vectors.") || s.starts_with("embeddings.") {
                component_is_other_vector = true;
            }
        }
        // Non-lance vector artifacts are rejected in every mode.
        if component_is_other_vector {
            return Err(archive_error(
                "ARCHIVE_VECTOR_ARTIFACT",
                format!("archive contains vector artifact: {}", name),
                "share must not include vector data; rebuild vectors with CI",
            ));
        }
        if component_is_lance {
            match mode {
                ArchiveType::ShareBundle => {
                    return Err(archive_error(
                        "ARCHIVE_VECTOR_ARTIFACT",
                        format!("archive contains vector artifact: {}", name),
                        "share must not include vector data; rebuild vectors with CI",
                    ));
                }
                ArchiveType::RegistryRelease => {
                    has_lance = true;
                }
            }
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

    match mode {
        ArchiveType::ShareBundle => {
            if !has_chunks {
                return Err(archive_error(
                    "ARCHIVE_MISSING_CHUNKS",
                    "registry archive is missing chunks.jsonl",
                    "retry install, or report the broken registry release",
                ));
            }
            // M8: row-level validation of chunks.jsonl against the manifest.
            let manifest_entry = entries
                .iter()
                .find(|(n, _)| n.ends_with("manifest.json"))
                .expect("has_manifest checked above");
            let manifest_json = std::str::from_utf8(&manifest_entry.1).map_err(|_| {
                archive_error(
                    "ARCHIVE_INVALID_MANIFEST",
                    "manifest.json is not valid UTF-8",
                    "report the broken registry release",
                )
            })?;
            let m = manifest::parse_manifest(manifest_json).map_err(|e| {
                archive_error(
                    "ARCHIVE_INVALID_MANIFEST",
                    format!("manifest.json failed to parse: {e}"),
                    "report the broken registry release",
                )
            })?;
            let chunks_entry = entries
                .iter()
                .find(|(n, _)| n.ends_with("chunks.jsonl"))
                .expect("has_chunks checked above");
            let rows = parse_chunks_jsonl(&chunks_entry.1)?;
            validate_chunks_jsonl(&m, &rows)?;
        }
        ArchiveType::RegistryRelease => {
            if !has_lance {
                return Err(archive_error(
                    "ARCHIVE_MISSING_STORE",
                    "registry release archive is missing a .lance table directory",
                    "retry install, or report the broken registry release",
                ));
            }
        }
    }

    Ok(())
}

/// Parse a `chunks.jsonl` byte blob into rows (skipping blank lines).
fn parse_chunks_jsonl(data: &[u8]) -> Result<Vec<JsonlChunk>, NowdocsError> {
    let text = std::str::from_utf8(data).map_err(|_| {
        archive_error(
            "ARCHIVE_INVALID_CHUNKS",
            "chunks.jsonl is not valid UTF-8",
            "report the broken registry release",
        )
    })?;
    let mut rows = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let row: JsonlChunk = serde_json::from_str(line).map_err(|e| {
            archive_error(
                "ARCHIVE_INVALID_CHUNKS",
                format!("chunks.jsonl line failed to parse: {e}"),
                "report the broken registry release",
            )
        })?;
        rows.push(row);
    }
    Ok(rows)
}

/// M8: row-level validation of `chunks.jsonl` against its manifest.
///
/// - `idx` must be the contiguous sequence 0, 1, …, N-1 (no gaps, no dupes).
/// - the row count must equal `manifest.source.chunk_count`.
/// - `text` must be non-empty after sanitize (strips injection/whitespace).
/// - `chunk_type`, if present, must be "Code" or "Info".
fn validate_chunks_jsonl(manifest: &Manifest, rows: &[JsonlChunk]) -> Result<(), NowdocsError> {
    let expected = manifest.source.chunk_count as usize;
    if rows.len() != expected {
        return Err(archive_error(
            "ARCHIVE_INVALID_CHUNKS",
            format!(
                "chunks.jsonl has {} rows but manifest chunk_count is {}",
                rows.len(),
                expected
            ),
            "report the broken registry release",
        ));
    }
    for (i, row) in rows.iter().enumerate() {
        if row.idx as usize != i {
            return Err(archive_error(
                "ARCHIVE_INVALID_CHUNKS",
                format!(
                    "chunks.jsonl idx {} at position {} is not contiguous 0..N-1",
                    row.idx, i
                ),
                "report the broken registry release",
            ));
        }
        if sanitize::sanitize_chunk(&row.text).trim().is_empty() {
            return Err(archive_error(
                "ARCHIVE_INVALID_CHUNKS",
                format!("chunks.jsonl row {} has empty text after sanitize", i),
                "report the broken registry release",
            ));
        }
        match row.chunk_type.as_deref() {
            Some("Code") | Some("Info") | None => {}
            Some(other) => {
                return Err(archive_error(
                    "ARCHIVE_INVALID_CHUNKS",
                    format!("chunks.jsonl row {} has invalid chunk_type {:?}", i, other),
                    "report the broken registry release",
                ));
            }
        }
    }
    Ok(())
}

/// Install a docset from an archive URL (no sha256 enforcement).
///
/// Used by tests with `file://` fixtures and as a convenience entry point. The
/// production binary path uses `install_with_sha256` after looking up the
/// catalog hash, so registry installs always verify integrity.
///
/// **Security**: production URLs must be on `nowdocs-registry` domains.
/// Test `file://` URLs are allowed (test fixture bypass).
pub fn install(docset: &str, url: &str) -> Result<()> {
    install_inner(docset, url, None)
}

/// Install a docset, verifying the archive's sha256 against `expected_sha256`
/// (S2). A mismatch removes any transient download and bails with
/// `ARCHIVE_SHA256_MISMATCH` before any active cache path is touched.
pub fn install_with_sha256(docset: &str, url: &str, expected_sha256: &str) -> Result<()> {
    install_inner(docset, url, Some(expected_sha256))
}

fn install_inner(docset: &str, url: &str, expected_sha256: Option<&str>) -> Result<()> {
    let docset = input::validate_docset(docset)?;
    cache::ensure_layout()?;

    // N6: exclusive per-docset lock for the duration of the install. The guard
    // removes the lockfile on drop (success, failure, or panic).
    let _lock = acquire_install_lock(&docset)?;

    // Build a complete, verified candidate under staging (no active writes yet).
    let staging_path = install_to_staging(&docset, url, expected_sha256)?;

    // Atomically promote staging -> active (rename-based). On failure this
    // restores the previous active docset and leaves staging for diagnostics.
    promote_staging(&docset, &staging_path)?;

    Ok(())
}

/// N6: an exclusive per-docset install lock. Removed on drop.
struct InstallLock {
    path: PathBuf,
}

impl Drop for InstallLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Acquire the per-docset install lock using `O_EXCL` (`create_new`). The lock
/// file content is the epoch-seconds of creation; a lock older than one hour is
/// treated as stale and replaced. A fresh lock yields the spec-mandated busy
/// error: "docset {docset} is currently being installed by another process".
fn acquire_install_lock(docset: &str) -> Result<InstallLock> {
    std::fs::create_dir_all(cache::staging_root())?;
    let path = cache::staging_root().join(format!("{docset}.lock"));
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(mut f) => {
            use std::io::Write;
            let _ = writeln!(f, "{now}");
            Ok(InstallLock { path })
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            let stale = match std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok())
            {
                Some(t) => now.saturating_sub(t) >= 3600,
                None => {
                    // Unreadable or truncated lock (e.g. a crash between
                    // create_new and the timestamp write): fall back to the
                    // file mtime, and if even that is unavailable treat the
                    // lock as stale so a corrupt lockfile can never pin the
                    // docset busy until manual cleanup.
                    std::fs::metadata(&path)
                        .and_then(|m| m.modified())
                        .ok()
                        .and_then(|mt| mt.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| now.saturating_sub(d.as_secs()) >= 3600)
                        .unwrap_or(true)
                }
            };
            if stale {
                let _ = std::fs::remove_file(&path);
                let mut f = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)?;
                use std::io::Write;
                let _ = writeln!(f, "{now}");
                Ok(InstallLock { path })
            } else {
                anyhow::bail!("docset {docset} is currently being installed by another process")
            }
        }
        Err(e) => Err(e).with_context(|| format!("create install lock {}", path.display())),
    }
}

/// Install a docset to a staging directory (not active paths).
fn install_to_staging(docset: &str, url: &str, expected_sha256: Option<&str>) -> Result<PathBuf> {
    let staging_path = cache::new_staging_path(docset);
    std::fs::create_dir_all(&staging_path)?;

    // OQ6/P1: package downloads must resolve to a release-artifact URL, so a
    // catalog entry (or direct caller) cannot point install at an arbitrary
    // raw/branch repo path. file:// test fixtures are accepted ONLY in test
    // mode (S3); in production, file:// URLs are rejected so the library API
    // cannot be used to install arbitrary local files.
    if is_test_file_url(url) {
        if !is_test_mode() {
            anyhow::bail!(
                "file:// URLs are not allowed in production builds; \
                 registry downloads must use github.com/nowdocs-registry/ or registry.nowdocs.dev"
            );
        }
    } else if !is_allowed_package_url(url) {
        anyhow::bail!(
            "registry URL not in allowed domains: {url} (package downloads must be \
             a release artifact: github.com/nowdocs-registry/<repo>/releases, \
             registry.nowdocs.dev/releases)"
        );
    }

    let (archive_path, is_temp) = if is_test_file_url(url) {
        let path = url.strip_prefix("file://").unwrap();
        (PathBuf::from(path), false)
    } else {
        (download_to_temp(url, docset)?, true)
    };

    // S2: streaming sha256 integrity check. On mismatch, transient downloads
    // are removed (fixtures are never deleted) and we bail before any active
    // cache path is touched.
    if let Some(expected) = expected_sha256 {
        verify_archive_integrity(&archive_path, expected, is_temp)?;
    }

    let mut file = std::fs::File::open(&archive_path).context("open archive")?;
    let entries = extract_tar(&mut file)?;
    drop(file);
    if is_temp {
        let _ = std::fs::remove_file(&archive_path);
    }

    // S1: registry releases accept a prebuilt `.lance` table and do not require
    // `chunks.jsonl` (OQ1 Method A — vectors are CI-built with the pinned model).
    validate_archive_with_mode(&entries, ArchiveType::RegistryRelease)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Materialize every entry to staging, preserving relative paths so the
    // `.lance` table tree is reproduced for a rename-based promote.
    for (name, data) in &entries {
        let dest = staging_path.join(name);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&dest, data)?;
    }

    // S7: identity binding + schema/model/license validation.
    verify_staging(&staging_path, docset)?;
    // M12: install-context business invariants (chunk_count, traceable URL).
    let m = manifest::parse_manifest(&std::fs::read_to_string(
        staging_path.join("manifest.json"),
    )?)?;
    manifest::validate_manifest_for_docset(&m, docset)?;

    Ok(staging_path)
}

/// Verify that staging contains a valid manifest whose `docset` matches the
/// CLI-provided install name (S7: identity binding / integrity symmetry).
fn verify_staging(staging_path: &Path, expected_docset: &str) -> Result<()> {
    let manifest_path = staging_path.join("manifest.json");
    if !manifest_path.is_file() {
        anyhow::bail!("staging missing manifest.json");
    }

    let raw = std::fs::read_to_string(&manifest_path)?;
    let m = manifest::parse_manifest(&raw)?;
    manifest::validate(&m)?;

    if m.docset != expected_docset {
        anyhow::bail!(
            "manifest docset {:?} does not match install name {:?}",
            m.docset,
            expected_docset
        );
    }

    Ok(())
}

/// Locate the single `.lance` table directory materialized under staging.
fn find_lance_dir(staging_path: &Path) -> Result<PathBuf> {
    let mut candidates = Vec::new();
    for entry in std::fs::read_dir(staging_path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.ends_with(".lance") {
                candidates.push(entry.path());
            }
        }
    }
    match candidates.len() {
        0 => anyhow::bail!("staging contains no .lance table directory"),
        1 => Ok(candidates.remove(0)),
        _ => anyhow::bail!(
            "staging contains {} .lance directories (expected exactly one)",
            candidates.len()
        ),
    }
}

/// Promote a verified staging candidate to the active cache using rename-based
/// atomic replacement (S1+S4). No zero vectors, no `chunks.jsonl`
/// materialization, and no `std::fs::copy` for the store.
///
/// Flow: backup active (rename) -> rename staging `.lance` to active db ->
/// atomic-write manifest (tmp + rename) -> copy license text -> reopen store
/// and verify row count == manifest chunk_count. On any failure after backup,
/// restore the previous active docset and leave staging for diagnostics. On
/// success, best-effort cleanup of rollback + staging.
fn promote_staging(docset: &str, staging_path: &Path) -> Result<()> {
    let active_manifest = cache::manifest_path(docset);
    let active_db = cache::db_path(docset);
    let active_license = cache::license_text_path(docset);

    let staging_manifest = staging_path.join("manifest.json");
    let manifest_raw = std::fs::read_to_string(&staging_manifest)?;
    let expected_rows = manifest::parse_manifest(&manifest_raw)?.source.chunk_count as u64;

    let staging_lance = find_lance_dir(staging_path)?;
    let staging_license = staging_path.join("LICENSE");

    // 1. Backup the current active docset via rename (same filesystem).
    let had_active = active_db.exists() || active_manifest.is_file();
    let rollback = if had_active {
        let rb = cache::rollback_path(docset);
        std::fs::create_dir_all(&rb)?;

        let mut db_moved = false;
        let mut manifest_moved = false;
        let mut license_moved = false;

        let backup_res = (|| -> Result<()> {
            if active_db.exists() {
                std::fs::rename(&active_db, rb.join("db.lance"))?;
                db_moved = true;
            }
            if active_manifest.is_file() {
                std::fs::rename(&active_manifest, rb.join("manifest.json"))?;
                manifest_moved = true;
            }
            if active_license.is_file() {
                std::fs::rename(&active_license, rb.join("license.txt"))?;
                license_moved = true;
            }
            Ok(())
        })();

        if let Err(e) = backup_res {
            // Transactional rollback: restore any partially moved active files before returning the error
            if db_moved {
                let _ = std::fs::rename(rb.join("db.lance"), &active_db);
            }
            if manifest_moved {
                let _ = std::fs::rename(rb.join("manifest.json"), &active_manifest);
            }
            if license_moved {
                let _ = std::fs::rename(rb.join("license.txt"), &active_license);
            }
            let _ = std::fs::remove_dir_all(&rb);
            return Err(e);
        }
        Some(rb)
    } else {
        None
    };

    // 2. Promote + verify. Isolated in a closure so any failure restores backup.
    let promote_result = (|| -> Result<()> {
        if let Some(parent) = active_db.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::rename(&staging_lance, &active_db)?;

        if let Some(parent) = active_manifest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp_manifest = active_manifest.with_extension("manifest.json.tmp");
        std::fs::write(&tmp_manifest, &manifest_raw)?;
        std::fs::rename(&tmp_manifest, &active_manifest)?;

        if staging_license.is_file() {
            if let Some(parent) = active_license.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&staging_license, &active_license)?;
        }

        // 3. Verify: reopen the installed table and confirm it is readable and
        // has exactly the manifest-declared number of chunks.
        let store = Store::open(docset)?;
        let rows = store.row_count()?;
        if rows == 0 {
            anyhow::bail!("installed store for {docset} has 0 rows");
        }
        if rows != expected_rows {
            anyhow::bail!(
                "installed store row_count {rows} != manifest chunk_count {expected_rows} for {docset}"
            );
        }
        Ok(())
    })();

    match promote_result {
        Ok(()) => {
            if let Some(rb) = rollback {
                if let Err(e) = cleanup_rollback(&rb) {
                    eprintln!("warning: failed to clean up rollback: {e}");
                }
            }
            if let Err(e) = cleanup_staging(staging_path) {
                eprintln!("warning: failed to clean up staging: {e}");
            }
            Ok(())
        }
        Err(e) => {
            // Restore the previous active docset (or clear partial active for a
            // fresh install). Best-effort: never mask the original error.
            if active_db.exists() {
                let _ = std::fs::remove_dir_all(&active_db);
            }
            if active_manifest.is_file() {
                let _ = std::fs::remove_file(&active_manifest);
            }
            if active_license.is_file() {
                let _ = std::fs::remove_file(&active_license);
            }
            if let Some(rb) = rollback {
                let rb_db = rb.join("db.lance");
                let rb_manifest = rb.join("manifest.json");
                let rb_license = rb.join("license.txt");
                if rb_db.exists() {
                    let _ = std::fs::rename(&rb_db, &active_db);
                }
                if rb_manifest.is_file() {
                    let _ = std::fs::rename(&rb_manifest, &active_manifest);
                }
                if rb_license.is_file() {
                    let _ = std::fs::rename(&rb_license, &active_license);
                }
                let _ = cleanup_rollback(&rb);
            }
            Err(e)
        }
    }
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

/// Share a docset: write manifest + text chunks (NO vectors, D10) to `out_dir`.
pub fn share(docset: &str, out_dir: &Path) -> Result<PathBuf> {
    let docset = input::validate_docset(docset)?;
    let mp = cache::manifest_path(&docset);
    if !mp.is_file() {
        anyhow::bail!("docset not installed: {}", docset);
    }
    let raw = std::fs::read_to_string(&mp)?;
    let m = manifest::parse_manifest(&raw)?;
    manifest::validate(&m)?;

    let store = Store::open(&docset)?;
    let chunks = store.dump_chunks()?;

    let share_dir = out_dir.join(&docset);
    // M10: refuse to write into a pre-existing non-empty output directory so a
    // stale bundle from a prior share cannot silently mix with the fresh one.
    if share_dir.exists() {
        let is_empty = std::fs::read_dir(&share_dir)?.next().is_none();
        if !is_empty {
            anyhow::bail!(
                "output directory {} already exists and is non-empty; remove it first or use a different path",
                share_dir.display()
            );
        }
    }
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

// On-disk `chunks.jsonl` row schema. `idx`/`text`/`chunk_type` are checked by
// `validate_chunks_jsonl` (M8); the metadata fields are part of the contributor
// bundle schema and retained for forward-compatible deserialization.
#[allow(dead_code)]
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
/// (S3) The library `update()` reads `NOWDOCS_TEST_URL` for test fixtures.
/// The production binary never calls this function with a test URL because
/// `main.rs` `Update` handler calls `catalog_lookup_for` (which does NOT read
/// `NOWDOCS_TEST_URL`) and passes the canonical/catalog-paired URL to `install()` directly.
pub fn update(docset: &str) -> Result<()> {
    let docset = input::validate_docset(docset)?;
    if is_test_mode() && is_test_file_url(&std::env::var("NOWDOCS_TEST_URL").unwrap_or_default()) {
        let url = std::env::var("NOWDOCS_TEST_URL")?;
        return install(&docset, &url);
    }
    let url = format!(
        "https://github.com/nowdocs-registry/{docset}/releases/latest/download/{docset}.tar"
    );
    install(&docset, &url)
}

/// Uninstall a docset: remove its db and manifest from the cache, plus any
/// docset-scoped leftovers (stashed license, staging dirs, rollback dirs).
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

    // M9: best-effort cleanup of docset-scoped leftovers. Failures here must
    // not mask a successful uninstall of the active db/manifest above.
    let lic = cache::license_text_path(&docset);
    if lic.is_file() {
        let _ = std::fs::remove_file(&lic);
    }
    remove_docset_dirs(cache::staging_root(), &docset);
    remove_docset_dirs(cache::rollback_root(), &docset);
    Ok(())
}

/// Best-effort removal of `<root>/<docset>-<pid>-<timestamp>` directories (staging/rollback
/// leftovers for one docset). Other docsets' dirs are left untouched.
fn remove_docset_dirs(root: PathBuf, docset: &str) {
    let entries = match std::fs::read_dir(&root) {
        Ok(it) => it,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if is_dir {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                // Safely match <docset>-<pid>-<timestamp> without prefix-matching other
                // docsets (e.g. "foo" matching leftovers of "foo-bar").
                let mut parts = name.rsplitn(3, '-');
                let timestamp = parts.next();
                let pid = parts.next();
                let d = parts.next();
                let name_match = matches!(
                    (d, pid, timestamp),
                    (Some(expected), Some(p), Some(t))
                        if expected == docset
                            && p.chars().all(|c| c.is_ascii_digit())
                            && t.chars().all(|c| c.is_ascii_digit())
                );
                if name_match {
                    let _ = std::fs::remove_dir_all(&path);
                }
            }
        }
    }
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
    #[serde(default)]
    pub description: Option<String>,
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
        download_to_temp(url, "index")?
    };
    let text = std::fs::read_to_string(&tmp)
        .with_context(|| format!("reading registry index at {tmp:?}"))?;
    let idx: RegistryIndex = serde_json::from_str(&text).context("parsing registry index.json")?;
    // Security: every package must satisfy the catalog contract before it is
    // surfaced to users (plan §U3): allowed download host, allowed license,
    // and a valid 64-hex sha256 integrity value.
    for p in &idx.packages {
        // Package downloads must be a release-artifact shape (not an arbitrary
        // raw/branch repo path) so a catalog entry cannot bypass the
        // registry-release contract. The index itself is fetched under the
        // broader is_allowed_registry_url (it legitimately lives at /raw/).
        if !is_allowed_package_url(&p.download_url) {
            anyhow::bail!(
                "registry package {} has disallowed download_url: {}",
                p.docset,
                p.download_url
            );
        }
        if !ALLOWED_LICENSES.contains(&p.license.as_str()) {
            anyhow::bail!(
                "registry package {} has disallowed license: {} (allowed: {})",
                p.docset,
                p.license,
                ALLOWED_LICENSES.join(", ")
            );
        }
        if !is_valid_sha256(&p.sha256) {
            anyhow::bail!(
                "registry package {} has invalid sha256 (must be 64 hex chars): {}",
                p.docset,
                p.sha256
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
            p.docset.to_lowercase().contains(&q)
                || p.description
                    .as_deref()
                    .unwrap_or_default()
                    .to_lowercase()
                    .contains(&q)
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
        "{:<14} {:<10} {:<8} {:<12} {:<12} {:<10}",
        "DOCSET", "VERSION", "CHUNKS", "LICENSE", "FRESHNESS", "INSTALLED"
    );
    println!("{}", "-".repeat(70));
    for p in &idx.packages {
        let installed = cache::db_path(&p.docset).exists();
        println!(
            "{:<14} {:<10} {:<8} {:<12} {:<12} {:<10}",
            p.docset,
            p.version,
            p.chunk_count,
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
            p.docset,
            p.version,
            p.license,
            p.description.as_deref().unwrap_or("")
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- S2: sha256 integrity verification ---

    #[test]
    fn sha256_mismatch_deletes_temp_file() {
        // is_temp = true → the transient download is removed on mismatch.
        let path = std::env::temp_dir().join(download_temp_name("sha256-temp"));
        std::fs::write(&path, b"registry artifact bytes").unwrap();
        let wrong = "0".repeat(64);
        let res = verify_archive_integrity(&path, &wrong, true);
        assert!(res.is_err(), "wrong sha256 must be rejected");
        assert!(
            format!("{}", res.unwrap_err()).contains("ARCHIVE_SHA256_MISMATCH"),
            "error must carry the mismatch code"
        );
        assert!(
            !path.exists(),
            "transient download must be deleted on sha256 mismatch"
        );
    }

    #[test]
    fn sha256_mismatch_preserves_caller_file() {
        // is_temp = false → a caller-supplied file:// fixture is never deleted.
        let path = std::env::temp_dir().join(download_temp_name("sha256-keep"));
        std::fs::write(&path, b"registry artifact bytes").unwrap();
        let wrong = "0".repeat(64);
        let res = verify_archive_integrity(&path, &wrong, false);
        assert!(res.is_err(), "wrong sha256 must be rejected");
        assert!(
            path.exists(),
            "caller-supplied fixture must NOT be deleted on mismatch"
        );
        let _ = std::fs::remove_file(&path);
    }

    // --- OQ6: curl must cap redirect following ---

    #[test]
    fn curl_does_not_follow_redirects() {
        let args = curl_args(
            Path::new("/tmp/nowdocs_test_dl"),
            "https://github.com/nowdocs-registry/x/releases/download/v1/x.tar",
        );
        let idx = args
            .iter()
            .position(|a| a == "--max-redirs")
            .expect("curl args must include --max-redirs");
        assert_eq!(
            args.get(idx + 1).map(String::as_str),
            Some("1"),
            "--max-redirs must be capped at 1 (github.com -> its CDN only)"
        );
    }

    // --- M11: download temp filename is docset/pid/timestamp scoped ---

    #[test]
    fn concurrent_downloads_use_different_temp_files() {
        let name = download_temp_name("alpha");
        // Structure: nowdocs_dl_{docset}_{pid}_{timestamp_millis}
        let parts: Vec<&str> = name.split('_').collect();
        assert_eq!(
            parts.len(),
            5,
            "temp name must be nowdocs_dl_{{docset}}_{{pid}}_{{ts}}, got: {name}"
        );
        assert_eq!(parts[0], "nowdocs");
        assert_eq!(parts[1], "dl");
        assert_eq!(parts[2], "alpha", "temp name must include the docset");
        assert!(
            parts[3].parse::<u32>().is_ok(),
            "temp name must include the numeric pid, got: {}",
            parts[3]
        );
        assert!(
            parts[4].parse::<u64>().is_ok(),
            "temp name must include a numeric timestamp, got: {}",
            parts[4]
        );
    }

    // --- OQ6/P1: package downloads must be release-artifact URLs ---

    #[test]
    fn package_url_accepts_github_release_download() {
        assert!(is_allowed_package_url(
            "https://github.com/nowdocs-registry/nextjs/releases/download/nextjs-14.2.5/nextjs.tar"
        ));
    }

    #[test]
    fn package_url_rejects_github_raw_branch_path() {
        // A /raw/ branch file passes the broad index gate but must NOT pass the
        // package gate — a catalog entry cannot point install at arbitrary repo
        // content and bypass the registry-release contract.
        assert!(!is_allowed_package_url(
            "https://github.com/nowdocs-registry/evil/raw/main/evil.tar"
        ));
    }

    #[test]
    fn package_url_rejects_github_repo_path_without_releases() {
        assert!(!is_allowed_package_url(
            "https://github.com/nowdocs-registry/evil/some/path.tar"
        ));
    }

    #[test]
    fn index_gate_still_allows_github_raw_index_path() {
        // The catalog index itself legitimately lives at a /raw/ repo path; the
        // broad gate must keep admitting it so `fetch_index` keeps working.
        assert!(is_allowed_registry_url(
            "https://github.com/nowdocs-registry/registry-index/raw/main/index.json"
        ));
    }

    // --- S2: sha256 compare is case-insensitive ---

    #[test]
    fn sha256_verify_is_case_insensitive() {
        let path = std::env::temp_dir().join(download_temp_name("sha256-case"));
        std::fs::write(&path, b"registry artifact bytes").unwrap();
        let lower = sha256_hex(&std::fs::read(&path).unwrap());
        let upper = lower.to_ascii_uppercase();
        // Uppercase expected (as catalog validation permits) must still match.
        verify_archive_integrity(&path, &upper, false).expect("uppercase sha256 must match");
        let _ = std::fs::remove_file(&path);
    }

    // --- OQ6/P2: package URL must be a real GitHub Releases download ---

    #[test]
    fn package_url_rejects_github_raw_path_with_releases_segment() {
        // Contains "/releases/" but is a mutable raw/branch file, NOT a GitHub
        // Releases download — must not pass the package gate.
        assert!(!is_allowed_package_url(
            "https://github.com/nowdocs-registry/foo/raw/main/releases/foo.tar"
        ));
        assert!(!is_allowed_package_url(
            "https://github.com/nowdocs-registry/foo/blob/main/releases/foo.tar"
        ));
    }

    #[test]
    fn package_url_accepts_github_latest_release_download() {
        assert!(is_allowed_package_url(
            "https://github.com/nowdocs-registry/nextjs/releases/latest/download/nextjs.tar"
        ));
    }

    // --- P1: directory entries (typeflag 5) in release tarballs are skipped ---

    /// Minimal tar entry builder for `extract_tar` tests. `extract_tar` only
    /// reads name/size/typeflag, so checksum/ustar fields are left zeroed.
    fn tar_entry_for_test(name: &str, data: &[u8], typeflag: u8) -> Vec<u8> {
        let mut header = [0u8; 512];
        let nb = name.as_bytes();
        header[..nb.len()].copy_from_slice(nb);
        let size = format!("{:011o}\0", data.len());
        header[124..124 + size.len()].copy_from_slice(size.as_bytes());
        header[156] = typeflag;
        let mut entry = header.to_vec();
        entry.extend_from_slice(data);
        let padded = data.len().div_ceil(512) * 512;
        if padded > data.len() {
            entry.extend(vec![0u8; padded - data.len()]);
        }
        entry
    }

    #[test]
    fn extract_tar_skips_directory_entries() {
        // A normal `tar` over `<docset>.lance/` emits typeflag-5 directory
        // entries for the LanceDB tree. They must be skipped (not rejected as
        // ARCHIVE_UNSUPPORTED_ENTRY) so real CI-built release artifacts install.
        let mut archive = Vec::new();
        archive.extend(tar_entry_for_test("pkg.lance/", &[], b'5'));
        archive.extend(tar_entry_for_test("pkg.lance/data.bin", b"vec", 0));
        archive.extend_from_slice(&[0u8; 512]);
        archive.extend_from_slice(&[0u8; 512]);

        let mut cursor = std::io::Cursor::new(archive);
        let files = extract_tar(&mut cursor).expect("directory entries must not be rejected");
        let names: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(
            names,
            vec!["pkg.lance/data.bin"],
            "directory entry must be skipped, file entry preserved"
        );
        assert_eq!(files[0].1, b"vec");
    }

    // --- OQ6/P2 (wave-4): release URL must be anchored at the canonical
    // segment position, not merely contain "/releases/download/" ---

    #[test]
    fn package_url_rejects_github_raw_path_with_releases_download_segment() {
        // Extra "/raw/main/" segments before "/releases/download/" — still a
        // mutable branch file, must not pass even though it contains the
        // "/releases/download/" substring.
        assert!(!is_allowed_package_url(
            "https://github.com/nowdocs-registry/foo/raw/main/releases/download/foo.tar"
        ));
        // Trailing segments beyond the asset are rejected too.
        assert!(!is_allowed_package_url(
            "https://github.com/nowdocs-registry/foo/releases/download/v1/x.tar/extra"
        ));
    }

    // --- P2 (wave-4): a corrupt/truncated install lock must not pin a docset
    // busy forever; unreadable content falls back to mtime age ---

    #[test]
    fn acquire_lock_treats_fresh_corrupt_lock_as_busy() {
        let dir = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };
        let docset = "lock-corrupt-fresh";
        std::fs::create_dir_all(cache::staging_root()).unwrap();
        let lp = cache::staging_root().join(format!("{docset}.lock"));
        // Non-numeric content but a fresh mtime → treated as busy (not stale).
        std::fs::write(&lp, b"not-a-timestamp").unwrap();
        let err = match acquire_install_lock(docset) {
            Err(e) => e,
            Ok(_) => panic!("fresh-mtime corrupt lock must be busy"),
        };
        assert!(
            format!("{err}").contains("currently being installed"),
            "must surface the busy-lock error"
        );
        let _ = std::fs::remove_file(&lp);
    }

    #[test]
    fn acquire_lock_replaces_stale_corrupt_lock() {
        let dir = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };
        let docset = "lock-corrupt-stale";
        std::fs::create_dir_all(cache::staging_root()).unwrap();
        let lp = cache::staging_root().join(format!("{docset}.lock"));
        std::fs::write(&lp, b"not-a-timestamp").unwrap();
        // Backdate mtime beyond the 1h staleness threshold.
        let old = std::time::SystemTime::now() - std::time::Duration::from_secs(7200);
        let ft = std::fs::FileTimes::new().set_modified(old);
        std::fs::File::options()
            .write(true)
            .open(&lp)
            .unwrap()
            .set_times(ft)
            .unwrap();
        let res = acquire_install_lock(docset);
        assert!(
            res.is_ok(),
            "stale corrupt lock must be replaced and acquired"
        );
    }
}
