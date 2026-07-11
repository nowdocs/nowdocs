//! Local cache layout under the OS cache dir (`<cache>/nowdocs/`).
//!
//! Layout (v1):
//! ```text
//! <cache>/nowdocs/
//!   .layout_version        # contains "1"
//!   db/<docset>.lance      # one Lance table per docset
//!   models/<org>/<repo>/   # downloaded embedder weights
//! ```
//! `ensure_layout` gates on `CACHE_LAYOUT_VERSION`: an on-disk layout written
//! by an incompatible nowdocs is rejected with a reinitialize hint rather than
//! silently corrupting the cache.

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use serde::Serialize;

pub const CACHE_LAYOUT_VERSION: u32 = 1;

const APP_DIR: &str = "nowdocs";
const LAYOUT_VERSION_FILE: &str = ".layout_version";

/// `<cache_dir>/nowdocs`. Returns the path even if the cache dir does not yet
/// exist; callers use `ensure_layout` to materialize it.
pub fn cache_root() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        if !xdg.trim().is_empty() {
            let path = PathBuf::from(xdg);
            if path.is_absolute() {
                return path.join(APP_DIR);
            }
        }
    }
    dirs::cache_dir()
        .expect("no OS cache dir — set XDG_CACHE_HOME or HOME")
        .join(APP_DIR)
}

pub fn db_path(docset: &str) -> PathBuf {
    cache_root().join("db").join(format!("{docset}.lance"))
}

pub fn model_path(model_id: &str) -> PathBuf {
    cache_root().join("models").join(model_id)
}

/// `<cache>/nowdocs/db/<docset>.manifest.json` — manifest alongside the lance table.
pub fn manifest_path(docset: &str) -> PathBuf {
    cache_root()
        .join("db")
        .join(format!("{docset}.manifest.json"))
}

/// `<cache>/nowdocs/db/<docset>.license.txt` — verbatim upstream LICENSE text
/// stashed at ingest time. `nowdocs share` copies this into the bundle so the
/// derived work carries the source license (MIT/Apache notice retention,
/// CC-BY-4.0 attribution). Absent when the source had no LICENSE file.
pub fn license_text_path(docset: &str) -> PathBuf {
    cache_root()
        .join("db")
        .join(format!("{docset}.license.txt"))
}

/// `<cache>/nowdocs/staging/` — root for staging directories during install/update.
pub fn staging_root() -> PathBuf {
    cache_root().join("staging")
}

/// Check if a path is under the cache root (for security validation).
pub fn is_under_cache_root(path: &std::path::Path) -> bool {
    let root = cache_root();
    path.starts_with(&root)
}

/// List all staging directories (for stale staging detection).
pub fn list_staging_dirs() -> anyhow::Result<Vec<PathBuf>> {
    let staging_root = staging_root();
    let mut dirs = Vec::new();
    if staging_root.exists() {
        for entry in std::fs::read_dir(&staging_root)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                dirs.push(entry.path());
            }
        }
    }
    Ok(dirs)
}

#[derive(Debug, Serialize)]
pub struct CacheStatus {
    pub cache_root: String,
    pub total_bytes: u64,
    pub db_bytes: u64,
    pub manifests_bytes: u64,
    pub models_bytes: u64,
    pub staging_bytes: u64,
    pub rollback_bytes: u64,
    pub installed_docsets: usize,
    pub staging_count: usize,
}

#[derive(Debug)]
pub struct CleanStagingResult {
    pub removed: Vec<PathBuf>,
    pub skipped: Vec<PathBuf>,
}

pub fn cache_status() -> Result<CacheStatus> {
    let root = cache_root();
    let db_root = root.join("db");
    let models_root = root.join("models");
    let staging = staging_root();
    let rollback = root.join("rollback");

    let db_bytes = dir_size(&db_root)?;
    let manifests_bytes = manifest_bytes(&db_root)?;
    let models_bytes = dir_size(&models_root)?;
    let staging_bytes = dir_size(&staging)?;
    let rollback_bytes = dir_size(&rollback)?;
    let installed_docsets = installed_docset_count(&db_root)?;
    let staging_count = list_staging_dirs().unwrap_or_default().len();

    Ok(CacheStatus {
        cache_root: root.display().to_string(),
        total_bytes: db_bytes + models_bytes + staging_bytes + rollback_bytes,
        db_bytes,
        manifests_bytes,
        models_bytes,
        staging_bytes,
        rollback_bytes,
        installed_docsets,
        staging_count,
    })
}

pub fn clean_staging_older_than(older_than: Duration) -> Result<CleanStagingResult> {
    let mut result = CleanStagingResult {
        removed: Vec::new(),
        skipped: Vec::new(),
    };
    for path in list_staging_dirs().unwrap_or_default() {
        if !is_nowdocs_staging_dir(&path) || !is_old_enough(&path, older_than)? {
            result.skipped.push(path);
            continue;
        }
        std::fs::remove_dir_all(&path)
            .with_context(|| format!("remove staging directory {}", path.display()))?;
        result.removed.push(path);
    }
    Ok(result)
}

fn dir_size(path: &Path) -> Result<u64> {
    if !path.exists() {
        return Ok(0);
    }
    let mut bytes = 0;
    for entry in std::fs::read_dir(path).with_context(|| format!("read {}", path.display()))? {
        let entry = entry?;
        let meta = entry.metadata()?;
        if meta.is_dir() {
            bytes += dir_size(&entry.path())?;
        } else if meta.is_file() {
            bytes += meta.len();
        }
    }
    Ok(bytes)
}

