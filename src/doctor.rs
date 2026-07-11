//! `nowdocs doctor` diagnostics implementation.

use serde::Serialize;

use crate::cache;
use crate::manifest;

/// Check severity/status.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Ok,
    Warn,
    Fail,
}

/// A single diagnostic check result.
#[derive(Debug, Clone, Serialize)]
pub struct Check {
    pub id: String,
    pub severity: Severity,
    pub message: String,
    pub remediation: Option<String>,
}

/// Top-level doctor output.
#[derive(Debug, Clone, Serialize)]
pub struct DoctorOutput {
    pub status: Severity,
    pub checks: Vec<Check>,
    /// M23: cache/observability metrics attached to `doctor --json` output.
    pub metrics: DoctorMetrics,
}

/// M23: cache-size and per-docset metrics for `doctor --json`. All fields are
/// best-effort: unreadable paths report 0 so a corrupt cache still produces a
/// complete JSON document.
#[derive(Debug, Clone, Serialize, Default)]
pub struct DoctorMetrics {
    /// Total size of the model cache (`<cache>/nowdocs/models`) in bytes.
    pub model_cache_bytes: u64,
    /// Same as `model_cache_bytes`, rounded to 2-decimal MiB for quick reading.
    pub model_cache_mb: f64,
    /// Size of the docset store area (`db/`, `.lance` tables) in bytes.
    pub db_bytes: u64,
    /// Size of the manifest files in bytes.
    pub manifests_bytes: u64,
    /// Size of the staging area in bytes.
    pub staging_bytes: u64,
    /// Size of the rollback area in bytes.
    pub rollback_bytes: u64,
    /// Per-docset store/manifest counters.
    pub docsets: Vec<DocsetMetric>,
}

/// M23: per-docset metrics row inside [`DoctorMetrics::docsets`].
#[derive(Debug, Clone, Serialize)]
pub struct DocsetMetric {
    pub name: String,
    /// Unified install-state label (M22 `InstalledDocsetState::label`).
    pub state: String,
    /// Live row count of the lance table (0 when the store is absent/unreadable).
    pub store_rows: u64,
    /// `source.chunk_count` from the manifest (0 when absent/unparseable).
    pub manifest_chunk_count: u32,
}

impl DoctorMetrics {
    /// Collect current cache metrics. Pure read-only disk inspection.
    pub fn collect() -> Self {
        let mut m = DoctorMetrics::default();
        if let Ok(status) = cache::cache_status() {
            m.model_cache_bytes = status.models_bytes;
            m.model_cache_mb =
                (status.models_bytes as f64 / (1024.0 * 1024.0) * 100.0).round() / 100.0;
            m.db_bytes = status.db_bytes;
            m.manifests_bytes = status.manifests_bytes;
            m.staging_bytes = status.staging_bytes;
            m.rollback_bytes = status.rollback_bytes;
        }

        let db_dir = cache::cache_root().join("db");
        let mut names: Vec<String> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&db_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if let Some(stem) = name.strip_suffix(".lance") {
                            names.push(stem.to_string());
                        }
                    }
                }
            }
        }
        names.sort();
        for name in names {
            let state = cache::check_docset_state(&name).label().to_string();
            let store_rows = if cache::db_path(&name).exists() {
                crate::store::Store::open(&name)
                    .and_then(|s| s.row_count())
                    .unwrap_or(0)
            } else {
                0
            };
            let manifest_chunk_count = std::fs::read_to_string(cache::manifest_path(&name))
                .ok()
                .and_then(|raw| manifest::parse_manifest(&raw).ok())
                .map(|mm| mm.source.chunk_count)
                .unwrap_or(0);
            m.docsets.push(DocsetMetric {
                name,
                state,
                store_rows,
                manifest_chunk_count,
            });
        }
        m
    }
}

