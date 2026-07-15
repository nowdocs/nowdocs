//! Registry-aware docset ensure planning and idempotent apply.
//!
//! C4 owns the `nowdocs ensure <docset>` flow: offline planning returns
//! `registry_metadata_required` or `already_satisfied`; `--online` fetches only
//! the selected package metadata and persists it in a C3 typed plan; and
//! `--apply <plan-hash>` delegates install/update to the existing registry
//! services and verifies the result. Repeated success is `already_satisfied`
//! with no redownload or rewrite.

use anyhow::{Context, Result};

use crate::agent_contract::RiskLevel;
use crate::automation::lock;
use crate::automation::plan::{
    load_plan, new_plan, store_plan, DocsetPrecondition, PlanInputs, PlanPreconditions,
    PlannedAction,
};
use crate::cache::{self, InstalledDocsetState};
use crate::input;
use crate::registry::{self, RegistryPackage};

/// Outcome of `ensure_plan`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnsurePlanResult {
    /// The docset is already installed with the desired registry version.
    AlreadySatisfied { precondition: DocsetPrecondition },
    /// A plan was created; the caller must approve and apply `plan_id`.
    PlanCreated {
        plan_id: String,
        precondition: DocsetPrecondition,
    },
    /// Offline planning cannot determine registry state; run with `--online`.
    RegistryMetadataRequired { precondition: DocsetPrecondition },
}

/// Outcome of `ensure_apply`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnsureApplyResult {
    /// The requested state was already present; no write occurred.
    AlreadySatisfied,
    /// The plan was applied and the docset is healthy.
    Applied,
}

/// Stable prefixes used to store selected-package metadata inside a C3
/// `PlannedAction`'s `target_paths`. The strings are sorted so plan hashing is
/// deterministic, and each prefix is unambiguous so apply can decode them.
const PKG_VERSION_PREFIX: &str = "pkg:version:";
const PKG_URL_PREFIX: &str = "pkg:url:";
const PKG_SHA_PREFIX: &str = "pkg:sha:";

/// Plan an idempotent ensure for `docset`.
///
/// - Offline (`online == false`): returns `already_satisfied` if the docset is
///   installed, otherwise `registry_metadata_required`. No cache/model/network
///   mutation occurs.
/// - Online (`online == true`): fetches only the selected package metadata from
///   the trusted registry index, creates a C3 typed plan if install/update is
///   required, or returns `already_satisfied` if the installed version already
///   matches. The full index bytes are never persisted.
pub fn ensure_plan(docset: &str, online: bool, now_unix_secs: u64) -> Result<EnsurePlanResult> {
    let docset = input::validate_docset(docset)?;
    let state = cache::check_docset_state_pure(&docset);
    let precondition = docset_precondition(&docset, &state);

    if !online {
        return match state {
            InstalledDocsetState::Healthy => {
                Ok(EnsurePlanResult::AlreadySatisfied { precondition })
            }
            _ => Ok(EnsurePlanResult::RegistryMetadataRequired { precondition }),
        };
    }

    let package = registry::fetch_selected_package(&docset)?;

    if state == InstalledDocsetState::Healthy {
        let installed_version = cache::read_docset_meta(&docset).0;
        if installed_version == package.version {
            return Ok(EnsurePlanResult::AlreadySatisfied { precondition });
        }
    }

    let plan = build_ensure_plan(&docset, &package, &precondition, now_unix_secs)?;
    let plan_id = store_plan(&plan)?;
    Ok(EnsurePlanResult::PlanCreated {
        plan_id,
        precondition,
    })
}

