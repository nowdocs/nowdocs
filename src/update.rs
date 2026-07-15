//! Unified Update Service for binary-version and registry-docset reminders.
//!
//! Checks binary releases and registry docset updates after successful
//! completion of eligible CLI commands. The service is designed to be safe and
//! invisible: every failure path (network, parse, I/O, lock contention, cache
//! corruption) returns `None` and never changes the primary command's exit code.
//!
//! # Behavior
//! - Eligible commands: `install`, `update`, `ensure`, `registry`, `smoke`,
//!   `doctor`. Checked only after successful completion; never on `serve`,
//!   `--help`, `--version`, or primary-command failure.
//! - Binary channel: fetches GitHub's official latest-release metadata with an
//!   800 ms timeout. Rejects draft and prerelease responses.
//! - Registry channel: loads valid receipts, fetches the Registry index once
//!   through the bounded reader, compares semver versions.
//! - Cache at `cache::cache_root()/update-cache.json`. Freshness is 24 hours
//!   independently per channel.
//! - `NOWDOCS_UPDATE_CHECK=0`: no network check and no cached reminder.
//! - `serve` is cache-only: it may claim a fresh unnotified reminder and write
//!   it to stderr exactly once; it never initiates a request.
//! - Cross-process dedup via `fs2` advisory lock on the cache file.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use crate::cache;
use crate::registry;

/// Cache freshness window for the same running binary version.
const FRESHNESS_SECS: u64 = 24 * 60 * 60; // 24 hours

/// HTTP timeout for the GitHub latest-release metadata fetch.
const FETCH_TIMEOUT: Duration = Duration::from_millis(800);

/// HTTP timeout for the Registry index fetch during update checks.
const REGISTRY_FETCH_TIMEOUT: Duration = Duration::from_millis(800);

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
const SCHEMA_VERSION: u32 = 2;

// ===========================================================================
// Cache v2 types
// ===========================================================================

/// On-disk update cache (schema v2).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UpdateCacheV2 {
    schema_version: u32,
    #[serde(default)]
    binary: BinaryChannelCache,
    #[serde(default)]
    registry: RegistryChannelCache,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct BinaryChannelCache {
    /// The running binary version that last wrote this cache.
    #[serde(default)]
    running_version: String,
    /// Wall-clock seconds of the last fetch attempt (success or failure).
    #[serde(default)]
    last_attempt_secs: u64,
    /// Wall-clock seconds of the last successful fetch. 0 = never succeeded.
    #[serde(default)]
    last_success_secs: u64,
    /// The latest non-prerelease version discovered, or null if unknown.
    #[serde(default)]
    latest_version: Option<String>,
    /// The version the user has already been notified about, or null.
    #[serde(default)]
    notified_version: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RegistryChannelCache {
    /// Wall-clock seconds of the last fetch attempt (success or failure).
    #[serde(default)]
    last_attempt_secs: u64,
    /// Wall-clock seconds of the last successful fetch. 0 = never succeeded.
    #[serde(default)]
    last_success_secs: u64,
    /// Available docset updates sorted by docset name.
    #[serde(default)]
    available: Vec<DocsetUpdate>,
    /// Stable snapshot string of the most recently announced updates.
    /// Used for deduplication: same snapshot = already notified.
    #[serde(default)]
    notified_snapshot: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct DocsetUpdate {
    docset: String,
    installed_version: String,
    latest_version: String,
}

impl Default for UpdateCacheV2 {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            binary: BinaryChannelCache::default(),
            registry: RegistryChannelCache::default(),
        }
    }
}

// ===========================================================================
// Migration from v1
// ===========================================================================

/// Legacy v1 cache structure for migration.
#[derive(Debug, Clone, Deserialize)]
struct UpdateCacheV1 {
    schema_version: u32,
    running_version: String,
    last_attempt_secs: u64,
    last_success_secs: u64,
    latest_version: Option<String>,
    notified_version: Option<String>,
}

