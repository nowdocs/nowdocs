//! Global automation operation lock.
//!
//! One setup/ensure/rollback operation may hold this lock at a time
//! (parent design Section 11). C3 provides the lock primitive only; no
//! executor, journal, or CLI consumes it yet.
//!
//! # Semantics
//! The lock is a process-lifetime OS advisory exclusive lock
//! (`fs2::FileExt::try_lock_exclusive`) on the fixed, regular file
//! `<automation>/operation.lock`. On contention the caller receives an error
//! beginning `OPERATION_IN_PROGRESS`; elapsed time is never inspected, the file
//! is never deleted, there is no retry loop, and ownership is never stolen.
//! A crashed process releases its OS lock automatically (the OS reclaims it on
//! process exit), so a later process can acquire it regardless of stale text
//! left on disk. A live process keeps its OS lock despite any old or tampered
//! on-disk text/mtime.
//!
//! `Drop` unlocks without removing the fixed lock file (avoids pathname races).
//! A pre-existing symlink or non-regular `operation.lock` is refused with an
//! error beginning `OPERATION_LOCK_UNSAFE`.
//!
//! # No-follow I/O (C3-R1)
//! The lock file is opened with `O_NOFOLLOW` on Unix so the kernel refuses a
//! symlink at the final component at open time, closing the TOCTOU hole left by
//! `symlink_metadata`-then-open. On Windows the lock acquisition path fails
//! closed with `OPERATION_LOCK_UNSAFE: unsupported platform for no-follow I/O`.
//!
//! # On-disk content
//! The file may contain only non-sensitive advisory metadata (operation id +
//! PID); it is advisory only and is truncated/re-written while the OS lock is
//! held. It never contains secrets, environment values, full configuration, or
//! absolute user paths.

use std::fs::File;

use anyhow::{Context, Result};
use fs2::FileExt;

use crate::cache;

/// A symlinked or otherwise non-regular operation lock path is refused.
const UNSAFE_PREFIX: &str = "OPERATION_LOCK_UNSAFE";
/// Another process holds the operation lock.
const IN_PROGRESS_PREFIX: &str = "OPERATION_IN_PROGRESS";

/// Validate `operation_id` as lowercase ASCII `[a-z0-9-]{1,64}` before any path
/// construction. The id is advisory metadata only, but validating it keeps the
/// on-disk content bounded and predictable.
fn validate_operation_id(id: &str) -> Result<()> {
    if id.is_empty() || id.len() > 64 {
        anyhow::bail!("invalid operation_id (length must be 1..=64): {id:?}");
    }
    if !id
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        anyhow::bail!("invalid operation_id (must match [a-z0-9-]{{1,64}}): {id:?}");
    }
    Ok(())
}

/// A guard holding the global automation operation lock. Dropping releases the
/// OS lock; the fixed lock file is never removed.
pub struct OperationLock {
    file: File,
}

impl std::fmt::Debug for OperationLock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OperationLock").finish_non_exhaustive()
    }
}

impl OperationLock {
    /// The fixed lock file path: `<automation>/operation.lock`.
    fn lock_path() -> std::path::PathBuf {
        cache::automation_root().join("operation.lock")
    }
}

impl Drop for OperationLock {
    fn drop(&mut self) {
        // Unlock only; never remove the fixed lock file (pathname races).
        let _ = self.file.unlock();
    }
}

