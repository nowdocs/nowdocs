//! Tests for the read-only `nowdocs status` inspector (C2 pure inspector).
//!
//! The inspector observes existing nowdocs-owned filesystem metadata only. It
//! must never create files or directories, write a writability probe, load a
//! model, open a Lance store, spawn a process, access the network, read
//! client configuration, clean data, or follow symlinks. These tests lock
//! that behavior against an absent cache, an existing layout, populated
//! docsets, and a symlink-laced automation subtree.

use std::path::Path;
use std::sync::Mutex;

use nowdocs::cache::{self, CacheLayoutState};
use nowdocs::inspect::collect_status;

// cache_root() reads XDG_CACHE_HOME at call time, so tests that mutate the
// process-global cache environment must not run concurrently.
static ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvGuard {
    key: &'static str,
    old: Option<String>,
    _g: std::sync::MutexGuard<'static, ()>,
}

impl EnvGuard {
    fn set(key: &'static str, val: &str) -> Self {
        // A panicking test must not cascade into poisoned-lock failures for
        // the remaining env-mutating tests in this single-threaded binary.
        let g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
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

/// Recursively list `root` as sorted `kind:relative/path` entries without
/// following symlinks, so a test can prove observation changed nothing.
fn snapshot_tree(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let meta = std::fs::symlink_metadata(&path).expect("snapshot metadata");
                let rel = path
                    .strip_prefix(root)
                    .expect("entry under root")
                    .to_string_lossy()
                    .replace('\\', "/");
                if meta.file_type().is_symlink() {
                    out.push(format!("link:{rel}"));
                } else if meta.is_dir() {
                    out.push(format!("dir:{rel}"));
                    stack.push(path);
                } else {
                    out.push(format!("file:{rel}"));
                }
            }
        }
    }
    out.sort();
    out
}

#[cfg(unix)]
fn make_file_symlink(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).expect("create file symlink");
}

#[cfg(windows)]
fn make_file_symlink(target: &Path, link: &Path) {
    std::os::windows::fs::symlink_file(target, link).expect("create file symlink");
}

#[cfg(unix)]
fn make_dir_symlink(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).expect("create dir symlink");
}

#[cfg(windows)]
fn make_dir_symlink(target: &Path, link: &Path) {
    std::os::windows::fs::symlink_dir(target, link).expect("create dir symlink");
}

/// A manifest that parses and passes `manifest::validate` (allowlisted
/// license, current schema), matching the cache_tests M22 fixture.
fn valid_manifest_json(docset: &str) -> String {
    format!(
        r#"{{
            "docset": "{docset}",
            "doc_version": "1.0.0",
            "nowdocs_schema_version": 1,
            "embedder": {{
                "model_id": "jinaai/jina-embeddings-v2-small-en",
                "model_version": "0.1.0",
                "model_revision": "abc123",
                "model_sha256": "deadbeef",
                "vector_dim": 512,
                "engine": "candle",
                "dtype": "f16"
            }},
            "retrieval": {{ "tokenizer": "default", "chunk_size_tokens": 512, "window_tokens": 64 }},
            "source": {{ "entry_url": "https://example.com/docs", "source_url": "https://example.com", "scraped_at": "2026-01-01T00:00:00Z", "chunk_count": 2 }},
            "legal": {{ "license": "MIT", "copyright_holder": "Example", "attribution": "" }},
            "refresh_strategy": {{ "tier": "stable", "auto_days": 30 }}
        }}"#
    )
}

/// Collect every key and string value of a JSON document, so tests can assert
/// that no path, environment value, or secret leaks into status output.
fn collect_json_strings(v: &serde_json::Value, out: &mut Vec<String>) {
    match v {
        serde_json::Value::String(s) => out.push(s.clone()),
        serde_json::Value::Array(items) => {
            for item in items {
                collect_json_strings(item, out);
            }
        }
        serde_json::Value::Object(map) => {
            for (key, value) in map {
                out.push(key.clone());
                collect_json_strings(value, out);
            }
        }
        _ => {}
    }
}

