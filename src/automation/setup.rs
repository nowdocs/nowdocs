//! One-plan setup orchestration: planner, executor, and rollback.
//!
//! C7 owns the `nowdocs setup plan`/`setup apply`/`setup rollback` flow. A
//! single `setup plan` produces one reusable `AutomationPlan` covering the
//! needed docset action, client configuration action, and verification action.
//! `setup apply` accepts only that plan hash; `setup rollback` accepts only the
//! resulting operation id.
//!
//! The plan hash is an integrity/scope check, not cryptographic evidence of
//! human consent. The caller obtains user consent before `setup apply`.
//!
//! C7 preserves C4/C5/C6 behavior: docset work delegates to the extracted C4
//! helpers, client work delegates to the C6 adapter trait, and rollback uses
//! C5's digest-guarded operation journal. One `setup plan` never creates a
//! nested `ensure` plan.

use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::agent_contract::{CapabilitySupport, RiskLevel};
use crate::automation::docset;
use crate::automation::lock;
use crate::automation::operation::OperationId;
use crate::automation::plan::{
    load_plan, new_plan, store_plan, DocsetPrecondition, PlanInputs, PlanPreconditions,
    PlannedAction, TargetFilePrecondition,
};
use crate::cache::{self, InstalledDocsetState};
use crate::clients::{
    all_adapters, approved_root, compute_digest, read_target, safe_target, ApprovedRoot,
    ClientExecutionOutcome, ClientExecutionRequest,
};
use crate::input;
use crate::registry;

/// Stable action kind for a client apply step.
const KIND_CLIENT_APPLY: &str = "client_apply";
/// Stable action kind for a client verify step.
const KIND_CLIENT_VERIFY: &str = "client_verify";
/// Stable action kind for a docset verification step.
const KIND_VERIFY_DOCSET: &str = "verify_docset";
/// Stable action kind for a client manual guidance step.
const KIND_CLIENT_MANUAL_GUIDANCE: &str = "client_manual_guidance";

/// Filename for setup-owned operation metadata (client id), stored alongside
/// the C5 operation journal. This is the "trusted setup operation recorded its
/// own successful apply" record that guards rollback dispatch.
const SETUP_META_FILENAME: &str = "setup-meta.json";

/// The global Cursor MCP configuration relative target.
const CURSOR_TARGET_RELATIVE: &str = ".cursor/mcp.json";

/// Logical id for the Cursor target file precondition.
const CURSOR_TARGET_LOGICAL_ID: &str = "cursor-mcp-json";

/// Outcome of `setup_plan`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetupPlanResult {
    /// The docset is already installed and the client needs no automatic
    /// configuration change.
    AlreadySatisfied { precondition: DocsetPrecondition },
    /// A single plan was created; the caller must approve and apply `plan_hash`.
    PlanCreated {
        plan_hash: String,
        precondition: DocsetPrecondition,
    },
    /// Offline planning cannot determine registry state; run with `--online`.
    RegistryMetadataRequired { precondition: DocsetPrecondition },
}

/// Outcome of `setup_apply`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetupApplyResult {
    /// The plan was applied and fully verified.
    SetupComplete {
        operation_id: String,
        observations: Vec<String>,
    },
    /// The plan was applied and verified, but the client needs a reload.
    ClientReloadRequired {
        operation_id: String,
        observations: Vec<String>,
    },
    /// The plan could not be fully applied; the caller must take manual action.
    ActionRequired { observations: Vec<String> },
    /// Docset work succeeded but client application could not start. No client
    /// rollback metadata is retained because no client change committed.
    PartialNoRollback { observations: Vec<String> },
    /// A client change committed but final verification did not confirm it.
    Partial {
        operation_id: String,
        observations: Vec<String>,
    },
    /// A client change committed but setup metadata could not be safely
    /// persisted, so rollback is not available (exit 21). The observation is
    /// stable and redacted; no rollback metadata is offered.
    AppliedButUnverified { observations: Vec<String> },
}

/// Setup-owned operation metadata, stored alongside the C5 journal. Records
/// which client a setup operation targeted so rollback can dispatch
/// deterministically to exactly one adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SetupMeta {
    client: String,
}

