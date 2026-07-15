//! Integration tests for the Claude Code adapter (C6A).
//!
//! These tests prove the conditional user-scope configuration workflow through
//! the official `claude mcp` CLI: argv shape, conflict/no-overwrite, successful
//! apply, verification mismatch, operation-owned rollback, missing CLI, and
//! redaction. They isolate `PATH` and `HOME`/XDG and use a temporary fake
//! `claude` executable; no real client configuration or network is touched.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use nowdocs::agent_contract::CapabilitySupport;
use nowdocs::clients::{
    all_adapters, approved_root, ClientAdapter, ClientExecutionOutcome, ClientExecutionRequest,
    ClientId,
};

/// Serialize environment-mutating tests. `cargo test` runs threads in parallel
/// and these tests set `PATH` and `HOME` process-wide.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Find the Claude Code adapter in the registered adapter set.
fn claude_code_adapter() -> Box<dyn ClientAdapter> {
    all_adapters()
        .into_iter()
        .find(|a| a.id() == ClientId::ClaudeCode)
        .expect("claude-code adapter is registered")
}

/// A scratch environment with an isolated `PATH` and `HOME`/XDG plus an optional
/// fake `claude` executable.
struct FakeEnv {
    #[allow(dead_code)]
    tmp: tempfile::TempDir,
    bin_dir: PathBuf,
    home: PathBuf,
    argv_log: PathBuf,
    control: PathBuf,
}

impl FakeEnv {
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let bin_dir = tmp.path().join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        let home = tmp.path().join("home");
        std::fs::create_dir_all(&home).unwrap();
        let argv_log = tmp.path().join("argv.log");
        let control = tmp.path().join("control");
        Self {
            tmp,
            bin_dir,
            home,
            argv_log,
            control,
        }
    }

    /// Write a fake `claude` into the isolated bin dir. The script records its
    /// argv to `argv_log` and consults `control` to decide exit/output,
    /// mirroring the real `claude mcp` CLI text format:
    ///
    /// - `get`: exit 0 + emit human-readable server details if the control file
    ///   says `present`; exit 1 if `absent`; exit 2 if `ambiguous`.
    /// - `add`: exit 0 and record argv.
    /// - `remove`: exit 0 and record argv.
    fn install_fake_claude(&self, control_contents: &str) {
        std::fs::write(&self.control, control_contents).unwrap();
        let script = format!(
            r#"#!/bin/sh
printf 'claude\0' > "{argv_log}"
for a in "$@"; do
    printf '%s\0' "$a" >> "{argv_log}"
done
subcmd="$2"
case "$subcmd" in
  get)
    state=absent
    if [ -r "{control}" ]; then
      read -r state < "{control}" || true
    fi
    case "$state" in
      present)
        printf 'nowdocs:\n'
        printf '  Scope: User config\n'
        printf '  Type: stdio\n'
        printf '  Command: /bin/nowdocs\n'
        printf '  Args: serve\n'
        exit 0
        ;;
      absent)
        exit 1
        ;;
      *)
        exit 2
        ;;
    esac
    ;;
  add|remove)
    exit 0
    ;;
  *)
    exit 2
    ;;
esac
"#,
            argv_log = self.argv_log.display(),
            control = self.control.display(),
        );
        let claude = self.bin_dir.join("claude");
        std::fs::write(&claude, script).unwrap();
        make_executable(&claude);
    }

    /// Read the NUL-delimited argv recorded by the fake `claude`.
    fn recorded_argv(&self) -> Vec<String> {
        let bytes = std::fs::read(&self.argv_log).unwrap_or_default();
        bytes
            .split(|&b| b == 0)
            .filter(|s| !s.is_empty())
            .map(|s| String::from_utf8_lossy(s).to_string())
            .collect()
    }

    /// Activate the isolated environment on the current process. The caller
    /// must already hold `ENV_LOCK` so that environment mutations are
    /// serialized across parallel test threads. `PATH` is replaced with only
    /// the fake bin dir so no real `claude` (or other system binary) can leak
    /// into the test outcome. The returned guard restores the original
    /// environment values on drop.
    fn activate(&self) -> EnvGuard {
        let saved_path = std::env::var_os("PATH");
        let saved_home = std::env::var_os("HOME");
        let saved_xdg_config = std::env::var_os("XDG_CONFIG_HOME");
        let saved_xdg_data = std::env::var_os("XDG_DATA_HOME");
        let saved_xdg_cache = std::env::var_os("XDG_CACHE_HOME");

        // Replace PATH entirely with the isolated bin dir. The fake `claude`
        // uses an absolute `#!/bin/sh` shebang, so it does not need PATH.
        std::env::set_var("PATH", &self.bin_dir);
        std::env::set_var("HOME", &self.home);
        std::env::set_var("XDG_CONFIG_HOME", self.home.join(".config"));
        std::env::set_var("XDG_DATA_HOME", self.home.join(".local").join("share"));
        std::env::set_var("XDG_CACHE_HOME", self.home.join(".cache"));

        EnvGuard {
            saved_path,
            saved_home,
            saved_xdg_config,
            saved_xdg_data,
            saved_xdg_cache,
        }
    }
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).unwrap();
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) {}

