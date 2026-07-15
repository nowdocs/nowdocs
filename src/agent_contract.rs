//! Versioned machine contract for agent automation (schema v1).
//!
//! This module owns the stable JSON envelope, statuses, result codes, exit
//! classes, the action model, rollback metadata, and the `capabilities`
//! payload shared by all agent-automation commands. Human output may change
//! for clarity; the versioned machine contract must not change incompatibly
//! without a schema version increase. Additive fields are allowed within
//! schema version 1; consumers must ignore unknown additive fields.
//!
//! Contract types never carry secret-bearing environment values, and the
//! module performs no I/O: every builder here uses compile-time constants
//! only, so the `capabilities` code path is read-only and offline-safe.

use serde::{Deserialize, Serialize};

/// Machine-contract schema version for agent automation output.
pub const AGENT_CONTRACT_SCHEMA_VERSION: u32 = 1;

/// Top-level outcome of an agent-automation command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Ok,
    Warning,
    ActionRequired,
    Partial,
    Error,
}

/// Stable machine-readable result code. The JSON `code` is authoritative
/// within each exit class; agents must use it (not `summary`) for decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultCode {
    Ready,
    AlreadySatisfied,
    ActionRequired,
    SetupComplete,
    ModelMissing,
    DocsetMissing,
    DocsetCorrupt,
    RegistryMetadataRequired,
    ClientNotDetected,
    ClientReloadRequired,
    ConfigParseFailed,
    ConfigConflict,
    ConfigWriteUnsafe,
    PlanNotFound,
    PlanExpired,
    PlanStale,
    PlanTampered,
    OperationInProgress,
    PermissionDenied,
    NetworkUnavailable,
    VerificationFailed,
    AppliedButUnverified,
    UnsupportedPlatform,
    InvalidRequest,
    InternalError,
}

impl ResultCode {
    /// Pure mapping from a result code to the parent design's exit classes
    /// (Section 7.3): 0 completed, 2 invalid request, 10 plan/concurrency
    /// conflict, 20 operation failed before completing the approved goal,
    /// 21 applied but unverified, 30 policy refusal.
    pub fn exit_code(self) -> u8 {
        match self {
            ResultCode::Ready
            | ResultCode::AlreadySatisfied
            | ResultCode::ActionRequired
            | ResultCode::SetupComplete
            | ResultCode::ClientReloadRequired => 0,
            ResultCode::InvalidRequest => 2,
            ResultCode::PlanNotFound
            | ResultCode::PlanExpired
            | ResultCode::PlanStale
            | ResultCode::PlanTampered
            | ResultCode::OperationInProgress => 10,
            ResultCode::AppliedButUnverified => 21,
            ResultCode::ConfigWriteUnsafe | ResultCode::UnsupportedPlatform => 30,
            ResultCode::ModelMissing
            | ResultCode::DocsetMissing
            | ResultCode::DocsetCorrupt
            | ResultCode::RegistryMetadataRequired
            | ResultCode::ClientNotDetected
            | ResultCode::ConfigParseFailed
            | ResultCode::ConfigConflict
            | ResultCode::PermissionDenied
            | ResultCode::NetworkUnavailable
            | ResultCode::VerificationFailed
            | ResultCode::InternalError => 20,
        }
    }
}

/// Risk classification of a planned action (parent design Section 9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    ReadOnly,
    InternalEphemeral,
    Additive,
    Mutating,
    Destructive,
}

/// Whether an adapter capability is available. `Conditional` is advertised
/// only when runtime preconditions are satisfied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilitySupport {
    Supported,
    Conditional,
    Unsupported,
}

/// A structured next action an agent may present or execute. Sensitive
/// values are excluded from `target_paths`; there is no free-form map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NextAction {
    /// Stable action identifier.
    pub id: String,
    /// Stable action kind (for example `model_download`).
    pub kind: String,
    pub risk: RiskLevel,
    /// Concise English human explanation.
    pub summary: String,
    pub changes_state: bool,
    pub network_access: bool,
    pub requires_confirmation: bool,
    pub reversible: bool,
    /// Exact argv when the action maps to a nowdocs invocation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub argv: Option<Vec<String>>,
    /// Target paths with sensitive values excluded.
    pub target_paths: Vec<String>,
    /// Advisory download estimate; absent when unknown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_download_bytes: Option<u64>,
}

/// Rollback metadata for a completed or partially applied operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RollbackMetadata {
    pub operation_id: String,
    /// RFC 3339 expiry timestamp of the retained rollback data.
    pub expires_at: String,
    /// Exact argv that rolls back the named operation.
    pub argv: Vec<String>,
}

/// The single envelope shape emitted by every agent-automation command when
/// `--json` is selected. `nowdocs_version` always comes from the compiled
/// package version; it is never duplicated as a literal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentEnvelope<T> {
    pub schema_version: u32,
    pub nowdocs_version: String,
    pub command: String,
    pub status: AgentStatus,
    pub code: ResultCode,
    /// Concise English human explanation; not for agent decisions.
    pub summary: String,
    pub data: T,
    pub next_actions: Vec<NextAction>,
    pub rollback: Option<RollbackMetadata>,
}

impl<T> AgentEnvelope<T> {
    /// Build an envelope with no next actions and no rollback metadata.
    pub fn new(
        command: &str,
        status: AgentStatus,
        code: ResultCode,
        summary: &str,
        data: T,
    ) -> Self {
        Self {
            schema_version: AGENT_CONTRACT_SCHEMA_VERSION,
            nowdocs_version: env!("CARGO_PKG_VERSION").to_string(),
            command: command.to_string(),
            status,
            code,
            summary: summary.to_string(),
            data,
            next_actions: Vec::new(),
            rollback: None,
        }
    }
}

