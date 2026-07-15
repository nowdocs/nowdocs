//! Safe approved-root + relative-target configuration I/O for client adapters.
//!
//! All paths are validated as relative under an explicit approved root.
//! Absolute paths, parent traversal, backslash separators, symlinks, and
//! non-regular files are refused. Writes are atomic and digest-verified.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

/// A validated, no-symlink directory used as the root for all client-config
/// reads and writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovedRoot {
    root: PathBuf,
}

/// A validated relative path under an [`ApprovedRoot`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeTarget {
    approved: ApprovedRoot,
    relative: PathBuf,
}

impl ApprovedRoot {
    /// The validated absolute root path.
    pub fn path(&self) -> &Path {
        &self.root
    }
}

impl SafeTarget {
    /// The approved root this target is relative to.
    pub fn approved(&self) -> &ApprovedRoot {
        &self.approved
    }

    /// The validated relative path component.
    pub fn relative(&self) -> &Path {
        &self.relative
    }

    /// The absolute path resolved from the approved root and relative target.
    pub fn path(&self) -> PathBuf {
        self.approved.root.join(&self.relative)
    }

    /// The relative target as a logical path string suitable for journals.
    pub fn logical(&self) -> String {
        self.relative.to_string_lossy().to_string()
    }
}

/// Validate and take ownership of an approved root directory.
///
/// The path must be absolute, exist as a directory, contain no symlink
/// component, and (on Unix) have neither group nor other write bit set
/// (`mode & 0o022 == 0`). This accepts normal `0755` and private `0700`;
/// it refuses `0775`, `0777`, and any other group/world-writable mode.
/// The function is a pure validator: it never changes the directory's mode.
pub fn approved_root(path: &Path) -> Result<ApprovedRoot> {
    if !path.is_absolute() {
        anyhow::bail!("approved root must be absolute: {}", path.display());
    }
    let meta = std::fs::symlink_metadata(path)
        .with_context(|| format!("stat approved root {}", path.display()))?;
    if meta.file_type().is_symlink() {
        anyhow::bail!("approved root {} is a symlink (refused)", path.display());
    }
    if !meta.is_dir() {
        anyhow::bail!("approved root {} is not a directory", path.display());
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = meta.permissions().mode() & 0o777;
        if mode & 0o022 != 0 {
            anyhow::bail!(
                "approved root {} is group- or world-writable ({:03o})",
                path.display(),
                mode
            );
        }
        if mode & 0o700 == 0 {
            anyhow::bail!(
                "approved root {} has no owner permissions ({:03o})",
                path.display(),
                mode
            );
        }
    }
    #[cfg(not(unix))]
    {
        // Restrictive private storage cannot be proven on Windows without ACL
        // inspection. We validate the directory structure but do not enforce
        // owner-only mode here.
    }

    Ok(ApprovedRoot {
        root: path.to_path_buf(),
    })
}

/// Validate a relative target path under an approved root.
///
/// Rejects absolute paths, empty strings, `..` traversal, backslash separators,
/// control characters, and NUL.
pub fn safe_target(approved: &ApprovedRoot, relative: &str) -> Result<SafeTarget> {
    if relative.is_empty() {
        anyhow::bail!("target path must not be empty");
    }
    if relative.starts_with('/') || relative.starts_with('\\') {
        anyhow::bail!(
            "target path must be relative (absolute refused): {}",
            relative
        );
    }
    if relative.len() >= 2 {
        let b = relative.as_bytes();
        if b[0].is_ascii_alphabetic() && b[1] == b':' {
            anyhow::bail!(
                "target path must be relative (drive-letter refused): {}",
                relative
            );
        }
    }
    if relative.bytes().any(|b| b == 0 || b.is_ascii_control()) {
        anyhow::bail!(
            "target path contains NUL or control characters: {}",
            relative
        );
    }
    if relative.contains('\\') {
        anyhow::bail!(
            "target path must use forward slashes (backslash refused): {}",
            relative
        );
    }
    if relative == ".."
        || relative.starts_with("../")
        || relative.ends_with("/..")
        || relative.contains("/../")
    {
        anyhow::bail!(
            "target path must not contain parent traversal (..): {}",
            relative
        );
    }
    for component in relative.split('/') {
        if component == ".." {
            anyhow::bail!("target path component must not be ..: {}", relative);
        }
        if component.is_empty() {
            anyhow::bail!("target path contains empty component: {}", relative);
        }
    }

    Ok(SafeTarget {
        approved: approved.clone(),
        relative: PathBuf::from(relative),
    })
}

