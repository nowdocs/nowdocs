//! Tests for MCP tool handlers (nowdocs_search / nowdocs_list).

use serde_json::json;
use std::fs;
use std::sync::Mutex;
use tempfile::TempDir;

// Global lock to serialize tests that modify XDG_CACHE_HOME.
static ENV_LOCK: Mutex<()> = Mutex::new(());

// Helper: set XDG_CACHE_HOME to a temp dir and return it (kept alive for the test).
// Uses a unique subdirectory per caller to avoid conflicts.
fn setup_cache(test_name: &str) -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().expect("tempdir");
    let cache_root = tmp.path().join(test_name).join("nowdocs");
    fs::create_dir_all(cache_root.join("db")).expect("create db dir");
    // Set XDG_CACHE_HOME to the test-specific parent so cache::cache_root() picks it up.
    // Note: env::set_var is safe in tests (single-threaded per test binary).
    // We use a unique subdirectory per test to avoid conflicts when tests run in parallel.
    unsafe {
        std::env::set_var("XDG_CACHE_HOME", tmp.path().join(test_name));
    }
    (tmp, cache_root)
}

#[test]
fn test_list_two_docsets() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let (tmp, cache_root) = setup_cache("two_docsets");
    // Create two fake docset directories.
    fs::create_dir_all(cache_root.join("db").join("a.lance")).unwrap();
    fs::create_dir_all(cache_root.join("db").join("b.lance")).unwrap();

    let result = nowdocs::tools::handle_call("nowdocs_list", json!({}));
    // Should be a success result with content array.
    assert!(
        result.get("content").is_some(),
        "expected content array, got: {result:?}"
    );
    let content = result["content"].as_array().unwrap();
    assert!(!content.is_empty(), "content should not be empty");
    // The text should contain both docset names.
    let text = content[0]["text"].as_str().unwrap();
    assert!(text.contains("a"), "text should contain 'a', got: {text:?}");
    assert!(text.contains("b"), "text should contain 'b', got: {text:?}");
    drop(tmp);
}

#[test]
fn test_list_no_docsets() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let (tmp, _cache_root) = setup_cache("no_docsets");
    // db/ exists but is empty.
    let result = nowdocs::tools::handle_call("nowdocs_list", json!({}));
    let content = result["content"].as_array().unwrap();
    let text = content[0]["text"].as_str().unwrap();
    assert!(
        text.contains("no docsets installed"),
        "expected 'no docsets installed', got: {text:?}"
    );
    drop(tmp);
}

#[test]
fn test_search_invalid_docset() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let (_tmp, _) = setup_cache("invalid_docset");
    // docset with path-traversal attempt.
    let result = nowdocs::tools::handle_call(
        "nowdocs_search",
        json!({"query": "hello", "docset": "../bad"}),
    );
    // Should be an error.
    assert!(
        result.get("code").is_some(),
        "expected JSON-RPC error, got: {result:?}"
    );
    assert_eq!(result["code"].as_i64().unwrap(), -32602);
}

#[test]
fn test_search_rejects_empty_query() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let (_tmp, _) = setup_cache("empty_query");
    let result =
        nowdocs::tools::handle_call("nowdocs_search", json!({"query": "", "docset": "nextjs"}));
    assert!(
        result.get("code").is_some(),
        "expected JSON-RPC error for empty query, got: {result:?}"
    );
    assert_eq!(result["code"].as_i64().unwrap(), -32602);
}

#[test]
fn test_unknown_tool() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let (_tmp, _) = setup_cache("unknown_tool");
    let result = nowdocs::tools::handle_call("nonexistent_tool", json!({}));
    assert!(
        result.get("code").is_some(),
        "expected JSON-RPC error for unknown tool, got: {result:?}"
    );
    assert_eq!(result["code"].as_i64().unwrap(), -32601);
    let msg = result["message"].as_str().unwrap();
    assert!(
        msg.contains("nonexistent_tool"),
        "error should name the tool: {msg:?}"
    );
}

// E2E test — requires real embedder + ingested docset. Run manually.
#[test]
#[ignore]
fn test_search_end_to_end() {
    // This test requires:
    // 1. A real docset ingested via ingest::ingest_dir
    // 2. The candle+jina embedder available
    // Skipped by default; run with: cargo test test_search_end_to_end -- --ignored
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let (_tmp, _) = setup_cache("e2e");

    // TODO: ingest a fixture docset then search it.
    // For now, just verify handle_call returns something for a valid docset
    // (will fail with "manifest not found" which is expected without ingest).
    let result = nowdocs::tools::handle_call(
        "nowdocs_search",
        json!({"query": "how to use router", "docset": "nextjs"}),
    );
    // Either success or a retrieval error (not a validation error).
    if result.get("code").is_some() {
        let code = result["code"].as_i64().unwrap();
        // Should NOT be a validation error (-32602) — those are caught above.
        assert_ne!(code, -32602, "should not be a validation error: {result:?}");
    }
}
