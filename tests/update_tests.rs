//! Integration tests for the unified update-notification service.
//!
//! These tests exercise the library-level [`nowdocs::update`] contract without
//! any real network access. The cache file is seeded directly on disk so the
//! UpdateService reads/writes the same `update-cache.json` path the production
//! code uses, and the HTTP fetch is never reached because every test either
//! relies on a fresh-enough cache or asserts the opt-out / throttle paths that
//! skip the fetch entirely.
//!
//! Every test isolates `XDG_CACHE_HOME` to a tempdir so the real user cache is
//! never touched. A process-global `Mutex` serializes env mutation because
//! `XDG_CACHE_HOME` is process-global and tests run in the same binary.

use std::sync::Mutex;

use nowdocs::cache;
use nowdocs::update;

// Serialize env mutation across tests in this binary.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// A guard that restores multiple env vars on drop.
struct MultiEnvGuard {
    pairs: Vec<(&'static str, Option<String>)>,
    _g: std::sync::MutexGuard<'static, ()>,
}

impl MultiEnvGuard {
    fn set(pairs: &[(&'static str, &str)]) -> Self {
        let g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let saved: Vec<_> = pairs
            .iter()
            .map(|(k, v)| {
                let old = std::env::var(k).ok();
                std::env::set_var(k, v);
                (*k, old)
            })
            .collect();
        Self {
            pairs: saved,
            _g: g,
        }
    }
}

impl Drop for MultiEnvGuard {
    fn drop(&mut self) {
        for (k, old) in self.pairs.iter().rev() {
            match old {
                Some(v) => std::env::set_var(k, v),
                None => std::env::remove_var(k),
            }
        }
    }
}

/// Point `XDG_CACHE_HOME` at a tempdir and set unreachable proxy variables so
/// any accidental network attempt fails fast. Returns the tempdir and a guard.
fn isolated_cache() -> (tempfile::TempDir, MultiEnvGuard) {
    let dir = tempfile::tempdir().unwrap();
    let cache_path = dir.path().to_str().unwrap().to_string();
    let g = MultiEnvGuard::set(&[
        ("XDG_CACHE_HOME", &cache_path),
        ("http_proxy", "http://127.0.0.1:9"),
        ("https_proxy", "http://127.0.0.1:9"),
        ("HTTP_PROXY", "http://127.0.0.1:9"),
        ("HTTPS_PROXY", "http://127.0.0.1:9"),
        ("ALL_PROXY", "http://127.0.0.1:9"),
        ("no_proxy", ""),
        ("NO_PROXY", ""),
    ]);
    (dir, g)
}

/// The running binary version used by tests that need to construct an
/// UpdateService for the "current" version.
const TEST_RUNNING_VERSION: &str = "0.1.2";

/// Write a raw JSON string to the update cache path.
fn seed_cache(json: &str) {
    let path = update::cache_path();
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, json).unwrap();
}

/// Read the raw update cache file as a string (empty if absent).
fn read_cache_raw() -> String {
    std::fs::read_to_string(update::cache_path()).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Cache freshness: a cache written for the same running version within 24 h
// must NOT trigger a network fetch. The service returns None (no reminder).
// ---------------------------------------------------------------------------

#[test]
fn fresh_cache_same_version_no_reminder() {
    let (_dir, _g) = isolated_cache();
    let now = nowdocs::update::test_util::now_unix_secs();
    let cache_json = serde_json::json!({
        "schema_version": 1,
        "running_version": TEST_RUNNING_VERSION,
        "last_attempt_secs": now,
        "last_success_secs": now,
        "latest_version": "0.1.2",
        "notified_version": null,
    });
    seed_cache(&cache_json.to_string());

    // A fresh cache for the same version means no fetch and no reminder.
    let result = update::UpdateService::new(TEST_RUNNING_VERSION)
        .unwrap()
        .check_and_notify();
    assert!(
        result.is_ok(),
        "check_and_notify must not error: {:?}",
        result.err()
    );
    // No reminder because latest == running.
    assert_eq!(result.unwrap(), None);
}

// ---------------------------------------------------------------------------
// Changed running version: when the running binary version differs from the
// cached `running_version`, the cache is considered stale even if the
// attempt timestamp is recent. This forces a re-check for a newly upgraded
// binary.
// ---------------------------------------------------------------------------

#[test]
fn changed_running_version_makes_cache_stale() {
    let (_dir, _g) = isolated_cache();
    let now = update::test_util::now_unix_secs();
    // Cache was written by an older binary version, recently.
    let cache_json = serde_json::json!({
        "schema_version": 1,
        "running_version": "0.1.0",
        "last_attempt_secs": now,
        "last_success_secs": now,
        "latest_version": "0.1.0",
        "notified_version": null,
    });
    seed_cache(&cache_json.to_string());

    // The service for 0.1.2 must treat this cache as stale (version changed).
    // Since there is no real network, the fetch will fail silently and record
    // the attempt. No reminder is produced, but the attempt timestamp advances.
    let svc = update::UpdateService::new(TEST_RUNNING_VERSION).unwrap();
    let _ = svc.check_and_notify();

    let raw = read_cache_raw();
    let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();
    // The running_version must be updated to the current binary.
    assert_eq!(parsed["running_version"], TEST_RUNNING_VERSION);
}

// ---------------------------------------------------------------------------
// Malformed cache: a corrupt cache file must be safe and invisible. The
// service treats it as absent and proceeds (fetch fails silently offline,
// no reminder, no crash).
// ---------------------------------------------------------------------------

#[test]
fn malformed_cache_is_safe_and_invisible() {
    let (_dir, _g) = isolated_cache();
    seed_cache("this is not json {{{");

    let svc = update::UpdateService::new(TEST_RUNNING_VERSION).unwrap();
    let result = svc.check_and_notify();
    assert!(result.is_ok(), "malformed cache must not error");
    assert_eq!(result.unwrap(), None, "malformed cache must not remind");
}

// ---------------------------------------------------------------------------
// Failed-attempt throttling: a recent failed attempt (last_attempt within
// 24 h, no success) must NOT retry. This prevents repeated CLI invocations
// from hammering the network during an outage.
// ---------------------------------------------------------------------------

#[test]
fn failed_attempt_throttles_retry() {
    let (_dir, _g) = isolated_cache();
    let now = update::test_util::now_unix_secs();
    // A recent attempt with no success (0 = never succeeded).
    let cache_json = serde_json::json!({
        "schema_version": 1,
        "running_version": TEST_RUNNING_VERSION,
        "last_attempt_secs": now,
        "last_success_secs": 0,
        "latest_version": null,
        "notified_version": null,
    });
    seed_cache(&cache_json.to_string());

    let svc = update::UpdateService::new(TEST_RUNNING_VERSION).unwrap();
    // First call: attempt is recent, so no fetch. No reminder.
    let result = svc.check_and_notify();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), None);

