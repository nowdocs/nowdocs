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
        std::fs::write(&version_file, CACHE_LAYOUT_VERSION.to_string())?;
    }
    Ok(())
}
