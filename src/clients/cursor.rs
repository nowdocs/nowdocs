//! Cursor adapter.
//!
//! Targets the global `~/.cursor/mcp.json` by way of an approved root's
//! relative `.cursor/mcp.json` target. Emits a canonical stdio `nowdocs` entry,
//! preserves every unrelated top-level key and `mcpServers` entry, and uses
//! C5's safe-target, atomic-replacement, and operation-owned backup/journal
//! primitives for conditional apply, verify, and digest-guarded rollback.
//!
//! All observations are stable and redacted: they never contain absolute
//! paths, binary locations, or configuration values.

use std::path::Path;

use anyhow::Result;

use crate::agent_contract::CapabilitySupport;
use crate::automation::operation::{apply_with_backup, rollback as rollback_op, OperationId};
use crate::clients::{
    read_target, safe_target, AdapterCapabilities, ApprovedRoot, ClientAdapter,
    ClientExecutionOutcome, ClientExecutionRequest, ClientExecutionResult, ClientId, Detection,
    GeneratedConfig,
};

/// The global relative target for Cursor's MCP configuration.
const TARGET_RELATIVE: &str = ".cursor/mcp.json";

/// Stable observation: the client may need a reload to pick up the change.
/// Deliberately generic so it cannot leak paths or values.
const OBS_RELOAD_REQUIRED: &str = "client_reload_required";

/// Stable observation: an apply succeeded with an operation-owned backup.
const OBS_APPLIED_WITH_BACKUP: &str = "applied_with_operation_backup";

/// Stable observation: a rollback restored the prior content.
const OBS_ROLLBACK_RESTORED: &str = "rolled_back_via_operation";

pub struct CursorAdapter;

