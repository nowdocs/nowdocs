//! Integration tests for the Codex CLI adapter (C6E).
//!
//! These tests prove the conditional global registration workflow through the
//! official `codex mcp` CLI: exact argv shape, JSON-based detection,
//! absent-to-add ordering, conflict/no-overwrite, malformed JSON handling,
//! exact canonical verification, rollback refusal after a user replacement,
//! missing CLI, and observation redaction. They isolate `PATH` and `HOME`/XDG
//! and use a temporary fake `codex` executable; no real Codex configuration or
//! network is touched, and no test reads `~/.codex/config.toml`.

#[cfg(unix)]
use std::path::Path;
use std::path::PathBuf;
#[cfg(unix)]
use std::sync::Mutex;

use nowdocs::agent_contract::CapabilitySupport;
use nowdocs::clients::{all_adapters, ClientAdapter, ClientId};
#[cfg(unix)]
use nowdocs::clients::{approved_root, ClientExecutionOutcome, ClientExecutionRequest};

/// Serialize environment-mutating tests. `cargo test` runs threads in parallel
/// and these tests set `PATH` and `HOME` process-wide.
#[cfg(unix)]
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Find the Codex adapter in the registered adapter set.
fn codex_adapter() -> Box<dyn ClientAdapter> {
    all_adapters()
        .into_iter()
        .find(|a| a.id() == ClientId::Codex)
        .expect("codex adapter is registered")
}

/// A scratch environment with an isolated `PATH` and `HOME`/XDG plus an
/// optional fake `codex` executable.
#[cfg(unix)]
struct FakeEnv {
    #[allow(dead_code)]
    tmp: tempfile::TempDir,
    bin_dir: PathBuf,
    home: PathBuf,
    argv_log: PathBuf,
    control: PathBuf,
}

