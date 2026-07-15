//! Unified Update Service for binary-version reminders.
//!
//! Checks only binary releases (not registry/docset updates) after successful
//! completion of eligible CLI commands. The service is designed to be safe and
//! invisible: every failure path (network, parse, I/O, lock contention, cache
//! corruption) returns `None` and never changes the primary command's exit code.
//!
//! # Behavior
//! - Eligible commands: `install`, `update`, `ensure`, `registry`, `smoke`,
//!   `doctor`. Checked only after successful completion; never on `serve`,
//!   `--help`, `--version`, or primary-command failure.
//! - Fetches GitHub's official latest-release metadata with an 800 ms timeout.
//!   Rejects draft and prerelease responses. Silent failures on any error.
//! - Cache at `cache::cache_root()/update-cache.json`. Freshness is 24 hours for
//!   the same running binary version. A failed attempt still advances the
//!   attempt timestamp so repeated CLI invocations don't retry during an outage.
//! - `NOWDOCS_UPDATE_CHECK=0`: no network check and no cached reminder.
//! - One reminder per discovered newer version (package-manager-neutral text).
//! - `serve` is cache-only: it may claim a fresh unnotified reminder and write
//!   it to stderr exactly once; it never initiates a request.
//! - Cross-process dedup via `fs2` advisory lock on the cache file (same
//!   primitive used by the automation operation lock and registry install lock).
//!   `fs2` maps to `flock(2)` on Unix and `LockFileEx` on Windows - both are
//!   process-lifetime and auto-released on crash, satisfying the safe
//!   cross-platform cache/lock requirement without a new dependency.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use crate::cache;

/// Cache freshness window for the same running binary version.
const FRESHNESS_SECS: u64 = 24 * 60 * 60; // 24 hours

/// HTTP timeout for the GitHub latest-release metadata fetch.
const FETCH_TIMEOUT: Duration = Duration::from_millis(800);

/// GitHub latest-release API endpoint (redirects to the latest non-prerelease,
/// non-draft release tag).
const LATEST_RELEASE_URL: &str = "https://api.github.com/repos/nowdocs/nowdocs/releases/latest";

/// The minimum HTTP API User-Agent GitHub requires. Contains only the product
/// name and version - no user-specific or product-specific identifiers.
const USER_AGENT: &str = concat!("nowdocs/", env!("CARGO_PKG_VERSION"),);

/// Cache file name under the cache root.
const CACHE_FILE_NAME: &str = "update-cache.json";

/// Lock file name (sibling to the cache file).
const LOCK_FILE_NAME: &str = "update-cache.lock";

/// On-disk cache schema version. Bump if the cache structure changes.
const SCHEMA_VERSION: u32 = 1;

/// The on-disk update cache record.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UpdateCache {
    schema_version: u32,
    /// The running binary version that last wrote this cache. When the current
    /// binary differs, the cache is treated as stale (forces a re-check after
    /// an upgrade).
    running_version: String,
    /// Wall-clock seconds of the last fetch attempt (success or failure).
    /// Used to throttle retries during outages.
    last_attempt_secs: u64,
    /// Wall-clock seconds of the last successful fetch. 0 = never succeeded.
    last_success_secs: u64,
    /// The latest non-prerelease version discovered, or null if unknown.
    latest_version: Option<String>,
    /// The version the user has already been notified about, or null.
    /// Prevents re-notifying for the same newer version.
    notified_version: Option<String>,
}

impl Default for UpdateCache {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            running_version: env!("CARGO_PKG_VERSION").to_string(),
            last_attempt_secs: 0,
            last_success_secs: 0,
            latest_version: None,
            notified_version: None,
        }
    }
}

// ===========================================================================
// Public API
// ===========================================================================

/// Path to the update cache file: `<cache>/nowdocs/update-cache.json`.
pub fn cache_path() -> PathBuf {
    cache::cache_root().join(CACHE_FILE_NAME)
}

/// Path to the update cache lock file: `<cache>/nowdocs/update-cache.lock`.
fn lock_path() -> PathBuf {
    cache::cache_root().join(LOCK_FILE_NAME)
}

/// The exact package-manager-neutral reminder text for a newer version.
pub fn reminder_text(version: &str) -> String {
    format!(
        "A newer version of nowdocs is available ({version}).\n\
         Update using the package manager you used to install nowdocs.\n\
         https://github.com/nowdocs/nowdocs/releases/latest"
    )
}

