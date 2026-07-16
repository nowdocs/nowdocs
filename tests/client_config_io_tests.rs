use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use nowdocs::agent_contract::CapabilitySupport;
use nowdocs::clients::{
    all_adapters, approved_root, atomic_replace, compute_digest, read_target, safe_target, ClientId,
};

// dirs::cache_dir() reads XDG_CACHE_HOME / HOME at call time, so tests that
// mutate these env vars must not run concurrently. This lock serializes them.
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

// ---------------------------------------------------------------------------
// Task 1: safe configuration I/O primitives
// ---------------------------------------------------------------------------

#[test]
fn config_io_refuses_absolute_target() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = tmp_cache_guard(&tmp);
    let root = approved_root(tmp.path()).unwrap();
    let err = safe_target(&root, "/etc/passwd").unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("absolute") || msg.contains("unsafe"),
        "expected absolute-path refusal, got: {}",
        msg
    );
}

#[test]
fn config_io_refuses_traversal() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = tmp_cache_guard(&tmp);
    let root = approved_root(tmp.path()).unwrap();
    let err = safe_target(&root, "../secrets").unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("traversal") || msg.contains("..") || msg.contains("unsafe"),
        "expected traversal refusal, got: {}",
        msg
    );
}

#[test]
#[cfg(unix)]
fn config_io_refuses_symlinked_root() {
    let tmp = tempfile::tempdir().unwrap();
    let real = tmp.path().join("real");
    std::fs::create_dir(&real).unwrap();
    let link = tmp.path().join("link");
    std::os::unix::fs::symlink(&real, &link).unwrap();
    let err = approved_root(&link).unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("symlink") || msg.contains("unsafe"),
        "expected symlinked-root refusal, got: {}",
        msg
    );
}

#[test]
#[cfg(unix)]
fn config_io_refuses_symlinked_target() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = tmp_cache_guard(&tmp);
    let root = approved_root(tmp.path()).unwrap();

    let real = tmp.path().join("real.txt");
    std::fs::write(&real, b"secret").unwrap();
    let link = tmp.path().join("link.txt");
    std::os::unix::fs::symlink(&real, &link).unwrap();

    let target = safe_target(&root, "link.txt").unwrap();
    let err = read_target(&target).unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("symlink") || msg.contains("unsafe") || msg.contains("ELOOP"),
        "expected symlinked-target refusal, got: {}",
        msg
    );
}

#[test]
#[cfg(unix)]
fn config_io_refuses_symlinked_parent_component_for_reads_and_writes() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = tmp_cache_guard(&tmp);
    let root = approved_root(tmp.path()).unwrap();

    let external = tempfile::tempdir().unwrap();
    let external_target = external.path().join("config.json");
    std::fs::write(&external_target, b"outside").unwrap();

    let linked_parent = tmp.path().join("linked");
    std::os::unix::fs::symlink(external.path(), &linked_parent).unwrap();

    let target = safe_target(&root, "linked/config.json").unwrap();
    assert!(
        read_target(&target).is_err(),
        "a symlinked parent directory must not be traversed for reads"
    );
    assert!(
        atomic_replace(&target, b"must not escape").is_err(),
        "a symlinked parent directory must not be traversed for writes"
    );
    assert_eq!(std::fs::read(&external_target).unwrap(), b"outside");
}

#[test]
fn config_io_refuses_nonregular_file() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = tmp_cache_guard(&tmp);
    let root = approved_root(tmp.path()).unwrap();

    // A directory is not a regular file.
    std::fs::create_dir(tmp.path().join("dir")).unwrap();
    let target = safe_target(&root, "dir").unwrap();
    let err = read_target(&target).unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("regular file") || msg.contains("nonregular") || msg.contains("unsafe"),
        "expected nonregular-file refusal, got: {}",
        msg
    );
}

// --- Non-Unix: atomic replace/read fail closed; pre-existing target is
//     byte-for-byte unchanged ---