/// Apply a stored ensure plan.
///
/// Revalidates the plan (not expired/tampered), checks that the requested
/// docset matches the plan, revalidates the docset precondition, acquires C3's
/// global operation lock before the per-docset install lock, and delegates the
/// install/update to `registry::install_with_sha256`. If the desired state is
/// already present, returns `already_satisfied` without redownload or rewrite.
pub fn ensure_apply(docset: &str, plan_id: &str, now_unix_secs: u64) -> Result<EnsureApplyResult> {
    let docset = input::validate_docset(docset)?;
    let plan = load_plan(plan_id, now_unix_secs)?;

    // The requested docset must match the plan scope.
    if plan.inputs.docset.as_deref() != Some(&docset) {
        anyhow::bail!(
            "PLAN_STALE: plan scope docset {:?} does not match requested docset {:?}",
            plan.inputs.docset,
            docset
        );
    }

    let planned_precondition = plan
        .preconditions
        .docset_state
        .first()
        .context("plan missing docset precondition")?;

    // If the desired state is already achieved, skip work entirely.
    if let Some(package) = desired_package_from_plan(&plan) {
        if is_already_satisfied(&docset, &package) {
            return Ok(EnsureApplyResult::AlreadySatisfied);
        }
    }

    // Revalidate precondition: if state drifted and the goal is not already met,
    // the plan is stale.
    let current_state = cache::check_docset_state_pure(&docset);
    let current_precondition = docset_precondition(&docset, &current_state);
    if &current_precondition != planned_precondition {
        if let Some(package) = desired_package_from_plan(&plan) {
            if is_already_satisfied(&docset, &package) {
                return Ok(EnsureApplyResult::AlreadySatisfied);
            }
        }
        anyhow::bail!(
            "PLAN_STALE: docset precondition changed since plan was created \
             (planned {:?}, current {:?})",
            planned_precondition,
            current_precondition
        );
    }

    // Acquire C3's global operation lock before the legacy per-docset lock.
    // The lock id must be <= 64 ASCII chars, so truncate the plan hash.
    let op_id = format!("ensure-{}", &plan_id[..12]);
    let _op_lock = lock::acquire_operation_lock(&op_id)?;

    // Re-check state after taking the global lock.
    let current_state = cache::check_docset_state_pure(&docset);
    let current_precondition = docset_precondition(&docset, &current_state);
    if &current_precondition != planned_precondition {
        if let Some(package) = desired_package_from_plan(&plan) {
            if is_already_satisfied(&docset, &package) {
                return Ok(EnsureApplyResult::AlreadySatisfied);
            }
        }
        anyhow::bail!("PLAN_STALE: docset precondition changed after lock acquisition");
    }

    let package =
        desired_package_from_plan(&plan).context("plan missing selected package metadata")?;

    // Already satisfied after lock acquisition?
    if is_already_satisfied(&docset, &package) {
        return Ok(EnsureApplyResult::AlreadySatisfied);
    }

    // Delegate to the existing transactional registry service.
    registry::install_verified_package(&package)
        .with_context(|| format!("install docset {docset}"))?;

    // Verify the result.
    let final_state = cache::check_docset_state(&docset);
    if final_state != InstalledDocsetState::Healthy || !is_already_satisfied(&docset, &package) {
        anyhow::bail!(
            "VERIFICATION_FAILED: docset {docset} does not match the planned package after apply: {}",
            final_state.label(),
        );
    }

    Ok(EnsureApplyResult::Applied)
}

/// Capture a `DocsetPrecondition` fingerprint for the current docset state.
fn docset_precondition(docset: &str, state: &InstalledDocsetState) -> DocsetPrecondition {
    let installed = *state != InstalledDocsetState::NotInstalled;
    let manifest_sha256 = if *state == InstalledDocsetState::Healthy {
        manifest_sha256(docset).ok()
    } else {
        None
    };
    DocsetPrecondition {
        docset: docset.to_string(),
        installed,
        manifest_sha256,
    }
}

/// SHA-256 of the installed manifest.json file, if present.
fn manifest_sha256(docset: &str) -> Result<String> {
    let path = cache::manifest_path(docset);
    let bytes = std::fs::read(&path).with_context(|| format!("read manifest for {docset}"))?;
    Ok(registry::sha256_hex(&bytes))
}

