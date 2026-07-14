//! Typed client adapter model and generation-only configuration output.
//!
//! C5 adapters never write real client configuration files and never spawn
//! client processes. They only report capabilities, detect markers, and emit
//! deterministic, redacted stdio commands and JSON fragments for later manual
//! or orchestrated application.

use std::path::Path;
use std::str::FromStr;

use anyhow::Result;

use crate::agent_contract::CapabilitySupport;

pub mod config_io;

pub use config_io::{
    approved_root, atomic_replace, compute_digest, read_target, safe_target, ApprovedRoot,
    SafeTarget,
};

mod claude_code;
mod claude_desktop;
mod cursor;
mod generic;

/// Supported MCP clients. The set is fixed; there is no arbitrary map or
/// secret-bearing string field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClientId {
    ClaudeCode,
    ClaudeDesktop,
    Cursor,
    Generic,
}

impl ClientId {
    /// Stable kebab-case identifier used in the machine contract.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::ClaudeDesktop => "claude-desktop",
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

/// One generation-only client adapter.
pub trait ClientAdapter {
    fn id(&self) -> ClientId;
    fn capabilities(&self) -> AdapterCapabilities;
    fn detect(&self, root: &ApprovedRoot) -> Result<Detection>;
    fn generate(&self, binary_path: &Path) -> Result<GeneratedConfig>;
}

/// All supported client adapters in a stable order.
pub fn all_adapters() -> Vec<Box<dyn ClientAdapter>> {
    vec![
        Box::new(claude_code::ClaudeCodeAdapter),
        Box::new(claude_desktop::ClaudeDesktopAdapter),
        Box::new(cursor::CursorAdapter),
        Box::new(generic::GenericAdapter),
    ]
}