#[cfg(unix)]
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

    /// Write a fake `codex` into the isolated bin dir. The script appends its
    /// argv (NUL-delimited) to `argv_log` so tests can prove invocation
    /// ordering across multiple adapter calls, and consults `control` to
    /// decide exit/output, mirroring the observed `codex mcp` CLI contract:
    ///
    /// - `get --json`: exit 0 + a JSON registration object shaped like the
    ///   observed `{"name", "enabled", "transport":{"type","command","args"}}`
    ///   contract when the control file names a present variant; exit 1 when
    ///   `absent` (the observed missing-entry exit); exit 2 when `ambiguous`.
    /// - `add`/`remove`: exit 0 and record argv.
    fn install_fake_codex(&self, control_contents: &str) {
        std::fs::write(&self.control, control_contents).unwrap();
        let script = r#"#!/bin/sh
{
  printf 'codex\0'
  for a in "$@"; do
    printf '%s\0' "$a"
  done
} >> "__ARGV_LOG__"
subcmd="$2"
case "$subcmd" in
  get)
    state=absent
    if [ -r "__CONTROL__" ]; then
      read -r state < "__CONTROL__" || true
    fi
    case "$state" in
      present)
        printf '%s\n' '{"name":"nowdocs","enabled":true,"transport":{"type":"stdio","command":"/bin/nowdocs","args":["serve"]}}'
        exit 0
        ;;
      present-other)
        printf '%s\n' '{"name":"nowdocs","enabled":true,"transport":{"type":"stdio","command":"/bin/other-nowdocs","args":["serve"]}}'
        exit 0
        ;;
      disabled)
        printf '%s\n' '{"name":"nowdocs","enabled":false,"transport":{"type":"stdio","command":"/bin/nowdocs","args":["serve"]}}'
        exit 0
        ;;
      wrong-args)
        printf '%s\n' '{"name":"nowdocs","enabled":true,"transport":{"type":"stdio","command":"/bin/nowdocs","args":["serve","--verbose"]}}'
        exit 0
        ;;
      wrong-name)
        printf '%s\n' '{"name":"other-server","enabled":true,"transport":{"type":"stdio","command":"/bin/nowdocs","args":["serve"]}}'
        exit 0
        ;;
      malformed)
        printf 'this is not json\n'
        exit 0
        ;;
      ambiguous)
        exit 2
        ;;
      *)
        printf "Error: No MCP server named 'nowdocs' found.\n" >&2
        exit 1
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
"#;
        let script = script
            .replace("__ARGV_LOG__", &self.argv_log.display().to_string())
            .replace("__CONTROL__", &self.control.display().to_string());
        let codex = self.bin_dir.join("codex");
        std::fs::write(&codex, script).unwrap();
        make_executable(&codex);
    }

    /// Read the NUL-delimited argv recorded by the fake `codex`, flattened
    /// across invocations so ordering is observable.
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
    /// the fake bin dir so no real `codex` (or other system binary) can leak
    /// into the test outcome. The returned guard restores the original
    /// environment values on drop.
    fn activate(&self) -> EnvGuard {
        let saved_path = std::env::var_os("PATH");
        let saved_home = std::env::var_os("HOME");
        let saved_xdg_config = std::env::var_os("XDG_CONFIG_HOME");
        let saved_xdg_data = std::env::var_os("XDG_DATA_HOME");
        let saved_xdg_cache = std::env::var_os("XDG_CACHE_HOME");

        // Replace PATH entirely with the isolated bin dir. The fake `codex`
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

/// RAII guard that restores the original environment values on drop so that
/// isolated `PATH`/`HOME` mutations never leak across serially-run tests.
#[cfg(unix)]
struct EnvGuard {
    saved_path: Option<std::ffi::OsString>,
    saved_home: Option<std::ffi::OsString>,
    saved_xdg_config: Option<std::ffi::OsString>,
    saved_xdg_data: Option<std::ffi::OsString>,
    saved_xdg_cache: Option<std::ffi::OsString>,
}

#[cfg(unix)]
impl Drop for EnvGuard {
    fn drop(&mut self) {
        restore("PATH", &self.saved_path);
        restore("HOME", &self.saved_home);
        restore("XDG_CONFIG_HOME", &self.saved_xdg_config);
        restore("XDG_DATA_HOME", &self.saved_xdg_data);
        restore("XDG_CACHE_HOME", &self.saved_xdg_cache);
    }
}

#[cfg(unix)]
fn restore(key: &str, saved: &Option<std::ffi::OsString>) {
    match saved {
        Some(v) => std::env::set_var(key, v),
        None => std::env::remove_var(key),
    }
}

#[cfg(unix)]
fn make_request(root: &nowdocs::clients::ApprovedRoot, binary: &Path) -> ClientExecutionRequest {
    ClientExecutionRequest::new("op-c6e-1", root.clone(), binary.to_path_buf()).unwrap()
}

/// Create a stub absolute nowdocs binary path inside the scratch dir. The
/// adapter only compares the path; it never executes or reads it.
#[cfg(unix)]
fn stub_binary(env: &FakeEnv) -> PathBuf {
    let binary = env.tmp.path().join("nowdocs");
    std::fs::write(&binary, b"#! /bin/sh\n").unwrap();
    make_executable(&binary);
    binary
}

// ---------------------------------------------------------------------------
// RED tests (written before implementation; expected to fail until GREEN).
// ---------------------------------------------------------------------------

#[test]
fn codex_client_id_is_stable_and_registered_in_lexical_order() {
    use std::str::FromStr;
    assert_eq!(ClientId::from_str("codex").unwrap(), ClientId::Codex);
    assert_eq!(ClientId::Codex.as_str(), "codex");

    let ids: Vec<String> = all_adapters().iter().map(|a| a.id().to_string()).collect();
    assert_eq!(
        ids,
        vec![
            "claude-code".to_string(),
            "claude-desktop".to_string(),
            "codex".to_string(),
            "cursor".to_string(),
            "generic".to_string(),
        ],
        "adapters must remain in deterministic lexical client order"
    );
}

#[test]
fn generate_produces_exact_global_argv() {
    let adapter = codex_adapter();
    let binary = PathBuf::from("/abs/path/to/nowdocs");
    let config = adapter.generate(&binary).unwrap();

    assert_eq!(
        config.stdio_command,
        vec![
            "codex".to_string(),
            "mcp".to_string(),
            "add".to_string(),
            "nowdocs".to_string(),
            "--".to_string(),
            "/abs/path/to/nowdocs".to_string(),
            "serve".to_string(),
        ]
    );
}

#[test]
fn generate_fragment_contains_placeholder_never_absolute_path() {
    let adapter = codex_adapter();
    let binary = PathBuf::from("/abs/path/to/nowdocs");
    let config = adapter.generate(&binary).unwrap();

    assert!(
        !config.redacted_fragment.contains("/abs/path/to/nowdocs"),
        "fragment leaked the absolute binary path: {}",
        config.redacted_fragment
    );
    assert!(
        config.redacted_fragment.contains("<binary>"),
        "fragment must use the <binary> placeholder: {}",
        config.redacted_fragment
    );
}

#[test]
fn capabilities_advertise_conditional_apply_and_verify() {
    let adapter = codex_adapter();
    let caps = adapter.capabilities();
    assert_eq!(caps.detect, CapabilitySupport::Supported);
    assert_eq!(caps.generate, CapabilitySupport::Supported);
    assert_eq!(caps.apply, CapabilitySupport::Conditional);
    assert_eq!(caps.verify, CapabilitySupport::Conditional);
}

#[test]
#[cfg(unix)]
fn detect_reports_structurally_valid_registration() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    env.install_fake_codex("present");
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let adapter = codex_adapter();
    let detection = adapter.detect(&root).unwrap();
    assert!(
        detection.detected,
        "a successful, structurally valid named registration must be detected"
    );
}

