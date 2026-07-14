//! Read-only inspection of local nowdocs state (parent design Section 5.2).
//!
//! `collect_status` aggregates pure, offline-safe observations into one
//! snapshot for `nowdocs status`: platform, cache layout and sizes, pinned
//! model presence, installed docsets, the MCP contract, and the private
//! automation subtree. Every observation is strictly read-only: nothing here
//! calls `ensure_layout`, creates or deletes files, writes a writability
//! probe, loads a model, opens a Lance store, spawns a process, touches the
//! network, reads client configuration, cleans data, or follows symlinks.
//! Unreadable or corrupt entries degrade to zero/false observations instead
//! of failing the snapshot.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::agent_contract::{AgentEnvelope, AgentStatus, ResultCode};
use crate::cache::{self, CacheLayoutState, InstalledDocsetState};

/// Pinned embedding model id. Mirrors the private `embedder::DEFAULT_MODEL_ID`
/// (that module is outside this task's editable scope); status observes the
/// model's cache directory but never constructs or loads an embedder.
const PINNED_MODEL_ID: &str = "jinaai/jina-embeddings-v2-small-en";

/// Operating system and CPU architecture of this binary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformStatus {
    pub os: String,
    pub arch: String,
}

/// Read-only cache observation. Deliberately omits the absolute cache path:
/// status output must never carry paths.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheObservation {
    pub layout: CacheLayoutState,
    pub total_bytes: u64,
    pub installed_docsets: u64,
    pub staging_count: u64,
}

/// Pinned-model cache presence. Never loads or downloads the model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelObservation {
    pub present: bool,
}

/// One installed docset row: name plus the `InstalledDocsetState` label.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocsetObservation {
    pub name: String,
    pub state: String,
}

/// MCP contract constants advertised to agents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpObservation {
    pub protocol_version: String,
    pub transport: String,
    pub tools: Vec<String>,
}

/// Read-only observation of `<cache>/nowdocs/automation`. C2 never creates,
/// cleans, parses, or follows anything there, so `expired_count` is always 0;
/// C3 replaces these counts with real plan/operation lifecycle data.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutomationObservation {
    pub storage_present: bool,
    pub plan_count: u64,
    pub operation_count: u64,
    pub rollback_count: u64,
    pub expired_count: u64,
    pub total_bytes: u64,
}

/// Payload of `nowdocs status --json` (agent-contract schema v1). Future
/// versions may only add fields; removing or retyping a field requires a
/// schema version increase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusData {
    pub platform: PlatformStatus,
    pub cache: CacheObservation,
    pub model: ModelObservation,
    pub docsets: Vec<DocsetObservation>,
    pub mcp: McpObservation,
    pub automation: AutomationObservation,
}

/// Collect one offline, strictly read-only snapshot of local nowdocs state.
pub fn collect_status() -> StatusData {
    let layout = cache::observe_layout_state();

    // Symlink boundary (C2-R1): the cache tree is traversed only when the
    // root itself is a plain directory. An absent, regular-file, unreadable,
    // or symlinked root yields safe zero/false/empty observations, and no
    // helper that could follow the root is ever called.
    let root_is_real_dir = std::fs::symlink_metadata(cache::cache_root())
        .map(|meta| meta.is_dir())
        .unwrap_or(false);

    let (cache_observation, model_present, docsets, automation) = if root_is_real_dir {
        let db_dir = cache::cache_root().join("db");
        let docsets: Vec<DocsetObservation> = if real_dir_below_root(&db_dir) {
            cache::installed_docset_names()
        } else {
            Vec::new()
        }
        .into_iter()
        .map(|name| DocsetObservation {
            state: cache::check_docset_state_pure(&name).label().to_string(),
            name,
        })
        .collect();
        (
            observe_cache_pure(layout, docsets.len() as u64),
            pinned_model_present(),
            docsets,
            observe_automation(),
        )
    } else {
        (
            CacheObservation {
                layout,
                total_bytes: 0,
                installed_docsets: 0,
                staging_count: 0,
            },
            false,
            Vec::new(),
            AutomationObservation::default(),
        )
    };

    StatusData {
        platform: PlatformStatus {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
        },
        cache: cache_observation,
        model: ModelObservation {
            present: model_present,
        },
        docsets,
        mcp: McpObservation {
            protocol_version: crate::mcp::PROTOCOL_VERSION.to_string(),
            transport: "stdio_ndjson".to_string(),
            tools: crate::mcp::MCP_TOOL_NAMES
                .iter()
                .map(|s| s.to_string())
                .collect(),
        },
        automation,
    }
}

