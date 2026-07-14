//! Claude Desktop adapter.
//!
//! v0.2 policy: detect where practical, explain the current `.mcpb` extension
//! requirement, and return manual next actions. Do not write legacy
//! `claude_desktop_config.json` automatically.

use std::path::Path;

use anyhow::Result;

use crate::agent_contract::CapabilitySupport;
use crate::clients::{
    AdapterCapabilities, ApprovedRoot, ClientAdapter, ClientId, Detection, GeneratedConfig,
};

pub struct ClaudeDesktopAdapter;

impl ClientAdapter for ClaudeDesktopAdapter {
    fn id(&self) -> ClientId {
        ClientId::ClaudeDesktop
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            detect: CapabilitySupport::Supported,
            generate: CapabilitySupport::Supported,
            apply: CapabilitySupport::Unsupported,
            verify: CapabilitySupport::Unsupported,
        }
    }

    fn detect(&self, _root: &ApprovedRoot) -> Result<Detection> {
        // Practical detection of a system-wide Claude Desktop installation is
        // platform-specific; C5 stays generation-only and records the policy.
        Ok(Detection {
            detected: false,
            target_path: None,
            observations: vec![
                "Claude Desktop is detected by the presence of the application; C5 does not edit legacy JSON.".to_string(),
            ],
        })
    }

    fn generate(&self, binary_path: &Path) -> Result<GeneratedConfig> {
        Ok(GeneratedConfig {
            stdio_command: vec![binary_path.display().to_string(), "serve".to_string()],
            redacted_fragment:
                r#"{"claude-desktop":{"mcpb":{"server":"nowdocs","command":"<binary>","args":["serve"]}}}"#
                    .to_string(),
            manual_steps: vec![
                "Claude Desktop requires a signed .mcpb extension.".to_string(),
                "Install the .mcpb through Claude Desktop Settings > Extensions.".to_string(),
            ],
        })
    }
}