/// Open or create the operation lock file with `O_NOFOLLOW` on Unix (C3-R1).
/// The kernel refuses a symlink at the final component at open time, closing
/// the TOCTOU hole. After opening, the handle is verified as a regular file.
/// On Windows, fail closed with `OPERATION_LOCK_UNSAFE`.
#[cfg(unix)]
fn open_lock_file_nofollow(path: &std::path::Path) -> Result<File> {
    use std::os::unix::fs::OpenOptionsExt;

    // Try to open/create with O_NOFOLLOW. For a new file, O_NOFOLLOW is a
    // no-op (there is no symlink to follow). For an existing file, O_NOFOLLOW
    // causes open(2) to fail with ELOOP if the final component is a symlink.
    //
    // We use create(true) (not create_new) so that a pre-existing regular lock
    // file from a prior process is reused. truncate(false) preserves the
    // advisory metadata until the OS lock is held.
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|e| {
            // ELOOP = symlink at final component (O_NOFOLLOW refusal).
            if e.raw_os_error() == Some(libc::ELOOP) {
                anyhow::anyhow!(
                    "{UNSAFE_PREFIX}: {} is a symlink (O_NOFOLLOW refused)",
                    path.display()
                )
            } else {
                anyhow::anyhow!("open operation lock {}: {e}", path.display())
            }
        })?;

    // Verify the opened handle is a regular file (not a directory, device,
    // or any non-regular type).
    let meta = file
        .metadata()
        .with_context(|| format!("fstat operation lock {}", path.display()))?;
    if !meta.is_file() {
        anyhow::bail!(
            "{UNSAFE_PREFIX}: {} is not a regular file (symlink/non-regular refused)",
            path.display()
        );
    }
    Ok(file)
}

/// Windows: fail closed for no-follow lock acquisition. Safe Windows no-follow
/// requires `FILE_FLAG_OPEN_REPARSE_POINT` via WinAPI, not available from
/// std+fs2.
#[cfg(not(unix))]
fn open_lock_file_nofollow(path: &std::path::Path) -> Result<File> {
    anyhow::bail!(
        "{UNSAFE_PREFIX}: unsupported platform for no-follow I/O at {}",
        path.display()
    );
}

/// Acquire the global automation operation lock for `operation_id`.
///
/// Validates the id, ensures the automation root exists, opens/creates the
/// regular lock file with `O_NOFOLLOW` (refusing a symlink at open time),
/// verifies the handle is a regular file, and takes an OS advisory exclusive
/// lock via `try_lock_exclusive`. On contention returns an error beginning
/// `OPERATION_IN_PROGRESS`; on an unsafe lock path returns an error beginning
/// `OPERATION_LOCK_UNSAFE`.
pub fn acquire_operation_lock(operation_id: &str) -> Result<OperationLock> {
    validate_operation_id(operation_id)?;

    // Ensure the automation root (and operations/) exists. The lock file lives
    // directly under automation/, which ensure_automation_root creates.
    crate::automation::plan::ensure_automation_root()?;

    let path = OperationLock::lock_path();

    // Open or create the lock file with O_NOFOLLOW. This refuses a symlink at
    // the final component at open(2) time (ELOOP on Unix), closing the TOCTOU
    // hole left by symlink_metadata-then-open.
    let file = open_lock_file_nofollow(&path)?;

    // Take the OS advisory exclusive lock without blocking. try_lock_exclusive
    // fails immediately if another process holds it.
    file.try_lock_exclusive().map_err(|e| {
        anyhow::anyhow!(
            "{IN_PROGRESS_PREFIX}: another automation operation holds the lock at {}: {e}",
            path.display()
        )
    })?;

    // Lock held: write non-sensitive advisory metadata (operation id + PID).
    // Truncate then rewrite while the OS lock is held.
    if let Err(e) = write_metadata(&file, operation_id) {
        // Best-effort: metadata is advisory only; release the lock on failure.
        let _ = file.unlock();
        return Err(e).context("write operation lock metadata");
    }

    Ok(OperationLock { file })
}

/// Truncate and write non-sensitive advisory metadata (operation id + PID)
/// to the lock file. Advisory only; never secrets or full configuration.
fn write_metadata(file: &File, operation_id: &str) -> Result<()> {
    use std::io::Write;
    // Set length to 0 (truncate) then write the new metadata.
    file.set_len(0).with_context(|| "truncate operation lock")?;
    let pid = std::process::id();
    let mut buf = String::new();
    buf.push_str("operation_id=");
    buf.push_str(operation_id);
    buf.push('\n');
    buf.push_str("pid=");
    buf.push_str(&pid.to_string());
    buf.push('\n');
    // `Write` is implemented for `&File`; bind mutably so the trait method
    // can take `&mut self` (`&mut &File`).
    let mut handle: &File = file;
    handle
        .write_all(buf.as_bytes())
        .with_context(|| "write operation lock metadata")?;
    handle.flush().with_context(|| "flush operation lock")?;
    Ok(())
}
