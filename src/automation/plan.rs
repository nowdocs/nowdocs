//! Deterministic automation plans: semantic data, SHA-256 identity, private
//! storage, 30-minute expiry, tamper detection, and precondition fingerprints.
//!
//! A plan is an ordered, immutable description of the actions a later
//! `setup apply`/`ensure --apply` will perform. C3 exposes the storage and
//! integrity layer only: there is no plan-creation flow, no registry-online
//! planning, no execution, no client configuration, and no rollback here.
//!
//! # Determinism
//! For identical inputs and observed preconditions the plan id is stable
//! across equivalent constructions. Variable-length lists are canonicalized
//! before hashing: docset/target-file preconditions sort by their declared
//! identity, target paths sort lexicographically within an action, and action
//! order is preserved because it is semantic execution order. Lifecycle
//! metadata (creation/expiration time) is part of the hash material, so two
//! separately created plans differ by id even when their actions match.
//!
//! # Integrity, not consent
//! The SHA-256 id is an integrity and scope-consistency check: it detects
//! tampering and stale/corrupt plan data. It does NOT authenticate a human or
//! prove consent; a malicious process running as the same OS user can forge
//! a plan. Human approval is enforced by the invoking agent/terminal workflow.
//!
//! # Lifecycle normalization gate (C3-R1)
//! Every public lifecycle entry point—`new_plan`, `plan_id`, `store_plan`,
//! `load_plan`—normalizes the plan through one private `normalize_plan`
//! helper before hashing, serializing, or returning. A hand-built public
//! `AutomationPlan` with destructive risk, wrong schema/version, unsafe
//! target paths, or invalid optional input is therefore rejected at every
//! entry point, not only at `new_plan`.
//!
//! # Storage
//! Plans live at `<cache>/nowdocs/automation/plans/<id>.json` as compact JSON.
//! The `automation/` subtree is additive and does not change the existing
//! cache layout version (1); older binaries safely ignore it. `store_plan`
//! uses `create_new` (never overwrites) and writes `0600` on Unix.
//! `load_plan` rejects non-regular files/symlinks, malformed JSON, unknown
//! fields, a hash/file-name mismatch, a nowdocs-version mismatch, and expired
//! plans. It performs no network/model/client/config I/O.
//!
//! # No-follow I/O (C3-R1)
//! Every automation directory component is verified as a real directory
//! (never a symlink) before creating or writing below it. Final file
//! components (`<id>.json`) are opened with `O_NOFOLLOW` on Unix so the
//! kernel refuses a symlink at open time, closing the TOCTOU hole left by a
//! `symlink_metadata`-then-open sequence. On Windows the plan mutation path
//! fails closed with `PLAN_TAMPERED: unsupported platform for no-follow I/O`.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::agent_contract::{self, AGENT_CONTRACT_SCHEMA_VERSION};
use crate::cache;
use crate::input;

/// Re-export the canonical action risk type from the agent contract. C3-R1
/// removes the duplicate `automation::plan::RiskLevel` enum; the contract is
/// the single owner of the action model.
pub use crate::agent_contract::RiskLevel;

/// Plan lifetime: 30 minutes (parent design Section 8).
pub const PLAN_TTL_SECS: u64 = 30 * 60;