impl ClientAdapter for CursorAdapter {
    fn id(&self) -> ClientId {
        ClientId::Cursor
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            detect: CapabilitySupport::Supported,
            generate: CapabilitySupport::Supported,
            apply: CapabilitySupport::Conditional,
            verify: CapabilitySupport::Conditional,
        }
    }

    fn detect(&self, root: &ApprovedRoot) -> Result<Detection> {
        let target = root.path().join(".cursor").join("mcp.json");
        let detected = target.is_file();
        Ok(Detection {
            detected,
            target_path: if detected {
                Some(TARGET_RELATIVE.to_string())
            } else {
                None
            },
            observations: Vec::new(),
        })
    }

    fn generate(&self, binary_path: &Path) -> Result<GeneratedConfig> {
        Ok(GeneratedConfig {
            stdio_command: vec![binary_path.display().to_string(), "serve".to_string()],
            redacted_fragment:
                r#"{"mcpServers":{"nowdocs":{"command":"<binary>","args":["serve"]}}}"#.to_string(),
            manual_steps: vec!["Add the nowdocs server entry to ~/.cursor/mcp.json.".to_string()],
        })
    }

    fn apply(&self, request: &ClientExecutionRequest) -> Result<ClientExecutionResult> {
        let id = match OperationId::new(request.operation_id()) {
            Ok(id) => id,
            Err(_) => {
                return Ok(ClientExecutionResult {
                    outcome: ClientExecutionOutcome::ManualRequired,
                    observations: vec!["invalid_operation_id".to_string()],
                });
            }
        };

        let target = match safe_target(request.approved_root(), TARGET_RELATIVE) {
            Ok(t) => t,
            Err(_) => {
                return Ok(manual("unsafe_target"));
            }
        };

        // Read the existing file via no-follow I/O. A missing target, a symlink,
        // or a non-regular file all downgrade to manual with no mutation.
        let original = match read_target(&target) {
            Ok(bytes) => bytes,
            Err(_) => {
                return Ok(manual("target_absent_or_unsafe"));
            }
        };

        let mut root: serde_json::Value = match serde_json::from_slice(&original) {
            Ok(v) => v,
            Err(_) => {
                return Ok(manual("malformed_json"));
            }
        };

        let root_obj = match root.as_object_mut() {
            Some(obj) => obj,
            None => {
                return Ok(manual("top_level_not_object"));
            }
        };

        let servers = match root_obj.get_mut("mcpServers") {
            Some(serde_json::Value::Object(servers)) => servers,
            Some(_) => {
                return Ok(manual("mcp_servers_not_object"));
            }
            None => {
                // No mcpServers key yet; create an empty object.
                root_obj.insert("mcpServers".to_string(), serde_json::json!({}));
                root_obj
                    .get_mut("mcpServers")
                    .and_then(|v| v.as_object_mut())
                    .expect("mcpServers was just inserted as an object")
            }
        };

        if servers.contains_key("nowdocs") {
            return Ok(ClientExecutionResult {
                outcome: ClientExecutionOutcome::Conflict,
                observations: vec!["nowdocs_entry_exists".to_string()],
            });
        }

        // Add exactly the canonical nowdocs entry using the absolute request binary.
        servers.insert(
            "nowdocs".to_string(),
            serde_json::json!({
                "command": request.binary_path().display().to_string(),
                "args": ["serve"],
            }),
        );

        let content = serde_json::to_vec_pretty(&root)
            .map_err(|e| anyhow::anyhow!("serialize cursor config: {e}"))?;

        apply_with_backup(&id, &target, &content)?;

        Ok(ClientExecutionResult {
            outcome: ClientExecutionOutcome::Applied,
            observations: vec![
                OBS_APPLIED_WITH_BACKUP.to_string(),
                OBS_RELOAD_REQUIRED.to_string(),
            ],
        })
    }

    fn verify(&self, request: &ClientExecutionRequest) -> Result<ClientExecutionResult> {
        let target = match safe_target(request.approved_root(), TARGET_RELATIVE) {
            Ok(t) => t,
            Err(_) => {
                return Ok(manual("unsafe_target"));
            }
        };

        let bytes = match read_target(&target) {
            Ok(b) => b,
            Err(_) => {
                return Ok(manual("target_absent_or_unsafe"));
            }
        };

        let root: serde_json::Value = match serde_json::from_slice(&bytes) {
            Ok(v) => v,
            Err(_) => {
                return Ok(manual("malformed_json"));
            }
        };

        let expected_command = request.binary_path().display().to_string();
        let nowdocs = root.get("mcpServers").and_then(|s| s.get("nowdocs"));

        let matches = nowdocs
            .and_then(|n| n.get("command"))
            .and_then(|c| c.as_str())
            .map(|c| c == expected_command)
            .unwrap_or(false)
            && nowdocs
                .and_then(|n| n.get("args"))
                .and_then(|a| a.as_array())
                .map(|args| {
                    args.len() == 1 && args[0].as_str().map(|s| s == "serve").unwrap_or(false)
                })
                .unwrap_or(false);

        if matches {
            Ok(ClientExecutionResult {
                outcome: ClientExecutionOutcome::Verified,
                observations: vec![OBS_RELOAD_REQUIRED.to_string()],
            })
        } else {
            Ok(manual("entry_mismatch"))
        }
    }

    fn rollback(&self, request: &ClientExecutionRequest) -> Result<ClientExecutionResult> {
        let id = match OperationId::new(request.operation_id()) {
            Ok(id) => id,
            Err(_) => {
                return Ok(manual("invalid_operation_id"));
            }
        };

        match rollback_op(&id) {
            Ok(_) => Ok(ClientExecutionResult {
                outcome: ClientExecutionOutcome::RolledBack,
                observations: vec![OBS_ROLLBACK_RESTORED.to_string()],
            }),
            Err(_) => Ok(manual("rollback_refused_or_unknown")),
        }
    }
}

/// Construct a stable, redacted `ManualRequired` result.
fn manual(reason: &str) -> ClientExecutionResult {
    ClientExecutionResult {
        outcome: ClientExecutionOutcome::ManualRequired,
        observations: vec![reason.to_string()],
    }
}