/// Pure cache size/count observation for the status path (C2-R1).
///
/// Unlike legacy [`cache::cache_status`], this never follows symlinks: each
/// component directory (`db`, `models`, `staging`, `rollback`) is verified
/// with `symlink_metadata` before it is walked, so a symlinked component
/// contributes zero bytes and zero counts. Unreadable entries degrade to
/// zero, matching the rest of the inspector.
fn observe_cache_pure(layout: CacheLayoutState, installed_docsets: u64) -> CacheObservation {
    let root = cache::cache_root();
    let component_bytes = |name: &str| {
        let dir = root.join(name);
        if real_dir_below_root(&dir) {
            total_bytes_without_symlinks(&dir)
        } else {
            0
        }
    };
    let staging = cache::staging_root();
    let staging_count = if real_dir_below_root(&staging) {
        cache::list_staging_dirs().unwrap_or_default().len() as u64
    } else {
        0
    };
    CacheObservation {
        layout,
        total_bytes: component_bytes("db")
            + component_bytes("models")
            + component_bytes("staging")
            + component_bytes("rollback"),
        installed_docsets,
        staging_count,
    }
}

/// True when `path` is a plain directory reachable from the cache root
/// without traversing any symlink: every component below the root, including
/// `path` itself, must be a plain directory per `symlink_metadata`. Callers
/// must already have established that the root itself is a plain directory
/// (see [`collect_status`]).
fn real_dir_below_root(path: &Path) -> bool {
    let root = cache::cache_root();
    let Ok(rel) = path.strip_prefix(&root) else {
        return false;
    };
    let mut current = root;
    for component in rel.components() {
        current.push(component.as_os_str());
        match std::fs::symlink_metadata(&current) {
            Ok(meta) if meta.is_dir() => {}
            _ => return false,
        }
    }
    true
}

/// Wrap a status snapshot in the schema-v1 agent envelope. `status` always
/// exits 0: degraded observations (uninitialized/mismatched/unreadable cache,
/// missing model, non-healthy docsets) surface as `status=warning` with
/// `code=ready`, never as a process error.
pub fn status_envelope(data: &StatusData) -> AgentEnvelope<StatusData> {
    let degraded = data.cache.layout != CacheLayoutState::Ready
        || !data.model.present
        || data
            .docsets
            .iter()
            .any(|d| d.state != InstalledDocsetState::Healthy.label());
    let status = if degraded {
        AgentStatus::Warning
    } else {
        AgentStatus::Ok
    };
    AgentEnvelope::new(
        "status",
        status,
        ResultCode::Ready,
        &status_summary(data),
        data.clone(),
    )
}

fn status_summary(data: &StatusData) -> String {
    let docsets = data.docsets.len();
    match data.cache.layout {
        CacheLayoutState::Ready => {
            if data.model.present {
                format!("nowdocs is ready: {docsets} docset(s) installed, embedding model cached")
            } else {
                format!(
                    "nowdocs cache is ready but the embedding model is not cached: {docsets} docset(s) installed"
                )
            }
        }
        layout => format!(
            "nowdocs cache layout is {}; no state was changed",
            layout.as_str()
        ),
    }
}