/// A reusable update-check service. Owns all update-check logic for one
/// invocation of an eligible CLI command.
pub struct UpdateService {
    running_version: String,
}

impl UpdateService {
    /// Create a service for the given running binary version.
    pub fn new(running_version: &str) -> Result<Self> {
        Ok(Self {
            running_version: running_version.to_string(),
        })
    }

    /// Run the update check and return the reminder text if a newer version
    /// should be surfaced, or `None` otherwise. All failures are silent and
    /// invisible: this method never returns an error that a caller needs to
    /// act on (the `Result` is only for structural correctness).
    ///
    /// Called by eligible CLI commands after successful completion.
    ///
    /// Flow:
    /// 1. Opt-out -> None immediately.
    /// 2. Read cache. If fresh (same running version, attempt < 24h), skip
    ///    fetch but still claim the reminder under lock if one is pending.
    /// 3. If stale, acquire lock, re-check, fetch, update cache.
    /// 4. A reminder is only returned when we hold the lock AND we mark
    ///    `notified_version` atomically, so a concurrent process that also
    ///    reads the same `latest_version` will see it as already notified.
    pub fn check_and_notify(&self) -> Result<Option<String>> {
        // Opt-out: no check, no reminder.
        if is_opted_out() {
            return Ok(None);
        }

        let cache = read_cache();

        if !cache_needs_fetch(&cache, &self.running_version) {
            // Cache is fresh. If there's an unnotified newer version, claim it
            // under the lock so only one process notifies. If we can't get the
            // lock, another process is handling it - return None.
            return Ok(self.claim_reminder_under_lock(&cache));
        }

        // Cache is stale: try to acquire the lock and fetch.
        let Some(_lock) = acquire_lock() else {
            // Another process holds the lock; don't fetch or notify.
            return Ok(None);
        };

        // Re-read after acquiring the lock (another process may have updated).
        let cache = read_cache();
        if !cache_needs_fetch(&cache, &self.running_version) {
            // Another process already refreshed the cache while we waited.
            // Claim the reminder if one is pending.
            return Ok(self.claim_reminder_locked(&cache));
        }

        // Attempt the fetch. On failure, advance the attempt timestamp and
        // return None (no reminder).
        let now = now_unix_secs();
        match fetch_latest_release() {
            Ok(Some(version)) => {
                let mut updated = cache.clone();
                updated.running_version = self.running_version.clone();
                updated.last_attempt_secs = now;
                updated.last_success_secs = now;
                updated.latest_version = Some(version.clone());
                // Claim the reminder: if newer and not yet notified, mark it.
                let reminder = compute_reminder(&updated, &self.running_version);
                if reminder.is_some() {
                    updated.notified_version = Some(version);
                }
                if write_cache(&updated).is_ok() {
                    Ok(reminder)
                } else {
                    Ok(None)
                }
            }
            Ok(None) => {
                // No newer version or rejected (draft/prerelease/parse).
                let mut updated = cache.clone();
                updated.running_version = self.running_version.clone();
                updated.last_attempt_secs = now;
                if updated.latest_version.is_some() {
                    updated.last_success_secs = now;
                }
                if write_cache(&updated).is_ok() {
                    Ok(self.claim_reminder_locked(&updated))
                } else {
                    Ok(None)
                }
            }
            Err(_) => {
                // Network/timeout/parse failure: advance attempt timestamp so
                // repeated invocations don't retry during an outage.
                let mut updated = cache.clone();
                updated.running_version = self.running_version.clone();
                updated.last_attempt_secs = now;
                let _write_result = write_cache(&updated);
                Ok(None)
            }
        }
    }

    /// Claim a pending reminder from a fresh cache. Acquires the lock, marks
    /// `notified_version`, and returns the reminder text. Returns None if the
    /// lock can't be acquired or there's no pending reminder.
    fn claim_reminder_under_lock(&self, _cache: &UpdateCache) -> Option<String> {
        let _lock = acquire_lock()?;
        // Re-read under lock in case another process already claimed.
        let cache = read_cache();
        self.claim_reminder_locked(&cache)
    }