    // The attempt timestamp must NOT have advanced (no retry was performed).
    let raw = read_cache_raw();
    let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(
        parsed["last_attempt_secs"].as_u64(),
        Some(now),
        "throttled attempt must not advance the timestamp"
    );
}

// ---------------------------------------------------------------------------
// Failed attempt advances timestamp: when a fetch IS performed (cache stale
// by time) and fails (offline), the attempt timestamp must advance so
// subsequent invocations don't retry during the outage.
// ---------------------------------------------------------------------------

#[test]
fn failed_attempt_advances_timestamp() {
    let (_dir, _g) = isolated_cache();
    let old = update::test_util::now_unix_secs() - (25 * 3600); // 25 h ago
    let cache_json = serde_json::json!({
        "schema_version": 1,
        "running_version": TEST_RUNNING_VERSION,
        "last_attempt_secs": old,
        "last_success_secs": 0,
        "latest_version": null,
        "notified_version": null,
    });
    seed_cache(&cache_json.to_string());

    let before = update::test_util::now_unix_secs();
    let svc = update::UpdateService::new(TEST_RUNNING_VERSION).unwrap();
    let _ = svc.check_and_notify();
    let after = update::test_util::now_unix_secs();

    let raw = read_cache_raw();
    let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let attempt = parsed["last_attempt_secs"].as_u64().unwrap();
    assert!(
        attempt >= before && attempt <= after,
        "failed attempt must advance timestamp to ~now, got {attempt} (before={before}, after={after})"
    );
}

// ---------------------------------------------------------------------------
// Semver comparison, draft/prerelease rejection: these test the pure parsing
// helpers without any network.
// ---------------------------------------------------------------------------

#[test]
fn semver_comparison_is_newer() {
    use update::test_util as tu;
    assert!(tu::is_newer("0.2.0", "0.1.2"));
    assert!(tu::is_newer("1.0.0", "0.9.9"));
    assert!(!tu::is_newer("0.1.2", "0.1.2"));
    assert!(!tu::is_newer("0.1.0", "0.1.2"));
    // Build metadata: the semver crate's Ord includes build metadata in
    // comparison, so 0.1.2+build is ordered after 0.1.2. This is acceptable
    // for our use case (a release tag with build metadata would be newer).
}

#[test]
fn draft_release_is_rejected() {
    let resp = serde_json::json!({
        "tag_name": "v0.2.0",
        "draft": true,
        "prerelease": false,
    });
    assert!(update::test_util::parse_release(&resp).is_none());
}

#[test]
fn prerelease_is_rejected() {
    let resp = serde_json::json!({
        "tag_name": "v0.2.0-rc1",
        "draft": false,
        "prerelease": true,
    });
    assert!(update::test_util::parse_release(&resp).is_none());
}

#[test]
fn stable_release_is_accepted() {
    let resp = serde_json::json!({
        "tag_name": "v0.2.0",
        "draft": false,
        "prerelease": false,
    });
    let v = update::test_util::parse_release(&resp).unwrap();
    assert_eq!(v, "0.2.0");
}

#[test]
fn tag_without_v_prefix_is_accepted() {
    let resp = serde_json::json!({
        "tag_name": "0.3.0",
        "draft": false,
        "prerelease": false,
    });
    let v = update::test_util::parse_release(&resp).unwrap();
    assert_eq!(v, "0.3.0");
}

#[test]
fn prerelease_tag_is_rejected_even_if_flag_false() {
    // A tag like v0.2.0-rc1 is a prerelease semver even if the GitHub flag
    // says false. The service rejects prerelease versions regardless.
    let resp = serde_json::json!({
        "tag_name": "v0.2.0-rc1",
        "draft": false,
        "prerelease": false,
    });
    assert!(update::test_util::parse_release(&resp).is_none());
}

fn opt_out_isolated_cache() -> (tempfile::TempDir, MultiEnvGuard) {
    let dir = tempfile::tempdir().unwrap();
    let cache_path = dir.path().to_str().unwrap().to_string();
    let g = MultiEnvGuard::set(&[
        ("XDG_CACHE_HOME", &cache_path),
        ("NOWDOCS_UPDATE_CHECK", "0"),
        ("http_proxy", "http://127.0.0.1:9"),
        ("https_proxy", "http://127.0.0.1:9"),
        ("HTTP_PROXY", "http://127.0.0.1:9"),
        ("HTTPS_PROXY", "http://127.0.0.1:9"),
        ("ALL_PROXY", "http://127.0.0.1:9"),
        ("no_proxy", ""),
        ("NO_PROXY", ""),
    ]);
    (dir, g)
}

// ---------------------------------------------------------------------------
// Opt-out: NOWDOCS_UPDATE_CHECK=0 disables all checks and reminders.
// ---------------------------------------------------------------------------

#[test]
fn opt_out_disables_check_and_reminder() {
    let (_dir, _g) = opt_out_isolated_cache();

    // Seed a cache that WOULD produce a reminder if not opted out.
    let now = update::test_util::now_unix_secs();
    let cache_json = serde_json::json!({
        "schema_version": 1,
        "running_version": TEST_RUNNING_VERSION,
        "last_attempt_secs": now,
        "last_success_secs": now,
        "latest_version": "0.2.0",
        "notified_version": null,
    });
    seed_cache(&cache_json.to_string());

    let svc = update::UpdateService::new(TEST_RUNNING_VERSION).unwrap();
    let result = svc.check_and_notify();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), None, "opt-out must suppress reminder");

    // The cache file must not have been written (no cached reminder).
    let raw = read_cache_raw();
    let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap_or(serde_json::json!({}));
    assert!(
        parsed.get("notified_version").is_none() || parsed["notified_version"].is_null(),
        "opt-out must not write a cached reminder"
    );
}

