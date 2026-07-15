//! Offline verification: report whether a named docset can be searched locally
//! and whether an optional client configuration needs a reload.
//!
//! C8/C8-R1 contract (see the durable spec). Verification is offline-safe:
//! - It never downloads a model, fetches registry metadata, or starts a
//!   network request. The model is loaded through the cached-only API
//!   (`embedder::load_default_cached_only`), which resolves the pinned revision
//!   exclusively through the local hf-hub cache and cannot download.
//! - It never changes client configuration or an installed docset.
//! - The library entry point accepts only explicit validated inputs and never
//!   resolves HOME/XDG itself; `main.rs` resolves an approved root only at the
//!   binary boundary when `--client` is present.
//!
//! Output never exposes chunk/query text, local paths, raw configuration
//! bytes, environment values, or arbitrary underlying error text.

use std::path::Path;
use std::str::FromStr;

use serde_json::json;

use crate::agent_contract::{
    AgentEnvelope, AgentStatus, NextAction, ResultCode, AGENT_CONTRACT_SCHEMA_VERSION,
};
use crate::cache::{self, InstalledDocsetState};
use crate::clients::{self, ClientExecutionOutcome, ClientExecutionRequest, ClientId};
use crate::embedder;
use crate::retrieve;

/// Fixed internal verification query. The query text is never exposed in the
/// envelope (contract §5); it only exercises the real retrieval path.
const VERIFICATION_QUERY: &str = "installation configuration example";

/// Structured outcome of a `nowdocs verify` invocation. Carries the
/// agent-contract fields. The exit code is derived authoritatively from
/// `self.code.exit_code()` via [`VerifyResult::exit_code`]; there is no
/// independently mutable exit field, so code and exit can never drift.
#[derive(Debug, Clone)]
pub struct VerifyResult {
    pub status: AgentStatus,
    pub code: ResultCode,
    pub summary: String,
    pub data: serde_json::Value,
    pub next_actions: Vec<NextAction>,
}

impl VerifyResult {
    fn new(status: AgentStatus, code: ResultCode, summary: &str) -> Self {
        Self {
            status,
            code,
            summary: summary.to_string(),
            data: json!({}),
            next_actions: Vec::new(),
        }
    }

    /// The authoritative process exit code, derived only from
    /// `self.code.exit_code()`. Required mappings (contract §2):
    /// `ready`, `action_required`, `client_reload_required` -> 0;
    /// `invalid_request` -> 2; `model_missing`, `docset_missing`,
    /// `docset_corrupt`, `client_not_detected`, `verification_failed`,
    /// `internal_error` -> 20.
    pub fn exit_code(&self) -> u8 {
        self.code.exit_code()
    }

    /// Build the single `AgentEnvelope` document for `verify --json`.
    pub fn to_envelope(&self) -> AgentEnvelope<&serde_json::Value> {
        AgentEnvelope {
            schema_version: AGENT_CONTRACT_SCHEMA_VERSION,
            nowdocs_version: env!("CARGO_PKG_VERSION").to_string(),
            command: "verify".to_string(),
            status: self.status,
            code: self.code,
            summary: self.summary.clone(),
            data: &self.data,
            next_actions: self.next_actions.clone(),
            rollback: None,
        }
    }
}