fn aggregate_status(checks: &[Check]) -> Severity {
    if checks.iter().any(|c| c.severity == Severity::Fail) {
        Severity::Fail
    } else if checks.iter().any(|c| c.severity == Severity::Warn) {
        Severity::Warn
    } else {
        Severity::Ok
    }
}

/// Run all default doctor checks.
pub fn run_default_checks() -> DoctorOutput {
    let mut checks = Vec::new();

    // Check cache root exists or can be created
    checks.push(check_cache_root());

    // Check cache root is writable
    checks.push(check_cache_writable());

    // Check expected directories exist
    checks.push(check_cache_directories());

    // Check installed docsets
    checks.extend(check_installed_docsets());

    // Check for stale staging directories
    checks.push(check_stale_staging());

    // Check curl is available for registry downloads (OQ6)
    checks.push(check_curl());

    // Check model cache status (M15)
    checks.extend(run_model_check().checks);

    // Check MCP handler status (M15)
    checks.extend(run_mcp_check().checks);

    // Determine overall status
    let status = if checks.iter().any(|c| c.severity == Severity::Fail) {
        Severity::Fail
    } else if checks.iter().any(|c| c.severity == Severity::Warn) {
        Severity::Warn
    } else {
        Severity::Ok
    };

    DoctorOutput {
        status,
        checks,
        metrics: DoctorMetrics::collect(),
    }
}

