//! Claude Code adapter.
//!
//! C6A policy: manage user-scope `nowdocs` registration exclusively through
//! the official `claude mcp` CLI. The adapter never reads, parses, modifies,
//! or exposes `~/.claude.json`. It resolves `claude` from `PATH`, invokes it
//! with argument vectors (never a shell), bounds captured output, and emits
//! only redacted observations containing no absolute path, HOME, token, or raw
//! CLI output.

use std::path::Path;
use std::process::Command;

use anyhow::Result;

use crate::agent_contract::CapabilitySupport;
use crate::clients::{
    AdapterCapabilities, ApprovedRoot, ClientAdapter, ClientExecutionOutcome,
    ClientExecutionRequest, ClientExecutionResult, ClientId, Detection, GeneratedConfig,
};

/// Maximum bytes of captured stdout/stderr retained for classification. The
/// content is never emitted in observations; it is only inspected locally.
const CAPTURE_LIMIT: usize = 8 * 1024;

pub struct ClaudeCodeAdapter;

impl ClientAdapter for ClaudeCodeAdapter {
    fn id(&self) -> ClientId {
        ClientId::ClaudeCode
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            detect: CapabilitySupport::Supported,
            generate: CapabilitySupport::Supported,
            apply: CapabilitySupport::Conditional,
            verify: CapabilitySupport::Conditional,
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

    fn apply(&self, request: &ClientExecutionRequest) -> Result<ClientExecutionResult> {
        let binary = request.binary_path();

        // First inspect whether a nowdocs entry already exists through the
        // official CLI. A present or indeterminate entry is never replaced.
        match claude_entry_state() {
            EntryState::Present => Ok(ClientExecutionResult {
                outcome: ClientExecutionOutcome::Conflict,
                observations: vec![
                    "claude-code already has a nowdocs entry; refusing to overwrite.".to_string(),
                ],
            }),
            EntryState::Absent => {
                // Clear absent result: invoke the exact user-scope add form.
                let mut cmd = claude_command();
                cmd.args([
                    "mcp",
                    "add",
                    "--transport",
                    "stdio",
                    "--scope",
                    "user",
                    "nowdocs",
                    "--",
                ]);
                cmd.arg(binary);
                cmd.arg("serve");
                match run_capture(&mut cmd) {
                    Captured::Ok { .. } => Ok(ClientExecutionResult {
                        outcome: ClientExecutionOutcome::Applied,
                        observations: vec![
                            "claude-code user-scope nowdocs entry added.".to_string()
                        ],
                    }),
                    Captured::Missing => Ok(manual_required("claude CLI not found on PATH")),
                    Captured::Denied | Captured::NonZero { .. } => Ok(manual_required(
                        "claude mcp add did not complete cleanly; manual review required",
                    )),
                }
            }
            EntryState::Ambiguous => Ok(manual_required(
                "claude mcp get returned an indeterminate result; manual review required",
            )),
            EntryState::Missing => Ok(manual_required("claude CLI not found on PATH")),
        }
    }

    fn verify(&self, request: &ClientExecutionRequest) -> Result<ClientExecutionResult> {
        let binary = request.binary_path();
        let mut cmd = claude_command();
        cmd.args(["mcp", "get", "nowdocs"]);
        match run_capture(&mut cmd) {
            Captured::Ok { stdout } => {
                if entry_matches(&stdout, binary) {
                    Ok(ClientExecutionResult {
                        outcome: ClientExecutionOutcome::Verified,
                        observations: vec![
                            "claude-code user-scope nowdocs entry verified.".to_string()
                        ],
                    })
                } else {
                    Ok(manual_required(
                        "claude-code nowdocs entry does not match the expected binary and serve argument",
                    ))
                }
            }
            Captured::Missing => Ok(manual_required("claude CLI not found on PATH")),
            Captured::Denied | Captured::NonZero { .. } => Ok(manual_required(
                "claude mcp get did not complete cleanly; manual review required",
            )),
        }
    }

    fn rollback(&self, _request: &ClientExecutionRequest) -> Result<ClientExecutionResult> {
        let mut cmd = claude_command();
        cmd.args(["mcp", "remove", "--scope", "user", "nowdocs"]);
        match run_capture(&mut cmd) {
            Captured::Ok { .. } => Ok(ClientExecutionResult {
                outcome: ClientExecutionOutcome::RolledBack,
                observations: vec!["claude-code user-scope nowdocs entry removed.".to_string()],
            }),
            Captured::Missing => Ok(manual_required("claude CLI not found on PATH")),
            Captured::Denied | Captured::NonZero { .. } => Ok(manual_required(
                "claude mcp remove did not complete cleanly; user configuration left intact",
            )),
        }
    }
}

/// Observed state of the `nowdocs` entry from `claude mcp get`.
enum EntryState {
    Present,
    Absent,
    Ambiguous,
    Missing,
}

/// Inspect the `nowdocs` entry through the official CLI without modifying it.
fn claude_entry_state() -> EntryState {
    let mut cmd = claude_command();
    cmd.args(["mcp", "get", "nowdocs"]);
    match run_capture(&mut cmd) {
        Captured::Ok { .. } => EntryState::Present,
        Captured::Missing => EntryState::Missing,
        Captured::Denied => EntryState::Ambiguous,
        // `claude mcp get` exits 1 when the named server does not exist. Any
        // other nonzero code is treated as indeterminate (Ambiguous) so that a
        // transient CLI error can never be mistaken for a clear absence and
        // trigger an unintended `mcp add`.
        Captured::NonZero { code: Some(1), .. } => EntryState::Absent,
        Captured::NonZero { .. } => EntryState::Ambiguous,
    }
}

/// Check whether captured `claude mcp get` output confirms the expected
/// absolute binary and `serve` argument. The real `claude mcp get` command
/// emits human-readable text (not JSON), e.g.:
///
/// ```text
/// nowdocs:
///   Scope: User config (available in all your projects)
///   Type: stdio
///   Command: /absolute/path/to/nowdocs
///   Args: serve
/// ```
///
/// Returns false on any parse ambiguity rather than risking a false positive.
fn entry_matches(stdout: &[u8], expected_binary: &Path) -> bool {
    let text = match std::str::from_utf8(stdout) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let command = extract_field(text, "Command");
    let args = extract_field(text, "Args");
    match (command, args) {
        (Some(cmd), Some(args_str)) => {
            Path::new(cmd.trim()) == expected_binary
                && args_str.split_whitespace().any(|a| a == "serve")
        }
        _ => false,
    }
}

/// Extract the value following a `Field:` label in human-readable CLI output.
/// Returns the trimmed text after the label on the first matching line.
fn extract_field(text: &str, field: &str) -> Option<String> {
    let label = format!("{field}:");
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix(&label) {
            return Some(rest.trim().to_string());
        }
    }
    None
}