/// Outcome of `setup_rollback`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetupRollbackResult {
    /// The operation was rolled back successfully.
    RolledBack { observations: Vec<String> },
    /// The operation could not be rolled back automatically; manual action needed.
    ManualRequired { observations: Vec<String> },
}

/// Plan one setup for `docset` + `client`.
///
/// - Offline (`online == false`): returns `already_satisfied` if the docset is
///   installed AND the client is canonically verified, otherwise
///   `registry_metadata_required`. No cache/model/network mutation occurs.
/// - Online (`online == true`): fetches the selected package metadata, creates
///   one `AutomationPlan` with docset install/update + verify, followed by
///   client apply + verify for conditional adapters. Claude Desktop and Generic
///   produce docset + `client_manual_guidance` plans (no client write actions).
///
/// The `approved_root` is supplied by the binary boundary; library code never
/// resolves HOME. A healthy docset alone is never `already_satisfied`: the
/// client must be canonically verified too.
pub fn setup_plan(
    docset: &str,
    client: &str,
    approved_root: &ApprovedRoot,
    online: bool,
    now_unix_secs: u64,
) -> Result<SetupPlanResult> {
    let docset_id = input::validate_docset(docset)?;
    let client_id = parse_client(client)?;

    let state = cache::check_docset_state_pure(&docset_id);
    let precondition = docset::docset_precondition(&docset_id, &state);

    // Capture the Cursor target fingerprint for conditional adapters.
    let target_fingerprint = cursor_target_fingerprint(approved_root)?;

    if !online {
        // Offline: already_satisfied requires BOTH healthy docset AND canonical
        // client verification.
        if state == InstalledDocsetState::Healthy
            && is_client_canonically_verified(&client_id, approved_root)
        {
            return Ok(SetupPlanResult::AlreadySatisfied { precondition });
        }
        return Ok(SetupPlanResult::RegistryMetadataRequired { precondition });
    }

    let package = registry::fetch_selected_package(&docset_id)?;

    if state == InstalledDocsetState::Healthy {
        let installed_version = cache::read_docset_meta(&docset_id).0;
        if installed_version == package.version
            && is_client_canonically_verified(&client_id, approved_root)
        {
            return Ok(SetupPlanResult::AlreadySatisfied { precondition });
        }
    }

    // Check for noncanonical existing Cursor entry before building the plan.
    // A noncanonical entry means we plan client_manual_guidance instead of
    // client_apply (contract 3).
    let cursor_noncanonical = cursor_has_noncanonical_entry(&client_id, approved_root);

    let plan = build_setup_plan(
        &docset_id,
        &client_id,
        &package,
        &precondition,
        &target_fingerprint,
        cursor_noncanonical,
        now_unix_secs,
    )?;
    let plan_hash = store_plan(&plan)?;
    Ok(SetupPlanResult::PlanCreated {
        plan_hash,
        precondition,
    })
}

