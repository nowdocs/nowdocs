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
use crate::automation::operation::{init_operation_dir, OperationId};
use crate::automation::plan::{
    load_plan, new_plan, store_plan, DocsetPrecondition, PlanInputs, PlanPreconditions,
    PlannedAction,
};
use crate::cache::{self, InstalledDocsetState};
use crate::clients::{all_adapters, approved_root, ClientExecutionOutcome, ClientExecutionRequest};
use crate::input;
use crate::registry;

/// Stable action kind for a client apply step.
const KIND_CLIENT_APPLY: &str = "client_apply";
/// Stable action kind for a client verify step.
const KIND_CLIENT_VERIFY: &str = "client_verify";
/// Stable action kind for a docset verification step.
const KIND_VERIFY_DOCSET: &str = "verify_docset";

/// Filename for setup-owned operation metadata (client id), stored alongside
/// the C5 operation journal. This is the "trusted setup operation recorded its
/// own successful apply" record that guards rollback dispatch.
const SETUP_META_FILENAME: &str = "setup-meta.json";

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
///   installed, otherwise `registry_metadata_required`. No cache/model/network
///   mutation occurs.
/// - Online (`online == true`): fetches the selected package metadata, creates
///   one `AutomationPlan` with docset install/update + verify, followed by
///   client apply + verify for conditional adapters. Claude Desktop and Generic
///   produce docset-only plans (no client write actions).
pub fn setup_plan(
    docset: &str,
    client: &str,
    online: bool,
    now_unix_secs: u64,
) -> Result<SetupPlanResult> {
    let docset_id = input::validate_docset(docset)?;
    let client_id = parse_client(client)?;

    let state = cache::check_docset_state_pure(&docset_id);
    let precondition = docset::docset_precondition(&docset_id, &state);

    if !online {
        return match state {
            InstalledDocsetState::Healthy => Ok(SetupPlanResult::AlreadySatisfied { precondition }),
            _ => Ok(SetupPlanResult::RegistryMetadataRequired { precondition }),
        };
    }

    let package = registry::fetch_selected_package(&docset_id)?;

    if state == InstalledDocsetState::Healthy {
        let installed_version = cache::read_docset_meta(&docset_id).0;
        if installed_version == package.version {
            return Ok(SetupPlanResult::AlreadySatisfied { precondition });
        }
    }

    let plan = build_setup_plan(
        &docset_id,
        &client_id,
        &package,
        &precondition,
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

    // If the client has no conditional apply capability, this is a manual-only
    // client (Claude Desktop, Generic). Return ActionRequired with manual guidance.
    if caps.apply != CapabilitySupport::Conditional {
        return Ok(SetupApplyResult::ActionRequired {
            observations: docset_observations,
        });
    }

    // Resolve the nowdocs binary path at apply time.
    let binary_path = std::env::current_exe().context("resolve nowdocs executable path")?;
    if !binary_path.is_absolute() {
        anyhow::bail!(
            "nowdocs executable path is not absolute: {}",
            binary_path.display()
        );
    }

    // Validate the approved root.
    let root = approved_root(approved_root_path)?;

    // Construct the execution request.
    let request = ClientExecutionRequest::new(&op_id, root.clone(), binary_path.clone())?;

    // Apply the client configuration.
    let apply_result = adapter.apply(&request)?;
    match apply_result.outcome {
        ClientExecutionOutcome::Applied => {
            // Fall through to verify.
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
    write_setup_meta(&OperationId::new(&op_id)?, &client_id.to_string())?;

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

/// Build a single `AutomationPlan` for a setup covering docset + client.
fn build_setup_plan(
    docset: &str,
    client: &crate::clients::ClientId,
    package: &registry::RegistryPackage,
    precondition: &DocsetPrecondition,
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

    // Add client apply + verify actions only for conditional adapters.
    let adapter = all_adapters()
        .into_iter()
        .find(|a| a.id() == *client)
        .context("adapter not found for client")?;
    let caps = adapter.capabilities();

    if caps.apply == CapabilitySupport::Conditional {
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
        target_files: vec![],
    };

    new_plan(inputs, preconditions, actions, now_unix_secs)
}

/// Write setup-owned operation metadata (client id) alongside the C5 journal.
/// This records that setup successfully applied a client change, which
/// `setup_rollback` checks before dispatching.
fn write_setup_meta(id: &OperationId, client: &str) -> Result<()> {
    let dir = init_operation_dir(id)?;
    let path = dir.join(SETUP_META_FILENAME);
    let meta = SetupMeta {
        client: client.to_string(),
    };
    let bytes = serde_json::to_vec_pretty(&meta).context("serialize setup meta")?;
    // The operation directory is already private (0700) and verified by C5's
    // init_operation_dir. Writing with std::fs::write is safe here because the
    // parent directory was created single-component with no symlink races.
    std::fs::write(&path, &bytes)
        .with_context(|| format!("write setup meta {}", path.display()))?;
    Ok(())
}

/// Read setup-owned operation metadata. Returns an error if the file is absent
/// (the operation was not recorded by setup) or malformed.
fn read_setup_meta(id: &OperationId) -> Result<SetupMeta> {
    let dir = init_operation_dir(id)?;
    let path = dir.join(SETUP_META_FILENAME);
    let bytes =
        std::fs::read(&path).with_context(|| format!("read setup meta {}", path.display()))?;
    let meta: SetupMeta = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse setup meta {}", path.display()))?;
    Ok(meta)
}