#[test]
#[cfg(unix)]
fn detect_reports_absent_entry_as_not_detected() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    env.install_fake_codex("absent");
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let adapter = codex_adapter();
    let detection = adapter.detect(&root).unwrap();
    assert!(!detection.detected);
    assert!(
        !detection.observations.is_empty(),
        "a not-detected result must carry a stable observation"
    );
}

#[test]
#[cfg(unix)]
fn detect_reports_malformed_json_as_not_detected() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    env.install_fake_codex("malformed");
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let adapter = codex_adapter();
    let detection = adapter.detect(&root).unwrap();
    assert!(
        !detection.detected,
        "malformed JSON must never be reported as detected"
    );
}

#[test]
#[cfg(unix)]
fn detect_reports_wrong_name_as_not_detected() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    env.install_fake_codex("wrong-name");
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let adapter = codex_adapter();
    let detection = adapter.detect(&root).unwrap();
    assert!(
        !detection.detected,
        "a registration under another name must not count as detected"
    );
}

#[test]
#[cfg(unix)]
fn detect_reports_missing_cli_as_not_detected() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    // No fake `codex` installed: PATH points only at the empty bin dir.
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let adapter = codex_adapter();
    let detection = adapter.detect(&root).unwrap();
    assert!(!detection.detected);
    assert!(!detection.observations.is_empty());
}

#[test]
#[cfg(unix)]
fn apply_runs_get_then_exact_add_when_absent() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    env.install_fake_codex("absent");
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let binary = stub_binary(&env);
    let request = make_request(&root, &binary);

    let adapter = codex_adapter();
    let result = adapter.apply(&request).unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::Applied);

    // The flattened transcript proves ordering: the read-only `get --json`
    // absence check runs before exactly one `add` with the locked argv.
    let argv = env.recorded_argv();
    assert_eq!(
        argv,
        vec![
            "codex".to_string(),
            "mcp".to_string(),
            "get".to_string(),
            "nowdocs".to_string(),
            "--json".to_string(),
            "codex".to_string(),
            "mcp".to_string(),
            "add".to_string(),
            "nowdocs".to_string(),
            "--".to_string(),
            binary.display().to_string(),
            "serve".to_string(),
        ]
    );
}

#[test]
#[cfg(unix)]
fn apply_refuses_existing_nowdocs_entry_as_conflict() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    // The fake `codex mcp get nowdocs --json` reports a valid registration.
    env.install_fake_codex("present");
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let binary = stub_binary(&env);
    let request = make_request(&root, &binary);

    let adapter = codex_adapter();
    let result = adapter.apply(&request).unwrap();
    assert_eq!(
        result.outcome,
        ClientExecutionOutcome::Conflict,
        "an existing nowdocs registration must be a Conflict, never overwritten"
    );
    // The fake `codex` must NOT have received an `add` invocation.
    assert!(
        env.recorded_argv().iter().all(|a| a != "add"),
        "apply must not invoke `codex mcp add` when the registration already exists"
    );
}

