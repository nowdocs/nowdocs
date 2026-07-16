//! Codex CLI adapter.
//!
//! C6E policy: manage the global `nowdocs` registration exclusively through
//! the official `codex mcp` CLI. The adapter never reads, parses, modifies,
//! or exposes `~/.codex/config.toml` or any other Codex-owned file, and never
//! derives config paths from HOME/XDG. It resolves `codex` from `PATH`,
//! invokes it with argument vectors (never a shell), bounds captured output,
//! and emits only redacted observations containing no absolute path, HOME,
//! token, or raw CLI output.

use std::io::Read;
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

pub struct CodexAdapter;

impl ClientAdapter for CodexAdapter {
    fn id(&self) -> ClientId {
        ClientId::Codex
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            detect: CapabilitySupport::Supported,
            generate: CapabilitySupport::Supported,
            apply: CapabilitySupport::Conditional,
            verify: CapabilitySupport::Conditional,
        }
    }

    fn detect(&self, _root: &ApprovedRoot) -> Result<Detection> {
        // Read-only: the only probe is `codex mcp get nowdocs --json`. The
        // approved root is an explicit trait input, not a permission to
        // inspect a real home directory, so no filesystem path is touched.
        let mut cmd = codex_command();
        cmd.args(["mcp", "get", "nowdocs", "--json"]);
        let (detected, observation) = match run_capture(&mut cmd) {
            Captured::Ok { stdout } => match parse_registration(&stdout) {
                Some(registration) if registration.name == "nowdocs" => {
                    (true, "codex mcp get returned a valid nowdocs registration")
                }
                _ => (
                    false,
                    "codex mcp get returned an unrecognized registration payload",
                ),
            },
            Captured::Missing => (false, "codex CLI not found on PATH"),
            Captured::Denied | Captured::NonZero { .. } => {
                (false, "codex mcp get did not complete cleanly")
            }
        };
        Ok(Detection {
            detected,
            target_path: None,
            observations: vec![observation.to_string()],
        })
    }

    fn generate(&self, binary_path: &Path) -> Result<GeneratedConfig> {
        let stdio_command = vec![
            "codex".to_string(),
            "mcp".to_string(),
            "add".to_string(),
            "nowdocs".to_string(),
            "--".to_string(),
            binary_path.display().to_string(),
            "serve".to_string(),
        ];

        Ok(GeneratedConfig {
            stdio_command,
            redacted_fragment: "codex mcp add nowdocs -- <binary> serve".to_string(),
            manual_steps: vec![
                "Run the generated codex mcp command to create the global nowdocs registration."
                    .to_string(),
            ],
        })
    }

    fn apply(&self, request: &ClientExecutionRequest) -> Result<ClientExecutionResult> {
        let binary = request.binary_path();

        // First inspect whether a nowdocs registration already exists through
        // the official CLI. A present or indeterminate entry is never replaced.
        match codex_entry_state() {
            EntryState::Present => Ok(ClientExecutionResult {
                outcome: ClientExecutionOutcome::Conflict,
                observations: vec![
                    "codex already has a nowdocs registration; refusing to overwrite".to_string(),
                ],
            }),
            EntryState::Absent => {
                // Clear absent result: invoke the exact global add form.
                let mut cmd = codex_command();
                cmd.args(["mcp", "add", "nowdocs", "--"]);
                cmd.arg(binary);
                cmd.arg("serve");
                match run_capture(&mut cmd) {
                    Captured::Ok { .. } => Ok(ClientExecutionResult {
                        outcome: ClientExecutionOutcome::Applied,
                        observations: vec!["codex global nowdocs registration added".to_string()],
                    }),
                    Captured::Missing => Ok(manual_required("codex CLI not found on PATH")),
                    Captured::Denied | Captured::NonZero { .. } => Ok(manual_required(
                        "codex mcp add did not complete cleanly; manual review required",
                    )),
                }
            }
            EntryState::Ambiguous => Ok(manual_required(
                "codex mcp get returned an indeterminate result; manual review required",
            )),
            EntryState::Missing => Ok(manual_required("codex CLI not found on PATH")),
        }
    }

    fn verify(&self, request: &ClientExecutionRequest) -> Result<ClientExecutionResult> {
        let binary = request.binary_path();
        let mut cmd = codex_command();
        cmd.args(["mcp", "get", "nowdocs", "--json"]);
        match run_capture(&mut cmd) {
            Captured::Ok { stdout } => {
                if registration_matches(&stdout, binary) {
                    Ok(ClientExecutionResult {
                        outcome: ClientExecutionOutcome::Verified,
                        observations: vec!["codex global nowdocs registration verified".to_string()],
                    })
                } else {
                    Ok(manual_required(
                        "codex nowdocs registration does not match the expected binary and serve argument",
                    ))
                }
            }
            Captured::Missing => Ok(manual_required("codex CLI not found on PATH")),
            Captured::Denied | Captured::NonZero { .. } => Ok(manual_required(
                "codex mcp get did not complete cleanly; manual review required",
            )),
        }
    }

    fn rollback(&self, request: &ClientExecutionRequest) -> Result<ClientExecutionResult> {
        let binary = request.binary_path();

        // Re-read the registration and require the same exact canonical entry
        // as `verify` before removal, so a later user replacement is never
        // removed.
        let mut cmd = codex_command();
        cmd.args(["mcp", "get", "nowdocs", "--json"]);
        match run_capture(&mut cmd) {
            Captured::Ok { stdout } => {
                if !registration_matches(&stdout, binary) {
                    return Ok(manual_required(
                        "codex nowdocs registration does not match the canonical entry; refusing to remove",
                    ));
                }
                let mut remove = codex_command();
                remove.args(["mcp", "remove", "nowdocs"]);
                match run_capture(&mut remove) {
                    Captured::Ok { .. } => Ok(ClientExecutionResult {
                        outcome: ClientExecutionOutcome::RolledBack,
                        observations: vec!["codex global nowdocs registration removed".to_string()],
                    }),
                    Captured::Missing => Ok(manual_required("codex CLI not found on PATH")),
                    Captured::Denied | Captured::NonZero { .. } => Ok(manual_required(
                        "codex mcp remove did not complete cleanly; configuration left intact",
                    )),
                }
            }
            Captured::Missing => Ok(manual_required("codex CLI not found on PATH")),
            Captured::Denied | Captured::NonZero { .. } => Ok(manual_required(
                "codex mcp get did not complete cleanly; configuration left intact",
            )),
        }
    }
}