/// Lowercase hex sha256 of an in-memory byte slice.
fn hex_sha256(bytes: &[u8]) -> String {
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

/// Validate that `id` is exactly 64 lowercase ASCII hex characters.
fn is_valid_plan_id(id: &str) -> bool {
    id.len() == 64
        && id
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

/// Validate a digest field as exactly 64 lowercase hex characters (C3-R1
/// tightens from mixed-case hex to lowercase-only).
fn is_valid_digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

/// Validate a bounded ASCII identifier: non-empty, ≤128 bytes, no control
/// characters, no path separators (`/`, `\`), no `..`, no NUL, and not
/// absolute/path-like (no leading `/` or drive letter on Windows).
fn is_safe_identifier(s: &str) -> bool {
    if s.is_empty() || s.len() > 128 {
        return false;
    }
    if s == ".." || s.contains("..") {
        return false;
    }
    s.bytes()
        .all(|b| b.is_ascii_graphic() && b != b'/' && b != b'\\')
}

/// Validate a target path string: must be relative, must not contain `..`,
/// backslash separators, NUL, or control characters. Individual safe
/// relative components are sorted lexicographically by the caller.
fn is_safe_target_path(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    // Reject absolute paths (Unix or Windows drive-letter).
    if s.starts_with('/') || s.starts_with('\\') {
        return false;
    }
    if s.len() >= 2 && s.as_bytes()[1] == b':' && s.as_bytes()[0].is_ascii_alphabetic() {
        return false;
    }
    // Reject parent traversal, backslash separators, NUL, control chars.
    !s.bytes().any(|b| {
        b == b'\\' || b == 0 || b.is_ascii_control() || {
            // Check for ".." as a path component.
            false
        }
    }) && !s.split('/').any(|component| component == "..")
}

// ---- Plan data model (parent design Section 7.1 / task §4.2) ----

/// Normalized user inputs captured by a plan. Values are identifiers only;
/// no tokens, environment values, or full configuration bytes are stored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanInputs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docset: Option<String>,
    pub online: bool,
}

/// Precondition fingerprint for one installed docset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocsetPrecondition {
    pub docset: String,
    pub installed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_sha256: Option<String>,
}

/// Precondition fingerprint for one target file (e.g. a client config entry).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetFilePrecondition {
    pub logical_id: String,
    pub exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}

/// Observed preconditions snapshotted at plan creation. A drift in any of
/// these at apply time makes the plan stale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanPreconditions {
    pub cache_layout: String,
    pub model_present: bool,
    pub docset_state: Vec<DocsetPrecondition>,
    pub target_files: Vec<TargetFilePrecondition>,
}

/// One ordered, semantic action in a plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlannedAction {
    pub id: String,
    pub kind: String,
    pub risk: RiskLevel,
    pub summary: String,
    pub changes_state: bool,
    pub network_access: bool,
    pub requires_confirmation: bool,
    pub reversible: bool,
    pub target_paths: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_download_bytes: Option<u64>,
}

/// A complete automation plan. Public fields allow construction for testing,
/// but every lifecycle entry point normalizes through [`normalize_plan`]
/// before hashing, storing, or returning, so a semantically invalid plan can
/// never be persisted or loaded as valid.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AutomationPlan {
    pub schema_version: u32,
    pub nowdocs_version: String,
    pub created_at_unix_secs: u64,
    pub expires_at_unix_secs: u64,
    pub inputs: PlanInputs,
    pub preconditions: PlanPreconditions,
    pub actions: Vec<PlannedAction>,
}

/// Canonical hash material: every plan field except `plan_hash` (which is
/// derived from this material, never stored on the plan). Serialized with
/// `serde_json::to_vec` and sha256-hashed to produce the plan id.
///
/// This is a separate struct so the hashing contract is explicit and stable:
/// adding a field to [`AutomationPlan`] requires adding it here too, otherwise
/// the id would silently stop covering it.
#[derive(Debug, Clone, Serialize)]
struct PlanHashMaterial<'a> {
    schema_version: u32,
    nowdocs_version: &'a str,
    created_at_unix_secs: u64,
    expires_at_unix_secs: u64,
    inputs: &'a PlanInputs,
    preconditions: &'a PlanPreconditions,
    actions: &'a [PlannedAction],
}

// ---- Validation & canonicalization ----

/// True when `risk` implies state mutation (parent design Section 9).
fn risk_implies_state_change(risk: RiskLevel) -> bool {
    matches!(risk, RiskLevel::Additive | RiskLevel::Mutating)
}

