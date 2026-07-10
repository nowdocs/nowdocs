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
