//! C6C Cursor adapter integration tests.
//!
//! Every test uses a temporary approved root for the client configuration file
//! and a temporary `XDG_CACHE_HOME` for the operation journal/backup subtree.
//! No test reads or writes real HOME, XDG, network, or process state.

use std::path::PathBuf;
use std::sync::Mutex;

use nowdocs::agent_contract::CapabilitySupport;
use nowdocs::automation::operation::{apply_with_backup, rollback, OperationId};
use nowdocs::clients::{
    all_adapters, approved_root, atomic_replace, safe_target, ClientExecutionOutcome,
    ClientExecutionRequest, ClientId,
};

// dirs::cache_dir() reads XDG_CACHE_HOME / HOME at call time, so tests that
// mutate these env vars must not run concurrently. This lock serializes them
// (the test runner is already pinned to a single thread by .cargo/config.toml).
static ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvGuard {
    key: &'static str,
    old: Option<String>,
    _g: std::sync::MutexGuard<'static, ()>,
}

impl EnvGuard {
    fn set(key: &'static str, val: &str) -> Self {
        let g = ENV_LOCK.lock().unwrap();
        let old = std::env::var(key).ok();
        std::env::set_var(key, val);
        Self { key, old, _g: g }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.old {
            Some(v) => std::env::set_var(self.key, v),
            None => std::env::remove_var(self.key),
        }
    }
}

fn tmp_cache_guard(tmp: &tempfile::TempDir) -> EnvGuard {
    EnvGuard::set("XDG_CACHE_HOME", tmp.path().to_str().unwrap())
}

/// A self-contained fixture: a temporary cache (for the journal/backup subtree)
/// plus a temporary approved root (for the client configuration file).
struct Harness {
    _cache: tempfile::TempDir,
    _cache_guard: EnvGuard,
    root_dir: tempfile::TempDir,
}

impl Harness {
    fn new() -> Self {
        let cache = tempfile::tempdir().unwrap();
        let guard = tmp_cache_guard(&cache);
        let root_dir = tempfile::tempdir().unwrap();
        Self {
            _cache: cache,
            _cache_guard: guard,
            root_dir,
        }
    }

    fn root(&self) -> nowdocs::clients::ApprovedRoot {
        approved_root(self.root_dir.path()).unwrap()
    }

    /// An absolute fake binary path inside the approved root so requests are
    /// valid without touching a real executable.
    fn binary(&self) -> PathBuf {
        self.root_dir.path().join("nowdocs")
    }

    fn request(&self, op: &str) -> ClientExecutionRequest {
        ClientExecutionRequest::new(op, self.root(), self.binary()).unwrap()
    }
}

fn cursor_adapter() -> Box<dyn nowdocs::clients::ClientAdapter> {
    all_adapters()
        .into_iter()
        .find(|a| a.id() == ClientId::Cursor)
        .unwrap()
}

/// Write a `.cursor/mcp.json` file under the approved root with the given
/// bytes. The `.cursor` directory is created first.
fn write_cursor_config(root: &nowdocs::clients::ApprovedRoot, bytes: impl AsRef<[u8]>) {
    std::fs::create_dir_all(root.path().join(".cursor")).unwrap();
    std::fs::write(root.path().join(".cursor").join("mcp.json"), bytes.as_ref()).unwrap();
}

fn read_cursor_config(root: &nowdocs::clients::ApprovedRoot) -> Vec<u8> {
    std::fs::read(root.path().join(".cursor").join("mcp.json")).unwrap()
}

// ---------------------------------------------------------------------------
// Capability declaration
// ---------------------------------------------------------------------------

#[test]
fn cursor_capabilities_declare_conditional_apply_and_verify() {
    let cu = cursor_adapter();
    let caps = cu.capabilities();
    assert_eq!(caps.detect, CapabilitySupport::Supported);
    assert_eq!(caps.generate, CapabilitySupport::Supported);
    assert_eq!(caps.apply, CapabilitySupport::Conditional);
    assert_eq!(caps.verify, CapabilitySupport::Conditional);
}

// ---------------------------------------------------------------------------
// Canonical generation (deterministic, redacted)
// ---------------------------------------------------------------------------