/// Clone, validate, and canonicalize a plan. This is the single normalization
/// gate for every lifecycle entry point (C3-R1 defect 3). A hand-built public
/// `AutomationPlan` that fails any semantic check is rejected here.
///
/// Canonicalization:
/// - `schema_version` must equal `AGENT_CONTRACT_SCHEMA_VERSION`;
/// - `nowdocs_version` must equal the compiled binary version;
/// - `expires_at_unix_secs` must equal `created_at + PLAN_TTL_SECS`;
/// - `inputs.docset` validated via `input::validate_docset` when present;
/// - `inputs.client`, action IDs/kinds, and target logical IDs validated as
///   bounded ASCII identifiers (reject empty, controls, separators,
///   absolute/path-like values, and `..`);
/// - `RiskLevel::Destructive` rejected; duplicate action/docset/target IDs
///   rejected; risk/`changes_state` mismatch rejected;
/// - digest fields validated as 64 lowercase hex;
/// - target paths reject absolute, parent traversal, backslash, NUL, control;
///   safe relative components sorted lexicographically within each action;
/// - docset/target-file preconditions sorted by their declared identity while
///   action order is preserved (it is semantic execution order).
fn normalize_plan(plan: &AutomationPlan) -> Result<AutomationPlan> {
    // Schema + version pin.
    if plan.schema_version != AGENT_CONTRACT_SCHEMA_VERSION {
        anyhow::bail!(
            "plan schema_version {} != contract {}",
            plan.schema_version,
            AGENT_CONTRACT_SCHEMA_VERSION
        );
    }
    if plan.nowdocs_version != env!("CARGO_PKG_VERSION") {
        anyhow::bail!(
            "plan nowdocs_version {} does not match binary {}",
            plan.nowdocs_version,
            env!("CARGO_PKG_VERSION")
        );
    }
    if plan.expires_at_unix_secs != plan.created_at_unix_secs.saturating_add(PLAN_TTL_SECS) {
        anyhow::bail!(
            "plan expires_at_unix_secs must equal created_at_unix_secs + {}",
            PLAN_TTL_SECS
        );
    }

    // Validate optional inputs.
    if let Some(docset) = &plan.inputs.docset {
        input::validate_docset(docset)
            .with_context(|| format!("invalid inputs.docset: {:?}", docset))?;
    }
    if let Some(client) = &plan.inputs.client {
        if !is_safe_identifier(client) {
            anyhow::bail!(
                "invalid inputs.client (must be a bounded ASCII identifier): {:?}",
                client
            );
        }
    }

    let mut normalized = plan.clone();

    // Validate + canonicalize docset preconditions (sort by docset identity).
    let mut seen_docsets = std::collections::HashSet::new();
    for d in &normalized.preconditions.docset_state {
        input::validate_docset(&d.docset)
            .with_context(|| format!("invalid docset in precondition: {:?}", d.docset))?;
        if !seen_docsets.insert(d.docset.clone()) {
            anyhow::bail!("duplicate docset precondition: {}", d.docset);
        }
        if let Some(sha) = &d.manifest_sha256 {
            if !is_valid_digest(sha) {
                anyhow::bail!(
                    "docset {} manifest_sha256 must be 64 lowercase hex chars, got {:?}",
                    d.docset,
                    sha
                );
            }
        }
    }
    normalized
        .preconditions
        .docset_state
        .sort_by(|a, b| a.docset.cmp(&b.docset));

    // Validate + canonicalize target-file preconditions (sort by logical id).
    let mut seen_targets = std::collections::HashSet::new();
    for t in &normalized.preconditions.target_files {
        if !is_safe_identifier(&t.logical_id) {
            anyhow::bail!(
                "target file logical_id must be a bounded ASCII identifier: {:?}",
                t.logical_id
            );
        }
        if !seen_targets.insert(t.logical_id.clone()) {
            anyhow::bail!("duplicate target file logical_id: {}", t.logical_id);
        }
        if let Some(sha) = &t.sha256 {
            if !is_valid_digest(sha) {
                anyhow::bail!(
                    "target file {} sha256 must be 64 lowercase hex chars, got {:?}",
                    t.logical_id,
                    sha
                );
            }
        }
    }
    normalized
        .preconditions
        .target_files
        .sort_by(|a, b| a.logical_id.cmp(&b.logical_id));

    // Validate + canonicalize actions (preserve order; sort target paths).
    let mut seen_action_ids = std::collections::HashSet::new();
    for a in &mut normalized.actions {
        if !is_safe_identifier(&a.id) {
            anyhow::bail!("action id must be a bounded ASCII identifier: {:?}", a.id);
        }
        if !is_safe_identifier(&a.kind) {
            anyhow::bail!(
                "action kind must be a bounded ASCII identifier: {:?}",
                a.kind
            );
        }
        if !seen_action_ids.insert(a.id.clone()) {
            anyhow::bail!("duplicate action id: {}", a.id);
        }
        if matches!(a.risk, RiskLevel::Destructive) {
            anyhow::bail!(
                "destructive actions are excluded from automated setup (action {})",
                a.id
            );
        }
        if a.changes_state != risk_implies_state_change(a.risk) {
            anyhow::bail!(
                "action {} risk {:?} is inconsistent with changes_state={} \
                 (additive/mutating must change state; read_only/internal_ephemeral must not)",
                a.id,
                a.risk,
                a.changes_state
            );
        }
        // Validate and sort target paths.
        for tp in &a.target_paths {
            if !is_safe_target_path(tp) {
                anyhow::bail!(
                    "action {} target_path is unsafe (absolute/traversal/backslash/control): {:?}",
                    a.id,
                    tp
                );
            }
        }
        a.target_paths.sort();
    }

    Ok(normalized)
}