/// Run offline verification for a docset and an optional client.
///
/// `docset` and `client` are validated before any path is constructed
/// (contract §2). Invalid input returns `invalid_request` (exit 2) and creates
/// no directories. `approved_root` is the resolved client-configuration root;
/// it is only consulted when `client` is `Some`, and is supplied by the binary
/// boundary (never resolved from HOME/XDG by the library).
pub fn verify(docset: &str, client: Option<&str>, approved_root: Option<&Path>) -> VerifyResult {
    // §2: validate identifiers before constructing any path.
    if crate::input::validate_docset(docset).is_err() {
        return VerifyResult::new(
            AgentStatus::ActionRequired,
            ResultCode::InvalidRequest,
            "invalid docset identifier",
        );
    }
    let client_id = match client {
        Some(c) => match ClientId::from_str(c) {
            Ok(id) => Some(id),
            Err(_) => {
                return VerifyResult::new(
                    AgentStatus::ActionRequired,
                    ResultCode::InvalidRequest,
                    "invalid client identifier",
                );
            }
        },
        None => None,
    };

    // §4: check prerequisites before retrieval. No model load/download here.
    match check_prerequisites(docset) {
        Prerequisite::Ok => {}
        Prerequisite::DocsetMissing => {
            return VerifyResult::new(
                AgentStatus::ActionRequired,
                ResultCode::DocsetMissing,
                "docset is not installed",
            );
        }
        Prerequisite::DocsetCorrupt => {
            return VerifyResult::new(
                AgentStatus::ActionRequired,
                ResultCode::DocsetCorrupt,
                "docset is installed but corrupt",
            );
        }
        Prerequisite::ModelMissing => {
            return VerifyResult::new(
                AgentStatus::ActionRequired,
                ResultCode::ModelMissing,
                "the pinned embedding model is not available locally",
            );
        }
    }

    // §5 + C8-R1: load the default model through the cached-only API before
    // calling retrieve::search. The cached-only path resolves the pinned
    // revision exclusively through the local hf-hub cache; it cannot download
    // or write. The loaded handle enters the in-process cache, so retrieve's
    // subsequent Embedder::load() is a cache hit and cannot reach its
    // downloader. If cached-only loading fails, return model_missing (the
    // model is not locally usable) with redacted output.
    if embedder::load_default_cached_only().is_err() {
        return VerifyResult::new(
            AgentStatus::ActionRequired,
            ResultCode::ModelMissing,
            "the pinned embedding model is not available locally",
        );
    }

    // §5: prerequisites are local and healthy. Call the existing real retrieval
    // path with the fixed internal verification query.
    match retrieve::search(docset, VERIFICATION_QUERY, None, None) {
        Ok(_result) => {
            // Retrieval success. Do not expose chunk/query text (contract §5).
            let mut outcome = VerifyResult::new(
                AgentStatus::Ok,
                ResultCode::Ready,
                "docset is searchable locally",
            );
            outcome.data = json!({ "docset": docset });
            // §6: with --client, verify the client configuration too.
            if let Some(id) = client_id {
                apply_client_verification(&mut outcome, id, approved_root, docset);
            }
            outcome
        }
        Err(_e) => {
            // Retrieval failure: redacted output, no chunk/query/paths/errors.
            let mut outcome = VerifyResult::new(
                AgentStatus::ActionRequired,
                ResultCode::VerificationFailed,
                "retrieval verification failed",
            );
            outcome.data = json!({ "docset": docset });
            // Client verification is skipped when docset retrieval failed.
            outcome
        }
    }
}

/// The prerequisite state for a docset + pinned model. Checks are offline and
/// never load or download the model.
enum Prerequisite {
    Ok,
    DocsetMissing,
    DocsetCorrupt,
    ModelMissing,
}

/// Check docset and model prerequisites without loading the model
/// (contract §4). Docset state uses the unified on-disk classifier.
fn check_prerequisites(docset: &str) -> Prerequisite {
    match cache::check_docset_state(docset) {
        InstalledDocsetState::NotInstalled => Prerequisite::DocsetMissing,
        InstalledDocsetState::ManifestOnly
        | InstalledDocsetState::StoreOnly
        | InstalledDocsetState::SchemaMismatch
        | InstalledDocsetState::RowCountMismatch => Prerequisite::DocsetCorrupt,
        InstalledDocsetState::Healthy => {
            // The docset is healthy; now check the pinned model is available
            // locally without loading or downloading it. default_model_cached()
            // and load_default_cached_only() share the same CacheRepo::get
            // resolution rules (C8-R1 finding #6).
            if embedder::default_model_cached() {
                Prerequisite::Ok
            } else {
                Prerequisite::ModelMissing
            }
        }
    }
}

/// Pure mapping from a client adapter execution outcome to a
/// (status, code, summary) triple. This is the single source of truth for
/// client-result mapping; tests exercise it directly without touching real
/// client config.
///
/// - `Verified` without reload -> `ready`, exit 0.
/// - `Verified` with reload -> `client_reload_required`, exit 0.
/// - `ManualRequired`, `Unsupported`, `Conflict`, `Applied`, `RolledBack` ->
///   `action_required`, exit 0 (truthful; never claims a reload occurred).
fn map_client_outcome(
    outcome: ClientExecutionOutcome,
    reload_required: bool,
) -> (AgentStatus, ResultCode, &'static str) {
    match outcome {
        ClientExecutionOutcome::Verified => {
            if reload_required {
                (
                    AgentStatus::Ok,
                    ResultCode::ClientReloadRequired,
                    "docset is searchable; client reload required",
                )
            } else {
                (
                    AgentStatus::Ok,
                    ResultCode::Ready,
                    "docset is searchable; client verified",
                )
            }
        }
        ClientExecutionOutcome::ManualRequired
        | ClientExecutionOutcome::Conflict
        | ClientExecutionOutcome::Unsupported
        | ClientExecutionOutcome::Applied
        | ClientExecutionOutcome::RolledBack => (
            AgentStatus::ActionRequired,
            ResultCode::ActionRequired,
            "docset is searchable; client requires manual action",
        ),
    }
}