/// Apply a stored setup plan.
///
/// Revalidates the plan (not expired/tampered/stale), acquires the global
/// operation lock, delegates docset installation to the C4 helper, and then
/// applies + verifies the client configuration via the C6 adapter. The
/// `approved_root_path` is the explicit approved client configuration root
/// supplied by the setup boundary; C7 never silently uses a real home directory.
pub fn setup_apply(
    plan_hash: &str,
    approved_root_path: &Path,
    now_unix_secs: u64,
) -> Result<SetupApplyResult> {
    let plan = load_plan(plan_hash, now_unix_secs)?;

    let client_id_str = plan
        .inputs
        .client
        .as_deref()
        .context("setup plan missing client input")?;
    let client_id = parse_client(client_id_str)?;
    let docset_id = plan
        .inputs
        .docset
        .as_deref()
        .context("setup plan missing docset input")?
        .to_string();

    // Revalidate docset precondition before any state change.
    let planned_precondition = plan
        .preconditions
        .docset_state
        .first()
        .context("plan missing docset precondition")?;

    let current_state = cache::check_docset_state_pure(&docset_id);
    let current_precondition = docset::docset_precondition(&docset_id, &current_state);
    if &current_precondition != planned_precondition {
        // If the desired state is already met, that's fine (idempotent).
        if let Some(package) = docset::desired_package_from_plan(&plan) {
            if docset::is_already_satisfied(&docset_id, &package) {
                // Fall through to client work below.
            } else {
                anyhow::bail!("PLAN_STALE: docset precondition changed since plan was created");
            }
        } else {
            anyhow::bail!("PLAN_STALE: docset precondition changed since plan was created");
        }
    }

    // Acquire the global operation lock before any state change.
    let op_id = format!("setup-{}", &plan_hash[..12]);
    let _op_lock = lock::acquire_operation_lock(&op_id)?;

    // Re-check precondition after lock acquisition.
    let current_state = cache::check_docset_state_pure(&docset_id);
    let current_precondition = docset::docset_precondition(&docset_id, &current_state);
    if &current_precondition != planned_precondition {
        if let Some(package) = docset::desired_package_from_plan(&plan) {
            if docset::is_already_satisfied(&docset_id, &package) {
                // Fall through.
            } else {
                anyhow::bail!("PLAN_STALE: docset precondition changed after lock acquisition");
            }
        } else {
            anyhow::bail!("PLAN_STALE: docset precondition changed after lock acquisition");
        }
    }

    // --- Target fingerprint drift check (contract 2) ---
    // After the global lock and before docset installation or adapter
    // invocation: verify the Cursor target fingerprint matches the plan.
    let root = approved_root(approved_root_path)?;
    let current_fingerprint = cursor_target_fingerprint(&root)?;
    let planned_fingerprints: Vec<&TargetFilePrecondition> =
        plan.preconditions.target_files.iter().collect();
    for planned_tf in &planned_fingerprints {
        let current_tf = current_fingerprint
            .iter()
            .find(|tf| tf.logical_id == planned_tf.logical_id);
        match (planned_tf, current_tf) {
            (planned, Some(current)) => {
                if planned.exists != current.exists || planned.sha256 != current.sha256 {
                    anyhow::bail!(
                        "PLAN_STALE: target file {} fingerprint drifted since plan creation",
                        planned.logical_id
                    );
                }
            }
            (planned, None) => {
                // Should not happen if the fingerprint was captured, but if the
                // plan has a target file precondition and we can't find the
                // current fingerprint, the plan is stale.
                anyhow::bail!(
                    "PLAN_STALE: target file {} fingerprint cannot be verified",
                    planned.logical_id
                );
            }
        }
    }

    // --- Docset phase ---
    // Delegate to the C4 install/update logic if the docset is not already
    // satisfied.
    let mut docset_observations: Vec<String> = Vec::new();
    let needs_docset_work = match docset::desired_package_from_plan(&plan) {
        Some(package) => !docset::is_already_satisfied(&docset_id, &package),
        None => false,
    };

    if needs_docset_work {
        let package = docset::desired_package_from_plan(&plan)
            .context("plan missing selected package metadata")?;
        registry::install_with_sha256(&docset_id, &package.download_url, &package.sha256)
            .with_context(|| format!("install docset {docset_id}"))?;

        let final_state = cache::check_docset_state(&docset_id);
        if final_state != InstalledDocsetState::Healthy
            || !docset::is_already_satisfied(&docset_id, &package)
        {
            anyhow::bail!(
                "VERIFICATION_FAILED: docset {docset_id} does not match the planned package after apply: {}",
                final_state.label()
            );
        }
        docset_observations.push("docset_installed".to_string());
    } else {
        docset_observations.push("docset_already_satisfied".to_string());
    }

    // --- Client phase ---
    // Find the adapter for the planned client.
    let adapter = all_adapters()
        .into_iter()
        .find(|a| a.id() == client_id)
        .context("adapter not found for client")?;

    let caps = adapter.capabilities();

    // If the plan has a client_manual_guidance action (noncanonical entry or
    // manual-only client), return ActionRequired with manual guidance without
    // invoking the adapter.
    let has_manual_guidance = plan
        .actions
        .iter()
        .any(|a| a.kind == KIND_CLIENT_MANUAL_GUIDANCE);

    if has_manual_guidance {
        let mut observations = docset_observations;
        observations.push("client_manual_guidance".to_string());
        return Ok(SetupApplyResult::ActionRequired { observations });
    }

    // If the client has no conditional apply capability, this is a manual-only
    // client (Claude Desktop, Generic). Return ActionRequired with manual guidance.
    if caps.apply != CapabilitySupport::Conditional {
        let mut observations = docset_observations;
        observations.push("client_manual_guidance".to_string());
        return Ok(SetupApplyResult::ActionRequired { observations });
    }

    // Resolve the nowdocs binary path at apply time.
    let binary_path = std::env::current_exe().context("resolve nowdocs executable path")?;
    if !binary_path.is_absolute() {
        anyhow::bail!(
            "nowdocs executable path is not absolute: {}",
            binary_path.display()
        );
    }

    // Construct the execution request.
    let request = ClientExecutionRequest::new(&op_id, root.clone(), binary_path.clone())?;

    // Apply the client configuration.
    let apply_result = adapter.apply(&request)?;
    match apply_result.outcome {
        ClientExecutionOutcome::Applied => {
            // Fall through to metadata + verify.
        }
        ClientExecutionOutcome::Conflict
        | ClientExecutionOutcome::ManualRequired
        | ClientExecutionOutcome::Unsupported => {
            // Docset succeeded but client could not start. No client change
            // committed, so there is no rollback metadata to retain.
            let mut observations = docset_observations;
            observations.extend(apply_result.observations);
            return Ok(SetupApplyResult::PartialNoRollback { observations });
        }
        ClientExecutionOutcome::Verified | ClientExecutionOutcome::RolledBack => {
            // Unexpected for apply, but treat as partial-no-rollback.
            let mut observations = docset_observations;
            observations.extend(apply_result.observations);
            return Ok(SetupApplyResult::PartialNoRollback { observations });
        }
    }

    // Record the successful apply so rollback can dispatch deterministically.
    // This is the "trusted setup operation recorded its own successful apply"
    // guard: without this file, setup_rollback refuses to touch the operation.
    // If the metadata cannot be safely persisted, return AppliedButUnverified
    // (contract 6).
    if write_setup_meta(&OperationId::new(&op_id)?, &client_id.to_string()).is_err() {
        let mut observations = docset_observations;
        observations.extend(apply_result.observations);
        observations.push("metadata_persist_failed".to_string());
        return Ok(SetupApplyResult::AppliedButUnverified { observations });
    }

    // Verify the client configuration.
    let verify_result = adapter.verify(&request)?;
    let reload_required = verify_result
        .observations
        .iter()
        .any(|o| o == "client_reload_required");
    let mut all_observations = docset_observations;
    all_observations.extend(apply_result.observations);
    all_observations.extend(verify_result.observations);

    match verify_result.outcome {
        ClientExecutionOutcome::Verified => {
            if reload_required {
                Ok(SetupApplyResult::ClientReloadRequired {
                    operation_id: op_id,
                    observations: all_observations,
                })
            } else {
                Ok(SetupApplyResult::SetupComplete {
                    operation_id: op_id,
                    observations: all_observations,
                })
            }
        }
        _ => {
            // Client change committed but verification did not confirm.
            Ok(SetupApplyResult::Partial {
                operation_id: op_id,
                observations: all_observations,
            })
        }
    }
}