#[test]
fn status_observes_absent_cache_without_creating_it() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", tmp.path().to_str().unwrap());

    let data = collect_status();

    assert_eq!(data.cache.layout, CacheLayoutState::NotInitialized);
    assert_eq!(data.cache.total_bytes, 0);
    assert_eq!(data.cache.installed_docsets, 0);
    assert_eq!(data.cache.staging_count, 0);
    assert!(!data.model.present, "absent cache must report model absent");
    assert!(
        data.docsets.is_empty(),
        "absent cache must report no docsets"
    );
    assert!(!data.automation.storage_present);
    assert_eq!(data.automation.plan_count, 0);
    assert_eq!(data.automation.operation_count, 0);
    assert_eq!(data.automation.rollback_count, 0);
    assert_eq!(data.automation.expired_count, 0);
    assert_eq!(data.automation.total_bytes, 0);
    let expected_tools: Vec<String> = nowdocs::mcp::MCP_TOOL_NAMES
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(data.mcp.tools, expected_tools);

    // The absent root is observed, never created: the isolated XDG root and
    // the cache root both stay absent/empty.
    assert!(
        !cache::cache_root().exists(),
        "status must not create the cache root"
    );
    let mut entries = std::fs::read_dir(tmp.path()).expect("read isolated root");
    assert!(
        entries.next().is_none(),
        "status must create no files or directories under an absent cache root"
    );
}

#[test]
fn status_observes_existing_layout_without_writability_probe() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", tmp.path().to_str().unwrap());
    cache::ensure_layout().expect("create a real layout to observe");

    let before = snapshot_tree(&cache::cache_root());
    let data = collect_status();
    let after = snapshot_tree(&cache::cache_root());

    assert_eq!(data.cache.layout, CacheLayoutState::Ready);
    assert_eq!(
        before, after,
        "status must not create, modify, or delete anything in an existing layout"
    );
    assert!(
        before.iter().all(|entry| !entry.contains(".write_test")),
        "status must not write the legacy doctor writability probe"
    );
}

#[test]
fn status_data_is_deterministic_and_contains_no_paths() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", tmp.path().to_str().unwrap());
    cache::ensure_layout().unwrap();

    // Two docsets that need no Lance store to classify: one store-only, one
    // with a valid manifest alongside its store directory.
    std::fs::create_dir_all(cache::db_path("alpha")).unwrap();
    std::fs::create_dir_all(cache::db_path("beta")).unwrap();
    std::fs::write(cache::manifest_path("beta"), valid_manifest_json("beta")).unwrap();

    let first = collect_status();
    let second = collect_status();
    let first_json = serde_json::to_string(&first).expect("status data serializes");
    let second_json = serde_json::to_string(&second).expect("status data serializes");
    assert_eq!(first_json, second_json, "status data must be deterministic");

    let names: Vec<&str> = first.docsets.iter().map(|d| d.name.as_str()).collect();
    assert_eq!(names, ["alpha", "beta"], "docsets must be lexical by name");
    let states: Vec<&str> = first.docsets.iter().map(|d| d.state.as_str()).collect();
    for state in &states {
        assert!(
            [
                "ok",
                "no-store",
                "no-manifest",
                "schema-mismatch",
                "count-mismatch",
                "not-installed"
            ]
            .contains(state),
            "docset state must use the InstalledDocsetState label, got: {state}"
        );
    }

    // No absolute paths, environment values, or cache internals anywhere in
    // the serialized data — including object keys.
    let root_str = tmp.path().to_string_lossy().to_string();
    let value: serde_json::Value =
        serde_json::from_str(&first_json).expect("status data parses back");
    let mut strings = Vec::new();
    collect_json_strings(&value, &mut strings);
    for s in &strings {
        assert!(
            !s.contains(&root_str),
            "status output must not contain the cache path, got: {s:?}"
        );
        assert!(
            !s.contains("/Users/"),
            "status output must not contain user paths, got: {s:?}"
        );
        assert!(
            !s.contains("XDG_CACHE_HOME"),
            "status output must not contain environment values, got: {s:?}"
        );
        assert_ne!(
            s, "cache_root",
            "status data must not expose the legacy cache_root field"
        );
    }
}

