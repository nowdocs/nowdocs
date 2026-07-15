//! Claude Code adapter.
//!
//! v0.2 policy: generate the official `claude mcp add --transport stdio`
//! command at user scope; detect the presence of `~/.claude.json`; never
//! parse or rewrite the CLI JSON directly in C5.

use std::path::Path;

use anyhow::Result;

use crate::agent_contract::CapabilitySupport;
use crate::clients::{
    AdapterCapabilities, ApprovedRoot, ClientAdapter, ClientId, Detection, GeneratedConfig,
};

pub struct ClaudeCodeAdapter;

impl ClientAdapter for ClaudeCodeAdapter {
    fn id(&self) -> ClientId {
        ClientId::ClaudeCode
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
        let target = root.path().join(".claude.json");
        let detected = target.is_file();
        Ok(Detection {
            detected,
            target_path: if detected {
                Some(".claude.json".to_string())
            } else {
                None
            },
            observations: Vec::new(),
        })
    }

    fn generate(&self, binary_path: &Path) -> Result<GeneratedConfig> {
        let mut stdio_command = vec![
            "claude".to_string(),
            "mcp".to_string(),
            "add".to_string(),
            "--transport".to_string(),
            "stdio".to_string(),
            "--scope".to_string(),
            "user".to_string(),
            "nowdocs".to_string(),
            "--".to_string(),
        ];
        stdio_command.push(binary_path.display().to_string());
        stdio_command.push("serve".to_string());

        Ok(GeneratedConfig {
            stdio_command,
            redacted_fragment:
                r#"{"mcpServers":{"nowdocs":{"command":"<binary>","args":["serve"]}}}"#.to_string(),
            manual_steps: vec!["Run the generated claude mcp command at user scope.".to_string()],
        })
    }
}