// ---- Public lifecycle API (task §4.3) ----

/// Create a new, canonicalized plan. `created_at_unix_secs` is explicit so
/// tests control time deterministically; no clock is read here. Expiry is set
/// to `created_at_unix_secs + PLAN_TTL_SECS`.
#[allow(clippy::too_many_arguments)]
pub fn new_plan(
    inputs: PlanInputs,
    preconditions: PlanPreconditions,
    actions: Vec<PlannedAction>,
    created_at_unix_secs: u64,
) -> Result<AutomationPlan> {
    let plan = AutomationPlan {
        schema_version: AGENT_CONTRACT_SCHEMA_VERSION,
        nowdocs_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at_unix_secs,
        expires_at_unix_secs: created_at_unix_secs.saturating_add(PLAN_TTL_SECS),
        inputs,
        preconditions,
        actions,
    };
    normalize_plan(&plan)
}

/// Compute the plan id: 64 lowercase ASCII hex characters derived from the
/// canonical hash material. Stable for equivalent semantics (see module docs).
/// Normalizes the plan first (C3-R1), so a hand-built invalid plan is rejected
/// rather than hashed.
pub fn plan_id(plan: &AutomationPlan) -> Result<String> {
    let normalized = normalize_plan(plan)?;
    let material = PlanHashMaterial {
        schema_version: normalized.schema_version,
        nowdocs_version: &normalized.nowdocs_version,
        created_at_unix_secs: normalized.created_at_unix_secs,
        expires_at_unix_secs: normalized.expires_at_unix_secs,
        inputs: &normalized.inputs,
        preconditions: &normalized.preconditions,
        actions: &normalized.actions,
    };
    let bytes = serde_json::to_vec(&material).context("serialize plan hash material")?;
    Ok(hex_sha256(&bytes))
}

/// Ensure the private automation root and its `plans/` + `operations/`
/// subdirectories exist. This is the *only* C3 initializer: it creates
/// `automation/`, `plans/`, and `operations/` only when a caller explicitly
/// creates or acquires C3 state. Every component in the path is verified as a
/// real directory (never a symlink) before creating below it (C3-R1). On Unix,
/// newly created directories are repaired to owner-only mode (`0700`); an
/// existing user directory is never silently chmoded.
pub fn ensure_automation_root() -> Result<()> {
    let root = cache::automation_root();
    ensure_private_dir(&root)?;
    ensure_private_dir(&root.join("plans"))?;
    ensure_private_dir(&root.join("operations"))?;
    Ok(())
}