/// Run deep checks for a specific docset.
pub fn run_docset_checks(docset: &str) -> DoctorOutput {
    let mut checks = Vec::new();

    // Validate docset name
    match crate::input::validate_docset(docset) {
        Ok(_) => checks.push(Check {
            id: "docset-name-valid".to_string(),
            severity: Severity::Ok,
            message: format!("Docset name '{}' is valid", docset),
            remediation: None,
        }),
        Err(e) => {
            checks.push(Check {
                id: "docset-name-valid".to_string(),
                severity: Severity::Fail,
                message: format!("Invalid docset name '{}': {}", docset, e),
                remediation: Some("Use lowercase alphanumeric with hyphens only".to_string()),
            });
            return DoctorOutput {
                status: Severity::Fail,
                checks,
                metrics: DoctorMetrics::default(),
            };
        }
    }

    // M22: unified install-state summary for this docset, sourced from the same
    // `check_docset_state` used by list-installed / smoke / nowdocs_list.
    {
        let state = cache::check_docset_state(docset);
        let (severity, message) = match state {
            cache::InstalledDocsetState::Healthy => (
                Severity::Ok,
                format!(
                    "Docset state: {} (manifest + store consistent)",
                    state.label()
                ),
            ),
            cache::InstalledDocsetState::ManifestOnly => (
                Severity::Warn,
                format!(
                    "Docset state: {} (manifest present, store missing)",
                    state.label()
                ),
            ),
            cache::InstalledDocsetState::StoreOnly => (
                Severity::Fail,
                format!(
                    "Docset state: {} (store present, manifest missing)",
                    state.label()
                ),
            ),
            cache::InstalledDocsetState::SchemaMismatch => (
                Severity::Fail,
                format!("Docset state: {} (schema incompatible)", state.label()),
            ),
            cache::InstalledDocsetState::RowCountMismatch => (
                Severity::Warn,
                format!(
                    "Docset state: {} (store row count != manifest chunk_count)",
                    state.label()
                ),
            ),
            cache::InstalledDocsetState::NotInstalled => (
                Severity::Fail,
                format!(
                    "Docset state: {} (neither manifest nor store)",
                    state.label()
                ),
            ),
        };
        checks.push(Check {
            id: "docset-state".to_string(),
            severity,
            message,
            remediation: None,
        });
    }

    // Check manifest exists
    let manifest_path = cache::manifest_path(docset);
    if manifest_path.is_file() {
        checks.push(Check {
            id: "docset-manifest-exists".to_string(),
            severity: Severity::Ok,
            message: "Manifest file exists".to_string(),
            remediation: None,
        });

        // Validate manifest content
        match std::fs::read_to_string(&manifest_path) {
            Ok(raw) => match manifest::parse_manifest(&raw) {
                Ok(m) => match manifest::validate(&m) {
                    Ok(()) => checks.push(Check {
                        id: "docset-manifest-valid".to_string(),
                        severity: Severity::Ok,
                        message: "Manifest is valid".to_string(),
                        remediation: None,
                    }),
                    Err(e) => checks.push(Check {
                        id: "docset-manifest-valid".to_string(),
                        severity: Severity::Fail,
                        message: format!("Manifest validation failed: {}", e),
                        remediation: Some("Reinstall the docset".to_string()),
                    }),
                },
                Err(e) => checks.push(Check {
                    id: "docset-manifest-valid".to_string(),
                    severity: Severity::Fail,
                    message: format!("Manifest parse error: {}", e),
                    remediation: Some("Reinstall the docset".to_string()),
                }),
            },
            Err(e) => checks.push(Check {
                id: "docset-manifest-readable".to_string(),
                severity: Severity::Fail,
                message: format!("Cannot read manifest: {}", e),
                remediation: Some("Check file permissions".to_string()),
            }),
        }
    } else {
        checks.push(Check {
            id: "docset-manifest-exists".to_string(),
            severity: Severity::Fail,
            message: "Manifest file missing".to_string(),
            remediation: Some("Install the docset with `nowdocs install <docset>`".to_string()),
        });
    }

    // Check store path exists
    let db_path = cache::db_path(docset);
    if db_path.exists() {
        checks.push(Check {
            id: "docset-store-exists".to_string(),
            severity: Severity::Ok,
            message: "Store directory exists".to_string(),
            remediation: None,
        });
    } else {
        checks.push(Check {
            id: "docset-store-exists".to_string(),
            severity: Severity::Warn,
            message: "Store directory missing".to_string(),
            remediation: Some("Reinstall the docset to rebuild the store".to_string()),
        });
    }

    // Check license/notice files
    let license_path = cache::license_text_path(docset);
    if license_path.is_file() {
        checks.push(Check {
            id: "docset-license-exists".to_string(),
            severity: Severity::Ok,
            message: "License file exists".to_string(),
            remediation: None,
        });
    } else {
        checks.push(Check {
            id: "docset-license-exists".to_string(),
            severity: Severity::Warn,
            message: "License file missing".to_string(),
            remediation: Some("This is normal for docsets without upstream LICENSE".to_string()),
        });
    }

    // Determine overall status
    let status = if checks.iter().any(|c| c.severity == Severity::Fail) {
        Severity::Fail
    } else if checks.iter().any(|c| c.severity == Severity::Warn) {
        Severity::Warn
    } else {
        Severity::Ok
    };

    DoctorOutput {
        status,
        checks,
        metrics: DoctorMetrics::collect(),
    }
}