/// Declaration of one agent-automation command in the locked CLI order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandCapability {
    pub id: String,
    pub implemented: bool,
    pub read_only: bool,
    pub network_access: bool,
}

/// Per-client capability matrix row (detect/generate/apply/verify).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientCapability {
    pub id: String,
    pub detect: CapabilitySupport,
    pub generate: CapabilitySupport,
    pub apply: CapabilitySupport,
    pub verify: CapabilitySupport,
}

/// Product security boundaries advertised to agents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityBoundaries {
    pub mcp_read_only: bool,
    pub stdio_only: bool,
    pub telemetry: bool,
    pub writable_mcp_tools: bool,
    pub search_requires_docset: bool,
}

/// Payload of `nowdocs capabilities --json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilitiesData {
    pub agent_contract_schema_version: u32,
    pub mcp_protocol_version: String,
    pub transport: String,
    pub mcp_tools: Vec<String>,
    pub commands: Vec<CommandCapability>,
    pub clients: Vec<ClientCapability>,
    pub security_boundaries: SecurityBoundaries,
}

/// Build the deterministic capabilities payload from compile-time constants
/// only. This performs no I/O, no cache access, and no network access.
pub fn capabilities_data() -> CapabilitiesData {
    CapabilitiesData {
        agent_contract_schema_version: AGENT_CONTRACT_SCHEMA_VERSION,
        // Single source of truth shared with MCP initialize.
        mcp_protocol_version: crate::mcp::PROTOCOL_VERSION.to_string(),
        transport: "stdio_ndjson".to_string(),
        // Derived from the single mcp::MCP_TOOL_NAMES source (P2 repair).
        mcp_tools: crate::mcp::MCP_TOOL_NAMES
            .iter()
            .map(|s| s.to_string())
            .collect(),
        commands: vec![
            command("capabilities", true, true, false),
            command("status", true, true, false),
            command("setup.plan", true, false, true),
            command("setup.apply", true, false, true),
            command("setup.rollback", true, false, false),
            command("ensure.plan", true, false, true),
            command("ensure.apply", true, false, true),
            command("verify", true, true, false),
        ],
        clients: vec![
            client(
                "claude-code",
                CapabilitySupport::Supported,
                CapabilitySupport::Supported,
                CapabilitySupport::Conditional,
                CapabilitySupport::Conditional,
            ),
            client(
                "claude-desktop",
                CapabilitySupport::Supported,
                CapabilitySupport::Conditional,
                CapabilitySupport::Unsupported,
                CapabilitySupport::Unsupported,
            ),
            client(
                "cursor",
                CapabilitySupport::Supported,
                CapabilitySupport::Supported,
                CapabilitySupport::Conditional,
                CapabilitySupport::Conditional,
            ),
            client(
                "generic",
                CapabilitySupport::Unsupported,
                CapabilitySupport::Supported,
                CapabilitySupport::Unsupported,
                CapabilitySupport::Unsupported,
            ),
        ],
        security_boundaries: SecurityBoundaries {
            mcp_read_only: true,
            stdio_only: true,
            telemetry: false,
            writable_mcp_tools: false,
            search_requires_docset: true,
        },
    }
}

fn command(
    id: &str,
    implemented: bool,
    read_only: bool,
    network_access: bool,
) -> CommandCapability {
    CommandCapability {
        id: id.to_string(),
        implemented,
        read_only,
        network_access,
    }
}

fn client(
    id: &str,
    detect: CapabilitySupport,
    generate: CapabilitySupport,
    apply: CapabilitySupport,
    verify: CapabilitySupport,
) -> ClientCapability {
    ClientCapability {
        id: id.to_string(),
        detect,
        generate,
        apply,
        verify,
    }
}

/// Concise English human rendering of the capabilities payload, derived from
/// the same `CapabilitiesData` as the JSON output. Human text is not part of
/// schema v1 and may change for clarity.
pub fn format_capabilities_human(data: &CapabilitiesData) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "nowdocs capabilities (agent contract schema version: {})\n",
        data.agent_contract_schema_version
    ));
    out.push_str(&format!(
        "MCP protocol version: {}\n",
        data.mcp_protocol_version
    ));
    out.push_str(&format!("transport: {}\n", data.transport));
    let b = &data.security_boundaries;
    out.push_str(&format!(
        "security boundaries: MCP read-only={}, stdio only={}, telemetry={}, \
         writable MCP tools={}, search requires explicit docset={}\n",
        b.mcp_read_only, b.stdio_only, b.telemetry, b.writable_mcp_tools, b.search_requires_docset
    ));
    out.push_str("commands:\n");
    for c in &data.commands {
        let state = if c.implemented {
            "implemented"
        } else {
            "not implemented"
        };
        out.push_str(&format!("  {}: {}\n", c.id, state));
    }
    out.push_str("clients:\n");
    for c in &data.clients {
        out.push_str(&format!(
            "  {}: detect={} generate={} apply={} verify={}\n",
            c.id,
            support_str(c.detect),
            support_str(c.generate),
            support_str(c.apply),
            support_str(c.verify)
        ));
    }
    out
}

fn support_str(support: CapabilitySupport) -> &'static str {
    match support {
        CapabilitySupport::Supported => "supported",
        CapabilitySupport::Conditional => "conditional",
        CapabilitySupport::Unsupported => "unsupported",
    }
}