    /// Claim a pending reminder when the lock is already held. Marks
    /// `notified_version` and returns the reminder text, or None if there's
    /// no pending newer version.
    fn claim_reminder_locked(&self, cache: &UpdateCache) -> Option<String> {
        let reminder = compute_reminder(cache, &self.running_version);
        if let Some(reminder) = reminder {
            let mut updated = cache.clone();
            if let Some(ref v) = updated.latest_version {
                updated.notified_version = Some(v.clone());
            }
            return write_cache(&updated).ok().map(|_| reminder);
        }
        None
    }
}

/// Claim a cached unnotified reminder for `serve` without any network request.
///
/// After cache layout initialization, `nowdocs serve` calls this to surface a
/// previously-discovered newer version exactly once on stderr. It never
/// initiates a network request. Returns the reminder text if a fresh
/// unnotified newer version exists, or `None` otherwise.
pub fn serve_claim_cached_reminder(running_version: &str) -> Result<Option<String>> {
    if is_opted_out() {
        return Ok(None);
    }

    let Some(_lock) = acquire_lock() else {
        return Ok(None);
    };

    let cache = read_cache();
    let reminder = compute_reminder(&cache, running_version);
    if let Some(reminder) = reminder {
        // Mark as notified so a second serve process doesn't re-notify.
        let mut updated = cache;
        if let Some(ref v) = updated.latest_version {
            updated.notified_version = Some(v.clone());
        }
        if write_cache(&updated).is_err() {
            return Ok(None);
        }
        return Ok(Some(reminder));
    }
    Ok(None)
}

// ===========================================================================
// Internal: cache logic
// ===========================================================================

/// True if `NOWDOCS_UPDATE_CHECK=0` is set.
fn is_opted_out() -> bool {
    matches!(std::env::var("NOWDOCS_UPDATE_CHECK").as_deref(), Ok("0"))
}

/// Read the cache from disk. Returns a default cache on any error (missing
/// file, malformed JSON, schema mismatch).
fn read_cache() -> UpdateCache {
    let path = cache_path();
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return UpdateCache::default();
    };
    match serde_json::from_str::<UpdateCache>(&raw) {
        Ok(c) if c.schema_version == SCHEMA_VERSION => c,
        _ => UpdateCache::default(),
    }
}

/// Write the cache atomically (unique same-directory tempfile + replacement)
/// so a crash mid-write never leaves a corrupt cache. The caller decides
/// whether a persistence failure suppresses a notification.
fn write_cache(cache: &UpdateCache) -> Result<()> {
    let path = cache_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string(cache)?;
    let parent = path.parent().expect("cache path has a parent");
    let mut tmp = NamedTempFile::new_in(parent)?;
    tmp.write_all(json.as_bytes())?;
    tmp.flush()?;
    tmp.as_file().sync_all()?;
    tmp.persist(&path)
        .map_err(|e| anyhow::Error::new(e.error))?;
    Ok(())
}

/// Determine whether a network fetch is needed based on cache freshness.
///
/// A fetch is needed when:
/// - The running version changed (upgrade detected), OR
/// - The last attempt is older than 24 hours.
fn cache_needs_fetch(cache: &UpdateCache, running_version: &str) -> bool {
    if cache.running_version != running_version {
        return true;
    }
    let now = now_unix_secs();
    now.saturating_sub(cache.last_attempt_secs) >= FRESHNESS_SECS
}

/// Compute the reminder from a cache record, if a newer unnotified version
/// exists.
fn compute_reminder(cache: &UpdateCache, running_version: &str) -> Option<String> {
    let latest = cache.latest_version.as_ref()?;
    // Already notified for this version.
    if cache.notified_version.as_deref() == Some(latest.as_str()) {
        return None;
    }
    if is_newer(latest, running_version) {
        Some(reminder_text(latest))
    } else {
        None
    }
}

// ===========================================================================
// Internal: locking (cross-platform via fs2)
// ===========================================================================

/// A guard holding the advisory lock on the update cache. Dropping releases
/// the lock; the lock file is never removed (avoids pathname races).
pub struct UpdateLock {
    _file: File,
}

/// Try to acquire the exclusive advisory lock. Returns `None` on contention
/// or any error (safe and invisible). The lock file is created if absent.
fn acquire_lock() -> Option<UpdateLock> {
    let path = lock_path();
    if let Some(parent) = path.parent() {
        if std::fs::create_dir_all(parent).is_err() {
            return None;
        }
    }
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .ok()?;
    file.try_lock_exclusive().ok()?;
    Some(UpdateLock { _file: file })
}