fn migrate_v1_to_v2(v1: UpdateCacheV1) -> UpdateCacheV2 {
    UpdateCacheV2 {
        schema_version: SCHEMA_VERSION,
        binary: BinaryChannelCache {
            running_version: v1.running_version,
            last_attempt_secs: v1.last_attempt_secs,
            last_success_secs: v1.last_success_secs,
            latest_version: v1.latest_version,
            notified_version: v1.notified_version,
        },
        registry: RegistryChannelCache::default(),
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

/// The exact package-manager-neutral reminder text for a newer binary version.
pub fn reminder_text(version: &str) -> String {
    format!(
        "A newer version of nowdocs is available ({version}).\n\
         Update using the package manager you used to install nowdocs.\n\
         https://github.com/nowdocs/nowdocs/releases/latest"
    )
}

/// Render the docset-update reminder section.
fn docset_reminder_section(updates: &[DocsetUpdate]) -> String {
    let mut lines = Vec::new();
    lines.push("Updates available for installed docsets:".to_string());
    for u in updates {
        lines.push(format!(
            "- {}: {} -> {}",
            u.docset, u.installed_version, u.latest_version
        ));
    }
    lines.push(String::new());
    lines.push("Run:".to_string());
    for u in updates {
        lines.push(format!("nowdocs update {}", u.docset));
    }
    lines.join("\n")
}

/// Compute the combined reminder text from binary and registry channels.
fn combined_reminder_text(binary: Option<&str>, docsets: &[DocsetUpdate]) -> Option<String> {
    let binary_section = binary.map(reminder_text);
    let docset_section = if docsets.is_empty() {
        None
    } else {
        Some(docset_reminder_section(docsets))
    };

    match (binary_section, docset_section) {
        (Some(b), Some(d)) => Some(format!("{b}\n\n{d}")),
        (Some(b), None) => Some(b),
        (None, Some(d)) => Some(d),
        (None, None) => None,
    }
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
    pub fn check_and_notify(&self) -> Result<Option<String>> {
        if is_opted_out() {
            return Ok(None);
        }

        let Some(_lock) = acquire_lock() else {
            return Ok(None);
        };

        let mut cache = read_cache();
        let now = now_unix_secs();

        // --- Binary channel ---
        if channel_needs_fetch(
            cache.binary.last_attempt_secs,
            &cache.binary.running_version,
            &self.running_version,
        ) {
            match fetch_latest_release() {
                Ok(Some(version)) => {
                    cache.binary.running_version = self.running_version.clone();
                    cache.binary.last_attempt_secs = now;
                    cache.binary.last_success_secs = now;
                    cache.binary.latest_version = Some(version);
                }
                Ok(None) => {
                    cache.binary.running_version = self.running_version.clone();
                    cache.binary.last_attempt_secs = now;
                    if cache.binary.latest_version.is_some() {
                        cache.binary.last_success_secs = now;
                    }
                }
                Err(_) => {
                    cache.binary.running_version = self.running_version.clone();
                    cache.binary.last_attempt_secs = now;
                }
            }
        }

        // --- Registry channel ---
        if channel_needs_fetch(
            cache.registry.last_attempt_secs,
            "", // registry has no running_version concept
            "",
        ) {
            cache.registry.last_attempt_secs = now;
            match check_registry_updates() {
                Ok(updates) => {
                    cache.registry.last_success_secs = now;
                    cache.registry.available = updates;
                }
                Err(_) => {
                    // Silent failure — leave available as-is.
                }
            }
        }

        // --- Compute and claim reminders ---
        let binary_version = cache
            .binary
            .latest_version
            .as_deref()
            .filter(|v| cache.binary.notified_version.as_deref() != Some(v))
            .filter(|v| is_newer(v, &self.running_version));

        let docset_snapshot = make_snapshot(&cache.registry.available);
        let docset_updates =
            if cache.registry.notified_snapshot.as_deref() == Some(&docset_snapshot) {
                vec![]
            } else {
                cache.registry.available.clone()
            };

        let reminder = combined_reminder_text(binary_version, &docset_updates);

        if reminder.is_some() {
            // Mark as notified.
            if let Some(v) = binary_version {
                cache.binary.notified_version = Some(v.to_string());
            }
            if !docset_updates.is_empty() {
                cache.registry.notified_snapshot = Some(docset_snapshot);
            }
        }

        if write_cache(&cache).is_err() {
            return Ok(None);
        }

        Ok(reminder)
    }
}

/// Claim a cached unnotified reminder for `serve` without any network request.
///
/// After cache layout initialization, `nowdocs serve` calls this to surface a
/// previously-discovered newer version exactly once on stderr. It never
/// initiates a network request.
pub fn serve_claim_cached_reminder(running_version: &str) -> Result<Option<String>> {
    if is_opted_out() {
        return Ok(None);
    }

    let Some(_lock) = acquire_lock() else {
        return Ok(None);
    };

    let mut cache = read_cache();

    let binary_version = cache
        .binary
        .latest_version
        .as_deref()
        .filter(|v| cache.binary.notified_version.as_deref() != Some(v))
        .filter(|v| is_newer(v, running_version));

    let docset_snapshot = make_snapshot(&cache.registry.available);
    let docset_updates = if cache.registry.notified_snapshot.as_deref() == Some(&docset_snapshot) {
        vec![]
    } else {
        cache.registry.available.clone()
    };

    let reminder = combined_reminder_text(binary_version, &docset_updates);

    if reminder.is_some() {
        if let Some(v) = binary_version {
            cache.binary.notified_version = Some(v.to_string());
        }
        if !docset_updates.is_empty() {
            cache.registry.notified_snapshot = Some(docset_snapshot);
        }
        if write_cache(&cache).is_err() {
            return Ok(None);
        }
    }

    Ok(reminder)
}

// ===========================================================================
// Internal: cache logic
// ===========================================================================

/// True if `NOWDOCS_UPDATE_CHECK=0` is set.
fn is_opted_out() -> bool {
    matches!(std::env::var("NOWDOCS_UPDATE_CHECK").as_deref(), Ok("0"))
}

/// Read the cache from disk. Returns a default cache on any error (missing
/// file, malformed JSON, schema mismatch). Handles migration from v1.
fn read_cache() -> UpdateCacheV2 {
    let path = cache_path();
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return UpdateCacheV2::default();
    };

    // Try v2 first.
    if let Ok(c) = serde_json::from_str::<UpdateCacheV2>(&raw) {
        if c.schema_version == SCHEMA_VERSION {
            return c;
        }
    }

    // Try v1 migration.
    if let Ok(v1) = serde_json::from_str::<UpdateCacheV1>(&raw) {
        if v1.schema_version == 1 {
            return migrate_v1_to_v2(v1);
        }
    }

    UpdateCacheV2::default()
}

/// Write the cache atomically (unique same-directory tempfile + replacement).
fn write_cache(cache: &UpdateCacheV2) -> Result<()> {
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
fn channel_needs_fetch(
    last_attempt_secs: u64,
    cached_version: &str,
    running_version: &str,
) -> bool {
    if !cached_version.is_empty() && cached_version != running_version {
        return true;
    }
    let now = now_unix_secs();
    now.saturating_sub(last_attempt_secs) >= FRESHNESS_SECS
}

/// Make a stable snapshot string from sorted docset updates.
fn make_snapshot(updates: &[DocsetUpdate]) -> String {
    let parts: Vec<String> = updates
        .iter()
        .map(|u| format!("{}:{}->{}", u.docset, u.installed_version, u.latest_version))
        .collect();
    parts.join(",")
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
// Internal: registry update check
// ===========================================================================

/// Check for registry docset updates by loading receipts and comparing against
/// the Registry index. Returns a sorted list of available updates.
/// Any error returns an empty list (silent failure).
fn check_registry_updates() -> Result<Vec<DocsetUpdate>> {
    let receipts = crate::registry_receipt::load_matching_installed();
    if receipts.is_empty() {
        return Ok(vec![]);
    }

    let idx = registry::fetch_index_for_update(REGISTRY_FETCH_TIMEOUT)?;

    let mut updates = Vec::new();
    for receipt in &receipts {
        if let Some(pkg) = idx.packages.iter().find(|p| p.docset == receipt.docset) {
            if is_newer(&pkg.version, &receipt.package_version) {
                updates.push(DocsetUpdate {
                    docset: receipt.docset.clone(),
                    installed_version: receipt.package_version.clone(),
                    latest_version: pkg.version.clone(),
                });
            }
        }
    }

    updates.sort_by(|a, b| a.docset.cmp(&b.docset));
    Ok(updates)
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
/// (non-draft, non-prerelease) release, or None otherwise.
fn parse_release_inner(release: &GithubRelease) -> Option<String> {
    if release.draft || release.prerelease {
        return None;
    }
    let tag = release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&release.tag_name);
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
// Test-only utilities
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
    pub fn acquire_lock() -> Option<super::UpdateLock> {
        super::acquire_lock()
    }

    /// Read the update cache as a JSON value (test-only).
    pub fn read_update_cache_json() -> serde_json::Value {
        let path = super::cache_path();
        let raw = std::fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&raw).unwrap_or(serde_json::json!({}))
    }

    /// Render the docset reminder text for a set of updates (test-only).
    pub fn registry_reminder_text(updates: &[(&str, &str, &str)]) -> String {
        let docset_updates: Vec<super::DocsetUpdate> = updates
            .iter()
            .map(|(docset, installed, latest)| super::DocsetUpdate {
                docset: docset.to_string(),
                installed_version: installed.to_string(),
                latest_version: latest.to_string(),
            })
            .collect();
        super::docset_reminder_section(&docset_updates)
    }
}

use anyhow::Context as _;
