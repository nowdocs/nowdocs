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
//! by a newer nowdocs is rejected with a migration hint (D15) rather than
//! silently corrupting the cache.

use std::path::PathBuf;

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
    cache_root().join("rollback").join(format!("{}-{}-{}", docset, pid, timestamp))
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
            anyhow::anyhow!("corrupt .layout_version (not a number): run `nowdocs migrate`")
        })?;
        if on_disk != CACHE_LAYOUT_VERSION {
            anyhow::bail!(
                "cache layout version mismatch: on disk {}, nowdocs {} — run `nowdocs migrate`",
                on_disk,
                CACHE_LAYOUT_VERSION
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