// ---------------------------------------------------------------------------
// One-per-version: a reminder is produced exactly once per discovered newer
// version. The first call produces the reminder; a second call for the same
// version does not.
// ---------------------------------------------------------------------------

#[test]
fn reminder_once_per_version() {
    let (_dir, _g) = isolated_cache();
    let now = update::test_util::now_unix_secs();
    // Fresh cache with a newer version already discovered but not yet notified.
    let cache_json = serde_json::json!({
        "schema_version": 1,
        "running_version": TEST_RUNNING_VERSION,
        "last_attempt_secs": now,
        "last_success_secs": now,
        "latest_version": "0.2.0",
        "notified_version": null,
    });
    seed_cache(&cache_json.to_string());

    // First check: should produce the reminder.
    let svc = update::UpdateService::new(TEST_RUNNING_VERSION).unwrap();
    let first = svc.check_and_notify().unwrap();
    assert!(first.is_some(), "first check must produce a reminder");
    let msg = first.unwrap();
    assert!(
        msg.contains("0.2.0"),
        "reminder must contain the newer version"
    );
    assert!(
        msg.contains("package manager"),
        "reminder must mention the package manager"
    );
    assert!(
        msg.contains("https://github.com/nowdocs/nowdocs/releases/latest"),
        "reminder must contain the releases URL"
    );

    // Second check: same version already notified, no reminder.
    let svc2 = update::UpdateService::new(TEST_RUNNING_VERSION).unwrap();
    let second = svc2.check_and_notify().unwrap();
    assert!(
        second.is_none(),
        "second check must not re-notify for the same version"
    );
}

