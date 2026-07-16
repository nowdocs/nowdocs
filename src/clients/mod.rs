//! Typed client adapter model and generation-only configuration output.
//!
//! The base adapter contract emits deterministic, redacted stdio commands and
//! configuration fragments. Conditional C6 adapters may use their official
//! client CLIs for explicitly approved apply, verify, and rollback actions;
//! they must never edit client configuration files directly.

use std::path::Path;
use std::str::FromStr;

use anyhow::Result;

use crate::agent_contract::CapabilitySupport;

pub mod config_io;

pub use config_io::{
    approved_root, atomic_replace, compute_digest, read_target, safe_target, ApprovedRoot,
    SafeTarget,
};

pub(crate) mod claude_code;
pub(crate) mod claude_desktop;
pub(crate) mod codex;
pub(crate) mod cursor;
pub(crate) mod generic;

/// Supported MCP clients. The set is fixed; there is no arbitrary map or
/// secret-bearing string field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClientId {
    ClaudeCode,
    ClaudeDesktop,
    Codex,
    Cursor,
    Generic,
}

impl ClientId {
    /// Stable kebab-case identifier used in the machine contract.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::ClaudeDesktop => "claude-desktop",
            Self::Codex => "codex",
            Self::Cursor => "cursor",
            Self::Generic => "generic",
        }
    }
}

impl FromStr for ClientId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "claude-code" => Ok(Self::ClaudeCode),
            "claude-desktop" => Ok(Self::ClaudeDesktop),
            "codex" => Ok(Self::Codex),
            "cursor" => Ok(Self::Cursor),
            "generic" => Ok(Self::Generic),
            _ => anyhow::bail!("unsupported client id: {s}"),
        }
    }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Capability matrix for one client adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdapterCapabilities {
    pub detect: CapabilitySupport,
    pub generate: CapabilitySupport,
    pub apply: CapabilitySupport,
    pub verify: CapabilitySupport,
}

/// Result of detecting a client installation in an approved root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Detection {
    pub detected: bool,
    pub target_path: Option<String>,
    pub observations: Vec<String>,
}

/// Deterministic, generation-only client configuration output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedConfig {
    /// Exact argv for a stdio MCP server registration.
    pub stdio_command: Vec<String>,
    /// Redacted JSON fragment suitable for logs (secrets and absolute paths
    /// replaced by placeholders).
    pub redacted_fragment: String,
    /// Manual steps required because C5 does not perform real config writes.
    pub manual_steps: Vec<String>,
}

/// Explicit, non-serialized inputs for a confirmed client execution step.
///
/// C6 adapters receive an approved configuration root and an absolute nowdocs
/// binary path; neither value is included in machine-facing observations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientExecutionRequest {
    operation_id: String,
    approved_root: ApprovedRoot,
    binary_path: std::path::PathBuf,
}

impl ClientExecutionRequest {
    /// Construct a validated execution request.
    pub fn new(
        operation_id: &str,
        approved_root: ApprovedRoot,
        binary_path: std::path::PathBuf,
    ) -> Result<Self> {
        if operation_id.is_empty()
            || operation_id.len() > 64
            || !operation_id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            anyhow::bail!("invalid client execution operation id");
        }
        if !binary_path.is_absolute() {
            anyhow::bail!("client execution binary path must be absolute");
        }
        Ok(Self {
            operation_id: operation_id.to_string(),
            approved_root,
            binary_path,
        })
    }

    /// Stable operation identifier for operation-owned rollback.
    pub fn operation_id(&self) -> &str {
        &self.operation_id
    }

    /// Explicitly approved configuration root.
    pub fn approved_root(&self) -> &ApprovedRoot {
        &self.approved_root
    }

    /// Absolute nowdocs binary path for stdio registration.
    pub fn binary_path(&self) -> &Path {
        &self.binary_path
    }
}

/// Closed result state for a client execution attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientExecutionOutcome {
    Applied,
    Verified,
    RolledBack,
    ManualRequired,
    Conflict,
    Unsupported,
}

/// Redacted result of a client execution attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientExecutionResult {
    pub outcome: ClientExecutionOutcome,
    pub observations: Vec<String>,
}

impl ClientExecutionResult {
    /// Return a stable result for a capability that this adapter does not own.
    pub fn unsupported(client: ClientId, operation: &str) -> Self {
        Self {
            outcome: ClientExecutionOutcome::Unsupported,
            observations: vec![format!(
                "{} does not support {} in this build",
                client, operation
            )],
        }
    }
}

/// One generation-only client adapter.
pub trait ClientAdapter {
    fn id(&self) -> ClientId;
    fn capabilities(&self) -> AdapterCapabilities;
    fn detect(&self, root: &ApprovedRoot) -> Result<Detection>;
    fn generate(&self, binary_path: &Path) -> Result<GeneratedConfig>;

    /// Apply an approved configuration change. C5 adapters default-deny until
    /// their owning C6 slice implements this method.
    fn apply(&self, _request: &ClientExecutionRequest) -> Result<ClientExecutionResult> {
        Ok(ClientExecutionResult::unsupported(self.id(), "apply"))
    }

    /// Verify a previously requested configuration change.
    fn verify(&self, _request: &ClientExecutionRequest) -> Result<ClientExecutionResult> {
        Ok(ClientExecutionResult::unsupported(self.id(), "verify"))
    }

    /// Roll back an operation-owned configuration change.
    fn rollback(&self, _request: &ClientExecutionRequest) -> Result<ClientExecutionResult> {
        Ok(ClientExecutionResult::unsupported(self.id(), "rollback"))
    }
}

/// All supported client adapters in a stable order.
pub fn all_adapters() -> Vec<Box<dyn ClientAdapter>> {
    vec![
        Box::new(claude_code::ClaudeCodeAdapter),
        Box::new(claude_desktop::ClaudeDesktopAdapter),
        Box::new(codex::CodexAdapter),
        Box::new(cursor::CursorAdapter),
        Box::new(generic::GenericAdapter),
    ]
}
