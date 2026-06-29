use std::path::PathBuf;

pub const CACHE_LAYOUT_VERSION: u32 = 1;

pub fn cache_root() -> PathBuf {
    todo!("1e")
}

pub fn db_path(_docset: &str) -> PathBuf {
    todo!("1e")
}

pub fn model_path(_model_id: &str) -> PathBuf {
    todo!("1e")
}

pub fn ensure_layout() -> anyhow::Result<()> {
    todo!("1e")
}