#[test]
fn generate_emits_canonical_redacted_fragment() {
    let cu = cursor_adapter();
    let generated = cu
        .generate(&PathBuf::from("/usr/local/bin/nowdocs"))
        .unwrap();
    // The redacted fragment must use a placeholder, never a real path.
    assert_eq!(
        generated.redacted_fragment,
        r#"{"mcpServers":{"nowdocs":{"command":"<binary>","args":["serve"]}}}"#
    );
    assert!(!generated
        .redacted_fragment
        .contains("/usr/local/bin/nowdocs"));
    // The stdio command carries the real absolute binary plus serve.
    assert_eq!(
        generated.stdio_command,
        vec!["/usr/local/bin/nowdocs", "serve"]
    );
}

#[test]
fn generate_is_deterministic_across_calls() {
    let cu = cursor_adapter();
    let binary = PathBuf::from("/opt/nowdocs");
    let a = cu.generate(&binary).unwrap();
    let b = cu.generate(&binary).unwrap();
    assert_eq!(a, b);
}

// ---------------------------------------------------------------------------
// Detection
// ---------------------------------------------------------------------------

#[test]
fn detect_finds_global_cursor_marker() {
    let h = Harness::new();
    write_cursor_config(&h.root(), b"{}");
    let cu = cursor_adapter();
    let detection = cu.detect(&h.root()).unwrap();
    assert!(detection.detected);
    assert_eq!(detection.target_path, Some(".cursor/mcp.json".to_string()));
}

#[test]
fn detect_reports_absent_when_no_marker() {
    let h = Harness::new();
    let cu = cursor_adapter();
    let detection = cu.detect(&h.root()).unwrap();
    assert!(!detection.detected);
    assert_eq!(detection.target_path, None);
}

// ---------------------------------------------------------------------------
// Apply: preserving unrelated data
// ---------------------------------------------------------------------------

#[test]
fn apply_adds_nowdocs_entry_preserving_unrelated_servers() {
    let h = Harness::new();
    let original = serde_json::json!({
        "mcpServers": {
            "filesystem": {
                "command": "/usr/bin/fs",
                "args": ["--root", "/tmp"]
            }
        },
        "cursor": "settings"
    });
    write_cursor_config(&h.root(), serde_json::to_vec_pretty(&original).unwrap());

    let result = cursor_adapter()
        .apply(&h.request("apply-preserve-1"))
        .unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::Applied);

    let after: serde_json::Value = serde_json::from_slice(&read_cursor_config(&h.root())).unwrap();
    let servers = after["mcpServers"].as_object().unwrap();
    // nowdocs was added with the absolute request binary and serve.
    assert_eq!(
        servers["nowdocs"]["command"],
        h.binary().to_string_lossy().to_string()
    );
    assert_eq!(servers["nowdocs"]["args"], serde_json::json!(["serve"]));
    // The unrelated filesystem entry and top-level cursor key survived.
    assert_eq!(servers["filesystem"]["command"], "/usr/bin/fs");
    assert_eq!(
        servers["filesystem"]["args"],
        serde_json::json!(["--root", "/tmp"])
    );
    assert_eq!(after["cursor"], "settings");
}

#[test]
fn apply_creates_nowdocs_in_empty_mcpservers() {
    let h = Harness::new();
    write_cursor_config(&h.root(), br#"{"mcpServers":{}}"#);

    let result = cursor_adapter().apply(&h.request("apply-empty-1")).unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::Applied);

    let after: serde_json::Value = serde_json::from_slice(&read_cursor_config(&h.root())).unwrap();
    assert_eq!(
        after["mcpServers"]["nowdocs"]["command"],
        h.binary().to_string_lossy().to_string()
    );
    assert_eq!(
        after["mcpServers"]["nowdocs"]["args"],
        serde_json::json!(["serve"])
    );
}

#[test]
fn apply_preserves_top_level_keys_without_mcp_servers() {
    let h = Harness::new();
    let original = serde_json::json!({
        "version": 2,
        "telemetry": false
    });
    write_cursor_config(&h.root(), serde_json::to_vec_pretty(&original).unwrap());

    let result = cursor_adapter()
        .apply(&h.request("apply-noservers-1"))
        .unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::Applied);

    let after: serde_json::Value = serde_json::from_slice(&read_cursor_config(&h.root())).unwrap();
    assert_eq!(after["version"], 2);
    assert_eq!(after["telemetry"], false);
    assert_eq!(
        after["mcpServers"]["nowdocs"]["command"],
        h.binary().to_string_lossy().to_string()
    );
}