#[test]
#[cfg(not(unix))]
fn config_io_read_and_atomic_replace_fail_closed_on_unsupported_platform() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = tmp_cache_guard(&tmp);
    let root = approved_root(tmp.path()).unwrap();

    // Pre-create the target with known content via plain std::fs (bypassing
    // the no-follow I/O path so the fixture exists on the unsupported platform).
    let original = b"pre-existing content";
    std::fs::write(tmp.path().join("config.json"), original).unwrap();

    let target = safe_target(&root, "config.json").unwrap();

    // read_target must fail closed on the unsupported no-follow path.
    let err = read_target(&target).unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("unsupported platform for no-follow I/O"),
        "read_target must fail closed with the stable platform prefix, got: {msg}"
    );

    // atomic_replace must also fail closed.
    let err = atomic_replace(&target, b"replacement").unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("unsupported platform for no-follow I/O"),
        "atomic_replace must fail closed with the stable platform prefix, got: {msg}"
    );

    // Zero-mutation: the pre-existing target is byte-for-byte unchanged.
    let after = std::fs::read(tmp.path().join("config.json")).unwrap();
    assert_eq!(
        after, original,
        "pre-existing target must remain byte-for-byte unchanged after fail-closed rejection"
    );
}

#[test]
#[cfg(unix)]
fn config_io_atomic_replace_verifies_digest_and_reopens() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = tmp_cache_guard(&tmp);
    let root = approved_root(tmp.path()).unwrap();

    let content = b"hello, nowdocs";
    let target = safe_target(&root, "config.json").unwrap();
    let digest = atomic_replace(&target, content).unwrap();
    assert_eq!(digest, compute_digest(content));

    let read_back = read_target(&target).unwrap();
    assert_eq!(read_back, content);
}

#[test]
#[cfg(unix)]
fn config_io_atomic_replace_refuses_changed_target() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = tmp_cache_guard(&tmp);
    let root = approved_root(tmp.path()).unwrap();

    let content = b"first";
    let target = safe_target(&root, "config.json").unwrap();
    atomic_replace(&target, content).unwrap();

    // Swap the target for a symlink to an external file between replace calls.
    let external = tmp.path().join("external.txt");
    std::fs::write(&external, b"external").unwrap();
    let target_path = tmp.path().join("config.json");
    std::fs::remove_file(&target_path).unwrap();
    std::os::unix::fs::symlink(&external, &target_path).unwrap();

    let err = atomic_replace(&target, b"second").unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("symlink") || msg.contains("unsafe") || msg.contains("ELOOP"),
        "expected refusal after target swapped to symlink, got: {}",
        msg
    );
}

#[test]
#[cfg(unix)]
fn config_io_preserves_restrictive_permissions() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = tmp_cache_guard(&tmp);
    let root = approved_root(tmp.path()).unwrap();

    let content = b"secret";
    let target = safe_target(&root, "config.json").unwrap();
    atomic_replace(&target, content).unwrap();

    use std::os::unix::fs::PermissionsExt;
    let mode = std::fs::metadata(tmp.path().join("config.json"))
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600, "new file must be owner-only");
}

// ---------------------------------------------------------------------------
// Task C7R1: safe client-root validation (pure, no chmod)
// ---------------------------------------------------------------------------

#[test]
fn approved_root_accepts_0755_without_chmod() {
    let tmp = tempfile::tempdir().unwrap();
    let root_dir = tmp.path().join("root_0755");
    std::fs::create_dir(&root_dir).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&root_dir, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    let root = approved_root(&root_dir).unwrap();
    assert_eq!(root.path(), root_dir);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let actual = std::fs::metadata(&root_dir).unwrap().permissions().mode() & 0o777;
        assert_eq!(actual, 0o755, "approved_root must not change 0755 mode");
    }
}

#[test]
fn approved_root_accepts_0700_without_chmod() {
    let tmp = tempfile::tempdir().unwrap();
    let root_dir = tmp.path().join("root_0700");
    std::fs::create_dir(&root_dir).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&root_dir, std::fs::Permissions::from_mode(0o700)).unwrap();
    }

    let root = approved_root(&root_dir).unwrap();
    assert_eq!(root.path(), root_dir);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let actual = std::fs::metadata(&root_dir).unwrap().permissions().mode() & 0o777;
        assert_eq!(actual, 0o700, "approved_root must not change 0700 mode");
    }
}