/// Roll back a setup operation by its operation id.
///
/// Dispatches to the owning conditional adapter, determined from the
/// setup-owned metadata file written during apply. Refuses unknown, unsafe, or
/// later-user-edited state without overwriting it. Unsupported/manual-only
/// clients have no automatic rollback.
pub fn setup_rollback(
    operation_id: &str,
    approved_root_path: &Path,
) -> Result<SetupRollbackResult> {
    // Only operation ids generated by setup are accepted.
    if !operation_id.starts_with("setup-") {
        return Ok(SetupRollbackResult::ManualRequired {
            observations: vec!["operation_id_not_generated_by_setup".to_string()],
        });
    }

    let id = OperationId::new(operation_id)?;

    // Read the setup metadata to determine which client owns this operation.
    // This is the "only after the trusted setup operation recorded its own
    // successful apply" guard: without the metadata file, rollback refuses.
    // read_setup_meta must NOT create an operation directory (contract 5).
    let meta = match read_setup_meta(&id) {
        Ok(m) => m,
        Err(_) => {
            return Ok(SetupRollbackResult::ManualRequired {
                observations: vec!["operation_not_recorded_by_setup".to_string()],
            });
        }
    };

    let client_id = parse_client(&meta.client)?;
    let root = approved_root(approved_root_path)?;
    let binary_path = std::env::current_exe().context("resolve nowdocs executable path")?;

    // Find the owning adapter and dispatch to it exclusively.
    let adapter = all_adapters()
        .into_iter()
        .find(|a| a.id() == client_id)
        .context("adapter not found for client")?;

    let caps = adapter.capabilities();
    if caps.apply != CapabilitySupport::Conditional {
        return Ok(SetupRollbackResult::ManualRequired {
            observations: vec!["client_does_not_support_rollback".to_string()],
        });
    }

    let request = ClientExecutionRequest::new(&id.to_string(), root, binary_path)?;
    let result = adapter.rollback(&request)?;
    match result.outcome {
        ClientExecutionOutcome::RolledBack => Ok(SetupRollbackResult::RolledBack {
            observations: result.observations,
        }),
        _ => Ok(SetupRollbackResult::ManualRequired {
            observations: result.observations,
        }),
    }
}