#[test]
fn apply_observations_contain_no_paths_or_values() {
    let h = Harness::new();
    write_cursor_config(&h.root(), br#"{"mcpServers":{}}"#);

    let result = cursor_adapter()
        .apply(&h.request("apply-redact-1"))
        .unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::Applied);
    for obs in &result.observations {
        assert!(
            !obs.contains(&h.binary().to_string_lossy().to_string()),
            "observation leaked binary path: {obs}"
        );
        assert!(
            !obs.contains(".cursor/mcp.json"),
            "observation leaked target path: {obs}"
        );
        assert!(
            !obs.contains(&h.root_dir.path().to_string_lossy().to_string()),
            "observation leaked approved root: {obs}"
        );
    }
}

// ---------------------------------------------------------------------------
// Apply: conflict (nowdocs entry already exists) - no mutation
// ---------------------------------------------------------------------------

#[test]
fn apply_returns_conflict_when_nowdocs_already_exists() {
    let h = Harness::new();
    let original = serde_json::json!({
        "mcpServers": {
            "nowdocs": {
                "command": "/some/other/nowdocs",
                "args": ["serve"]
            }
        }
    });
    let original_bytes = serde_json::to_vec_pretty(&original).unwrap();
    write_cursor_config(&h.root(), &original_bytes);

    let result = cursor_adapter().apply(&h.request("conflict-1")).unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::Conflict);

    // The file must be byte-identical (no mutation).
    assert_eq!(read_cursor_config(&h.root()), original_bytes);
}

// ---------------------------------------------------------------------------
// Apply: malformed JSON - no mutation, ManualRequired
// ---------------------------------------------------------------------------

#[test]
fn apply_returns_manual_required_for_malformed_json_without_mutation() {
    let h = Harness::new();
    let malformed = b"{ this is not valid json ";
    write_cursor_config(&h.root(), malformed);

    let result = cursor_adapter().apply(&h.request("malformed-1")).unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::ManualRequired);

    // The file must be unchanged.
    assert_eq!(read_cursor_config(&h.root()), malformed);
}

#[test]
fn apply_returns_manual_required_when_top_level_is_not_object() {
    let h = Harness::new();
    write_cursor_config(&h.root(), br#"[1, 2, 3]"#);

    let result = cursor_adapter().apply(&h.request("notobject-1")).unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::ManualRequired);
}

#[test]
fn apply_returns_manual_required_when_mcp_servers_is_not_object() {
    let h = Harness::new();
    write_cursor_config(&h.root(), br#"{"mcpServers":"not-an-object"}"#);

    let result = cursor_adapter().apply(&h.request("badserver-1")).unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::ManualRequired);
}

// ---------------------------------------------------------------------------
// Apply: target does not exist - no surprising .cursor directory
// ---------------------------------------------------------------------------

#[test]
fn apply_returns_manual_required_when_target_absent_and_creates_no_cursor_dir() {
    let h = Harness::new();
    let result = cursor_adapter().apply(&h.request("absent-1")).unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::ManualRequired);

    // A `.cursor` directory must not be created as a side effect.
    assert!(!h.root_dir.path().join(".cursor").exists());
}

// ---------------------------------------------------------------------------
// Apply: symlink refusal - no mutation
// ---------------------------------------------------------------------------

#[test]
fn apply_refuses_symlinked_target_without_mutation() {
    let h = Harness::new();
    // Create a real file outside .cursor and a symlink at .cursor/mcp.json.
    let real = h.root_dir.path().join("real-mcp.json");
    std::fs::write(&real, br#"{"mcpServers":{}}"#).unwrap();
    std::fs::create_dir_all(h.root_dir.path().join(".cursor")).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&real, h.root_dir.path().join(".cursor/mcp.json")).unwrap();

    let result = cursor_adapter().apply(&h.request("symlink-1")).unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::ManualRequired);

    // The symlink target content must be unchanged.
    assert_eq!(std::fs::read(&real).unwrap(), br#"{"mcpServers":{}}"#);
}