#[test]
#[cfg(unix)]
fn approved_root_refuses_0775_and_preserves_mode() {
    let tmp = tempfile::tempdir().unwrap();
    let root_dir = tmp.path().join("root_0775");
    std::fs::create_dir(&root_dir).unwrap();

    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&root_dir, std::fs::Permissions::from_mode(0o775)).unwrap();
    let err = approved_root(&root_dir).unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("writable") || msg.contains("group") || msg.contains("other"),
        "expected group/world-writable refusal, got: {}",
        msg
    );
    let actual = std::fs::metadata(&root_dir).unwrap().permissions().mode() & 0o777;
    assert_eq!(
        actual, 0o775,
        "mode must not be changed by failed validation"
    );
}

#[test]
#[cfg(unix)]
fn approved_root_refuses_0777_and_preserves_mode() {
    let tmp = tempfile::tempdir().unwrap();
    let root_dir = tmp.path().join("root_0777");
    std::fs::create_dir(&root_dir).unwrap();

    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&root_dir, std::fs::Permissions::from_mode(0o777)).unwrap();
    let err = approved_root(&root_dir).unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("writable") || msg.contains("group") || msg.contains("other"),
        "expected group/world-writable refusal, got: {}",
        msg
    );
    let actual = std::fs::metadata(&root_dir).unwrap().permissions().mode() & 0o777;
    assert_eq!(
        actual, 0o777,
        "mode must not be changed by failed validation"
    );
}

#[test]
#[cfg(unix)]
fn approved_root_refuses_symlink_root() {
    let tmp = tempfile::tempdir().unwrap();
    let real = tmp.path().join("real");
    std::fs::create_dir(&real).unwrap();
    let link = tmp.path().join("link");
    std::os::unix::fs::symlink(&real, &link).unwrap();
    let err = approved_root(&link).unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("symlink") || msg.contains("unsafe"),
        "expected symlinked-root refusal, got: {}",
        msg
    );
}

// ---------------------------------------------------------------------------
// Task 2: typed adapter model and staged execution capabilities
// ---------------------------------------------------------------------------

#[test]
fn adapter_capability_matrix_tracks_staged_execution_capabilities() {
    let adapters = all_adapters();
    assert_eq!(adapters.len(), 4);

    let by_id: std::collections::HashMap<_, _> = adapters.iter().map(|a| (a.id(), a)).collect();
    assert!(by_id.contains_key(&ClientId::ClaudeCode));
    assert!(by_id.contains_key(&ClientId::ClaudeDesktop));
    assert!(by_id.contains_key(&ClientId::Cursor));
    assert!(by_id.contains_key(&ClientId::Generic));

    let cc = by_id[&ClientId::ClaudeCode].capabilities();
    assert_eq!(cc.detect, CapabilitySupport::Supported);
    assert_eq!(cc.generate, CapabilitySupport::Supported);
    assert!(matches!(
        cc.apply,
        CapabilitySupport::Unsupported | CapabilitySupport::Conditional
    ));
    assert!(matches!(
        cc.verify,
        CapabilitySupport::Unsupported | CapabilitySupport::Conditional
    ));

    let cd = by_id[&ClientId::ClaudeDesktop].capabilities();
    assert_eq!(cd.detect, CapabilitySupport::Supported);
    assert_eq!(cd.generate, CapabilitySupport::Supported);
    assert_eq!(cd.apply, CapabilitySupport::Unsupported);
    assert_eq!(cd.verify, CapabilitySupport::Unsupported);

    let cu = by_id[&ClientId::Cursor].capabilities();
    assert_eq!(cu.detect, CapabilitySupport::Supported);
    assert_eq!(cu.generate, CapabilitySupport::Supported);
    // Cursor's C6 implementation is present, so its conditional execution
    // support is an exact current-state assertion.
    assert_eq!(cu.apply, CapabilitySupport::Conditional);
    assert_eq!(cu.verify, CapabilitySupport::Conditional);

    let gen = by_id[&ClientId::Generic].capabilities();
    assert_eq!(gen.detect, CapabilitySupport::Unsupported);
    assert_eq!(gen.generate, CapabilitySupport::Supported);
    assert_eq!(gen.apply, CapabilitySupport::Unsupported);
    assert_eq!(gen.verify, CapabilitySupport::Unsupported);
}