fn manifest_bytes(db_root: &Path) -> Result<u64> {
    if !db_root.exists() {
        return Ok(0);
    }
    let mut bytes = 0;
    for entry in
        std::fs::read_dir(db_root).with_context(|| format!("read {}", db_root.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file()
            && path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".manifest.json"))
        {
            bytes += entry.metadata()?.len();
        }
    }
    Ok(bytes)
}

fn installed_docset_count(db_root: &Path) -> Result<usize> {
    if !db_root.exists() {
        return Ok(0);
    }
    let mut count = 0;
    for entry in
        std::fs::read_dir(db_root).with_context(|| format!("read {}", db_root.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir()
            && path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".lance"))
        {
            count += 1;
        }
    }
    Ok(count)
}

fn is_old_enough(path: &Path, older_than: Duration) -> Result<bool> {
    let modified = path.metadata()?.modified()?;
    Ok(modified
        .elapsed()
        .map(|age| age >= older_than)
        .unwrap_or(false))
}

fn is_nowdocs_staging_dir(path: &Path) -> bool {
    let staging_root = staging_root();
    if path.parent() != Some(staging_root.as_path()) {
        return false;
    }
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    let mut parts = name.rsplitn(3, '-');
    let timestamp = parts.next();
    let pid = parts.next();
    let docset = parts.next();
    matches!(
        (docset, pid, timestamp),
        (Some(d), Some(p), Some(t))
            if !d.is_empty()
                && p.chars().all(|c| c.is_ascii_digit())
                && t.chars().all(|c| c.is_ascii_digit())
    )
}

/// Create a unique staging path for a docset install/update.
/// Format: `<staging_root>/<docset>-<pid>-<timestamp>/`
pub fn new_staging_path(docset: &str) -> PathBuf {
    let pid = std::process::id();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    staging_root().join(format!("{}-{}-{}", docset, pid, timestamp))
}

/// `<cache>/nowdocs/rollback/<docset>-<pid>-<timestamp>/` — rollback path for active replacement.
pub fn rollback_path(docset: &str) -> PathBuf {
    let pid = std::process::id();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    cache_root()
        .join("rollback")
        .join(format!("{}-{}-{}", docset, pid, timestamp))
}

/// Create the cache tree if absent and gate on the layout version.
///
/// - First run (no `.layout_version`): create `db/` + `models/`, write version.
/// - Existing matching version: no-op success.
/// - Existing mismatched version: `Err` with a `nowdocs migrate` hint.
pub fn ensure_layout() -> anyhow::Result<()> {
    let root = cache_root();
    let version_file = root.join(LAYOUT_VERSION_FILE);

    if version_file.is_file() {
        let on_disk = std::fs::read_to_string(&version_file)?;
        let on_disk: u32 = on_disk.trim().parse().map_err(|_| {
            anyhow::anyhow!(
                "corrupt .layout_version (not a number) at {} — remove the cache directory and re-run `nowdocs` to reinitialize",
                version_file.display()
            )
        })?;
        if on_disk != CACHE_LAYOUT_VERSION {
            anyhow::bail!(
                "cache layout version mismatch: on disk {} but nowdocs expects {} — \
                 the cache at {} was created by an incompatible nowdocs build; \
                 remove that directory and re-run `nowdocs` to reinitialize (installed docsets will be lost)",
                on_disk,
                CACHE_LAYOUT_VERSION,
                root.display()
            );
        }
    } else {
        std::fs::create_dir_all(root.join("db"))?;
        std::fs::create_dir_all(root.join("models"))?;
        std::fs::create_dir_all(root.join("staging"))?;
        std::fs::create_dir_all(root.join("rollback"))?;
        std::fs::write(&version_file, CACHE_LAYOUT_VERSION.to_string())?;
    }
    Ok(())
}

/// Metadata about an installed docset.
#[derive(Debug, Serialize)]
pub struct InstalledDocset {
    pub name: String,
    pub version: String,
    pub chunks: String,
    pub license: String,
    pub status: String,
}

/// Read manifest metadata for a docset, returning (version, chunk_count, license).
pub fn read_docset_meta(docset: &str) -> (String, String, String) {
    let manifest_path = manifest_path(docset);
    if let Ok(raw) = std::fs::read_to_string(&manifest_path) {
        if let Ok(m) = crate::manifest::parse_manifest(&raw) {
            return (
                m.doc_version,
                m.source.chunk_count.to_string(),
                m.legal.license,
            );
        }
    }
    ("?".into(), "?".into(), "?".into())
}

/// Check if a docset manifest parses and validates successfully.
pub fn is_docset_healthy(docset: &str) -> bool {
    let manifest_path = manifest_path(docset);
    if let Ok(raw) = std::fs::read_to_string(&manifest_path) {
        if let Ok(m) = crate::manifest::parse_manifest(&raw) {
            return crate::manifest::validate(&m).is_ok();
        }
    }
    false
}

/// List installed docsets with metadata.
pub fn list_installed() -> std::io::Result<Vec<InstalledDocset>> {
    let db_dir = cache_root().join("db");
    let mut docsets: Vec<InstalledDocset> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&db_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if let Some(stem) = name.strip_suffix(".lance") {
                        let (version, chunks, license) = read_docset_meta(stem);
                        let status = if is_docset_healthy(stem) {
                            "ok"
                        } else if manifest_path(stem).is_file() {
                            "broken"
                        } else {
                            "no-manifest"
                        };
                        docsets.push(InstalledDocset {
                            name: stem.to_string(),
                            version,
                            chunks,
                            license,
                            status: status.to_string(),
                        });
                    }
                }
            }
        }
    }
    docsets.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(docsets)
}