#[test]
fn status_omits_invalid_filesystem_docset_names() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", tmp.path().to_str().unwrap());
    cache::ensure_layout().unwrap();

    // Filesystem entries are untrusted input: only names accepted by the
    // established docset validator may reach agent-facing status output.
    std::fs::create_dir_all(cache::db_path("safe-docset")).unwrap();
    std::fs::create_dir_all(cache::cache_root().join("db/.lance")).unwrap();
    std::fs::create_dir_all(cache::cache_root().join("db/unsafe\ninstruction.lance")).unwrap();

    let data = collect_status();
    let names: Vec<&str> = data
        .docsets
        .iter()
        .map(|docset| docset.name.as_str())
        .collect();
    assert_eq!(names, ["safe-docset"]);
}

#[test]
fn status_automation_observation_is_read_only_and_does_not_follow_symlinks() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", tmp.path().to_str().unwrap());
    cache::ensure_layout().unwrap();

    // Outside content that observation must never traverse or count.
    let outside = tempfile::tempdir().unwrap();
    let secret_file = outside.path().join("secret.txt");
    std::fs::write(&secret_file, vec![b'x'; 1000]).unwrap();
    let outside_dir = outside.path().join("big");
    std::fs::create_dir(&outside_dir).unwrap();
    std::fs::write(outside_dir.join("big.bin"), vec![b'y'; 4096]).unwrap();

    let automation = cache::cache_root().join("automation");
    let plans = automation.join("plans");
    let operations = automation.join("operations");
    let rollback = automation.join("rollback");
    std::fs::create_dir_all(plans.join("nested")).unwrap();
    std::fs::create_dir_all(&operations).unwrap();
    std::fs::create_dir_all(&rollback).unwrap();
    std::fs::write(plans.join("p1.json"), b"12345").unwrap(); // 5 bytes
    std::fs::write(plans.join("p2.json"), b"1234567").unwrap(); // 7 bytes
    std::fs::write(plans.join("nested").join("deep.bin"), b"12345678901").unwrap(); // 11 bytes
    std::fs::write(operations.join("op1.json"), b"123").unwrap(); // 3 bytes
    make_file_symlink(&secret_file, &plans.join("link-to-secret"));
    make_dir_symlink(&outside_dir, &automation.join("link-to-big"));

    let before = snapshot_tree(&cache::cache_root());
    let data = collect_status();
    let after = snapshot_tree(&cache::cache_root());

    assert_eq!(
        before, after,
        "automation observation must be strictly read-only"
    );

    let a = &data.automation;
    assert!(
        a.storage_present,
        "existing automation dir must be observed"
    );
    assert_eq!(
        a.plan_count, 2,
        "only immediate regular files in plans/ count"
    );
    assert_eq!(a.operation_count, 1);
    assert_eq!(a.rollback_count, 0);
    assert_eq!(
        a.expired_count, 0,
        "C2 never parses plans: expired_count is always 0"
    );
    assert_eq!(
        a.total_bytes, 26,
        "total_bytes must sum regular files without following symlinks (5+7+11+3)"
    );
}

// ---- C2-R1: read-only symlink boundary ----
//
// status must never inspect bytes or entries outside its owned non-symlink
// cache tree: a symlinked cache root or symlinked child component contributes
// no bytes, files, docsets, or counts, and the external target stays
// untouched.