/// Concise English rendering of the status snapshot, derived from the same
/// `StatusData` as the JSON output. Contains no absolute paths or environment
/// values; human text is not part of schema v1 and may change for clarity.
pub fn format_status_human(data: &StatusData) -> String {
    let mut out = String::new();
    out.push_str("nowdocs status\n");
    out.push_str(&format!("cache layout: {}\n", data.cache.layout.as_str()));
    out.push_str(&format!(
        "cache: {} bytes, {} docset(s), {} staging dir(s)\n",
        data.cache.total_bytes, data.cache.installed_docsets, data.cache.staging_count
    ));
    out.push_str(if data.model.present {
        "model: cached\n"
    } else {
        "model: not cached\n"
    });
    out.push_str(&format!("docsets: {} installed\n", data.docsets.len()));
    for docset in &data.docsets {
        out.push_str(&format!("  {}: {}\n", docset.name, docset.state));
    }
    out.push_str(&format!(
        "MCP: protocol {} over {}; tools: {}\n",
        data.mcp.protocol_version,
        data.mcp.transport,
        data.mcp.tools.join(", ")
    ));
    let a = &data.automation;
    if a.storage_present {
        out.push_str(&format!(
            "automation: {} plan(s), {} operation(s), {} rollback(s), {} expired, {} bytes\n",
            a.plan_count, a.operation_count, a.rollback_count, a.expired_count, a.total_bytes
        ));
    } else {
        out.push_str("automation: no storage present\n");
    }
    out
}

/// True only when the pinned model's cache directory exists and contains at
/// least one regular file. Never constructs or loads an embedder; never
/// follows symlinks — a symlinked `models/` component or model directory
/// reports absent (C2-R1).
fn pinned_model_present() -> bool {
    let dir = cache::model_path(PINNED_MODEL_ID);
    real_dir_below_root(&dir) && contains_regular_file(&dir)
}

/// Whether `dir` contains any regular file, walking without symlink traversal
/// and tolerating unreadable entries.
fn contains_regular_file(dir: &Path) -> bool {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            // DirEntry::metadata does not follow symlinks.
            let Ok(meta) = entry.metadata() else {
                continue;
            };
            if meta.is_file() {
                return true;
            }
            if meta.is_dir() {
                stack.push(entry.path());
            }
        }
    }
    false
}

/// Observe `<cache>/nowdocs/automation` without creating, cleaning, parsing,
/// or following anything. A missing or non-directory subtree reports
/// zero/false; C3 replaces these counts.
fn observe_automation() -> AutomationObservation {
    let root = cache::cache_root().join("automation");
    // A symlinked automation directory (or any symlinked component on the way
    // down) reports storage absent rather than being followed (C2-R1).
    if !real_dir_below_root(&root) {
        return AutomationObservation::default();
    }
    AutomationObservation {
        storage_present: true,
        plan_count: count_immediate_regular_files(&root.join("plans")),
        operation_count: count_immediate_regular_files(&root.join("operations")),
        rollback_count: count_immediate_regular_files(&root.join("rollback")),
        // C2 never parses plans, so expiration cannot be evaluated yet.
        expired_count: 0,
        total_bytes: total_bytes_without_symlinks(&root),
    }
}

/// Count immediate regular files in `dir` (no symlink following, tolerant of
/// unreadable entries). A symlinked `dir` itself is never followed (C2-R1).
fn count_immediate_regular_files(dir: &Path) -> u64 {
    match std::fs::symlink_metadata(dir) {
        Ok(meta) if meta.is_dir() => {}
        _ => return 0,
    }
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.metadata().map(|m| m.is_file()).unwrap_or(false) {
                count += 1;
            }
        }
    }
    count
}

/// Recursively total regular-file bytes under `root` without symlink
/// traversal; unreadable entries contribute 0.
fn total_bytes_without_symlinks(root: &Path) -> u64 {
    let mut total = 0;
    let mut stack = vec![root.to_path_buf()];
    while let Some(current) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(meta) = entry.metadata() else {
                continue;
            };
            if meta.is_file() {
                total += meta.len();
            } else if meta.is_dir() {
                stack.push(entry.path());
            }
        }
    }
    total
}