#[test]
#[cfg(unix)]
fn apply_returns_manual_required_on_malformed_get_output() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    env.install_fake_codex("malformed");
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let binary = stub_binary(&env);
    let request = make_request(&root, &binary);

    let adapter = codex_adapter();
    let result = adapter.apply(&request).unwrap();
    assert_eq!(
        result.outcome,
        ClientExecutionOutcome::ManualRequired,
        "malformed get output is ambiguous and must not trigger an add"
    );
    assert!(env.recorded_argv().iter().all(|a| a != "add"));
}

#[test]
#[cfg(unix)]
fn apply_returns_manual_required_on_non_absence_get_failure() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    // Exit 2 is not the observed missing-entry exit, so absence is not proven.
    env.install_fake_codex("ambiguous");
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let binary = stub_binary(&env);
    let request = make_request(&root, &binary);

    let adapter = codex_adapter();
    let result = adapter.apply(&request).unwrap();
    assert_eq!(
        result.outcome,
        ClientExecutionOutcome::ManualRequired,
        "a non-absence get failure must never be mistaken for a clear absence"
    );
    assert!(env.recorded_argv().iter().all(|a| a != "add"));
}

#[test]
#[cfg(unix)]
fn apply_returns_manual_required_when_cli_missing() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    // No fake `codex` installed: PATH points only at the empty bin dir.
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let binary = stub_binary(&env);
    let request = make_request(&root, &binary);

    let adapter = codex_adapter();
    let result = adapter.apply(&request).unwrap();
    assert_eq!(
        result.outcome,
        ClientExecutionOutcome::ManualRequired,
        "a missing `codex` CLI must be ManualRequired, never a false success"
    );
}

#[test]
#[cfg(unix)]
fn verify_confirms_exact_canonical_registration() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    // `get --json` reports an enabled stdio registration whose command is
    // `/bin/nowdocs` with args exactly `["serve"]`, matching the request.
    env.install_fake_codex("present");
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let binary = PathBuf::from("/bin/nowdocs");
    let request = make_request(&root, &binary);

    let adapter = codex_adapter();
    let result = adapter.verify(&request).unwrap();
    assert_eq!(
        result.outcome,
        ClientExecutionOutcome::Verified,
        "a canonical registration (name, enabled, stdio, exact binary, [serve]) must verify"
    );
}

#[test]
#[cfg(unix)]
fn verify_returns_manual_required_on_command_mismatch() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    // Registered command is `/bin/other-nowdocs`, not the requested binary.
    env.install_fake_codex("present-other");
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let binary = PathBuf::from("/bin/nowdocs");
    let request = make_request(&root, &binary);

    let adapter = codex_adapter();
    let result = adapter.verify(&request).unwrap();
    assert_eq!(
        result.outcome,
        ClientExecutionOutcome::ManualRequired,
        "a command mismatch must never be a false Verified"
    );
}

#[test]
#[cfg(unix)]
fn verify_returns_manual_required_on_disabled_entry() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    env.install_fake_codex("disabled");
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let binary = PathBuf::from("/bin/nowdocs");
    let request = make_request(&root, &binary);

    let adapter = codex_adapter();
    let result = adapter.verify(&request).unwrap();
    assert_eq!(
        result.outcome,
        ClientExecutionOutcome::ManualRequired,
        "a disabled registration must not verify"
    );
}

#[test]
#[cfg(unix)]
fn verify_returns_manual_required_on_wrong_args() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    env.install_fake_codex("wrong-args");
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let binary = PathBuf::from("/bin/nowdocs");
    let request = make_request(&root, &binary);

    let adapter = codex_adapter();
    let result = adapter.verify(&request).unwrap();
    assert_eq!(
        result.outcome,
        ClientExecutionOutcome::ManualRequired,
        "args other than exactly [\"serve\"] must not verify"
    );
}

#[test]
#[cfg(unix)]
fn verify_returns_manual_required_when_cli_missing() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let binary = PathBuf::from("/bin/nowdocs");
    let request = make_request(&root, &binary);

    let adapter = codex_adapter();
    let result = adapter.verify(&request).unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::ManualRequired);
}