#[test]
fn status_does_not_follow_symlinked_cache_root() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", tmp.path().to_str().unwrap());

    // External tree the symlink points at; must never be inspected.
    let outside = tempfile::tempdir().unwrap();
    std::fs::create_dir(outside.path().join("db")).unwrap();
    std::fs::write(
        outside.path().join("db").join("outside.bin"),
        vec![b'z'; 4096],
    )
    .unwrap();
    let before = snapshot_tree(outside.path());

    // <XDG_CACHE_HOME>/nowdocs is a symlink to the external tree.
    make_dir_symlink(outside.path(), &tmp.path().join("nowdocs"));

    let data = collect_status();

    assert_eq!(
        data.cache.layout,
        CacheLayoutState::Unreadable,
        "a symlinked cache root must report layout unreadable"
    );
    assert_eq!(
        data.cache.total_bytes, 0,
        "status must not count bytes behind a symlinked cache root"
    );
    assert_eq!(data.cache.installed_docsets, 0);
    assert_eq!(data.cache.staging_count, 0);
    assert!(
        !data.model.present,
        "status must not inspect a model behind a symlinked cache root"
    );
    assert!(
        data.docsets.is_empty(),
        "status must not list docsets behind a symlinked cache root"
    );
    assert!(!data.automation.storage_present);
    assert_eq!(data.automation.plan_count, 0);
    assert_eq!(data.automation.operation_count, 0);
    assert_eq!(data.automation.rollback_count, 0);
    assert_eq!(data.automation.expired_count, 0);
    assert_eq!(data.automation.total_bytes, 0);

    let after = snapshot_tree(outside.path());
    assert_eq!(
        before, after,
        "the external symlink target must be left completely unchanged"
    );
}

#[test]
fn status_does_not_follow_symlinked_cache_children() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", tmp.path().to_str().unwrap());
    cache::ensure_layout().unwrap();
    let root = cache::cache_root();

    // External populated trees that symlinked children would point at.
    let outside = tempfile::tempdir().unwrap();
    let outside_db = outside.path().join("db");
    std::fs::create_dir_all(outside_db.join("evil.lance")).unwrap();
    std::fs::write(
        outside_db.join("evil.manifest.json"),
        valid_manifest_json("evil"),
    )
    .unwrap();
    std::fs::write(
        outside_db.join("evil.lance").join("data.bin"),
        vec![b'd'; 2048],
    )
    .unwrap();
    let outside_models = outside.path().join("models");
    std::fs::create_dir_all(
        outside_models
            .join("jinaai")
            .join("jina-embeddings-v2-small-en"),
    )
    .unwrap();
    std::fs::write(
        outside_models
            .join("jinaai")
            .join("jina-embeddings-v2-small-en")
            .join("weights.bin"),
        vec![b'w'; 1234],
    )
    .unwrap();
    let outside_staging = outside.path().join("staging");
    std::fs::create_dir_all(outside_staging.join("junk-1-2")).unwrap();
    let outside_automation = outside.path().join("automation");
    std::fs::create_dir_all(outside_automation.join("plans")).unwrap();
    std::fs::write(
        outside_automation.join("plans").join("p1.json"),
        vec![b'p'; 999],
    )
    .unwrap();
    let before = snapshot_tree(outside.path());

    // Replace real child components with symlinks to the external trees.
    std::fs::remove_dir(root.join("db")).unwrap();
    make_dir_symlink(&outside_db, &root.join("db"));
    std::fs::remove_dir(root.join("models")).unwrap();
    make_dir_symlink(&outside_models, &root.join("models"));
    std::fs::remove_dir(root.join("staging")).unwrap();
    make_dir_symlink(&outside_staging, &root.join("staging"));
    make_dir_symlink(&outside_automation, &root.join("automation"));

    let data = collect_status();

    assert_eq!(
        data.cache.layout,
        CacheLayoutState::Ready,
        "the real root with a valid version file still reports ready"
    );
    assert_eq!(
        data.cache.total_bytes, 0,
        "symlinked db/models/staging children must contribute no bytes"
    );
    assert_eq!(data.cache.installed_docsets, 0);
    assert_eq!(data.cache.staging_count, 0);
    assert!(
        !data.model.present,
        "a model behind a symlinked models/ must report absent"
    );
    assert!(
        data.docsets.is_empty(),
        "docsets behind a symlinked db/ must not be listed"
    );
    assert!(
        !data.automation.storage_present,
        "a symlinked automation dir must report storage absent"
    );
    assert_eq!(data.automation.plan_count, 0);
    assert_eq!(data.automation.total_bytes, 0);

    let after = snapshot_tree(outside.path());
    assert_eq!(
        before, after,
        "external symlink targets must be left completely unchanged"
    );
}