/// SHA-256 hex digest of a byte slice.
pub fn compute_digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut s = String::with_capacity(digest.len() * 2);
    for b in digest {
        use std::fmt::Write;
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Read the contents of a safe target via a no-follow open.
pub fn read_target(target: &SafeTarget) -> Result<Vec<u8>> {
    ensure_real_parent_components(target)?;
    let path = target.path();
    let mut file = open_nofollow_read(&path).with_context(|| {
        format!(
            "unsafe target {} (symlink or non-regular file refused)",
            path.display()
        )
    })?;
    use std::io::Read;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .with_context(|| format!("read {}", path.display()))?;
    Ok(buf)
}

/// Atomically replace a safe target with `content`, verifying the digest after
/// rename. Returns the SHA-256 hex digest of the written content.
pub fn atomic_replace(target: &SafeTarget, content: &[u8]) -> Result<String> {
    ensure_real_parent_components(target)?;
    let path = target.path();
    let expected = compute_digest(content);

    // If the target already exists, verify it is a real regular file (not a
    // symlink or directory) before we replace it.
    if let Ok(meta) = std::fs::symlink_metadata(&path) {
        if meta.file_type().is_symlink() || !meta.is_file() {
            anyhow::bail!(
                "target {} is not a regular file (symlink/non-regular refused)",
                path.display()
            );
        }
    }

    let parent = path
        .parent()
        .with_context(|| format!("target {} has no parent directory", path.display()))?;

    let tmp_name = format!(".tmp.nowdocs.{}.{}", std::process::id(), timestamp_nanos());
    let tmp_path = parent.join(&tmp_name);

    // Write temp file with restrictive permissions.
    {
        let mut file = open_private_temp(&tmp_path)?;
        use std::io::Write;
        file.write_all(content)
            .with_context(|| format!("write temp file {}", tmp_path.display()))?;
        file.flush()
            .with_context(|| format!("flush temp file {}", tmp_path.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let rc = unsafe { libc::fchmod(file.as_raw_fd(), 0o600) };
            if rc != 0 {
                return Err(std::io::Error::last_os_error())
                    .with_context(|| format!("fchmod 0600 {}", tmp_path.display()));
            }
        }
    }

    std::fs::rename(&tmp_path, &path)
        .with_context(|| format!("atomic rename {} -> {}", tmp_path.display(), path.display()))?;

    // Fsync parent directory on Unix so the rename is durable.
    #[cfg(unix)]
    {
        let dir = std::fs::File::open(parent)
            .with_context(|| format!("open parent directory {}", parent.display()))?;
        let _ = dir.sync_all();
    }

    // Reopen no-follow and verify digest.
    let mut file = open_nofollow_read(&path)?;
    use std::io::Read;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .with_context(|| format!("read back {}", path.display()))?;
    let actual = compute_digest(&buf);
    if actual != expected {
        anyhow::bail!(
            "digest mismatch after atomic replace at {} (expected {expected}, got {actual})",
            path.display()
        );
    }
    Ok(expected)
}

fn timestamp_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

/// Verify that the approved root and every existing parent component of a
/// target are real directories. `O_NOFOLLOW` protects only the final component
/// of an `open(2)` call, so this explicit walk is required before opening a
/// nested target such as `root/config/file.json`.
fn ensure_real_parent_components(target: &SafeTarget) -> Result<()> {
    let mut current = target.approved.root.clone();
    ensure_real_dir(&current)?;

    let components: Vec<_> = target.relative.components().collect();
    for component in components.iter().take(components.len().saturating_sub(1)) {
        current.push(component.as_os_str());
        ensure_real_dir(&current)?;
    }
    Ok(())
}

fn ensure_real_dir(path: &Path) -> Result<()> {
    let meta = std::fs::symlink_metadata(path)
        .with_context(|| format!("stat target parent {}", path.display()))?;
    if meta.file_type().is_symlink() || !meta.is_dir() {
        anyhow::bail!(
            "target parent {} is not a real directory (symlink refused)",
            path.display()
        );
    }
    Ok(())
}

/// Create a temporary replacement file without following or truncating an
/// existing path. A collision is refused rather than risking a write through a
/// pre-created symlink; callers can safely retry the whole operation.
#[cfg(unix)]
fn open_private_temp(path: &Path) -> Result<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .with_context(|| format!("create private temp file {}", path.display()))
}

#[cfg(not(unix))]
fn open_private_temp(path: &Path) -> Result<std::fs::File> {
    anyhow::bail!(
        "unsupported platform for no-follow I/O at {}",
        path.display()
    );
}

/// Open a file for reading with `O_NOFOLLOW` on Unix. On Windows, fail closed.
#[cfg(unix)]
fn open_nofollow_read(path: &Path) -> Result<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    let file = std::fs::OpenOptions::new()
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
    Ok(file)
}

#[cfg(not(unix))]
fn open_nofollow_read(path: &Path) -> Result<std::fs::File> {
    anyhow::bail!(
        "unsupported platform for no-follow I/O at {}",
        path.display()
    );
}