/// Observed state of the `nowdocs` registration from `codex mcp get --json`.
enum EntryState {
    Present,
    Absent,
    Ambiguous,
    Missing,
}

/// Inspect the `nowdocs` registration through the official CLI without
/// modifying it.
fn codex_entry_state() -> EntryState {
    let mut cmd = codex_command();
    cmd.args(["mcp", "get", "nowdocs", "--json"]);
    match run_capture(&mut cmd) {
        Captured::Ok { stdout } => match parse_registration(&stdout) {
            Some(registration) if registration.name == "nowdocs" => EntryState::Present,
            // A successful exit whose payload is malformed or names another
            // server is ambiguous: absence is not proven.
            _ => EntryState::Ambiguous,
        },
        Captured::Missing => EntryState::Missing,
        Captured::Denied => EntryState::Ambiguous,
        // `codex mcp get` exits 1 when the named server does not exist
        // (observed against codex-cli 0.144.x: "Error: No MCP server named
        // '<name>' found."). Any other nonzero code is treated as
        // indeterminate (Ambiguous) so that a transient CLI error can never be
        // mistaken for a clear absence and trigger an unintended `mcp add`.
        Captured::NonZero { code: Some(1), .. } => EntryState::Absent,
        Captured::NonZero { .. } => EntryState::Ambiguous,
    }
}

/// A parsed `codex mcp get nowdocs --json` registration payload, matching the
/// shape established by the C6E smoke: `name`, `enabled`, and a `transport`
/// object with `type`, `command`, and `args`.
struct Registration {
    name: String,
    enabled: bool,
    transport_type: String,
    command: String,
    args: Vec<String>,
}