/// The captured outcome of a subprocess invocation.
enum Captured {
    /// Exit status zero, with bounded stdout retained for local inspection.
    Ok { stdout: Vec<u8> },
    /// `claude` could not be resolved on `PATH`.
    Missing,
    /// Execution was denied by policy or failed to spawn for a non-missing reason.
    Denied,
    /// Exited nonzero. `code` distinguishes exit 1 (the documented "not found"
    /// signal from `claude mcp get`) from other nonzero codes (ambiguous).
    NonZero { code: Option<i32>, _stdout: Vec<u8> },
}

/// Build a `Command` for `claude` resolved from `PATH`, using an argument
/// vector (never a shell). No environment secrets are injected.
fn claude_command() -> Command {
    Command::new("claude")
}

/// Run a command, capturing bounded stdout/stderr, and classify the outcome.
/// Captured output is bounded to `CAPTURE_LIMIT` bytes and never emitted in
/// observations.
fn run_capture(cmd: &mut Command) -> Captured {
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Captured::Missing,
        Err(_) => return Captured::Denied,
    };

    let bound = output.stdout.len().min(CAPTURE_LIMIT);
    let stdout = output.stdout[..bound].to_vec();

    if output.status.success() {
        Captured::Ok { stdout }
    } else {
        Captured::NonZero {
            code: output.status.code(),
            _stdout: stdout,
        }
    }
}

/// Build a redacted `ManualRequired` result with a stable observation.
fn manual_required(reason: &str) -> ClientExecutionResult {
    ClientExecutionResult {
        outcome: ClientExecutionOutcome::ManualRequired,
        observations: vec![reason.to_string()],
    }
}
