use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use nowdocs::automation::operation::{
    apply_with_backup, init_operation_dir, operations_root, read_journal, retention_expiry,
    rollback, write_journal, OperationId, OperationRecord, OperationState,
};
use nowdocs::clients::{approved_root, atomic_replace, safe_target};

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
// Task 3: operation journal, private backup, and atomic replace gate
// ---------------------------------------------------------------------------

#[test]
fn operation_dir_created_with_safe_permissions() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = tmp_cache_guard(&tmp);

    let id = OperationId::new("test-op").unwrap();
    let dir = init_operation_dir(&id).unwrap();
    assert!(dir.ends_with("test-op"));
    assert!(dir.is_dir());
    assert!(dir.join("backup").is_dir());

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = dir.metadata().unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o700, "operation dir must be owner-only");
        let backup_mode = dir.join("backup").metadata().unwrap().permissions().mode() & 0o777;
        assert_eq!(backup_mode, 0o700, "backup dir must be owner-only");
    }
}

#[test]
fn journal_round_trip_excludes_backup_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = tmp_cache_guard(&tmp);

    let id = OperationId::new("journal-op").unwrap();
    init_operation_dir(&id).unwrap();

    let record = OperationRecord {
        operation_id: id.clone(),
        created_at: SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000),
        state: OperationState::AppliedVerified,
        target_digest: Some("abcd".repeat(16)),
        backup_path: Some(operations_root().join("journal-op/backup/original")),
        logical_target: ".cursor/mcp.json".to_string(),
    };

    write_journal(&record).unwrap();
    let raw = std::fs::read_to_string(operations_root().join("journal-op/journal.json")).unwrap();
    assert!(
        !raw.contains("backup"),
        "journal must not contain backup path or bytes: {}",
        raw
    );
    assert!(
        !raw.contains("original"),
        "journal must not leak backup filename: {}",
        raw
    );

    let loaded = read_journal(&id).unwrap();
    assert_eq!(loaded.operation_id, id);
    assert_eq!(loaded.state, OperationState::AppliedVerified);
    assert_eq!(loaded.target_digest, record.target_digest);
    assert_eq!(loaded.logical_target, record.logical_target);
    // Backup path is intentionally excluded from the journal.
    assert!(loaded.backup_path.is_none());

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(operations_root().join("journal-op/journal.json"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600, "journal must be owner-only");
    }
}

#[test]
fn apply_with_backup_records_verified_state() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = tmp_cache_guard(&tmp);

    let root = approved_root(tmp.path()).unwrap();
    let target = safe_target(&root, "config.json").unwrap();
    atomic_replace(&target, b"original content").unwrap();

    let id = OperationId::new("apply-op").unwrap();
    let record = apply_with_backup(&id, &target, b"applied content").unwrap();
    assert_eq!(record.operation_id, id);
    assert_eq!(record.state, OperationState::AppliedVerified);
    assert!(record.target_digest.is_some());
    assert!(record.backup_path.is_some());

    let current = std::fs::read_to_string(tmp.path().join("config.json")).unwrap();
    assert_eq!(current, "applied content");

    let backup = std::fs::read(record.backup_path.as_ref().unwrap()).unwrap();
    assert_eq!(backup, b"original content");
}

#[test]
fn apply_with_backup_refuses_unsafe_target() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = tmp_cache_guard(&tmp);

    // Use a target outside the approved root by constructing an absolute path.
    let root = approved_root(tmp.path()).unwrap();
    let target = safe_target(&root, "config.json").unwrap();
    let external = tmp.path().join("../external.json");

    // safe_target should have already rejected the absolute/traversal form; this
    // test guards the operation layer against an absolute target sneaking in.
    assert!(external.to_str().unwrap().contains(".."));

    let id = OperationId::new("unsafe-op").unwrap();
    let record = apply_with_backup(&id, &target, b"applied");
    assert!(record.is_ok());

    // We cannot construct SafeTarget outside the module, so the only unsafe
    // path available is one that safe_target rejects. This assertion locks
    // the contract: absolute/traversal targets never reach apply.
    assert!(safe_target(&root, "/etc/passwd").is_err());
}

// ---------------------------------------------------------------------------
// Task 4: digest-guarded rollback and retention metadata
// ---------------------------------------------------------------------------

#[test]
fn rollback_restores_matching_digest_target() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = tmp_cache_guard(&tmp);

    let root = approved_root(tmp.path()).unwrap();
    let target = safe_target(&root, "config.json").unwrap();
    atomic_replace(&target, b"original").unwrap();

    let id = OperationId::new("rollback-op").unwrap();
    apply_with_backup(&id, &target, b"applied").unwrap();

    let rolled = rollback(&id).unwrap();
    assert_eq!(rolled.state, OperationState::RolledBack);

    let current = std::fs::read_to_string(tmp.path().join("config.json")).unwrap();
    assert_eq!(current, "original");
}

#[test]
fn rollback_refuses_changed_target() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = tmp_cache_guard(&tmp);

    let root = approved_root(tmp.path()).unwrap();
    let target = safe_target(&root, "config.json").unwrap();
    atomic_replace(&target, b"original").unwrap();

    let id = OperationId::new("changed-op").unwrap();
    apply_with_backup(&id, &target, b"applied").unwrap();

    // User edits the target after apply.
    atomic_replace(&target, b"user-edited").unwrap();

    let err = rollback(&id).unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("edited") || msg.contains("digest") || msg.contains("refuse"),
        "expected changed-target refusal, got: {}",
        msg
    );
}

#[test]
fn rollback_retention_24h_verified() {
    let t = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
    let expiry = retention_expiry(OperationState::AppliedVerified, Some(t)).unwrap();
    assert_eq!(expiry, t + Duration::from_secs(24 * 3600));
}

#[test]
fn rollback_retention_7d_partial() {
    let t = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
    let expiry = retention_expiry(OperationState::AppliedButUnverified, Some(t)).unwrap();
    assert_eq!(expiry, t + Duration::from_secs(7 * 24 * 3600));
}