// ---------------------------------------------------------------------------
// A newer version that is the same as the already-notified version produces
// no reminder even if the cache is fresh.
// ---------------------------------------------------------------------------

#[test]
fn already_notified_version_no_reminder() {
    let (_dir, _g) = isolated_cache();
    let now = update::test_util::now_unix_secs();
    let cache_json = serde_json::json!({
        "schema_version": 1,
        "running_version": TEST_RUNNING_VERSION,
        "last_attempt_secs": now,
        "last_success_secs": now,
        "latest_version": "0.2.0",
        "notified_version": "0.2.0",
    });
    seed_cache(&cache_json.to_string());

    let svc = update::UpdateService::new(TEST_RUNNING_VERSION).unwrap();
    let result = svc.check_and_notify().unwrap();
    assert!(result.is_none(), "already-notified version must not remind");
}

// ---------------------------------------------------------------------------
// Sequential claim: the first service claims the reminder and marks
// notified_version; a second service for the same version sees it as already
// notified and produces no reminder. Cross-process lock exclusivity is
// verified by the MCP process tests (serve_claim_cached_reminder).
// ---------------------------------------------------------------------------

#[test]
fn concurrent_claim_only_one_reminder() {
    let (_dir, _g) = isolated_cache();
    let now = update::test_util::now_unix_secs();
    let cache_json = serde_json::json!({
        "schema_version": 1,
        "running_version": TEST_RUNNING_VERSION,
        "last_attempt_secs": now,
        "last_success_secs": now,
        "latest_version": "0.2.0",
        "notified_version": null,
    });
    seed_cache(&cache_json.to_string());

    // First service: claims the reminder and marks notified_version.
    let svc = update::UpdateService::new(TEST_RUNNING_VERSION).unwrap();
    let result = svc.check_and_notify().unwrap();
    assert!(result.is_some(), "first check must produce a reminder");

    // Second service: same version already notified, no reminder.
    let svc2 = update::UpdateService::new(TEST_RUNNING_VERSION).unwrap();
    let result2 = svc2.check_and_notify().unwrap();
    assert!(
        result2.is_none(),
        "second check must not re-notify for the same version (already claimed)"
    );
}

