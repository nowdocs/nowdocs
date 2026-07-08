use nowdocs::cache::{
    cache_root, cache_status, clean_staging_older_than, db_path, ensure_layout, model_path,
    new_staging_path, CACHE_LAYOUT_VERSION,
};
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;

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
    assert!(
        p.ends_with(Path::new("nowdocs/db/nextjs.lance")),
        "got {:?}",
        p
    );
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
    assert!(
        msg.contains("migrate"),
        "err should hint migration, got: {}",
        msg
    );
}

#[test]
fn ensure_layout_idempotent_on_repeat() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", tmp.path().to_str().unwrap());
    ensure_layout().unwrap();
    ensure_layout().unwrap(); // second call on matching layout must succeed
}

#[test]
fn cache_status_reports_installed_docsets_and_staging() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", tmp.path().to_str().unwrap());
    ensure_layout().unwrap();

    std::fs::create_dir_all(db_path("nextjs")).unwrap();
    std::fs::write(cache_root().join("db/nextjs.manifest.json"), "{}").unwrap();
    let staging = new_staging_path("nextjs");
    std::fs::create_dir_all(&staging).unwrap();

    let status = cache_status().unwrap();
    assert_eq!(status.installed_docsets, 1);
    assert_eq!(status.staging_count, 1);
    assert!(status.cache_root.ends_with("nowdocs"));
    assert!(status.db_bytes >= 2);
}

#[test]
fn clean_staging_removes_only_nowdocs_owned_staging_dirs() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", tmp.path().to_str().unwrap());
    ensure_layout().unwrap();

    let owned = new_staging_path("safe-docset");
    let unrelated = cache_root().join("staging").join("do-not-delete");
    let active = db_path("safe-docset");
    std::fs::create_dir_all(&owned).unwrap();
    std::fs::create_dir_all(&unrelated).unwrap();
    std::fs::create_dir_all(&active).unwrap();

    let cleaned = clean_staging_older_than(Duration::from_secs(0)).unwrap();

    assert_eq!(cleaned.removed.len(), 1);
    assert!(!owned.exists(), "owned staging dir should be removed");
    assert!(
        unrelated.exists(),
        "unrelated staging dir must be preserved"
    );
    assert!(active.exists(), "active db path must never be removed");
}

#[test]
fn clean_staging_respects_age_threshold() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", tmp.path().to_str().unwrap());
    ensure_layout().unwrap();

    let owned = new_staging_path("recent-docset");
    std::fs::create_dir_all(&owned).unwrap();

    let cleaned = clean_staging_older_than(Duration::from_secs(60 * 60)).unwrap();

    assert!(cleaned.removed.is_empty());
    assert!(owned.exists(), "recent staging dir should be skipped");
}

#[test]
fn cli_cache_status_json_parses() {
    let tmp = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_nowdocs"))
        .args(["cache", "status", "--json"])
        .current_dir(cwd.path())
        .env("XDG_CACHE_HOME", tmp.path())
        .output()
        .expect("run nowdocs cache status");

    assert!(
        out.status.success(),
        "cache status should exit 0, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(json.get("cache_root").is_some());
    assert!(json.get("installed_docsets").is_some());
    assert!(json.get("staging_count").is_some());
}