#[test]
#[cfg(unix)]
fn rollback_removes_only_after_canonical_match() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    env.install_fake_codex("present");
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let binary = PathBuf::from("/bin/nowdocs");
    let request = make_request(&root, &binary);

    let adapter = codex_adapter();
    let result = adapter.rollback(&request).unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::RolledBack);

    // The canonical re-read must precede exactly one `remove` invocation.
    let argv = env.recorded_argv();
    assert_eq!(
        argv,
        vec![
            "codex".to_string(),
            "mcp".to_string(),
            "get".to_string(),
            "nowdocs".to_string(),
            "--json".to_string(),
            "codex".to_string(),
            "mcp".to_string(),
            "remove".to_string(),
            "nowdocs".to_string(),
        ]
    );
}

#[test]
#[cfg(unix)]
fn rollback_refuses_after_user_replacement() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    // The registration now points at a different binary: a later user
    // replacement that rollback must not remove.
    env.install_fake_codex("present-other");
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let binary = PathBuf::from("/bin/nowdocs");
    let request = make_request(&root, &binary);

    let adapter = codex_adapter();
    let result = adapter.rollback(&request).unwrap();
    assert_eq!(
        result.outcome,
        ClientExecutionOutcome::ManualRequired,
        "a replaced registration must be left intact, never removed"
    );
    assert!(
        env.recorded_argv().iter().all(|a| a != "remove"),
        "rollback must not invoke `codex mcp remove` after a user replacement"
    );
}

#[test]
#[cfg(unix)]
fn rollback_returns_manual_required_when_cli_missing() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let binary = PathBuf::from("/bin/nowdocs");
    let request = make_request(&root, &binary);

    let adapter = codex_adapter();
    let result = adapter.rollback(&request).unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::ManualRequired);
}

#[test]
#[cfg(unix)]
fn observations_are_redacted_across_all_outcomes() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let env = FakeEnv::new();
    env.install_fake_codex("absent");
    let _env_guard = env.activate();

    let root = approved_root(&env.home).unwrap();
    let binary = stub_binary(&env);
    let request = make_request(&root, &binary);
    let adapter = codex_adapter();

    // Helper: every observation must be free of absolute paths, HOME, tokens,
    // raw CLI argv fragments, and captured command output.
    let assert_redacted = |label: &str, observations: &[String]| {
        for obs in observations {
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
            assert!(
                !obs.contains(&binary.display().to_string()),
                "{label}: observation must not leak the binary path: {obs}"
            );
        }
    };

    // Detect paths (absent then present).
    let absent_detection = adapter.detect(&root).unwrap();
    assert!(!absent_detection.detected);
    assert_redacted("detect-absent", &absent_detection.observations);

    // Applied path (absent -> add succeeds).
    let applied = adapter.apply(&request).unwrap();
    assert_eq!(applied.outcome, ClientExecutionOutcome::Applied);
    assert_redacted("apply-applied", &applied.observations);

    // Conflict path (present -> refuse).
    std::fs::write(&env.control, "present").unwrap();
    let conflict = adapter.apply(&request).unwrap();
    assert_eq!(conflict.outcome, ClientExecutionOutcome::Conflict);
    assert_redacted("apply-conflict", &conflict.observations);

    // Detected path.
    let present_detection = adapter.detect(&root).unwrap();
    assert!(present_detection.detected);
    assert_redacted("detect-present", &present_detection.observations);

    // Verified path (present, matching binary).
    let matched_request = make_request(&root, &PathBuf::from("/bin/nowdocs"));
    let verified = adapter.verify(&matched_request).unwrap();
    assert_eq!(verified.outcome, ClientExecutionOutcome::Verified);
    assert_redacted("verify-verified", &verified.observations);

    // ManualRequired path (present, mismatched binary).
    let mismatched = adapter.verify(&request).unwrap();
    assert_eq!(mismatched.outcome, ClientExecutionOutcome::ManualRequired);
    assert_redacted("verify-mismatch", &mismatched.observations);

    // RolledBack path (canonical match).
    let rolled = adapter.rollback(&matched_request).unwrap();
    assert_eq!(rolled.outcome, ClientExecutionOutcome::RolledBack);
    assert_redacted("rollback", &rolled.observations);

    // Rollback refusal path (user replacement).
    std::fs::write(&env.control, "present-other").unwrap();
    let refused = adapter.rollback(&matched_request).unwrap();
    assert_eq!(refused.outcome, ClientExecutionOutcome::ManualRequired);
    assert_redacted("rollback-refused", &refused.observations);
}
