//! Generic MCP client adapter.
//!
//! Emits deterministic stdio configuration and manual steps only. Detection and
//! automatic application are unsupported.

use std::path::Path;

use anyhow::Result;

use crate::agent_contract::CapabilitySupport;
use crate::clients::{
    AdapterCapabilities, ApprovedRoot, ClientAdapter, ClientId, Detection, GeneratedConfig,
};

pub struct GenericAdapter;

impl ClientAdapter for GenericAdapter {
    fn id(&self) -> ClientId {
        ClientId::Generic
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            detect: CapabilitySupport::Unsupported,
            generate: CapabilitySupport::Supported,
            apply: CapabilitySupport::Unsupported,
            verify: CapabilitySupport::Unsupported,
        }
    }

    fn detect(&self, _root: &ApprovedRoot) -> Result<Detection> {
        Ok(Detection {
            detected: false,
            target_path: None,
            observations: vec!["Generic adapter cannot detect a specific client.".to_string()],
        })
    }

    fn generate(&self, binary_path: &Path) -> Result<GeneratedConfig> {
        Ok(GeneratedConfig {
            stdio_command: vec![binary_path.display().to_string(), "serve".to_string()],
            redacted_fragment:
                r#"{"mcpServers":{"nowdocs":{"command":"<binary>","args":["serve"]}}}"#.to_string(),
            manual_steps: vec![
                "Add this stdio server to your MCP client configuration.".to_string()
            ],
        })
    }
}