/// Run MCP smoke test (in-process, no network).
///
/// Verifies the MCP handler can produce a valid initialize response and
/// a non-empty tools list. No I/O, no subprocess — calls the handler
/// functions directly.
pub fn run_mcp_check() -> DoctorOutput {
    let mut checks = Vec::new();

    // Check 1: initialize handler returns valid response
    let init = crate::mcp::handle_initialize();
    if init.get("protocolVersion").is_some()
        && init.get("capabilities").is_some()
        && init.get("serverInfo").is_some()
    {
        checks.push(Check {
            id: "mcp-initialize".to_string(),
            severity: Severity::Ok,
            message: format!(
                "MCP initialize ok (protocol {}, server {})",
                init["protocolVersion"], init["serverInfo"]["name"]
            ),
            remediation: None,
        });
    } else {
        checks.push(Check {
            id: "mcp-initialize".to_string(),
            severity: Severity::Fail,
            message: "MCP initialize handler returned unexpected shape".to_string(),
            remediation: Some("Check nowdocs build integrity".to_string()),
        });
    }

    // Check 2: tools/list returns expected tools
    let tools_list = crate::mcp::handle_tools_list();
    let tool_names: Vec<String> = tools_list
        .get("tools")
        .and_then(|t| t.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let has_search = tool_names.iter().any(|n| n == "nowdocs_search");
    let has_list = tool_names.iter().any(|n| n == "nowdocs_list");

    if has_search && has_list {
        checks.push(Check {
            id: "mcp-tools".to_string(),
            severity: Severity::Ok,
            message: format!("MCP tools ok: {}", tool_names.join(", ")),
            remediation: None,
        });
    } else {
        checks.push(Check {
            id: "mcp-tools".to_string(),
            severity: Severity::Fail,
            message: format!(
                "MCP tools incomplete: expected nowdocs_search + nowdocs_list, got {}",
                tool_names.join(", ")
            ),
            remediation: Some("Check nowdocs build integrity".to_string()),
        });
    }

    let status = if checks.iter().any(|c| c.severity == Severity::Fail) {
        Severity::Fail
    } else {
        Severity::Ok
    };

    DoctorOutput {
        status,
        checks,
        metrics: DoctorMetrics::collect(),
    }
}

/// Pre-warm hint appended to install/update output and shown by the
/// missing-model check (N5 / A1.3). Pre-downloading the ~66 MB embedder keeps
/// the first `nowdocs_search` from stalling long enough to hit MCP client
/// timeouts. Single source of truth shared by the CLI and doctor.
pub const MODEL_PREWARM_HINT: &str =
    "tip: run 'nowdocs doctor --model' to pre-download the embedding model";

/// Run model cache check.
pub fn run_model_check() -> DoctorOutput {
    let mut checks = Vec::new();

    let model_id = "jinaai/jina-embeddings-v2-small-en";
    let model_path = cache::model_path(model_id);

    if model_path.exists() {
        checks.push(Check {
            id: "model-cache-exists".to_string(),
            severity: Severity::Ok,
            message: format!("Model cache exists at {}", model_path.display()),
            remediation: None,
        });
    } else {
        checks.push(Check {
            id: "model-cache-exists".to_string(),
            severity: Severity::Warn,
            message: "Model cache not found".to_string(),
            remediation: Some(MODEL_PREWARM_HINT.to_string()),
        });
    }

    DoctorOutput {
        status: aggregate_status(&checks),
        checks,
        metrics: DoctorMetrics::default(),
    }
}

/// Pre-download the embedder model (`nowdocs doctor --model`).
///
/// Unlike [`run_model_check`] (read-only status), this intentionally performs
/// the ~66 MB download: it is the pre-warm path that install/update output and
/// the missing-model hint point users at (N5), so the first `nowdocs_search`
/// doesn't stall. On success the model is downloaded, sha256-verified, and
/// loaded; on failure the check is `Fail` with the error chain as remediation.
pub fn run_model_prewarm() -> DoctorOutput {
    let model_id = "jinaai/jina-embeddings-v2-small-en";
    let checks = match crate::embedder::Embedder::load() {
        Ok(_) => vec![Check {
            id: "model-prewarm".to_string(),
            severity: Severity::Ok,
            message: format!(
                "Embedding model {model_id} is cached and loadable at {}",
                cache::model_path(model_id).display()
            ),
            remediation: None,
        }],
        Err(e) => vec![Check {
            id: "model-prewarm".to_string(),
            severity: Severity::Fail,
            message: format!("Failed to pre-download embedding model {model_id}"),
            remediation: Some(format!("{e:#}")),
        }],
    };

    DoctorOutput {
        status: aggregate_status(&checks),
        checks,
        metrics: DoctorMetrics::collect(),
    }
}

/// Run repair mode (staging cleanup only).
pub fn run_repair() -> DoctorOutput {
    let checks = match cache::clean_staging_older_than(std::time::Duration::from_secs(60 * 60)) {
        Ok(cleaned) => {
            let removed_paths = if cleaned.removed.is_empty() {
                "none".to_string()
            } else {
                cleaned
                    .removed
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            vec![Check {
                id: "repair-staging-cleanup".to_string(),
                severity: Severity::Ok,
                message: format!(
                    "Removed {} stale staging director{} ({removed_paths}); skipped {} recent or non-nowdocs staging director{}",
                cleaned.removed.len(),
                if cleaned.removed.len() == 1 { "y" } else { "ies" },
                cleaned.skipped.len(),
                if cleaned.skipped.len() == 1 { "y" } else { "ies" }
            ),
                remediation: Some(
                    "For explicit cleanup thresholds, run `nowdocs cache clean-staging --older-than 1h`"
                        .to_string(),
                ),
            }]
        }
        Err(e) => vec![Check {
            id: "repair-staging-cleanup".to_string(),
            severity: Severity::Fail,
            message: format!("Failed to clean stale staging directories: {e}"),
            remediation: Some("Inspect cache permissions or run `nowdocs doctor`".to_string()),
        }],
    };

    DoctorOutput {
        status: aggregate_status(&checks),
        checks,
        metrics: DoctorMetrics::default(),
    }
}

// Helper functions for checks

fn check_cache_root() -> Check {
    let root = cache::cache_root();
    if root.exists() {
        Check {
            id: "cache-root-exists".to_string(),
            severity: Severity::Ok,
            message: format!("Cache root exists at {}", root.display()),
            remediation: None,
        }
    } else {
        // Try to create it
        match std::fs::create_dir_all(&root) {
            Ok(()) => Check {
                id: "cache-root-exists".to_string(),
                severity: Severity::Ok,
                message: format!("Created cache root at {}", root.display()),
                remediation: None,
            },
            Err(e) => Check {
                id: "cache-root-exists".to_string(),
                severity: Severity::Fail,
                message: format!("Cannot create cache root: {}", e),
                remediation: Some("Check directory permissions".to_string()),
            },
        }
    }
}

fn check_cache_writable() -> Check {
    let root = cache::cache_root();
    let test_file = root.join(".write_test");
    match std::fs::write(&test_file, "test") {
        Ok(()) => {
            let _ = std::fs::remove_file(&test_file);
            Check {
                id: "cache-writable".to_string(),
                severity: Severity::Ok,
                message: "Cache directory is writable".to_string(),
                remediation: None,
            }
        }
        Err(e) => Check {
            id: "cache-writable".to_string(),
            severity: Severity::Fail,
            message: format!("Cache directory not writable: {}", e),
            remediation: Some("Check directory permissions".to_string()),
        },
    }
}

fn check_cache_directories() -> Check {
    let root = cache::cache_root();
    let db_dir = root.join("db");
    let models_dir = root.join("models");

    let mut missing = Vec::new();

    if !db_dir.exists() {
        match std::fs::create_dir_all(&db_dir) {
            Ok(()) => {}
            Err(_) => missing.push("db"),
        }
    }

    if !models_dir.exists() {
        match std::fs::create_dir_all(&models_dir) {
            Ok(()) => {}
            Err(_) => missing.push("models"),
        }
    }

    if missing.is_empty() {
        Check {
            id: "cache-directories".to_string(),
            severity: Severity::Ok,
            message: "All expected cache directories exist".to_string(),
            remediation: None,
        }
    } else {
        Check {
            id: "cache-directories".to_string(),
            severity: Severity::Fail,
            message: format!("Missing directories: {}", missing.join(", ")),
            remediation: Some("Check directory permissions".to_string()),
        }
    }
}

fn check_installed_docsets() -> Vec<Check> {
    let mut checks = Vec::new();

    let db_dir = cache::cache_root().join("db");
    if !db_dir.exists() {
        return checks;
    }

    let mut docsets = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&db_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if let Some(stem) = name.strip_suffix(".lance") {
                        docsets.push(stem.to_string());
                    }
                }
            }
        }
    }

    for docset in &docsets {
        // M22: route through the unified state model so doctor agrees with
        // list-installed / smoke / nowdocs_list on partial installs.
        let state = cache::check_docset_state(docset);
        let (severity, message, remediation) = match state {
            cache::InstalledDocsetState::Healthy => {
                (Severity::Ok, format!("Docset '{docset}' is healthy"), None)
            }
            cache::InstalledDocsetState::ManifestOnly => (
                Severity::Warn,
                format!("Docset '{docset}' has manifest but no store"),
                Some("Reinstall the docset to rebuild the store".to_string()),
            ),
            cache::InstalledDocsetState::StoreOnly => (
                Severity::Fail,
                format!("Docset '{docset}' has store but no manifest"),
                Some("Reinstall the docset".to_string()),
            ),
            cache::InstalledDocsetState::SchemaMismatch => (
                Severity::Fail,
                format!("Docset '{docset}' schema version is incompatible with this binary"),
                Some(format!(
                    "Run `nowdocs rebuild {docset}` to migrate the local cache"
                )),
            ),
            cache::InstalledDocsetState::RowCountMismatch => (
                Severity::Warn,
                format!("Docset '{docset}' store row count disagrees with its manifest"),
                Some("Reinstall or rebuild the docset to repair the store".to_string()),
            ),
            cache::InstalledDocsetState::NotInstalled => (
                Severity::Fail,
                format!("Docset '{docset}' is not installed"),
                Some(format!(
                    "Install the docset with `nowdocs install {docset}`"
                )),
            ),
        };
        checks.push(Check {
            id: format!("docset-{docset}"),
            severity,
            message,
            remediation,
        });
    }

    checks
}

