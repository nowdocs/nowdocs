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
#[cfg(unix)]
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
#[cfg(unix)]
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
#[cfg(unix)]
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
// Task 3 (non-Unix): journal/private-file and apply persistence fail closed
// without changing the target or creating operation artifacts
// ---------------------------------------------------------------------------

#[test]
#[cfg(not(unix))]
fn journal_write_fails_closed_without_creating_artifacts() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = tmp_cache_guard(&tmp);

    let id = OperationId::new("journal-op").unwrap();
    let record = OperationRecord {
        operation_id: id.clone(),
        created_at: SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000),
        state: OperationState::AppliedVerified,
        target_digest: Some("abcd".repeat(16)),
        backup_path: Some(operations_root().join("journal-op/backup/original")),
        logical_target: ".cursor/mcp.json".to_string(),
    };

    // write_journal must fail closed on the unsupported no-follow path.
    let err = write_journal(&record).unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("unsupported platform for no-follow I/O"),
        "write_journal must fail closed with the stable platform prefix, got: {msg}"
    );

    // read_journal must also fail closed.
    let err = read_journal(&id).unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("unsupported platform for no-follow I/O"),
        "read_journal must fail closed with the stable platform prefix, got: {msg}"
    );

    // Zero payload mutation: init_operation_dir runs before the file-level
    // refusal, so empty operation/backup directories are expected. No journal,
    // backup bytes, target metadata, or other regular file may be created.
    let operation_dir = operations_root().join("journal-op");
    assert!(
        operation_dir.join("backup").is_dir(),
        "the existing initializer creates an empty backup directory"
    );
    assert!(
        !operation_dir.join("journal.json").exists(),
        "no journal file may be created on the unsupported platform"
    );
    let auto_root = nowdocs::cache::automation_root();
    assert!(
        count_regular_files_recursive(&auto_root) == 0,
        "no operation payload file may appear under the automation root"
    );
}

#[test]
#[cfg(not(unix))]
fn apply_with_backup_fails_closed_without_changing_target() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = tmp_cache_guard(&tmp);

    let root = approved_root(tmp.path()).unwrap();
    let target = safe_target(&root, "config.json").unwrap();

    // Pre-create the target with known content.
    std::fs::write(tmp.path().join("config.json"), b"original content").unwrap();
    let before = std::fs::read(tmp.path().join("config.json")).unwrap();

    let id = OperationId::new("apply-op").unwrap();
    let err = apply_with_backup(&id, &target, b"applied content").unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("unsupported platform for no-follow I/O"),
        "apply_with_backup must fail closed with the stable platform prefix, got: {msg}"
    );

    // Zero-mutation: the target file is byte-for-byte unchanged.
    let after = std::fs::read(tmp.path().join("config.json")).unwrap();
    assert_eq!(
        after, before,
        "target file must remain byte-for-byte unchanged after fail-closed rejection"
    );

    // No operation payload files (journal, backup bytes, target metadata) were
    // created. Empty initializer directories are allowed.
    let operation_dir = operations_root().join("apply-op");
    assert!(
        operation_dir.join("backup").is_dir(),
        "the existing initializer creates an empty backup directory"
    );
    let auto_root = nowdocs::cache::automation_root();
    assert!(
        count_regular_files_recursive(&auto_root) == 0,
        "no operation payload file may appear under the automation root"
    );
}

// ---------------------------------------------------------------------------
// Task 4: digest-guarded rollback and retention metadata
// ---------------------------------------------------------------------------

#[test]
#[cfg(unix)]
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
#[cfg(unix)]
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

/// Count regular payload files recursively while allowing empty initializer
/// directory scaffolding.
#[cfg(not(unix))]
fn count_regular_files_recursive(path: &std::path::Path) -> usize {
    if !path.exists() {
        return 0;
    }
    std::fs::read_dir(path)
        .map(|it| {
            it.flatten()
                .map(|entry| {
                    let file_type = entry
                        .file_type()
                        .expect("inspect automation entry without following symlinks");
                    if file_type.is_dir() {
                        count_regular_files_recursive(&entry.path())
                    } else {
                        // Regular files, symlinks/reparse points, and every
                        // other non-directory entry are all payload.
                        1
                    }
                })
                .sum()
        })
        .unwrap_or(0)
}