/// Build a C3 `AutomationPlan` for installing/updating `docset` to `package`.
fn build_ensure_plan(
    docset: &str,
    package: &RegistryPackage,
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
        package_action("ensure-install", action_kind, risk, &summary, package, None);

    let verify_action = PlannedAction {
        id: "ensure-verify".to_string(),
        kind: "verify_docset".to_string(),
        risk: RiskLevel::ReadOnly,
        summary: format!("Verify {docset} installation"),
        changes_state: false,
        network_access: false,
        requires_confirmation: false,
        reversible: true,
        target_paths: vec![],
        estimated_download_bytes: None,
    };

    let cache_layout = cache::observe_layout_state().as_str().to_string();
    let inputs = PlanInputs {
        client: None,
        docset: Some(docset.to_string()),
        online: true,
    };
    let preconditions = PlanPreconditions {
        cache_layout,
        model_present: false,
        docset_state: vec![precondition.clone()],
        target_files: vec![],
    };

    new_plan(
        inputs,
        preconditions,
        vec![install_action, verify_action],
        now_unix_secs,
    )
}

/// Build a `PlannedAction` whose `target_paths` encode the selected package
/// metadata deterministically.
fn package_action(
    id: &str,
    kind: &str,
    risk: RiskLevel,
    summary: &str,
    package: &RegistryPackage,
    estimated_download_bytes: Option<u64>,
) -> PlannedAction {
    let mut target_paths = vec![
        format!("{}{}", PKG_VERSION_PREFIX, package.version),
        format!("{}{}", PKG_URL_PREFIX, package.download_url),
        format!("{}{}", PKG_SHA_PREFIX, package.sha256),
    ];
    target_paths.sort();

    PlannedAction {
        id: id.to_string(),
        kind: kind.to_string(),
        risk,
        summary: summary.to_string(),
        changes_state: risk_implies_state_change(risk),
        network_access: true,
        requires_confirmation: true,
        reversible: true,
        target_paths,
        estimated_download_bytes,
    }
}

/// True when `risk` implies state mutation (mirrors C3 normalization rule).
fn risk_implies_state_change(risk: RiskLevel) -> bool {
    matches!(risk, RiskLevel::Additive | RiskLevel::Mutating)
}

/// Decode the selected package metadata stored in a plan's install/update action.
fn desired_package_from_plan(
    plan: &crate::automation::plan::AutomationPlan,
) -> Option<RegistryPackage> {
    let action = plan
        .actions
        .iter()
        .find(|a| a.kind == "registry_install" || a.kind == "registry_update")?;
    let docset = plan.inputs.docset.as_deref().unwrap_or("unknown");
    decode_package_action(action, docset).ok()
}

/// Decode package metadata from the `target_paths` of a registry install/update
/// action.
fn decode_package_action(action: &PlannedAction, docset: &str) -> Result<RegistryPackage> {
    let mut version = None;
    let mut download_url = None;
    let mut sha256 = None;
    for tp in &action.target_paths {
        if let Some(v) = tp.strip_prefix(PKG_VERSION_PREFIX) {
            version = Some(v.to_string());
        } else if let Some(u) = tp.strip_prefix(PKG_URL_PREFIX) {
            download_url = Some(u.to_string());
        } else if let Some(s) = tp.strip_prefix(PKG_SHA_PREFIX) {
            sha256 = Some(s.to_string());
        }
    }
    Ok(RegistryPackage {
        docset: docset.to_string(),
        version: version.context("missing pkg:version target_path")?,
        license: "MIT".to_string(),
        chunk_count: 0,
        freshness: String::new(),
        download_url: download_url.context("missing pkg:url target_path")?,
        sha256: sha256.context("missing pkg:sha target_path")?,
        description: None,
    })
}

/// True when the docset is installed and its version matches the desired
/// package version.
fn is_already_satisfied(docset: &str, package: &RegistryPackage) -> bool {
    if cache::check_docset_state_pure(docset) != InstalledDocsetState::Healthy {
        return false;
    }
    let installed_version = cache::read_docset_meta(docset).0;
    installed_version == package.version
}

// Keep agent_contract import for RiskLevel re-export stability.
#[allow(unused_imports)]
use crate::agent_contract as _agent_contract;