/// Parse the JSON registration emitted by `codex mcp get --json`. Returns
/// `None` on any structural deviation rather than risking a false positive.
fn parse_registration(stdout: &[u8]) -> Option<Registration> {
    let value: serde_json::Value = serde_json::from_slice(stdout).ok()?;
    let name = value.get("name")?.as_str()?.to_string();
    let enabled = value.get("enabled")?.as_bool()?;
    let transport = value.get("transport")?;
    let transport_type = transport.get("type")?.as_str()?.to_string();
    let command = transport.get("command")?.as_str()?.to_string();
    let args = transport
        .get("args")?
        .as_array()?
        .iter()
        .map(|arg| arg.as_str().map(str::to_string))
        .collect::<Option<Vec<String>>>()?;
    Some(Registration {
        name,
        enabled,
        transport_type,
        command,
        args,
    })
}

/// Check whether captured `codex mcp get --json` output confirms the exact
/// canonical registration: name `nowdocs`, enabled, stdio transport, command
/// exactly the requested absolute binary, and args exactly `["serve"]`.
fn registration_matches(stdout: &[u8], expected_binary: &Path) -> bool {
    match parse_registration(stdout) {
        Some(registration) => {
            registration.name == "nowdocs"
                && registration.enabled
                && registration.transport_type == "stdio"
                && Path::new(&registration.command) == expected_binary
                && registration.args.len() == 1
                && registration.args[0] == "serve"
        }
        None => false,
    }
}

/// The captured outcome of a subprocess invocation.
enum Captured {
    /// Exit status zero, with bounded stdout retained for local inspection.
    Ok { stdout: Vec<u8> },
    /// `codex` could not be resolved on `PATH`.
    Missing,
    /// Execution was denied by policy or failed to spawn for a non-missing reason.
    Denied,
    /// Exited nonzero. `code` distinguishes exit 1 (the observed "no such
    /// server" signal from `codex mcp get`) from other nonzero codes
    /// (ambiguous).
    NonZero { code: Option<i32>, _stdout: Vec<u8> },
}

/// Build a `Command` for `codex` resolved from `PATH`, using an argument
/// vector (never a shell). No environment secrets are injected.
fn codex_command() -> Command {
    Command::new("codex")
}

/// Run a command and classify the outcome without retaining unbounded output.
/// Stderr is discarded because classification depends only on the exit status;
/// stdout is drained completely but retains at most `CAPTURE_LIMIT` bytes for
/// local JSON classification and is never emitted in observations.
fn run_capture(cmd: &mut Command) -> Captured {
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::null());

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Captured::Missing,
        Err(_) => return Captured::Denied,
    };

    let Some(stdout_pipe) = child.stdout.take() else {
        return Captured::Denied;
    };
    let reader = std::thread::spawn(move || drain_capped(stdout_pipe));
    let status = match child.wait() {
        Ok(status) => status,
        Err(_) => return Captured::Denied,
    };
    let stdout = match reader.join() {
        Ok(Ok(stdout)) => stdout,
        _ => return Captured::Denied,
    };

    if status.success() {
        Captured::Ok { stdout }
    } else {
        Captured::NonZero {
            code: status.code(),
            _stdout: stdout,
        }
    }
}

/// Drain a readable stream to avoid a child-process pipe deadlock while
/// retaining only a bounded prefix for classification.
fn drain_capped<R: Read>(mut reader: R) -> std::io::Result<Vec<u8>> {
    let mut retained = Vec::with_capacity(CAPTURE_LIMIT);
    let mut buffer = [0_u8; 4096];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        let remaining = CAPTURE_LIMIT.saturating_sub(retained.len());
        retained.extend_from_slice(&buffer[..count.min(remaining)]);
    }
    Ok(retained)
}

/// Build a redacted `ManualRequired` result with a stable observation.
fn manual_required(reason: &str) -> ClientExecutionResult {
    ClientExecutionResult {
        outcome: ClientExecutionOutcome::ManualRequired,
        observations: vec![reason.to_string()],
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::{drain_capped, CAPTURE_LIMIT};

    struct RepeatingReader {
        remaining: usize,
    }

    impl Read for RepeatingReader {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if self.remaining == 0 {
                return Ok(0);
            }
            let count = self.remaining.min(buf.len());
            buf[..count].fill(b'x');
            self.remaining -= count;
            Ok(count)
        }
    }

    #[test]
    fn drain_capped_discards_excess_output_after_the_retention_limit() {
        let retained = drain_capped(RepeatingReader {
            remaining: CAPTURE_LIMIT + 1,
        })
        .expect("reader succeeds");

        assert_eq!(retained.len(), CAPTURE_LIMIT);
    }
}