fn check_stale_staging() -> Check {
    match cache::list_staging_dirs() {
        Ok(dirs) => {
            if dirs.is_empty() {
                Check {
                    id: "stale-staging".to_string(),
                    severity: Severity::Ok,
                    message: "No stale staging directories found".to_string(),
                    remediation: None,
                }
            } else {
                Check {
                    id: "stale-staging".to_string(),
                    severity: Severity::Warn,
                    message: format!("Found {} stale staging directory(ies)", dirs.len()),
                    remediation: Some(
                        "Run `nowdocs cache clean-staging --older-than 1h` or `nowdocs doctor --repair`"
                            .to_string(),
                    ),
                }
            }
        }
        Err(e) => Check {
            id: "stale-staging".to_string(),
            severity: Severity::Warn,
            message: format!("Cannot check staging directories: {}", e),
            remediation: None,
        },
    }
}

/// OQ6: registry downloads go through `curl`, so doctor verifies it is on PATH.
fn check_curl() -> Check {
    let path_var = std::env::var("PATH").unwrap_or_default();
    if is_curl_available_in_path(&path_var) {
        Check {
            id: "curl-available".to_string(),
            severity: Severity::Ok,
            message: "curl is available on PATH".to_string(),
            remediation: None,
        }
    } else {
        Check {
            id: "curl-available".to_string(),
            severity: Severity::Warn,
            message: "curl not found on PATH".to_string(),
            remediation: Some("install curl for registry downloads".to_string()),
        }
    }
}

/// OQ6: whether `curl` resolves as an executable on the given PATH string
/// (`:`-separated). Split out from [`check_curl`] so tests can mock PATH.
pub fn is_curl_available_in_path(path_var: &str) -> bool {
    if path_var.is_empty() {
        return false;
    }
    for dir in path_var.split(':') {
        if dir.is_empty() {
            continue;
        }
        let candidate = std::path::Path::new(dir).join("curl");
        if is_executable_file(&candidate) {
            return true;
        }
        #[cfg(windows)]
        if is_executable_file(&candidate.with_extension("exe")) {
            return true;
        }
    }
    false
}

fn is_executable_file(path: &std::path::Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        match path.metadata() {
            Ok(m) => m.is_file() && (m.permissions().mode() & 0o111) != 0,
            Err(_) => false,
        }
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}