/// Apply client adapter verification (contract §6) without calling
/// apply/rollback/generate. Updates `outcome` in place.
fn apply_client_verification(
    outcome: &mut VerifyResult,
    client_id: ClientId,
    approved_root: Option<&Path>,
    docset: &str,
) {
    let root = match approved_root {
        Some(p) => p,
        None => {
            // No approved root supplied: the client cannot be verified.
            set_client_result(
                outcome,
                AgentStatus::ActionRequired,
                ResultCode::ClientNotDetected,
                "docset is searchable; client root not available",
                docset,
                client_id,
            );
            return;
        }
    };

    let approved = match clients::approved_root(root) {
        Ok(r) => r,
        Err(_) => {
            // Absent/unusable root is a redacted observation (contract §3).
            set_client_result(
                outcome,
                AgentStatus::ActionRequired,
                ResultCode::ClientNotDetected,
                "docset is searchable; client root not available",
                docset,
                client_id,
            );
            return;
        }
    };

    let adapter = clients::all_adapters()
        .into_iter()
        .find(|a| a.id() == client_id);

    let Some(adapter) = adapter else {
        set_client_result(
            outcome,
            AgentStatus::ActionRequired,
            ResultCode::ClientNotDetected,
            "docset is searchable; client not supported",
            docset,
            client_id,
        );
        return;
    };

    // Verify needs an absolute binary path; current_exe is the nowdocs binary.
    let binary_path = match std::env::current_exe() {
        Ok(p) if p.is_absolute() => p,
        _ => {
            set_client_result(
                outcome,
                AgentStatus::ActionRequired,
                ResultCode::InternalError,
                "docset is searchable; client verification unavailable",
                docset,
                client_id,
            );
            return;
        }
    };

    let request = match ClientExecutionRequest::new("verify", approved, binary_path) {
        Ok(r) => r,
        Err(_) => {
            set_client_result(
                outcome,
                AgentStatus::ActionRequired,
                ResultCode::InternalError,
                "docset is searchable; client verification unavailable",
                docset,
                client_id,
            );
            return;
        }
    };

    let verify_result = match adapter.verify(&request) {
        Ok(r) => r,
        Err(_) => {
            set_client_result(
                outcome,
                AgentStatus::ActionRequired,
                ResultCode::VerificationFailed,
                "docset is searchable; client verification failed",
                docset,
                client_id,
            );
            return;
        }
    };

    // Map the adapter outcome through the pure mapping helper. Observations are
    // redacted: only the client id and docset are carried in `data`.
    let reload_required = verify_result
        .observations
        .iter()
        .any(|o| o == "client_reload_required");
    let (status, code, summary) = map_client_outcome(verify_result.outcome, reload_required);
    set_client_result(outcome, status, code, summary, docset, client_id);
}

/// Set the outcome fields for a client-verification result. This is the only
/// place client-verification state transitions happen, so code and exit (via
/// `exit_code()`) can never drift.
fn set_client_result(
    outcome: &mut VerifyResult,
    status: AgentStatus,
    code: ResultCode,
    summary: &str,
    docset: &str,
    client_id: ClientId,
) {
    outcome.status = status;
    outcome.code = code;
    outcome.summary = summary.to_string();
    outcome.data = json!({ "docset": docset, "client": client_id.as_str() });
}

/// Format the verify result as concise, English-only human output. Never
/// prints raw errors or paths (contract §7).
pub fn format_human(result: &VerifyResult) -> String {
    let mut out = String::new();
    let status_str = match result.status {
        AgentStatus::Ok => "ok",
        AgentStatus::Warning => "warning",
        AgentStatus::ActionRequired => "action required",
        AgentStatus::Partial => "partial",
        AgentStatus::Error => "error",
    };
    out.push_str(&format!("verify: {status_str}\n"));
    out.push_str(&format!("code: {}\n", snake_code(result.code)));
    out.push_str(&format!("{}\n", result.summary));
    out
}

