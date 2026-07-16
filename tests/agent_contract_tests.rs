//! Contract tests for the agent-automation machine contract (schema v1).
//!
//! These tests lock the JSON shape of the shared envelope, the snake_case
//! serialization of every contract enum, the result-code -> exit-class
//! mapping, and the deterministic content of `nowdocs capabilities` data.

use nowdocs::agent_contract::{
    capabilities_data, AgentEnvelope, AgentStatus, CapabilitySupport, NextAction, ResultCode,
    RiskLevel, RollbackMetadata,
};

#[test]
fn agent_envelope_serializes_required_top_level_fields() {
    let envelope = AgentEnvelope::new(
        "capabilities",
        AgentStatus::Ok,
        ResultCode::Ready,
        "nowdocs agent automation capabilities",
        serde_json::json!({}),
    );
    let v = serde_json::to_value(&envelope).expect("envelope must serialize");
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["nowdocs_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(v["command"], "capabilities");
    assert_eq!(v["status"], "ok");
    assert_eq!(v["code"], "ready");
    assert_eq!(v["summary"], "nowdocs agent automation capabilities");
    assert_eq!(v["data"], serde_json::json!({}));
    assert_eq!(v["next_actions"], serde_json::json!([]));
    assert!(v["rollback"].is_null(), "rollback must be null when absent");

    // A populated envelope keeps the locked action and rollback shapes from
    // the parent design (Sections 7 and 7.1).
    let mut envelope = AgentEnvelope::new(
        "setup.apply",
        AgentStatus::Error,
        ResultCode::AppliedButUnverified,
        "configuration applied; verification incomplete",
        serde_json::json!({}),
    );
    envelope.next_actions.push(NextAction {
        id: "prepare_model".to_string(),
        kind: "model_download".to_string(),
        risk: RiskLevel::Additive,
        summary: "Download the pinned embedding model".to_string(),
        changes_state: true,
        network_access: true,
        requires_confirmation: true,
        reversible: true,
        argv: None,
        target_paths: vec![],
        estimated_download_bytes: Some(69_206_016),
    });
    envelope.rollback = Some(RollbackMetadata {
        operation_id: "op-123".to_string(),
        expires_at: "2026-07-14T00:00:00Z".to_string(),
        argv: vec![
            "nowdocs".to_string(),
            "setup".to_string(),
            "rollback".to_string(),
            "--operation-id".to_string(),
            "op-123".to_string(),
        ],
    });
    let v = serde_json::to_value(&envelope).expect("envelope must serialize");
    let action = &v["next_actions"][0];
    assert_eq!(action["id"], "prepare_model");
    assert_eq!(action["kind"], "model_download");
    assert_eq!(action["risk"], "additive");
    assert_eq!(action["summary"], "Download the pinned embedding model");
    assert_eq!(action["changes_state"], true);
    assert_eq!(action["network_access"], true);
    assert_eq!(action["requires_confirmation"], true);
    assert_eq!(action["reversible"], true);
    assert_eq!(action["target_paths"], serde_json::json!([]));
    assert_eq!(action["estimated_download_bytes"], 69_206_016);
    let rollback = &v["rollback"];
    assert_eq!(rollback["operation_id"], "op-123");
    assert_eq!(rollback["expires_at"], "2026-07-14T00:00:00Z");
    assert_eq!(rollback["argv"][0], "nowdocs");
    assert_eq!(rollback["argv"][4], "op-123");
}

#[test]
fn agent_contract_enums_serialize_as_snake_case() {
    let cases: Vec<(serde_json::Value, &str)> = vec![
        (serde_json::to_value(AgentStatus::Ok).unwrap(), "ok"),
        (
            serde_json::to_value(AgentStatus::Warning).unwrap(),
            "warning",
        ),
        (
            serde_json::to_value(AgentStatus::ActionRequired).unwrap(),
            "action_required",
        ),
        (
            serde_json::to_value(AgentStatus::Partial).unwrap(),
            "partial",
        ),
        (serde_json::to_value(AgentStatus::Error).unwrap(), "error"),
        (
            serde_json::to_value(RiskLevel::ReadOnly).unwrap(),
            "read_only",
        ),
        (
            serde_json::to_value(RiskLevel::InternalEphemeral).unwrap(),
            "internal_ephemeral",
        ),
        (
            serde_json::to_value(RiskLevel::Additive).unwrap(),
            "additive",
        ),
        (
            serde_json::to_value(RiskLevel::Mutating).unwrap(),
            "mutating",
        ),
        (
            serde_json::to_value(RiskLevel::Destructive).unwrap(),
            "destructive",
        ),
        (
            serde_json::to_value(CapabilitySupport::Supported).unwrap(),
            "supported",
        ),
        (
            serde_json::to_value(CapabilitySupport::Conditional).unwrap(),
            "conditional",
        ),
        (
            serde_json::to_value(CapabilitySupport::Unsupported).unwrap(),
            "unsupported",
        ),
        (serde_json::to_value(ResultCode::Ready).unwrap(), "ready"),
        (
            serde_json::to_value(ResultCode::AlreadySatisfied).unwrap(),
            "already_satisfied",
        ),
        (
            serde_json::to_value(ResultCode::ActionRequired).unwrap(),
            "action_required",
        ),
        (
            serde_json::to_value(ResultCode::SetupComplete).unwrap(),
            "setup_complete",
        ),
        (
            serde_json::to_value(ResultCode::ModelMissing).unwrap(),
            "model_missing",
        ),
        (
            serde_json::to_value(ResultCode::DocsetMissing).unwrap(),
            "docset_missing",
        ),
        (
            serde_json::to_value(ResultCode::DocsetCorrupt).unwrap(),
            "docset_corrupt",
        ),
        (
            serde_json::to_value(ResultCode::RegistryMetadataRequired).unwrap(),
            "registry_metadata_required",
        ),
        (
            serde_json::to_value(ResultCode::ClientNotDetected).unwrap(),
            "client_not_detected",
        ),
        (
            serde_json::to_value(ResultCode::ClientReloadRequired).unwrap(),
            "client_reload_required",
        ),
        (
            serde_json::to_value(ResultCode::ConfigParseFailed).unwrap(),
            "config_parse_failed",
        ),
        (
            serde_json::to_value(ResultCode::ConfigConflict).unwrap(),
            "config_conflict",
        ),
        (
            serde_json::to_value(ResultCode::ConfigWriteUnsafe).unwrap(),
            "config_write_unsafe",
        ),
        (
            serde_json::to_value(ResultCode::PlanNotFound).unwrap(),
            "plan_not_found",
        ),
        (
            serde_json::to_value(ResultCode::PlanExpired).unwrap(),
            "plan_expired",
        ),
        (
            serde_json::to_value(ResultCode::PlanStale).unwrap(),
            "plan_stale",
        ),
        (
            serde_json::to_value(ResultCode::PlanTampered).unwrap(),
            "plan_tampered",
        ),
        (
            serde_json::to_value(ResultCode::OperationInProgress).unwrap(),
            "operation_in_progress",
        ),
        (
            serde_json::to_value(ResultCode::PermissionDenied).unwrap(),
            "permission_denied",
        ),
        (
            serde_json::to_value(ResultCode::NetworkUnavailable).unwrap(),
            "network_unavailable",
        ),
        (
            serde_json::to_value(ResultCode::VerificationFailed).unwrap(),
            "verification_failed",
        ),
        (
            serde_json::to_value(ResultCode::AppliedButUnverified).unwrap(),
            "applied_but_unverified",
        ),
        (
            serde_json::to_value(ResultCode::UnsupportedPlatform).unwrap(),
            "unsupported_platform",
        ),
        (
            serde_json::to_value(ResultCode::InvalidRequest).unwrap(),
            "invalid_request",
        ),
        (
            serde_json::to_value(ResultCode::InternalError).unwrap(),
            "internal_error",
        ),
    ];
    assert_eq!(cases.len(), 38, "every contract enum variant is covered");
    for (value, expected) in cases {
        assert_eq!(value, expected);
    }
}

#[test]
fn initial_result_codes_map_to_locked_exit_classes() {
    // Exit classes locked by the parent design Section 7.3: 0 success,
    // 2 invalid request, 10 plan/concurrency conflict, 20 operation failed,
    // 21 applied but unverified, 30 policy refusal.
    let cases: Vec<(ResultCode, u8)> = vec![
        (ResultCode::Ready, 0),
        (ResultCode::AlreadySatisfied, 0),
        (ResultCode::ActionRequired, 0),
        (ResultCode::SetupComplete, 0),
        (ResultCode::ClientReloadRequired, 0),
        (ResultCode::InvalidRequest, 2),
        (ResultCode::PlanNotFound, 10),
        (ResultCode::PlanExpired, 10),
        (ResultCode::PlanStale, 10),
        (ResultCode::PlanTampered, 10),
        (ResultCode::OperationInProgress, 10),
        (ResultCode::AppliedButUnverified, 21),
        (ResultCode::ConfigWriteUnsafe, 30),
        (ResultCode::UnsupportedPlatform, 30),
        (ResultCode::ModelMissing, 20),
        (ResultCode::DocsetMissing, 20),
        (ResultCode::DocsetCorrupt, 20),
        (ResultCode::RegistryMetadataRequired, 20),
        (ResultCode::ClientNotDetected, 20),
        (ResultCode::ConfigParseFailed, 20),
        (ResultCode::ConfigConflict, 20),
        (ResultCode::PermissionDenied, 20),
        (ResultCode::NetworkUnavailable, 20),
        (ResultCode::VerificationFailed, 20),
        (ResultCode::InternalError, 20),
    ];
    assert_eq!(cases.len(), 25, "every initial result code is covered");
    for (code, expected_exit) in cases {
        assert_eq!(code.exit_code(), expected_exit, "exit class for {code:?}");
    }
}

#[test]
fn capabilities_data_is_deterministically_ordered() {
    let data = capabilities_data();

    let mcp_tools: Vec<&str> = data.mcp_tools.iter().map(String::as_str).collect();
    assert_eq!(mcp_tools, ["nowdocs_list", "nowdocs_search"]);

    // Commands appear in the explicit locked order with per-command
    // (implemented, read_only, network_access) declarations. `capabilities`
    // (C1) and `status` (C2) are implemented.
    let expected_commands: [(&str, bool, bool, bool); 8] = [
        ("capabilities", true, true, false),
        ("status", true, true, false),
        ("setup.plan", true, false, true),
        ("setup.apply", true, false, true),
        ("setup.rollback", true, false, false),
        ("ensure.plan", true, false, true),
        ("ensure.apply", true, false, true),
        ("verify", true, true, false),
    ];
    assert_eq!(data.commands.len(), expected_commands.len());
    for (command, (id, implemented, read_only, network_access)) in
        data.commands.iter().zip(expected_commands)
    {
        assert_eq!(command.id, id);
        assert_eq!(command.implemented, implemented, "{id} implemented");
        assert_eq!(command.read_only, read_only, "{id} read_only");
        assert_eq!(
            command.network_access, network_access,
            "{id} network_access"
        );
    }

    let client_ids: Vec<&str> = data.clients.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(
        client_ids,
        [
            "claude-code",
            "claude-desktop",
            "codex",
            "cursor",
            "generic"
        ]
    );

    // Rebuilding yields byte-identical JSON: ordering is deterministic.
    let first = serde_json::to_string(&data).expect("capabilities data serializes");
    let second = serde_json::to_string(&capabilities_data()).expect("capabilities data serializes");
    assert_eq!(first, second);
}

#[test]
fn capabilities_matrix_matches_researched_clients() {
    let data = capabilities_data();

    // Matrix locked by client-adapter-research.md (2026-07-13) plus the C6E
    // Codex CLI adapter contract (2026-07-16).
    let expected: [(
        &str,
        CapabilitySupport,
        CapabilitySupport,
        CapabilitySupport,
        CapabilitySupport,
    ); 5] = [
        (
            "claude-code",
            CapabilitySupport::Supported,
            CapabilitySupport::Supported,
            CapabilitySupport::Conditional,
            CapabilitySupport::Conditional,
        ),
        (
            "claude-desktop",
            CapabilitySupport::Supported,
            CapabilitySupport::Conditional,
            CapabilitySupport::Unsupported,
            CapabilitySupport::Unsupported,
        ),
        (
            "codex",
            CapabilitySupport::Supported,
            CapabilitySupport::Supported,
            CapabilitySupport::Conditional,
            CapabilitySupport::Conditional,
        ),
        (
            "cursor",
            CapabilitySupport::Supported,
            CapabilitySupport::Supported,
            CapabilitySupport::Conditional,
            CapabilitySupport::Conditional,
        ),
        (
            "generic",
            CapabilitySupport::Unsupported,
            CapabilitySupport::Supported,
            CapabilitySupport::Unsupported,
            CapabilitySupport::Unsupported,
        ),
    ];
    assert_eq!(data.clients.len(), expected.len());
    for (client, (id, detect, generate, apply, verify)) in data.clients.iter().zip(expected) {
        assert_eq!(client.id, id);
        assert_eq!(client.detect, detect, "{id} detect");
        assert_eq!(client.generate, generate, "{id} generate");
        assert_eq!(client.apply, apply, "{id} apply");
        assert_eq!(client.verify, verify, "{id} verify");
    }

    // Aider is deliberately absent: upstream has no native MCP support.
    assert!(
        data.clients.iter().all(|c| c.id != "aider"),
        "Aider must not appear as a supported client"
    );
}

#[test]
fn capabilities_mcp_tools_derive_from_single_tool_name_source() {
    let data = capabilities_data();
    let expected: Vec<String> = nowdocs::mcp::MCP_TOOL_NAMES
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        data.mcp_tools, expected,
        "capabilities mcp_tools must derive from mcp::MCP_TOOL_NAMES (P2 single source)"
    );
}

#[test]
fn capabilities_declares_security_boundaries() {
    let data = capabilities_data();
    assert_eq!(data.agent_contract_schema_version, 1);
    assert_eq!(data.mcp_protocol_version, "2025-11-25");
    // Capabilities reports the same single protocol constant as MCP initialize.
    assert_eq!(data.mcp_protocol_version, nowdocs::mcp::PROTOCOL_VERSION);
    assert_eq!(data.transport, "stdio_ndjson");
    let boundaries = &data.security_boundaries;
    assert!(boundaries.mcp_read_only);
    assert!(boundaries.stdio_only);
    assert!(!boundaries.telemetry);
    assert!(!boundaries.writable_mcp_tools);
    assert!(boundaries.search_requires_docset);
}
