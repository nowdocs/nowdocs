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

    DoctorOutput { status, checks }
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
            };
        }
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

    DoctorOutput { status, checks }
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

    DoctorOutput { status, checks }
}

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
            remediation: Some("Model will be downloaded on first use".to_string()),
        });
    }

    DoctorOutput {
        status: aggregate_status(&checks),
        checks,
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
        let manifest_path = cache::manifest_path(docset);
        let db_path = cache::db_path(docset);

        if manifest_path.is_file() && db_path.exists() {
            checks.push(Check {
                id: format!("docset-{}", docset),
                severity: Severity::Ok,
                message: format!("Docset '{}' is healthy", docset),
                remediation: None,
            });
        } else if manifest_path.is_file() && !db_path.exists() {
            checks.push(Check {
                id: format!("docset-{}", docset),
                severity: Severity::Warn,
                message: format!("Docset '{}' has manifest but no store", docset),
                remediation: Some("Reinstall the docset to rebuild the store".to_string()),
            });
        } else if !manifest_path.is_file() && db_path.exists() {
            checks.push(Check {
                id: format!("docset-{}", docset),
                severity: Severity::Fail,
                message: format!("Docset '{}' has store but no manifest", docset),
                remediation: Some("Reinstall the docset".to_string()),
            });
        }
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