/// RAII guard that restores the original environment values on drop so that
/// isolated `PATH`/`HOME` mutations never leak across serially-run tests.
struct EnvGuard {
    saved_path: Option<std::ffi::OsString>,
    saved_home: Option<std::ffi::OsString>,
    saved_xdg_config: Option<std::ffi::OsString>,
    saved_xdg_data: Option<std::ffi::OsString>,
    saved_xdg_cache: Option<std::ffi::OsString>,
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        restore("PATH", &self.saved_path);
        restore("HOME", &self.saved_home);
        restore("XDG_CONFIG_HOME", &self.saved_xdg_config);
        restore("XDG_DATA_HOME", &self.saved_xdg_data);
        restore("XDG_CACHE_HOME", &self.saved_xdg_cache);
    }
}

fn restore(key: &str, saved: &Option<std::ffi::OsString>) {
    match saved {
        Some(v) => std::env::set_var(key, v),
        None => std::env::remove_var(key),
    }
}

fn make_request(root: &nowdocs::clients::ApprovedRoot, binary: &Path) -> ClientExecutionRequest {
    ClientExecutionRequest::new("op-c6a-1", root.clone(), binary.to_path_buf()).unwrap()
}

// ---------------------------------------------------------------------------
// RED tests (written before implementation; expected to fail until GREEN).
// ---------------------------------------------------------------------------

#[test]
fn generate_produces_exact_user_scope_argv() {
    let adapter = claude_code_adapter();
    let binary = PathBuf::from("/abs/path/to/nowdocs");
    let config = adapter.generate(&binary).unwrap();

    assert_eq!(
        config.stdio_command,
        vec![
            "claude".to_string(),
            "mcp".to_string(),
            "add".to_string(),
            "--transport".to_string(),
            "stdio".to_string(),
            "--scope".to_string(),
            "user".to_string(),
            "nowdocs".to_string(),
            "--".to_string(),
            "/abs/path/to/nowdocs".to_string(),
            "serve".to_string(),
        ]
    );
}

#[test]
fn capabilities_advertise_conditional_apply_and_verify() {
    let adapter = claude_code_adapter();
    let caps = adapter.capabilities();
    assert_eq!(caps.detect, CapabilitySupport::Supported);
    assert_eq!(caps.generate, CapabilitySupport::Supported);
    assert_eq!(caps.apply, CapabilitySupport::Conditional);
    assert_eq!(caps.verify, CapabilitySupport::Conditional);
}

#[test]
fn apply_refuses_existing_nowdocs_entry_as_conflict() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    // The fake `claude mcp get nowdocs` reports the entry as present.
    env.install_fake_claude("present");
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let binary = env.tmp.path().join("nowdocs");
    std::fs::write(&binary, b"#! /bin/sh\n").unwrap();
    make_executable(&binary);
    let request = make_request(&root, &binary);

    let adapter = claude_code_adapter();
    let result = adapter.apply(&request).unwrap();
    assert_eq!(
        result.outcome,
        ClientExecutionOutcome::Conflict,
        "an existing nowdocs entry must be a Conflict, never overwritten"
    );
    // The fake `claude` must NOT have received an `add` invocation.
    assert!(
        env.recorded_argv().iter().all(|a| a != "add"),
        "apply must not invoke `claude mcp add` when the entry already exists"
    );
}

#[test]
fn apply_invokes_exact_user_scope_add_when_absent() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    env.install_fake_claude("absent");
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let binary = env.tmp.path().join("nowdocs");
    std::fs::write(&binary, b"#! /bin/sh\n").unwrap();
    make_executable(&binary);
    let request = make_request(&root, &binary);

    let adapter = claude_code_adapter();
    let result = adapter.apply(&request).unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::Applied);

    let argv = env.recorded_argv();
    assert_eq!(
        argv,
        vec![
            "claude".to_string(),
            "mcp".to_string(),
            "add".to_string(),
            "--transport".to_string(),
            "stdio".to_string(),
            "--scope".to_string(),
            "user".to_string(),
            "nowdocs".to_string(),
            "--".to_string(),
            binary.display().to_string(),
            "serve".to_string(),
        ]
    );
}

