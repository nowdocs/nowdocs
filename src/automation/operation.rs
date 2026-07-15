//! Operation journaling and rollback boundary.
//!
//! Records the lifecycle of an automation operation under the private C3
//! `automation/operations/` subtree. Journals contain only operation IDs,
//! states, digests, and logical target paths; backup bytes and secrets live
//! outside JSON in a private `backup/` directory with restrictive permissions.
//! C5 is generation-only at the adapter layer: `apply_with_backup` performs the
//! physical write, but it is only called by later orchestration that has
//! already obtained approval.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::clients::{atomic_replace, compute_digest, read_target, safe_target, SafeTarget};

/// Validated operation identifier: `[a-z0-9-]{1,64}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationId(String);

impl OperationId {
    /// Validate and construct an operation id.
    pub fn new(id: &str) -> Result<Self> {
        if id.is_empty() || id.len() > 64 {
            anyhow::bail!("invalid operation_id length (must be 1..=64): {id:?}");
        }
        if !id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            anyhow::bail!("invalid operation_id (must match [a-z0-9-]{{1,64}}): {id:?}");
        }
        Ok(Self(id.to_string()))
    }
}

impl std::fmt::Display for OperationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Lifecycle state of an operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationState {
    Planned,
    AppliedVerified,
    AppliedButUnverified,
    RolledBack,
    Failed,
}

/// Journal record for one operation. Backup paths are deliberately excluded
/// from serialization: they are recomputed from the operation id at runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationRecord {
    pub operation_id: OperationId,
    pub created_at: SystemTime,
    pub state: OperationState,
    pub target_digest: Option<String>,
    #[serde(skip)]
    pub backup_path: Option<PathBuf>,
    pub logical_target: String,
}

/// Path to the operations subtree: `<cache>/nowdocs/automation/operations`.
pub fn operations_root() -> PathBuf {
    crate::cache::automation_root().join("operations")
}

/// Create the private directory tree for an operation: `<operations>/<id>/` and
/// `<operations>/<id>/backup/`, both `0700` on Unix.
pub fn init_operation_dir(id: &OperationId) -> Result<PathBuf> {
    crate::automation::plan::ensure_automation_root()?;
    let dir = operations_root().join(&id.0);
    ensure_private_dir(&dir)?;
    ensure_private_dir(&dir.join("backup"))?;
    Ok(dir)
}

/// Write a journal record as compact JSON. Backup paths are not serialized.
pub fn write_journal(record: &OperationRecord) -> Result<()> {
    init_operation_dir(&record.operation_id)?;
    let path = journal_path(&record.operation_id);
    let bytes = serde_json::to_vec_pretty(record).context("serialize journal")?;
    write_owner_only_file(&path, &bytes)?;
    Ok(())
}

/// Read a journal record via no-follow open.
pub fn read_journal(id: &OperationId) -> Result<OperationRecord> {
    let path = journal_path(id);
    let bytes = read_nofollow(&path)?;
    let record: OperationRecord = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse journal {}", path.display()))?;
    Ok(record)
}

/// Store a private backup of the original target bytes. Returns the backup path.
pub fn store_backup(id: &OperationId, _target: &SafeTarget, original: &[u8]) -> Result<PathBuf> {
    let dir = init_operation_dir(id)?;
    let path = dir.join("backup").join("backup.bin");
    write_owner_only_file(&path, original)?;
    Ok(path)
}

/// Apply new content to a safe target with backup and journal. Returns the
/// updated operation record in `AppliedVerified` state.
pub fn apply_with_backup(
    id: &OperationId,
    target: &SafeTarget,
    content: &[u8],
) -> Result<OperationRecord> {
    let _ = init_operation_dir(id)?;

    // Snapshot current target, if any, and store a private backup.
    let original = read_target(target).ok();
    let backup_path = original
        .as_ref()
        .map(|bytes| store_backup(id, target, bytes))
        .transpose()?;

    // Remember the approved root + relative path so rollback can locate the
    // target without requiring the caller to resupply it.
    let target_meta = TargetMetadata {
        approved_root: target.approved().path().to_path_buf(),
        relative: target.logical(),
    };
    write_owner_only_file(
        &target_metadata_path(id),
        &serde_json::to_vec(&target_meta).context("serialize target metadata")?,
    )?;

    let digest = atomic_replace(target, content)?;

    let record = OperationRecord {
        operation_id: id.clone(),
        created_at: SystemTime::now(),
        state: OperationState::AppliedVerified,
        target_digest: Some(digest),
        backup_path,
        logical_target: target.logical(),
    };
    write_journal(&record)?;
    Ok(record)
}

/// Roll back an operation by restoring its private backup. Refuses if the
/// current target digest does not match the committed digest (i.e., the target
/// was edited later).
pub fn rollback(id: &OperationId) -> Result<OperationRecord> {
    let mut record = read_journal(id)?;
    let meta = load_target_metadata(id)?;
    let root = crate::clients::approved_root(&meta.approved_root)?;
    let target = safe_target(&root, &meta.relative)?;

    let current_digest = read_target(&target).map(|b| compute_digest(&b)).ok();
    match (&record.target_digest, current_digest) {
        (Some(expected), Some(actual)) if expected == &actual => {
            let backup_path = backup_file_path(id);
            if backup_path.exists() {
                let backup = read_nofollow(&backup_path)?;
                atomic_replace(&target, &backup)?;
            } else {
                // Target was absent when created; restore absence by removing it.
                std::fs::remove_file(target.path())
                    .with_context(|| format!("remove target {}", target.path().display()))?;
            }
            record.state = OperationState::RolledBack;
            write_journal(&record)?;
            Ok(record)
        }
        (None, None) => {
            // Target was absent and remains absent; nothing to restore.
            record.state = OperationState::RolledBack;
            write_journal(&record)?;
            Ok(record)
        }
        _ => anyhow::bail!(
            "rollback refused: target {} was edited later (digest mismatch)",
            target.path().display()
        ),
    }
}

