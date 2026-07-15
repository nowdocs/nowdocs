//! Cursor adapter.
//!
//! v0.2 policy: target the global `~/.cursor/mcp.json` by default; emit a
//! canonical stdio entry; do not write the file automatically in C5.

use std::path::Path;

use anyhow::Result;

use crate::agent_contract::CapabilitySupport;
use crate::clients::{
    AdapterCapabilities, ApprovedRoot, ClientAdapter, ClientId, Detection, GeneratedConfig,
};

pub struct CursorAdapter;

impl ClientAdapter for CursorAdapter {
    fn id(&self) -> ClientId {
        ClientId::Cursor
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            detect: CapabilitySupport::Supported,
            generate: CapabilitySupport::Supported,
            apply: CapabilitySupport::Unsupported,
            verify: CapabilitySupport::Unsupported,
        }
    }

    fn detect(&self, root: &ApprovedRoot) -> Result<Detection> {
        let target = root.path().join(".cursor").join("mcp.json");
        let detected = target.is_file();
        Ok(Detection {
            detected,
            target_path: if detected {
                Some(".cursor/mcp.json".to_string())
            } else {
                None
            },
            observations: Vec::new(),
        })
    }

    fn generate(&self, binary_path: &Path) -> Result<GeneratedConfig> {
        Ok(GeneratedConfig {
            stdio_command: vec![binary_path.display().to_string(), "serve".to_string()],
            redacted_fragment:
                r#"{"mcpServers":{"nowdocs":{"command":"<binary>","args":["serve"]}}}"#.to_string(),
            manual_steps: vec!["Add the nowdocs server entry to ~/.cursor/mcp.json.".to_string()],
        })
    }
}