/// Render a `ResultCode` as its snake_case JSON string for human output.
fn snake_code(code: ResultCode) -> &'static str {
    match code {
        ResultCode::Ready => "ready",
        ResultCode::AlreadySatisfied => "already_satisfied",
        ResultCode::ActionRequired => "action_required",
        ResultCode::SetupComplete => "setup_complete",
        ResultCode::ModelMissing => "model_missing",
        ResultCode::DocsetMissing => "docset_missing",
        ResultCode::DocsetCorrupt => "docset_corrupt",
        ResultCode::RegistryMetadataRequired => "registry_metadata_required",
        ResultCode::ClientNotDetected => "client_not_detected",
        ResultCode::ClientReloadRequired => "client_reload_required",
        ResultCode::ConfigParseFailed => "config_parse_failed",
        ResultCode::ConfigConflict => "config_conflict",
        ResultCode::ConfigWriteUnsafe => "config_write_unsafe",
        ResultCode::PlanNotFound => "plan_not_found",
        ResultCode::PlanExpired => "plan_expired",
        ResultCode::PlanStale => "plan_stale",
        ResultCode::PlanTampered => "plan_tampered",
        ResultCode::OperationInProgress => "operation_in_progress",
        ResultCode::PermissionDenied => "permission_denied",
        ResultCode::NetworkUnavailable => "network_unavailable",
        ResultCode::VerificationFailed => "verification_failed",
        ResultCode::AppliedButUnverified => "applied_but_unverified",
        ResultCode::UnsupportedPlatform => "unsupported_platform",
        ResultCode::InvalidRequest => "invalid_request",
        ResultCode::InternalError => "internal_error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pure client-outcome mapper must produce the locked code/exit
    /// mappings for every adapter outcome.
    #[test]
    fn map_client_outcome_verified_no_reload_is_ready_exit_0() {
        let (status, code, _) = map_client_outcome(ClientExecutionOutcome::Verified, false);
        assert_eq!(status, AgentStatus::Ok);
        assert_eq!(code, ResultCode::Ready);
        assert_eq!(code.exit_code(), 0);
    }

    #[test]
    fn map_client_outcome_verified_reload_is_client_reload_required_exit_0() {
        let (status, code, _) = map_client_outcome(ClientExecutionOutcome::Verified, true);
        assert_eq!(status, AgentStatus::Ok);
        assert_eq!(code, ResultCode::ClientReloadRequired);
        assert_eq!(code.exit_code(), 0);
    }

    #[test]
    fn map_client_outcome_manual_required_is_action_required_exit_0() {
        let (status, code, _) = map_client_outcome(ClientExecutionOutcome::ManualRequired, false);
        assert_eq!(status, AgentStatus::ActionRequired);
        assert_eq!(code, ResultCode::ActionRequired);
        assert_eq!(code.exit_code(), 0);
    }

    #[test]
    fn map_client_outcome_unsupported_is_action_required_exit_0() {
        let (status, code, _) = map_client_outcome(ClientExecutionOutcome::Unsupported, false);
        assert_eq!(status, AgentStatus::ActionRequired);
        assert_eq!(code, ResultCode::ActionRequired);
        assert_eq!(code.exit_code(), 0);
    }

    #[test]
    fn map_client_outcome_conflict_is_action_required_exit_0() {
        let (status, code, _) = map_client_outcome(ClientExecutionOutcome::Conflict, false);
        assert_eq!(status, AgentStatus::ActionRequired);
        assert_eq!(code, ResultCode::ActionRequired);
        assert_eq!(code.exit_code(), 0);
    }

    /// exit_code() is always derived from code.exit_code(); they can never
    /// differ for any ResultCode.
    #[test]
    fn exit_code_always_equals_code_exit_code() {
        for code in [
            ResultCode::Ready,
            ResultCode::ActionRequired,
            ResultCode::ClientReloadRequired,
            ResultCode::InvalidRequest,
            ResultCode::ModelMissing,
            ResultCode::DocsetMissing,
            ResultCode::DocsetCorrupt,
            ResultCode::ClientNotDetected,
            ResultCode::VerificationFailed,
            ResultCode::InternalError,
        ] {
            let r = VerifyResult::new(AgentStatus::Ok, code, "test");
            assert_eq!(
                r.exit_code(),
                code.exit_code(),
                "exit_code drift for {code:?}"
            );
        }
    }
}