// ---- Private helpers ----

/// Parse a client id string into a `ClientId`.
fn parse_client(s: &str) -> Result<crate::clients::ClientId> {
    use std::str::FromStr;
    crate::clients::ClientId::from_str(s).context("parse client id")
}

/// Capture a `TargetFilePrecondition` fingerprint for the Cursor MCP config
/// using C5's `safe_target` + `read_target`. Rejects target or parent
/// symlinks/non-regular paths rather than treating them as absent.
fn cursor_target_fingerprint(root: &ApprovedRoot) -> Result<Vec<TargetFilePrecondition>> {
    let target = safe_target(root, CURSOR_TARGET_RELATIVE)?;

    match read_target(&target) {
        Ok(bytes) => Ok(vec![TargetFilePrecondition {
            logical_id: CURSOR_TARGET_LOGICAL_ID.to_string(),
            exists: true,
            sha256: Some(compute_digest(&bytes)),
        }]),
        Err(error) if error_is_not_found(&error) => Ok(vec![TargetFilePrecondition {
            logical_id: CURSOR_TARGET_LOGICAL_ID.to_string(),
            exists: false,
            sha256: None,
        }]),
        Err(error) => Err(error).context("refuse unsafe Cursor config target"),
    }
}

fn error_is_not_found(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        cause
            .downcast_ref::<std::io::Error>()
            .is_some_and(|io| io.kind() == std::io::ErrorKind::NotFound)
    })
}

/// True when the client's MCP configuration is canonically verified: the
/// Cursor entry has the current absolute binary and exactly `args: ["serve"]`.
fn is_client_canonically_verified(client: &crate::clients::ClientId, root: &ApprovedRoot) -> bool {
    match client {
        crate::clients::ClientId::Cursor => {
            let target = match safe_target(root, CURSOR_TARGET_RELATIVE) {
                Ok(t) => t,
                Err(_) => return false,
            };
            let bytes = match read_target(&target) {
                Ok(b) => b,
                Err(_) => return false,
            };
            let config: serde_json::Value = match serde_json::from_slice(&bytes) {
                Ok(v) => v,
                Err(_) => return false,
            };
            let binary_path = match std::env::current_exe() {
                Ok(p) => p,
                Err(_) => return false,
            };
            let expected_command = binary_path.display().to_string();
            let nowdocs = config.get("mcpServers").and_then(|s| s.get("nowdocs"));
            let command_matches = nowdocs
                .and_then(|n| n.get("command"))
                .and_then(|c| c.as_str())
                .map(|c| c == expected_command)
                .unwrap_or(false);
            let args_match = nowdocs
                .and_then(|n| n.get("args"))
                .and_then(|a| a.as_array())
                .map(|args| {
                    args.len() == 1 && args[0].as_str().map(|s| s == "serve").unwrap_or(false)
                })
                .unwrap_or(false);
            command_matches && args_match
        }
        // Manual-only clients are never "canonically verified" for
        // already_satisfied purposes (they don't have automatic apply).
        _ => false,
    }
}

/// True when the Cursor config has an existing `nowdocs` entry that is
/// noncanonical (wrong binary or wrong args).
fn cursor_has_noncanonical_entry(client: &crate::clients::ClientId, root: &ApprovedRoot) -> bool {
    if *client != crate::clients::ClientId::Cursor {
        return false;
    }
    let target = match safe_target(root, CURSOR_TARGET_RELATIVE) {
        Ok(t) => t,
        Err(_) => return false,
    };
    let bytes = match read_target(&target) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let config: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let nowdocs = config.get("mcpServers").and_then(|s| s.get("nowdocs"));
    if nowdocs.is_none() {
        return false;
    }
    // An entry exists; check if it is canonical.
    !is_client_canonically_verified(client, root)
}

