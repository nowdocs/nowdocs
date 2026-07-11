use nowdocs::doctor;

// These tests set XDG_CACHE_HOME and must not run in parallel.
// Run with: cargo test --test doctor_tests -- --test-threads=1

#[test]
fn test_doctor_default_checks_ok() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let output = doctor::run_default_checks();

    // Should have at least some checks
    assert!(!output.checks.is_empty(), "should have at least one check");

    // Status should be ok or warn (not fail) for clean temp dir
    assert!(
        output.status != doctor::Severity::Fail,
        "clean temp cache should not fail"
    );
}

#[test]
fn test_doctor_json_output_parses() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let output = doctor::run_default_checks();
    let json = serde_json::to_string(&output).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Should have status field
    assert!(
        parsed.get("status").is_some(),
        "JSON should have status field"
    );

    // Should have checks array
    assert!(
        parsed.get("checks").is_some(),
        "JSON should have checks field"
    );
    let checks = parsed["checks"].as_array().unwrap();
    assert!(!checks.is_empty(), "checks array should not be empty");

    // Each check should have required fields
    for check in checks {
        assert!(check.get("id").is_some(), "check should have id");
        assert!(
            check.get("severity").is_some(),
            "check should have severity"
        );
        assert!(check.get("message").is_some(), "check should have message");
    }
}

#[test]
fn test_doctor_docset_validates_name() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    // Invalid docset name with path traversal
    let output = doctor::run_docset_checks("../../tmp/victim");

    // Should fail with invalid name
    assert_eq!(output.status, doctor::Severity::Fail);

    // Should have a check about invalid name
    let name_check = output.checks.iter().find(|c| c.id == "docset-name-valid");
    assert!(name_check.is_some(), "should have docset-name-valid check");
    assert_eq!(name_check.unwrap().severity, doctor::Severity::Fail);
}

#[test]
fn test_doctor_docset_missing_manifest() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let output = doctor::run_docset_checks("nonexistent-docset");

    // Should have a check about missing manifest
    let manifest_check = output
        .checks
        .iter()
        .find(|c| c.id == "docset-manifest-exists");
    assert!(
        manifest_check.is_some(),
        "should have docset-manifest-exists check"
    );
    assert_eq!(manifest_check.unwrap().severity, doctor::Severity::Fail);
}

#[test]
fn test_doctor_mcp_check() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let output = doctor::run_mcp_check();

    // MCP check should succeed without network
    assert_eq!(output.status, doctor::Severity::Ok);

    // Should have MCP initialize and tools checks
    let init_check = output.checks.iter().find(|c| c.id == "mcp-initialize");
    assert!(init_check.is_some(), "should have mcp-initialize check");
    assert_eq!(init_check.unwrap().severity, doctor::Severity::Ok);
    let tools_check = output.checks.iter().find(|c| c.id == "mcp-tools");
    assert!(tools_check.is_some(), "should have mcp-tools check");
    assert_eq!(tools_check.unwrap().severity, doctor::Severity::Ok);
    // Tools check should mention the expected tool names
    assert!(
        tools_check.unwrap().message.contains("nowdocs_search"),
        "tools check should mention nowdocs_search"
    );
    assert!(
        tools_check.unwrap().message.contains("nowdocs_list"),
        "tools check should mention nowdocs_list"
    );
}

#[test]
fn test_doctor_repair_non_destructive() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let output = doctor::run_repair();

    // Repair should be wired to staging cleanup and remain non-destructive.
    assert_eq!(output.status, doctor::Severity::Ok);

    let repair_check = output
        .checks
        .iter()
        .find(|c| c.id == "repair-staging-cleanup");
    assert!(
        repair_check.is_some(),
        "should have repair-staging-cleanup check"
    );
    assert_eq!(repair_check.unwrap().severity, doctor::Severity::Ok);
}

#[test]
fn test_doctor_stale_staging_detected() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    // Create a stale staging directory
    let staging_root = nowdocs::cache::staging_root();
    std::fs::create_dir_all(&staging_root).unwrap();
    let stale_dir = staging_root.join("test-stale-123-456");
    std::fs::create_dir(&stale_dir).unwrap();

    let output = doctor::run_default_checks();

    // Should detect stale staging
    let staging_check = output.checks.iter().find(|c| c.id == "stale-staging");
    assert!(staging_check.is_some(), "should detect stale staging");
    assert_eq!(staging_check.unwrap().severity, doctor::Severity::Warn);
}

// M14: `run_model_check` must aggregate its checks instead of hardcoding
// `Severity::Ok`. When the model cache is missing, the overall status must be
// `Warn` (not `Ok`).
#[test]
fn test_run_model_check_missing_model_is_warn() {
    let cache = tempfile::tempdir().unwrap();
    // A temp XDG_CACHE_HOME means the model is NOT cached, so the check yields Warn.
    unsafe { std::env::set_var("XDG_CACHE_HOME", cache.path()) };

    let out = nowdocs::doctor::run_model_check();
    assert_eq!(
        out.status,
        nowdocs::doctor::Severity::Warn,
        "missing model cache must produce Warn status, got: {:?}",
        out.status
    );
}

#[test]
fn test_doctor_default_includes_model_check() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let output = doctor::run_default_checks();
    assert!(
        output.checks.iter().any(|c| c.id == "model-cache-exists"),
        "default checks must include model-cache-exists check"
    );
}

#[test]
fn test_doctor_default_includes_mcp_check() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let output = doctor::run_default_checks();
    assert!(
        output.checks.iter().any(|c| c.id == "mcp-initialize"),
        "default checks must include mcp-initialize check"
    );
    assert!(
        output.checks.iter().any(|c| c.id == "mcp-tools"),
        "default checks must include mcp-tools check"
    );
}

#[test]
fn test_doctor_default_reports_warn_when_model_missing() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let output = doctor::run_default_checks();
    assert_eq!(
        output.status,
        doctor::Severity::Warn,
        "missing model should make default checks report Warn status"
    );
}