/// Walk the ancestor chain of `dir` from the highest non-existent ancestor
/// down to `dir`, verifying each existing component is a real directory (never
/// a symlink) and creating missing components one at a time. This replaces the
/// unsafe `create_dir_all` which would silently follow a symlinked intermediate
/// component (C3-R1).
#[cfg(unix)]
fn ensure_private_dir(dir: &Path) -> Result<()> {
    use std::os::unix::fs::DirBuilderExt;

    // Collect the chain of ancestors that need to exist, from the highest
    // (closest to root) to the target itself.
    let mut chain = Vec::new();
    let mut current = dir.to_path_buf();
    loop {
        match std::fs::symlink_metadata(&current) {
            Ok(meta) => {
                // This ancestor exists. Verify it is a real directory (not a
                // symlink). symlink_metadata does not follow, so a symlink here
                // reports as "symlink", not "dir".
                if !meta.is_dir() {
                    anyhow::bail!(
                        "automation path {} exists but is not a directory \
                         (symlink/non-directory refused)",
                        current.display()
                    );
                }
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                chain.push(current.clone());
                if !current.pop() {
                    // Reached the filesystem root without finding an existing
                    // ancestor. This should not happen in practice.
                    anyhow::bail!(
                        "cannot find existing ancestor for automation path {}",
                        dir.display()
                    );
                }
            }
            Err(e) => {
                anyhow::bail!("cannot stat automation ancestor {}: {e}", current.display());
            }
        }
    }

    // Create each missing component from highest to lowest (parent-first).
    // Each create_dir is a single-component create (not create_dir_all), so a
    // symlink planted between the stat above and this create is handled at the
    // next ensure_private_dir call's stat check. The final file-level I/O uses
    // O_NOFOLLOW to close the residual TOCTOU window for files.
    for component in chain.into_iter().rev() {
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&component)
            .with_context(|| format!("create automation directory {}", component.display()))?;
    }

    Ok(())
}

#[cfg(not(unix))]
fn ensure_private_dir(dir: &Path) -> Result<()> {
    let mut chain = Vec::new();
    let mut current = dir.to_path_buf();
    loop {
        match std::fs::symlink_metadata(&current) {
            Ok(meta) => {
                if !meta.is_dir() {
                    anyhow::bail!(
                        "automation path {} exists but is not a directory \
                         (symlink/non-directory refused)",
                        current.display()
                    );
                }
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                chain.push(current.clone());
                if !current.pop() {
                    anyhow::bail!(
                        "cannot find existing ancestor for automation path {}",
                        dir.display()
                    );
                }
            }
            Err(e) => {
                anyhow::bail!("cannot stat automation ancestor {}: {e}", current.display());
            }
        }
    }
    for component in chain.into_iter().rev() {
        std::fs::create_dir(&component)
            .with_context(|| format!("create automation directory {}", component.display()))?;
    }
    Ok(())
}

/// The fixed path of the stored plan file for `id` (validated first).
fn plan_path(id: &str) -> Result<PathBuf> {
    if !is_valid_plan_id(id) {
        anyhow::bail!("invalid plan id (must be 64 lowercase hex chars): {id:?}");
    }
    Ok(cache::automation_root()
        .join("plans")
        .join(format!("{id}.json")))
}

/// Open a file for reading with `O_NOFOLLOW` on Unix (C3-R1). The kernel
/// refuses to follow a symlink in the final path component at `open(2)` time,
/// closing the TOCTOU hole left by `symlink_metadata`-then-open. After
/// opening, the handle is verified as a regular file. On Windows, fail closed
/// with a stable error (safe Windows no-follow requires WinAPI not available
/// from std+fs2).
#[cfg(unix)]
fn open_nofollow_read(path: &Path) -> Result<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .with_context(|| format!("open (O_NOFOLLOW) {}", path.display()))?;
    // Verify the opened handle is a regular file (not a symlink that was
    // somehow followed, not a device, not a directory).
    let meta = file
        .metadata()
        .with_context(|| format!("fstat {}", path.display()))?;
    if !meta.is_file() {
        anyhow::bail!("PLAN_TAMPERED: {} is not a regular file", path.display());
    }
    Ok(file)
}

/// Windows: fail closed for no-follow reads. Safe Windows no-follow requires
/// `FILE_FLAG_OPEN_REPARSE_POINT` via WinAPI, which is not available from
/// std+fs2. Rather than silently follow reparse points, refuse.
#[cfg(not(unix))]
fn open_nofollow_read(path: &Path) -> Result<std::fs::File> {
    anyhow::bail!(
        "PLAN_TAMPERED: unsupported platform for no-follow I/O at {}",
        path.display()
    );
}

/// Open or create a file for writing with `O_NOFOLLOW` on Unix (C3-R1). For
/// `create_new`, the file must not already exist; `O_NOFOLLOW` ensures a
/// symlink at the final component is not followed. Returns the raw io::Result
/// so callers can distinguish `AlreadyExists` from other errors. On Windows,
/// fail closed.
#[cfg(unix)]
fn open_nofollow_create_new(path: &Path) -> std::io::Result<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
}