/// Build a single `AutomationPlan` for a setup covering docset + client.
fn build_setup_plan(
    docset: &str,
    client: &crate::clients::ClientId,
    package: &registry::RegistryPackage,
    precondition: &DocsetPrecondition,
    target_fingerprint: &[TargetFilePrecondition],
    cursor_noncanonical: bool,
    now_unix_secs: u64,
) -> Result<crate::automation::plan::AutomationPlan> {
    let (action_kind, risk, summary_verb) = if precondition.installed {
        ("registry_update", RiskLevel::Mutating, "Update")
    } else {
        ("registry_install", RiskLevel::Additive, "Install")
    };

    let summary = format!(
        "{summary_verb} docset {docset} to version {version} from registry",
        version = package.version
    );

    let install_action =
        docset::package_action("setup-docset", action_kind, risk, &summary, package, None);

    let verify_action = PlannedAction {
        id: "setup-verify-docset".to_string(),
        kind: KIND_VERIFY_DOCSET.to_string(),
        risk: RiskLevel::ReadOnly,
        summary: format!("Verify {docset} installation"),
        changes_state: false,
        network_access: false,
        requires_confirmation: false,
        reversible: true,
        target_paths: vec![],
        estimated_download_bytes: None,
    };

    let mut actions = vec![install_action, verify_action];

    // Determine the client action based on adapter capabilities and config state.
    let adapter = all_adapters()
        .into_iter()
        .find(|a| a.id() == *client)
        .context("adapter not found for client")?;
    let caps = adapter.capabilities();

    if caps.apply == CapabilitySupport::Conditional {
        // For conditional adapters (Cursor, Claude Code): check if the config
        // has a noncanonical nowdocs entry. If so, plan manual guidance instead
        // of client_apply (contract 3).
        let needs_manual = match *client {
            crate::clients::ClientId::Cursor => cursor_noncanonical,
            _ => false,
        };

        if needs_manual {
            let manual_action = PlannedAction {
                id: "setup-client-manual".to_string(),
                kind: KIND_CLIENT_MANUAL_GUIDANCE.to_string(),
                risk: RiskLevel::ReadOnly,
                summary: format!("Manually configure {client} to use nowdocs"),
                changes_state: false,
                network_access: false,
                requires_confirmation: false,
                reversible: true,
                target_paths: vec![format!("client:{client}")],
                estimated_download_bytes: None,
            };
            actions.push(manual_action);
        } else {
            let client_apply = PlannedAction {
                id: "setup-client-apply".to_string(),
                kind: KIND_CLIENT_APPLY.to_string(),
                risk: RiskLevel::Additive,
                summary: format!("Configure {client} to use nowdocs"),
                changes_state: true,
                network_access: false,
                requires_confirmation: true,
                reversible: true,
                target_paths: vec![format!("client:{client}")],
                estimated_download_bytes: None,
            };
            let client_verify = PlannedAction {
                id: "setup-client-verify".to_string(),
                kind: KIND_CLIENT_VERIFY.to_string(),
                risk: RiskLevel::ReadOnly,
                summary: format!("Verify {client} nowdocs registration"),
                changes_state: false,
                network_access: false,
                requires_confirmation: false,
                reversible: true,
                target_paths: vec![format!("client:{client}")],
                estimated_download_bytes: None,
            };
            actions.push(client_apply);
            actions.push(client_verify);
        }
    } else {
        // Manual-only adapters (Claude Desktop, Generic): add
        // client_manual_guidance action (contract 4).
        let manual_action = PlannedAction {
            id: "setup-client-manual".to_string(),
            kind: KIND_CLIENT_MANUAL_GUIDANCE.to_string(),
            risk: RiskLevel::ReadOnly,
            summary: format!("Manually configure {client} to use nowdocs"),
            changes_state: false,
            network_access: false,
            requires_confirmation: false,
            reversible: true,
            target_paths: vec![format!("client:{client}")],
            estimated_download_bytes: None,
        };
        actions.push(manual_action);
    }

    let cache_layout = cache::observe_layout_state().as_str().to_string();
    let inputs = PlanInputs {
        client: Some(client.to_string()),
        docset: Some(docset.to_string()),
        online: true,
    };
    let preconditions = PlanPreconditions {
        cache_layout,
        model_present: false,
        docset_state: vec![precondition.clone()],
        target_files: target_fingerprint.to_vec(),
    };

    new_plan(inputs, preconditions, actions, now_unix_secs)
}

