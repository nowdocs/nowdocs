use nowdocs::cache::{cache_root, db_path, ensure_layout, model_path, CACHE_LAYOUT_VERSION};
use std::path::Path;
use std::sync::Mutex;

// dirs::cache_dir() reads XDG_CACHE_HOME / HOME at call time, so tests that
// mutate these env vars must not run concurrently. This lock serializes them.
static ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvGuard {
    key: &'static str,
    old: Option<String>,
    _g: std::sync::MutexGuard<'static, ()>,
}

impl EnvGuard {
    fn set(key: &'static str, val: &str) -> Self {
        let g = ENV_LOCK.lock().unwrap();
        let old = std::env::var(key).ok();
        std::env::set_var(key, val);
        Self { key, old, _g: g }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.old {
            Some(v) => std::env::set_var(self.key, v),
            None => std::env::remove_var(self.key),
        }
    }
}

#[test]
fn cache_root_ends_with_nowdocs() {
    let _g = EnvGuard::set("XDG_CACHE_HOME", "/tmp/nowdocs-fake-cache");
    let root = cache_root();
    assert!(root.ends_with(Path::new("nowdocs")), "got {:?}", root);
}

#[test]
fn db_path_ends_with_docset_lance() {
    let _g = EnvGuard::set("XDG_CACHE_HOME", "/tmp/nowdocs-fake-cache");
    let p = db_path("nextjs");
    assert!(p.ends_with(Path::new("nowdocs/db/nextjs.lance")), "got {:?}", p);
}

#[test]
fn model_path_nests_org_repo() {
    let _g = EnvGuard::set("XDG_CACHE_HOME", "/tmp/nowdocs-fake-cache");
    let p = model_path("jinaai/jina-embeddings-v2-small-en");
    assert!(
        p.ends_with(Path::new("models/jinaai/jina-embeddings-v2-small-en")),
        "got {:?}",
        p
    );
}

#[test]
fn ensure_layout_creates_tree_and_version_file() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", tmp.path().to_str().unwrap());
    ensure_layout().unwrap();
    let root = cache_root();
    assert!(root.join("db").is_dir(), "db/ missing");
    assert!(root.join("models").is_dir(), "models/ missing");
    let v = std::fs::read_to_string(root.join(".layout_version")).unwrap();
    assert_eq!(v.trim(), CACHE_LAYOUT_VERSION.to_string());
}

#[test]
fn ensure_layout_rejects_version_mismatch() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", tmp.path().to_str().unwrap());
    ensure_layout().unwrap();
    // Simulate an on-disk layout from a future version.
    std::fs::write(cache_root().join(".layout_version"), "99").unwrap();
    let err = ensure_layout().unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("migrate"), "err should hint migration, got: {}", msg);
}

#[test]
fn ensure_layout_idempotent_on_repeat() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", tmp.path().to_str().unwrap());
    ensure_layout().unwrap();
    ensure_layout().unwrap(); // second call on matching layout must succeed
}
