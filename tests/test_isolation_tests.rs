//! C0 regression policy: the default test suite stays hermetic.
//!
//! These tests read only repo-owned files through `CARGO_MANIFEST_DIR`. They
//! must never run Cargo recursively, spawn a shell, inspect a real model
//! cache, or read user paths.

use std::path::Path;

/// Real-model tests that need an isolated writable copy of the pinned model
/// cache or a network-prepared cache. They are explicit opt-in (`#[ignore]`)
/// and must stay out of the default suite.
const REAL_MODEL_TESTS: [&str; 5] = [
    "test_embed_dim_is_512",
    "test_embed_semantic_self_consistency",
    "test_load_for_rejects_tampered_sha",
    "test_load_delegates_to_load_for",
    "test_load_for_returns_cached_embedder_on_second_call",
];

/// Hermetic embedder tests that must keep running in the default suite.
const HERMETIC_TESTS: [&str; 3] = [
    "test_no_unsafe_set_var_in_embedder",
    "test_preload_skips_when_model_uncached",
    "test_default_model_cached_requires_all_files",
];

fn repo_file(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

#[test]
fn cargo_config_serializes_default_test_threads() {
    // Legacy tests mutate the process-global cache environment without a
    // shared lock, so the default test process must run single-threaded
    // until they are refactored to path injection.
    let cfg = repo_file(".cargo/config.toml");
    assert!(
        cfg.lines().any(|l| l.trim() == "[env]"),
        ".cargo/config.toml must define an [env] section for the test policy"
    );
    let line = cfg
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with("RUST_TEST_THREADS"))
        .expect(".cargo/config.toml must set RUST_TEST_THREADS");
    assert!(
        line.contains("value = \"1\""),
        "RUST_TEST_THREADS must be pinned to 1, got: {line}"
    );
    assert!(
        line.contains("force = true"),
        "RUST_TEST_THREADS must be forced so an ambient env var cannot re-enable races, got: {line}"
    );
}

struct TestFn {
    name: String,
    ignored: bool,
}

/// Minimal attribute-aware parser: tracks `#[ignore...]` attributes attached
/// to `fn` items in a Rust test source file. Attributes, comments, and blank
/// lines keep the pending state; any other code line resets it.
fn parse_test_functions(src: &str) -> Vec<TestFn> {
    let mut fns = Vec::new();
    let mut pending_ignore = false;
    for line in src.lines() {
        let t = line.trim();
        if t.starts_with("#[ignore") {
            pending_ignore = true;
        } else if t.starts_with("#[") || t.starts_with("//") || t.is_empty() {
            // keep pending attribute state
        } else if let Some(rest) = t.strip_prefix("fn ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                fns.push(TestFn {
                    name,
                    ignored: pending_ignore,
                });
            }
            pending_ignore = false;
        } else {
            pending_ignore = false;
        }
    }
    fns
}

#[test]
fn exactly_five_real_model_tests_are_ignored() {
    let src = repo_file("tests/embedder_tests.rs");
    let fns = parse_test_functions(&src);
    for name in REAL_MODEL_TESTS {
        let f = fns
            .iter()
            .find(|f| f.name == name)
            .unwrap_or_else(|| panic!("{name} must exist in tests/embedder_tests.rs"));
        assert!(
            f.ignored,
            "{name} needs an isolated writable model cache or a network-prepared cache and must be #[ignore]d"
        );
    }
}

#[test]
fn hermetic_embedder_tests_run_by_default() {
    let src = repo_file("tests/embedder_tests.rs");
    let fns = parse_test_functions(&src);
    for name in HERMETIC_TESTS {
        let f = fns
            .iter()
            .find(|f| f.name == name)
            .unwrap_or_else(|| panic!("{name} must exist in tests/embedder_tests.rs"));
        assert!(
            !f.ignored,
            "{name} is hermetic and must keep running in the default suite"
        );
    }
}

#[test]
fn embedder_source_has_no_global_env_mutation() {
    let src = repo_file("src/embedder.rs");
    assert!(
        !src.contains("set_var("),
        "src/embedder.rs must not call std::env::set_var (global process state)"
    );
    assert!(
        !src.contains("HF_HOME"),
        "src/embedder.rs must not reference HF_HOME; the cache dir goes through with_cache_dir"
    );
}