/// Restrict a newly created plan through its already-open descriptor. Using
/// `fchmod` avoids a path-based chmod race if the pathname is replaced after
/// `create_new` succeeds.
#[cfg(unix)]
fn set_owner_only(file: &std::fs::File, path: &Path) -> Result<()> {
    use std::os::unix::io::AsRawFd;

    let rc = unsafe { libc::fchmod(file.as_raw_fd(), 0o600) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error())
            .with_context(|| format!("fchmod 0600 {}", path.display()));
    }
    Ok(())
}

#[cfg(not(unix))]
fn open_nofollow_create_new(path: &Path) -> std::io::Result<std::fs::File> {
    Err(std::io::Error::other(format!(
        "PLAN_TAMPERED: unsupported platform for no-follow I/O at {}",
        path.display()
    )))
}

/// Store a plan as compact JSON using `create_new` (never overwrites). Returns
/// the computed plan id. The plan is normalized first (C3-R1); the normalized
/// value is hashed and serialized, never the caller's original. If a file with
/// the same id already exists, it is success only if its content is
/// byte-identical to the normalized candidate; otherwise an error is returned.
/// New plan files are `0600` on Unix.
pub fn store_plan(plan: &AutomationPlan) -> Result<String> {
    let normalized = normalize_plan(plan)?;
    ensure_automation_root()?;
    let id = plan_id(&normalized)?;
    let path = plan_path(&id)?;
    let bytes = serde_json::to_vec(&normalized).context("serialize plan")?;

    match open_nofollow_create_new(&path) {
        Ok(mut f) => {
            use std::io::Write;
            f.write_all(&bytes)
                .with_context(|| format!("write {}", path.display()))?;
            f.flush()
                .with_context(|| format!("flush {}", path.display()))?;
            #[cfg(unix)]
            {
                set_owner_only(&f, &path)?;
            }
            Ok(id)
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            // Same-id content is success only if byte-identical to the
            // normalized candidate. Read via no-follow to verify.
            let existing = read_nofollow(&path)?;
            if existing == bytes {
                Ok(id)
            } else {
                anyhow::bail!(
                    "plan file {} already exists with non-identical content; \
                     store_plan never overwrites",
                    path.display()
                );
            }
        }
        Err(e) => Err(map_plan_create_error(e, &path)),
    }
}

/// Add create-path context to ordinary I/O errors without obscuring a stable
/// plan-integrity classification emitted by a fail-closed platform branch.
fn map_plan_create_error(error: std::io::Error, path: &Path) -> anyhow::Error {
    let message = error.to_string();
    if message.starts_with("PLAN_TAMPERED:") {
        anyhow::anyhow!("{message}")
    } else {
        anyhow::Error::new(error).context(format!("create (O_NOFOLLOW) plan {}", path.display()))
    }
}