#[test]
fn apply_refuses_symlinked_cursor_parent_directory() {
    let h = Harness::new();
    // Create a real directory and symlink .cursor -> real_dir.
    let real_dir = h.root_dir.path().join("real-cursor");
    std::fs::create_dir_all(&real_dir).unwrap();
    std::fs::write(real_dir.join("mcp.json"), br#"{"mcpServers":{}}"#).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&real_dir, h.root_dir.path().join(".cursor")).unwrap();

    let result = cursor_adapter()
        .apply(&h.request("symlink-parent-1"))
        .unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::ManualRequired);
}

// ---------------------------------------------------------------------------
// Verify
// ---------------------------------------------------------------------------

#[test]
fn verify_returns_verified_when_canonical_entry_matches_request_binary() {
    let h = Harness::new();
    let config = serde_json::json!({
        "mcpServers": {
            "nowdocs": {
                "command": h.binary().to_string_lossy().to_string(),
                "args": ["serve"]
            }
        }
    });
    write_cursor_config(&h.root(), serde_json::to_vec_pretty(&config).unwrap());

    let result = cursor_adapter().verify(&h.request("verify-ok-1")).unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::Verified);
    // Verification may emit a reload-required observation but must not claim
    // the Cursor UI has already reloaded.
    for obs in &result.observations {
        assert!(
            !obs.to_lowercase().contains("reloaded"),
            "verify must not claim the UI reloaded: {obs}"
        );
    }
}

#[test]
fn verify_emits_reload_required_observation() {
    let h = Harness::new();
    let config = serde_json::json!({
        "mcpServers": {
            "nowdocs": {
                "command": h.binary().to_string_lossy().to_string(),
                "args": ["serve"]
            }
        }
    });
    write_cursor_config(&h.root(), serde_json::to_vec_pretty(&config).unwrap());

    let result = cursor_adapter()
        .verify(&h.request("verify-reload-1"))
        .unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::Verified);
    // A stable reload-required observation is expected.
    assert!(
        result
            .observations
            .iter()
            .any(|o| o.to_lowercase().contains("reload")),
        "expected a reload-required observation, got: {:?}",
        result.observations
    );
}

#[test]
fn verify_returns_manual_required_on_binary_mismatch() {
    let h = Harness::new();
    let config = serde_json::json!({
        "mcpServers": {
            "nowdocs": {
                "command": "/different/path/nowdocs",
                "args": ["serve"]
            }
        }
    });
    write_cursor_config(&h.root(), serde_json::to_vec_pretty(&config).unwrap());

    let result = cursor_adapter()
        .verify(&h.request("verify-mismatch-1"))
        .unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::ManualRequired);
}

#[test]
fn verify_returns_manual_required_on_arg_mismatch() {
    let h = Harness::new();
    let config = serde_json::json!({
        "mcpServers": {
            "nowdocs": {
                "command": h.binary().to_string_lossy().to_string(),
                "args": ["run"]
            }
        }
    });
    write_cursor_config(&h.root(), serde_json::to_vec_pretty(&config).unwrap());

    let result = cursor_adapter().verify(&h.request("verify-arg-1")).unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::ManualRequired);
}

#[test]
fn verify_returns_manual_required_when_nowdocs_absent() {
    let h = Harness::new();
    write_cursor_config(&h.root(), br#"{"mcpServers":{"other":{"command":"x"}}}"#);

    let result = cursor_adapter()
        .verify(&h.request("verify-absent-1"))
        .unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::ManualRequired);
}

#[test]
fn verify_returns_manual_required_when_target_missing() {
    let h = Harness::new();
    let result = cursor_adapter()
        .verify(&h.request("verify-nofile-1"))
        .unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::ManualRequired);
}

#[test]
fn verify_returns_manual_required_for_malformed_json() {
    let h = Harness::new();
    write_cursor_config(&h.root(), b"not json at all");

    let result = cursor_adapter().verify(&h.request("verify-bad-1")).unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::ManualRequired);
}