// ===========================================================================
// Internal: HTTP fetch (in-process ureq client)
// ===========================================================================

/// GitHub latest-release API response (only the fields we need).
#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    draft: bool,
    prerelease: bool,
}

/// Fetch the latest non-prerelease, non-draft release version from GitHub.
/// Returns `Ok(Some(version))` if a newer release was found, `Ok(None)` if
/// the response was rejected (draft/prerelease/parse/no tag), or `Err` on
/// network/timeout failure.
fn fetch_latest_release() -> Result<Option<String>> {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(FETCH_TIMEOUT))
        .https_only(true)
        .build();
    let agent = ureq::Agent::new_with_config(config);
    let mut response = agent
        .get(LATEST_RELEASE_URL)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/vnd.github+json")
        .call()
        .context("fetch release metadata")?;
    let body = response
        .body_mut()
        .read_to_string()
        .context("read release metadata response")?;
    let release: GithubRelease =
        serde_json::from_str(&body).context("parse release metadata JSON")?;
    Ok(parse_release_inner(&release))
}

/// Parse a release JSON value, returning the version string if it is a stable
/// (non-draft, non-prerelease) release, or None otherwise. Strips an optional
/// leading `v` prefix and rejects prerelease semver tags.
fn parse_release_inner(release: &GithubRelease) -> Option<String> {
    if release.draft || release.prerelease {
        return None;
    }
    let tag = release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&release.tag_name);
    // Reject prerelease semver (e.g. "0.2.0-rc1") even if the GitHub flag is
    // false. `semver::Version` parses prerelease tags, so we check via
    // `Version::pre` being non-empty.
    let Ok(ver) = semver::Version::parse(tag) else {
        return None;
    };
    if !ver.pre.is_empty() {
        return None;
    }
    Some(ver.to_string())
}

// ===========================================================================
// Internal: semver comparison
// ===========================================================================

/// True if `candidate` is strictly newer than `current` using semver ordering.
fn is_newer(candidate: &str, current: &str) -> bool {
    let Ok(c) = semver::Version::parse(candidate.strip_prefix('v').unwrap_or(candidate)) else {
        return false;
    };
    let Ok(cur) = semver::Version::parse(current.strip_prefix('v').unwrap_or(current)) else {
        return false;
    };
    c > cur
}

/// Current wall-clock time as whole seconds since the Unix epoch.
fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ===========================================================================
// Internal: eligibility
// ===========================================================================

/// Commands that trigger an update check after successful completion.
const ELIGIBLE_COMMANDS: &[&str] = &["install", "update", "ensure", "registry", "smoke", "doctor"];

/// True if a CLI command name is eligible for an update check.
fn is_eligible_command(cmd: &str) -> bool {
    ELIGIBLE_COMMANDS.contains(&cmd)
}

// ===========================================================================
// Test-only utilities (compiled in all builds; exposed under a test_util
// module so integration tests can exercise pure helpers without network).
// ===========================================================================

/// Test-only utilities for exercising pure helpers in integration tests.
pub mod test_util {
    /// Current wall-clock time as whole seconds since the Unix epoch.
    pub fn now_unix_secs() -> u64 {
        super::now_unix_secs()
    }

    /// True if `candidate` is strictly newer than `current` using semver.
    pub fn is_newer(candidate: &str, current: &str) -> bool {
        super::is_newer(candidate, current)
    }

    /// Parse a GitHub release JSON value (tag_name, draft, prerelease),
    /// returning the version string if stable, or None.
    pub fn parse_release(resp: &serde_json::Value) -> Option<String> {
        let release: super::GithubRelease = serde_json::from_value(resp.clone()).ok()?;
        super::parse_release_inner(&release)
    }

    /// True if a CLI command name is eligible for an update check.
    pub fn is_eligible_command(cmd: &str) -> bool {
        super::is_eligible_command(cmd)
    }

    /// Acquire the update cache lock. Returns the guard or None on failure.
    /// Used by tests to simulate a concurrent lock holder.
    pub fn acquire_lock() -> Option<super::UpdateLock> {
        super::acquire_lock()
    }
}

use anyhow::Context as _;
