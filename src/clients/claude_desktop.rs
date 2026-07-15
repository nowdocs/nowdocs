//! Claude Desktop adapter.
//!
//! v0.2 policy: the current official installation route for nowdocs on
//! Claude Desktop is a user-installed `.mcpb` extension installed through
//! Claude Desktop Settings > Extensions. A signed, cross-platform nowdocs
//! MCPB is a separate future deliverable; this adapter therefore stays
//! strictly generation-only. It does not create, edit, verify, or claim
//! installation of a Desktop extension, and it never writes
//! `claude_desktop_config.json`.

use std::path::Path;

use anyhow::Result;

use crate::agent_contract::CapabilitySupport;
use crate::clients::{
    AdapterCapabilities, ApprovedRoot, ClientAdapter, ClientId, Detection, GeneratedConfig,
};

/// Adapter for Claude Desktop.
pub struct ClaudeDesktopAdapter;

/// Conventional Claude Desktop configuration directory name used to detect
/// the presence of a Desktop installation in an approved root.
const DESKTOP_CONFIG_DIR: &str = "Claude";

/// Conventional Claude Desktop configuration file name. The adapter never
/// reads, parses, or writes this file; the name is used only as a detection
/// marker.
const DESKTOP_CONFIG_FILE: &str = "claude_desktop_config.json";

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

    fn detect(&self, root: &ApprovedRoot) -> Result<Detection> {
        // Presence of the conventional Claude Desktop configuration file
        // inside the approved root is the documented marker. The marker is
        // looked up only inside the explicitly approved root: this adapter
        // never reads real HOME, XDG, or absolute system paths.
        let marker = root
            .path()
            .join(DESKTOP_CONFIG_DIR)
            .join(DESKTOP_CONFIG_FILE);
        let detected = marker.is_file();

        let observations = vec![
            "Claude Desktop installation is detected by the conventional configuration directory inside the approved root.".to_string(),
            "nowdocs on Claude Desktop requires a user-installed .mcpb extension installed through Settings > Extensions; this adapter is generation-only and never writes the legacy configuration file.".to_string(),
        ];

        Ok(Detection {
            detected,
            target_path: if detected {
                Some(format!("{}/{}", DESKTOP_CONFIG_DIR, DESKTOP_CONFIG_FILE))
            } else {
                None
            },
            observations,
        })
    }

    fn generate(&self, binary_path: &Path) -> Result<GeneratedConfig> {
        // The stdio command is what a future signed `.mcpb` would launch
        // once a user installs it through Settings > Extensions. The
        // command itself stays deterministic and uses the supplied
        // absolute binary path so the future extension can be reproduced
        // without re-resolving anything.
        let stdio_command = vec![binary_path.display().to_string(), "serve".to_string()];

        // The redacted fragment is a deterministic, non-secret summary
        // that names the supported delivery channel (`.mcpb`) and the
        // future-deliverable status. It intentionally omits raw JSON
        // `command` / `args` / `stdio` keys that would mirror legacy
        // configuration files.
        let redacted_fragment = r#"{"claude-desktop":{"delivery":"mcpb","server":"nowdocs","status":"future_deliverable"}}"#.to_string();

        let manual_steps = vec![
            "Claude Desktop requires a user-installed .mcpb extension; nowdocs does not ship a signed, cross-platform MCPB in v0.2.".to_string(),
            "A signed, cross-platform nowdocs MCPB is a separate future deliverable tracked outside this adapter.".to_string(),
            "Install the .mcpb through Claude Desktop Settings > Extensions once a signed build is published.".to_string(),
        ];

        Ok(GeneratedConfig {
            stdio_command,
            redacted_fragment,
            manual_steps,
        })
    }
}
