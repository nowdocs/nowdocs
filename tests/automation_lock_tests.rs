//! C3 automation operation lock tests.
//!
//! C3-R1: tests use a poison-resistant `EnvGuard` (static mutex + save/restore
//! `XDG_CACHE_HOME` in Drop) so they are hermetic under explicit parallelism
//! (`--test-threads=4`). No test depends on `RUST_TEST_THREADS=1`.
//!
//! No test here reads a real cache, client config, model, or network. Lock
//! contention is exercised deterministically through drop sequencing - no
//! sleeps are used.

use std::sync::Mutex;

use nowdocs::automation::lock;

// C3-R1: env-mutation guard. A static mutex serializes XDG_CACHE_HOME access
// across tests; Drop restores the prior value. A poisoned mutex is recovered
// so subsequent tests can still run.
static ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvGuard {
    key: &'static str,
    old: Option<String>,
    _g: std::sync::MutexGuard<'static, ()>,
}

impl EnvGuard {
    fn set(key: &'static str, val: &str) -> Self {
        let g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
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

// --- Test 5: second operation lock is refused until the first drops ---

#[test]
fn second_operation_lock_is_refused_until_first_drops() {
    let dir = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", dir.path().to_str().unwrap());

    // Hold the first guard.
    let first = lock::acquire_operation_lock("op-contention").expect("first acquire");

    // A second acquisition must be refused while the first is held.
    let err = lock::acquire_operation_lock("op-contention-2").unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.starts_with("OPERATION_IN_PROGRESS"),
        "second lock must yield OPERATION_IN_PROGRESS, got: {msg}"
    );

    // Dropping the first releases the OS lock; a new acquire must succeed.
    drop(first);
    let _second = lock::acquire_operation_lock("op-contention-3").expect("acquire after drop");

    // The fixed lock file must still exist (Drop never removes it).
    let lock_file = nowdocs::cache::automation_root().join("operation.lock");
    assert!(
        lock_file.is_file(),
        "operation.lock must persist after drop (no pathname removal)"
    );
}

// --- Test 6: operation lock does not follow a symlink (open-time O_NOFOLLOW) ---

#[test]
fn operation_lock_does_not_follow_symlink() {
    let dir = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", dir.path().to_str().unwrap());

    // Ensure the automation root exists so we can plant the symlink.
    let root = nowdocs::cache::automation_root();
    std::fs::create_dir_all(&root).unwrap();
    let lock_file = root.join("operation.lock");

    // Plant a symlink pointing at an external target.
    #[cfg(unix)]
    {
        let external = dir.path().join("external-op-target");
        std::fs::write(&external, b"external").unwrap();
        std::os::unix::fs::symlink(&external, &lock_file).unwrap();

        let err = lock::acquire_operation_lock("op-symlink").unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.starts_with("OPERATION_LOCK_UNSAFE"),
            "symlinked operation.lock must yield OPERATION_LOCK_UNSAFE, got: {msg}"
        );

        // The external target must be unchanged (never followed/written).
        let after = std::fs::read_to_string(&external).unwrap();
        assert_eq!(after, "external", "external target must be untouched");
        // Remove the symlink so non-Unix-equivalent cleanup is consistent.
        std::fs::remove_file(&lock_file).unwrap();
    }
    #[cfg(not(unix))]
    {
        // On Windows the no-follow path fails closed. Create a directory where
        // the lock file should be to exercise the non-regular refusal (if the
        // platform even reaches that check before the fail-closed error).
        std::fs::create_dir_all(&lock_file).unwrap();
        let err = lock::acquire_operation_lock("op-symlink").unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.starts_with("OPERATION_LOCK_UNSAFE"),
            "non-regular operation.lock must yield OPERATION_LOCK_UNSAFE, got: {msg}"
        );
    }
}

// --- Supplementary: invalid operation_id is rejected before path construction ---

#[test]
fn operation_lock_rejects_invalid_operation_id() {
    let dir = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", dir.path().to_str().unwrap());

    // Uppercase, separators, path traversal, and too-long ids are all invalid.
    for bad in ["BadID", "has/slash", "..", "has space", &"a".repeat(65)] {
        let err = lock::acquire_operation_lock(bad).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("operation_id"),
            "invalid operation_id {bad:?} must be rejected, got: {msg}"
        );
    }
    // A valid id acquires.
    let _g2 = lock::acquire_operation_lock("valid-op-1").expect("valid id acquires");
}

// --- C3-R1: symlinked automation root is refused by operation lock ---

#[test]
#[cfg(unix)]
fn operation_lock_refuses_symlinked_automation_root() {
    let dir = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", dir.path().to_str().unwrap());

    // Create the cache root, then replace automation/ with a symlink.
    let cache_root = nowdocs::cache::cache_root();
    std::fs::create_dir_all(&cache_root).unwrap();
    let external = dir.path().join("external-auto-lock");
    std::fs::create_dir_all(&external).unwrap();
    let auto_root = cache_root.join("automation");
    std::os::unix::fs::symlink(&external, &auto_root).unwrap();

    // Operation lock acquisition must refuse to create through a symlinked root.
    let err = lock::acquire_operation_lock("op-symlink-root").unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("not a directory") || msg.contains("symlink"),
        "symlinked automation root must be refused, got: {msg}"
    );

    // The external target must not have operation.lock created inside it.
    assert!(
        !external.join("operation.lock").exists(),
        "external symlink target must not have operation.lock created inside it"
    );
}