#[test]
fn generic_generate_is_deterministic() {
    let adapters = all_adapters();
    let generic = adapters
        .iter()
        .find(|a| a.id() == ClientId::Generic)
        .unwrap();
    let binary = Path::new("/tmp/nowdocs");
    let a = generic.generate(binary).unwrap();
    let b = generic.generate(binary).unwrap();
    assert_eq!(a, b);
    assert_eq!(a.stdio_command, vec!["/tmp/nowdocs", "serve"]);
}

#[test]
fn generic_fragment_contains_no_secrets() {
    let adapters = all_adapters();
    let generic = adapters
        .iter()
        .find(|a| a.id() == ClientId::Generic)
        .unwrap();
    let binary = Path::new("/opt/nowdocs");
    let generated = generic.generate(binary).unwrap();
    // The redacted fragment must not contain a real filesystem path (a secret-ish
    // leakage vector in logs). It should use a placeholder.
    assert!(
        !generated.redacted_fragment.contains("/opt/nowdocs"),
        "fragment leaked binary path: {}",
        generated.redacted_fragment
    );
    assert!(generated.redacted_fragment.contains("<binary>"));
    assert!(generated.redacted_fragment.contains("mcpServers"));
}

#[test]
fn claude_code_generate_uses_official_cli_form() {
    let adapters = all_adapters();
    let cc = adapters
        .iter()
        .find(|a| a.id() == ClientId::ClaudeCode)
        .unwrap();
    let binary = Path::new("/usr/local/bin/nowdocs");
    let generated = cc.generate(binary).unwrap();
    assert_eq!(
        generated.stdio_command,
        vec![
            "claude",
            "mcp",
            "add",
            "--transport",
            "stdio",
            "--scope",
            "user",
            "nowdocs",
            "--",
            "/usr/local/bin/nowdocs",
            "serve",
        ]
    );
}

#[test]
fn claude_desktop_generate_returns_mcpb_guidance() {
    let adapters = all_adapters();
    let cd = adapters
        .iter()
        .find(|a| a.id() == ClientId::ClaudeDesktop)
        .unwrap();
    let generated = cd.generate(Path::new("/usr/local/bin/nowdocs")).unwrap();
    assert!(generated.manual_steps.iter().any(|s| s.contains("mcpb")));
    assert!(generated.redacted_fragment.contains("claude-desktop"));
}

#[test]
fn cursor_generate_returns_global_json_fragment() {
    let adapters = all_adapters();
    let cu = adapters
        .iter()
        .find(|a| a.id() == ClientId::Cursor)
        .unwrap();
    let generated = cu.generate(Path::new("/usr/local/bin/nowdocs")).unwrap();
    assert!(generated.redacted_fragment.contains("mcpServers"));
    assert!(generated
        .manual_steps
        .iter()
        .any(|s| s.contains(".cursor/mcp.json")));
}

#[test]
fn claude_code_detects_marker_in_approved_root() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = tmp_cache_guard(&tmp);
    let root = approved_root(tmp.path()).unwrap();
    std::fs::write(tmp.path().join(".claude.json"), b"{}").unwrap();

    let adapters = all_adapters();
    let cc = adapters
        .iter()
        .find(|a| a.id() == ClientId::ClaudeCode)
        .unwrap();
    let detection = cc.detect(&root).unwrap();
    assert!(detection.detected);
    assert_eq!(detection.target_path, Some(".claude.json".to_string()));
}

#[test]
fn cursor_detects_marker_in_approved_root() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = tmp_cache_guard(&tmp);
    let root = approved_root(tmp.path()).unwrap();
    std::fs::create_dir(tmp.path().join(".cursor")).unwrap();
    std::fs::write(tmp.path().join(".cursor/mcp.json"), b"{}").unwrap();

    let adapters = all_adapters();
    let cu = adapters
        .iter()
        .find(|a| a.id() == ClientId::Cursor)
        .unwrap();
    let detection = cu.detect(&root).unwrap();
    assert!(detection.detected);
    assert_eq!(detection.target_path, Some(".cursor/mcp.json".to_string()));
}

#[test]
fn operation_record_timestamp_uses_unix_epoch_in_tests() {
    // Sanity check that SystemTime / u64 conversions are available for tests.
    let _t = UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000);
    assert!(_t > SystemTime::UNIX_EPOCH);
}