#[test]
fn verify_returns_manual_required_on_mismatch() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    // `get` reports a present entry whose command is `/bin/nowdocs`, which will
    // not match the request's binary path.
    env.install_fake_claude("present");
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let binary = env.tmp.path().join("nowdocs");
    std::fs::write(&binary, b"#! /bin/sh\n").unwrap();
    make_executable(&binary);
    let request = make_request(&root, &binary);

    let adapter = claude_code_adapter();
    let result = adapter.verify(&request).unwrap();
    assert_eq!(
        result.outcome,
        ClientExecutionOutcome::ManualRequired,
        "a binary mismatch must never be a false Verified"
    );
}

#[test]
fn verify_confirms_matched_entry() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    // `get` reports a present entry whose command is `/bin/nowdocs` with
    // args `["serve"]`, matching the request's binary exactly.
    env.install_fake_claude("present");
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let binary = PathBuf::from("/bin/nowdocs");
    let request = make_request(&root, &binary);

    let adapter = claude_code_adapter();
    let result = adapter.verify(&request).unwrap();
    assert_eq!(
        result.outcome,
        ClientExecutionOutcome::Verified,
        "a matching binary and serve argument must verify"
    );
}

#[test]
fn rollback_invokes_scope_aware_remove() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    env.install_fake_claude("present");
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let binary = env.tmp.path().join("nowdocs");
    std::fs::write(&binary, b"#! /bin/sh\n").unwrap();
    make_executable(&binary);
    let request = make_request(&root, &binary);

    let adapter = claude_code_adapter();
    let result = adapter.rollback(&request).unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::RolledBack);

    let argv = env.recorded_argv();
    assert_eq!(
        argv,
        vec![
            "claude".to_string(),
            "mcp".to_string(),
            "remove".to_string(),
            "--scope".to_string(),
            "user".to_string(),
            "nowdocs".to_string(),
        ]
    );
}

#[test]
fn apply_returns_manual_required_when_cli_missing() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    // No fake `claude` installed: PATH points only at the empty bin dir.
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let binary = env.tmp.path().join("nowdocs");
    std::fs::write(&binary, b"#! /bin/sh\n").unwrap();
    make_executable(&binary);
    let request = make_request(&root, &binary);

    let adapter = claude_code_adapter();
    let result = adapter.apply(&request).unwrap();
    assert_eq!(
        result.outcome,
        ClientExecutionOutcome::ManualRequired,
        "a missing `claude` CLI must be ManualRequired, never a false success"
    );
}

#[test]
fn observations_are_redacted_across_all_outcomes() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    env.install_fake_claude("absent");
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let binary = env.tmp.path().join("nowdocs");
    std::fs::write(&binary, b"#! /bin/sh\n").unwrap();
    make_executable(&binary);
    let request = make_request(&root, &binary);
    let adapter = claude_code_adapter();

    // Helper: every observation must be free of absolute paths, HOME, tokens,
    // and raw CLI argv fragments.
    let assert_redacted = |label: &str, result: &nowdocs::clients::ClientExecutionResult| {
        for obs in &result.observations {
            assert!(
                !obs.contains('/'),
                "{label}: observation must not contain a path separator: {obs}"
            );
            assert!(
                !obs.contains("nowdocs serve"),
                "{label}: observation must not leak raw CLI argv: {obs}"
            );
            assert!(
                !obs.contains(&env.home.display().to_string()),
                "{label}: observation must not leak HOME: {obs}"
            );
        }
    };

    // Applied path (absent -> add succeeds).
    let applied = adapter.apply(&request).unwrap();
    assert_eq!(applied.outcome, ClientExecutionOutcome::Applied);
    assert_redacted("apply-applied", &applied);

    // Conflict path (present -> refuse).
    std::fs::write(&env.control, "present").unwrap();
    let conflict = adapter.apply(&request).unwrap();
    assert_eq!(conflict.outcome, ClientExecutionOutcome::Conflict);
    assert_redacted("apply-conflict", &conflict);

    // Verified path (present, matching binary).
    let matched_request = make_request(&root, &PathBuf::from("/bin/nowdocs"));
    let verified = adapter.verify(&matched_request).unwrap();
    assert_eq!(verified.outcome, ClientExecutionOutcome::Verified);
    assert_redacted("verify-verified", &verified);

    // ManualRequired path (present, mismatched binary).
    let mismatched = adapter.verify(&request).unwrap();
    assert_eq!(mismatched.outcome, ClientExecutionOutcome::ManualRequired);
    assert_redacted("verify-mismatch", &mismatched);

    // RolledBack path.
    let rolled = adapter.rollback(&request).unwrap();
    assert_eq!(rolled.outcome, ClientExecutionOutcome::RolledBack);
    assert_redacted("rollback", &rolled);
}