/// Read file bytes via no-follow open (C3-R1). On Unix uses `O_NOFOLLOW`;
/// on Windows fails closed.
fn read_nofollow(path: &Path) -> Result<Vec<u8>> {
    use std::io::Read;
    let mut file = open_nofollow_read(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .with_context(|| format!("read {}", path.display()))?;
    Ok(buf)
}

/// Load a plan by id, validating it against `now_unix_secs`. Returns errors
/// whose text begins with stable code labels: `PLAN_NOT_FOUND`, `PLAN_TAMPERED`,
/// or `PLAN_EXPIRED`. The plan is deserialized, normalized (C3-R1), and its
/// filename hash checked against the normalized plan. A semantically invalid
/// or noncanonical stored plan returns `PLAN_TAMPERED`, never a valid plan.
/// Rejects expired plans (`now_unix_secs >= expires_at_unix_secs`). Does not
/// delete expired plans and performs no network/model/client/config I/O.
pub fn load_plan(plan_id_str: &str, now_unix_secs: u64) -> Result<AutomationPlan> {
    if !is_valid_plan_id(plan_id_str) {
        anyhow::bail!("PLAN_NOT_FOUND: invalid plan id: {plan_id_str:?}");
    }
    let path = plan_path(plan_id_str)?;

    // No-follow read: on Unix the kernel refuses a symlink at open time
    // (O_NOFOLLOW). On Windows this fails closed.
    let raw = match read_nofollow(&path) {
        Ok(bytes) => bytes,
        Err(e) => {
            let msg = format!("{e}");
            if msg.contains("No such file")
                || msg.contains("PLAN_NOT_FOUND")
                || e.chain().any(|c| c.to_string().contains("No such file"))
            {
                anyhow::bail!("PLAN_NOT_FOUND: {}", path.display());
            }
            // Any other open/read error (including symlink refusal, permission
            // denied, or Windows unsupported) is treated as tampering.
            anyhow::bail!("PLAN_TAMPERED: cannot open/read {}: {e}", path.display());
        }
    };

    let plan: AutomationPlan = serde_json::from_slice(&raw).map_err(|e| {
        anyhow::anyhow!(
            "PLAN_TAMPERED: malformed plan JSON at {}: {e}",
            path.display()
        )
    })?;

    // Normalize the deserialized plan (C3-R1). A semantically invalid or
    // noncanonical stored plan returns PLAN_TAMPERED, never a valid plan.
    let normalized = normalize_plan(&plan).map_err(|e| {
        anyhow::anyhow!(
            "PLAN_TAMPERED: plan at {} failed normalization: {e}",
            path.display()
        )
    })?;

    // Hash/file-name mismatch -> tampered. The hash is computed on the
    // normalized plan, so a stored plan whose bytes hash to the file name
    // but whose semantics are invalid is caught by normalization above.
    let recomputed = plan_id(&normalized).context("PLAN_TAMPERED: recompute plan id")?;
    if recomputed != plan_id_str {
        anyhow::bail!(
            "PLAN_TAMPERED: plan id mismatch at {} (file name {}, recomputed {})",
            path.display(),
            plan_id_str,
            recomputed
        );
    }

    // Expiry: now >= expires -> expired. Does not delete the plan.
    if now_unix_secs >= normalized.expires_at_unix_secs {
        anyhow::bail!(
            "PLAN_EXPIRED: plan {} expired at {} (now {})",
            plan_id_str,
            normalized.expires_at_unix_secs,
            now_unix_secs
        );
    }

    Ok(normalized)
}

#[cfg(all(test, unix))]
mod tests {
    use super::set_owner_only;
    use std::fs::OpenOptions;
    use std::os::unix::fs::PermissionsExt;

    /// Regression for C3-R2: plan-file permissions must be applied through the
    /// already-open descriptor, not through a pathname an attacker can replace.
    #[test]
    fn owner_only_permissions_target_open_file_not_replaced_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("plan.json");
        let moved = dir.path().join("opened-plan.json");

        std::fs::write(&path, b"original").expect("write original");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644))
            .expect("set original mode");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .expect("open original");

        std::fs::rename(&path, &moved).expect("move opened file");
        std::fs::write(&path, b"replacement").expect("write replacement");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644))
            .expect("set replacement mode");

        set_owner_only(&file, &path).expect("chmod opened descriptor");

        assert_eq!(
            std::fs::metadata(&moved)
                .expect("opened metadata")
                .permissions()
                .mode()
                & 0o777,
            0o600,
            "the opened file must become owner-only"
        );
        assert_eq!(
            std::fs::metadata(&path)
                .expect("replacement metadata")
                .permissions()
                .mode()
                & 0o777,
            0o644,
            "a replacement at the old pathname must not be chmoded"
        );
    }
}

#[cfg(test)]
mod error_mapping_tests {
    use std::path::Path;

    use super::map_plan_create_error;

    #[test]
    fn create_error_preserves_plan_tampered_prefix() {
        let error = map_plan_create_error(
            std::io::Error::other("PLAN_TAMPERED: unsupported platform for no-follow I/O"),
            Path::new("plans/plan.json"),
        );

        assert!(
            error.to_string().starts_with("PLAN_TAMPERED:"),
            "classification must survive create-path context: {error}"
        );
    }
}

// Keep agent_contract import for potential future use by C4+.
#[allow(unused_imports)]
use agent_contract as _agent_contract;