/// Write setup-owned operation metadata (client id) alongside the C5 journal.
/// This records that setup successfully applied a client change, which
/// `setup_rollback` checks before dispatching.
///
/// Uses no-follow final-file opens, verifies regular files, uses `0600`, and
/// rejects pre-existing metadata (contract 5). On non-Unix, fails closed.
fn write_setup_meta(id: &OperationId, client: &str) -> Result<()> {
    let dir = crate::automation::operation::init_operation_dir(id)?;
    let path = dir.join(SETUP_META_FILENAME);

    // Reject pre-existing metadata: a symlink or regular file at this path
    // means the operation was already recorded or tampered with.
    if let Ok(meta) = std::fs::symlink_metadata(&path) {
        if meta.file_type().is_symlink() {
            anyhow::bail!("setup metadata path is a symlink (refused)");
        }
        if meta.is_file() {
            anyhow::bail!("setup metadata already exists (refused to overwrite)");
        }
    }

    let meta = SetupMeta {
        client: client.to_string(),
    };
    let bytes = serde_json::to_vec_pretty(&meta).context("serialize setup meta")?;

    // Write with no-follow open and 0600 on Unix. Fail closed on non-Unix.
    write_meta_nofollow(&path, &bytes)?;
    Ok(())
}

/// Read setup-owned operation metadata. Returns an error if the file is absent
/// (the operation was not recorded by setup) or malformed. Does NOT create an
/// operation directory (contract 5).
fn read_setup_meta(id: &OperationId) -> Result<SetupMeta> {
    let path = crate::automation::operation::operations_root()
        .join(id.to_string())
        .join(SETUP_META_FILENAME);

    // Use no-follow read. If the file doesn't exist, the operation was not
    // recorded by setup.
    let bytes = read_meta_nofollow(&path)?;
    let meta: SetupMeta = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse setup meta {}", path.display()))?;
    Ok(meta)
}

/// Write a metadata file with no-follow open and 0600 on Unix. Fail closed on
/// non-Unix (contract 5).
#[cfg(unix)]
fn write_meta_nofollow(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    use std::os::unix::io::AsRawFd;

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .with_context(|| format!("create (O_NOFOLLOW) setup meta {}", path.display()))?;

    use std::io::Write;
    file.write_all(bytes)
        .with_context(|| format!("write setup meta {}", path.display()))?;
    file.flush()
        .with_context(|| format!("flush setup meta {}", path.display()))?;

    let rc = unsafe { libc::fchmod(file.as_raw_fd(), 0o600) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error())
            .with_context(|| format!("fchmod 0600 setup meta {}", path.display()));
    }
    Ok(())
}

#[cfg(not(unix))]
fn write_meta_nofollow(path: &Path, _bytes: &[u8]) -> Result<()> {
    anyhow::bail!(
        "unsupported platform for no-follow I/O at {}",
        path.display()
    );
}

/// Read a metadata file with no-follow open on Unix. Fail closed on non-Unix.
#[cfg(unix)]
fn read_meta_nofollow(path: &Path) -> Result<Vec<u8>> {
    use std::os::unix::fs::OpenOptionsExt;

    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .with_context(|| format!("open (O_NOFOLLOW) setup meta {}", path.display()))?;

    let meta = file
        .metadata()
        .with_context(|| format!("fstat setup meta {}", path.display()))?;
    if !meta.is_file() {
        anyhow::bail!("setup meta {} is not a regular file", path.display());
    }

    use std::io::Read;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .with_context(|| format!("read setup meta {}", path.display()))?;
    Ok(buf)
}

#[cfg(not(unix))]
fn read_meta_nofollow(path: &Path) -> Result<Vec<u8>> {
    anyhow::bail!(
        "unsupported platform for no-follow I/O at {}",
        path.display()
    );
}