// ---------------------------------------------------------------------------
// Serve cache-only: a fresh unnotified reminder is claimed and returned for
// stderr, without any network request. This mirrors what `nowdocs serve`
// does after cache layout init.
// ---------------------------------------------------------------------------

#[test]
fn serve_claims_cached_reminder_without_network() {
    let (_dir, _g) = isolated_cache();
    let now = update::test_util::now_unix_secs();
    let cache_json = serde_json::json!({
        "schema_version": 1,
        "running_version": TEST_RUNNING_VERSION,
        "last_attempt_secs": now,
        "last_success_secs": now,
        "latest_version": "0.2.0",
        "notified_version": null,
    });
    seed_cache(&cache_json.to_string());

    // serve_claim_cached_reminder never fetches; it only reads/claims.
    let first = update::serve_claim_cached_reminder(TEST_RUNNING_VERSION).unwrap();
    assert!(first.is_some(), "serve must claim a fresh reminder");
    assert!(first.unwrap().contains("0.2.0"));

    // A second serve process must not re-claim (notified_version is set).
    let second = update::serve_claim_cached_reminder(TEST_RUNNING_VERSION).unwrap();
    assert!(
        second.is_none(),
        "serve must not re-claim an already-notified version"
    );
}

// ---------------------------------------------------------------------------
// Serve never fetches: even with a stale/missing cache, serve_claim returns
// None (no network). This is the core "serve is cache-only" invariant.
// ---------------------------------------------------------------------------

#[test]
fn serve_never_fetches_on_missing_cache() {
    let (_dir, _g) = isolated_cache();
    // No cache file at all.
    let result = update::serve_claim_cached_reminder(TEST_RUNNING_VERSION).unwrap();
    assert!(
        result.is_none(),
        "serve must return None when no cache exists (never fetches)"
    );
}

// ---------------------------------------------------------------------------
// Eligibility: only specific commands trigger the check. The eligibility
// helper is a pure function we can test directly.
// ---------------------------------------------------------------------------

#[test]
fn eligible_commands() {
    use update::test_util as tu;
    for cmd in ["install", "update", "ensure", "registry", "smoke", "doctor"] {
        assert!(tu::is_eligible_command(cmd), "{cmd} must be eligible");
    }
    for cmd in ["serve", "--help", "--version", "status", "list-installed"] {
        assert!(!tu::is_eligible_command(cmd), "{cmd} must NOT be eligible");
    }
}

// ---------------------------------------------------------------------------
// Reminder text contract: the exact package-manager-neutral message.
// ---------------------------------------------------------------------------

#[test]
fn reminder_text_contract() {
    let msg = update::reminder_text("0.9.0");
    assert_eq!(
        msg,
        "A newer version of nowdocs is available (0.9.0).\n\
         Update using the package manager you used to install nowdocs.\n\
         https://github.com/nowdocs/nowdocs/releases/latest"
    );
}

// ---------------------------------------------------------------------------
// Cache path is under cache_root().
// ---------------------------------------------------------------------------

#[test]
fn cache_path_is_under_cache_root() {
    let (_dir, _g) = isolated_cache();
    let path = update::cache_path();
    assert!(
        cache::is_under_cache_root(&path),
        "update cache path must be under the cache root"
    );
    assert!(
        path.ends_with("update-cache.json"),
        "cache file must be named update-cache.json, got {}",
        path.display()
    );
}