/// Compute rollback retention expiry: 24 hours for verified applies, 7 days for
/// partial applies, none otherwise.
pub fn retention_expiry(
    state: OperationState,
    verified_at: Option<SystemTime>,
) -> Option<SystemTime> {
    match state {
        OperationState::AppliedVerified => verified_at.map(|t| t + Duration::from_secs(24 * 3600)),
        OperationState::AppliedButUnverified => {
            verified_at.map(|t| t + Duration::from_secs(7 * 24 * 3600))
        }
        _ => None,
    }
}

// ---- Private helpers ----

#[derive(Debug, Serialize, Deserialize)]
struct TargetMetadata {
    approved_root: PathBuf,
    relative: String,
}

fn journal_path(id: &OperationId) -> PathBuf {
    operations_root().join(&id.0).join("journal.json")
}

fn backup_file_path(id: &OperationId) -> PathBuf {
    operations_root()
        .join(&id.0)
        .join("backup")
        .join("backup.bin")
}

fn target_metadata_path(id: &OperationId) -> PathBuf {
    operations_root().join(&id.0).join("target.json")
}

fn load_target_metadata(id: &OperationId) -> Result<TargetMetadata> {
    let path = target_metadata_path(id);
    let bytes = read_nofollow(&path)?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("parse target metadata {}", path.display()))
}

/// Create a directory and (on Unix) ensure it is owner-only (`0700`). Existing
/// directories are verified to be real directories but are not re-chmoded.
fn ensure_private_dir(dir: &Path) -> Result<()> {
    if let Ok(meta) = std::fs::symlink_metadata(dir) {
        if !meta.is_dir() || meta.file_type().is_symlink() {
            anyhow::bail!(
                "operation path {} exists but is not a real directory (symlink refused)",
                dir.display()
            );
        }
        return Ok(()); // Existing real directory: leave permissions alone.
    }

    // Path does not exist: create parent-first, single-component, so symlink
    // races are limited to one level.
    let mut chain = Vec::new();
    let mut current = dir.to_path_buf();
    loop {
        match std::fs::symlink_metadata(&current) {
            Ok(meta) => {
                if !meta.is_dir() || meta.file_type().is_symlink() {
                    anyhow::bail!(
                        "operation ancestor {} is not a real directory (symlink refused)",
                        current.display()
                    );
                }
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                chain.push(current.clone());
                if !current.pop() {
                    anyhow::bail!("cannot find existing ancestor for {}", dir.display());
                }
            }
            Err(e) => {
                anyhow::bail!("cannot stat operation ancestor {}: {e}", current.display());
            }
        }
    }

    for component in chain.into_iter().rev() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            std::fs::DirBuilder::new()
                .mode(0o700)
                .create(&component)
                .with_context(|| format!("create operation directory {}", component.display()))?;
        }
        #[cfg(not(unix))]
        {
            std::fs::create_dir(&component)
                .with_context(|| format!("create operation directory {}", component.display()))?;
        }
    }
    Ok(())
}

/// Write a file with no-follow open and `0600` permissions on Unix. On Windows
/// and other non-Unix platforms, fail closed because equivalent no-follow
/// safety cannot be proven with std+fs2.
fn write_owner_only_file(path: &Path, bytes: &[u8]) -> Result<()> {
    init_parent_dir(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        use std::os::unix::io::AsRawFd;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path)
            .with_context(|| format!("open (O_NOFOLLOW) {}", path.display()))?;
        use std::io::Write;
        file.write_all(bytes)
            .with_context(|| format!("write {}", path.display()))?;
        file.flush()
            .with_context(|| format!("flush {}", path.display()))?;
        let rc = unsafe { libc::fchmod(file.as_raw_fd(), 0o600) };
        if rc != 0 {
            return Err(std::io::Error::last_os_error())
                .with_context(|| format!("fchmod 0600 {}", path.display()));
        }
    }
    #[cfg(not(unix))]
    {
        anyhow::bail!(
            "unsupported platform for no-follow I/O at {}",
            path.display()
        );
    }
    Ok(())
}

fn init_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_private_dir(parent)?;
    }
    Ok(())
}

#[cfg(unix)]
fn read_nofollow(path: &Path) -> Result<Vec<u8>> {
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .with_context(|| format!("open (O_NOFOLLOW) {}", path.display()))?;
    let meta = file
        .metadata()
        .with_context(|| format!("fstat {}", path.display()))?;
    if !meta.is_file() {
        anyhow::bail!("{} is not a regular file", path.display());
    }
    use std::io::Read;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .with_context(|| format!("read {}", path.display()))?;
    Ok(buf)
}

#[cfg(not(unix))]
fn read_nofollow(path: &Path) -> Result<Vec<u8>> {
    anyhow::bail!(
        "unsupported platform for no-follow I/O at {}",
        path.display()
    );
}