#[test]
fn verify_observations_contain_no_paths_or_values() {
    let h = Harness::new();
    let config = serde_json::json!({
        "mcpServers": {
            "nowdocs": {
                "command": h.binary().to_string_lossy().to_string(),
                "args": ["serve"]
            }
        }
    });
    write_cursor_config(&h.root(), serde_json::to_vec_pretty(&config).unwrap());

    let result = cursor_adapter()
        .verify(&h.request("verify-redact-1"))
        .unwrap();
    for obs in &result.observations {
        assert!(
            !obs.contains(&h.binary().to_string_lossy().to_string()),
            "verify observation leaked binary path: {obs}"
        );
        assert!(
            !obs.contains(".cursor/mcp.json"),
            "verify observation leaked target path: {obs}"
        );
    }
}

// ---------------------------------------------------------------------------
// Rollback: restores original content
// ---------------------------------------------------------------------------

#[test]
fn rollback_restores_original_after_apply() {
    let h = Harness::new();
    let original = serde_json::json!({
        "mcpServers": {
            "filesystem": {"command": "/usr/bin/fs", "args": []}
        },
        "extra": true
    });
    write_cursor_config(&h.root(), serde_json::to_vec_pretty(&original).unwrap());

    let request = h.request("rollback-ok-1");
    cursor_adapter().apply(&request).unwrap();

    // The file now has a nowdocs entry.
    let after_apply: serde_json::Value =
        serde_json::from_slice(&read_cursor_config(&h.root())).unwrap();
    assert!(after_apply["mcpServers"]["nowdocs"].is_object());

    let result = cursor_adapter().rollback(&request).unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::RolledBack);

    // The original content is restored byte-for-byte.
    let restored: serde_json::Value =
        serde_json::from_slice(&read_cursor_config(&h.root())).unwrap();
    assert_eq!(restored, original);
    assert!(restored["mcpServers"]["nowdocs"].is_null());
}

#[test]
fn rollback_restores_absent_target_when_apply_created_it() {
    let h = Harness::new();
    // The adapter refuses absent targets, but C5's apply_with_backup can create
    // a file from nothing when the parent directory exists. Verify that
    // rollback of such a creation restores the pre-apply absence. This locks
    // the digest-guarded rollback contract for the cursor target shape.
    let root = h.root();
    std::fs::create_dir_all(root.path().join(".cursor")).unwrap();
    let target = safe_target(&root, ".cursor/mcp.json").unwrap();
    assert!(!root.path().join(".cursor/mcp.json").exists());

    let id = OperationId::new("rollback-absent-1").unwrap();
    apply_with_backup(&id, &target, br#"{"mcpServers":{"nowdocs":{}}}"#).unwrap();
    assert!(root.path().join(".cursor/mcp.json").is_file());

    rollback(&id).unwrap();
    // The file must be gone (restored to absence).
    assert!(!root.path().join(".cursor/mcp.json").exists());
}

// ---------------------------------------------------------------------------
// Rollback: refuses later user edit (digest mismatch)
// ---------------------------------------------------------------------------

#[test]
fn rollback_refuses_after_later_user_edit() {
    let h = Harness::new();
    let original = serde_json::json!({"mcpServers":{}});
    write_cursor_config(&h.root(), serde_json::to_vec_pretty(&original).unwrap());

    let request = h.request("rollback-refuse-1");
    cursor_adapter().apply(&request).unwrap();

    // User edits the file after apply.
    let root = h.root();
    let target = safe_target(&root, ".cursor/mcp.json").unwrap();
    atomic_replace(
        &target,
        br#"{"mcpServers":{"nowdocs":{"command":"edited","args":["serve"]}}}"#,
    )
    .unwrap();

    let result = cursor_adapter().rollback(&request).unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::ManualRequired);

    // The user-edited content is preserved (rollback did not overwrite it).
    let current = read_cursor_config(&h.root());
    assert!(String::from_utf8_lossy(&current).contains("edited"));
}

// ---------------------------------------------------------------------------
// Rollback: unknown operation id is manual
// ---------------------------------------------------------------------------

#[test]
fn rollback_returns_manual_required_for_unknown_operation() {
    let h = Harness::new();
    let request = h.request("rollback-unknown-1");
    let result = cursor_adapter().rollback(&request).unwrap();
    assert_eq!(result.outcome, ClientExecutionOutcome::ManualRequired);
}
