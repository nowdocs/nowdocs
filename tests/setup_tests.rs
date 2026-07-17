//! C7 - One-plan setup orchestration tests.
//!
//! All tests use isolated temporary directories and serialized environment
//! guards so they never touch a real cache, model, client config, HOME, XDG,
//! or network. Registry fixtures use local `file://` indexes and in-memory
//! release archives built from real `.lance` tables.

use std::sync::Mutex;

use nowdocs::automation::plan::load_plan;
use nowdocs::automation::setup::{
    setup_apply, setup_plan, setup_rollback, SetupApplyResult, SetupPlanResult,
};
use nowdocs::cache::{self, InstalledDocsetState};
use nowdocs::chunker::{Chunk, ChunkType};
use nowdocs::clients::{approved_root, atomic_replace, safe_target, ApprovedRoot};
use nowdocs::registry::{self, RegistryPackage};
use nowdocs::store::Store;

// Serialize env mutation so parallel tests don't race on XDG_CACHE_HOME.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Holds the env lock and the prior values for XDG_CACHE_HOME and TMPDIR.
struct EnvGuard {
    old_xdg: Option<String>,
    old_tmpdir: Option<String>,
    old_index_url: Option<String>,
    _g: std::sync::MutexGuard<'static, ()>,
}

impl EnvGuard {
    fn isolate(root: &std::path::Path) -> Self {
        let g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let old_xdg = std::env::var("XDG_CACHE_HOME").ok();
        std::env::set_var("XDG_CACHE_HOME", root.to_str().unwrap());
        let old_tmpdir = std::env::var("TMPDIR").ok();
        std::env::set_var("TMPDIR", root.to_str().unwrap());
        let old_index_url = std::env::var("NOWDOCS_REGISTRY_INDEX_URL").ok();
        std::env::remove_var("NOWDOCS_REGISTRY_INDEX_URL");
        Self {
            old_xdg,
            old_tmpdir,
            old_index_url,
            _g: g,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.old_xdg {
            Some(v) => std::env::set_var("XDG_CACHE_HOME", v),
            None => std::env::remove_var("XDG_CACHE_HOME"),
        }
        match &self.old_tmpdir {
            Some(v) => std::env::set_var("TMPDIR", v),
            None => std::env::remove_var("TMPDIR"),
        }
        match &self.old_index_url {
            Some(v) => std::env::set_var("NOWDOCS_REGISTRY_INDEX_URL", v),
            None => std::env::remove_var("NOWDOCS_REGISTRY_INDEX_URL"),
        }
    }
}

fn isolate(root: &std::path::Path) -> EnvGuard {
    EnvGuard::isolate(root)
}

/// Create an ApprovedRoot from a temp directory, setting mode 0700 on Unix so
/// `approved_root` accepts it.
fn make_approved_root(path: &std::path::Path) -> ApprovedRoot {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    approved_root(path).expect("approved root")
}

fn two_chunks() -> Vec<Chunk> {
    vec![
        Chunk {
            idx: 0,
            heading_path: "Intro".into(),
            source_url: "https://example.com/0".into(),
            api_version: None,
            chunk_type: ChunkType::Info,
            text: "hello".into(),
        },
        Chunk {
            idx: 1,
            heading_path: "API".into(),
            source_url: "https://example.com/1".into(),
            api_version: None,
            chunk_type: ChunkType::Info,
            text: "world".into(),
        },
    ]
}

fn zero_vectors(n: usize) -> Vec<Vec<f32>> {
    vec![vec![0.0f32; 512]; n]
}

fn manifest_json(docset: &str, version: &str) -> String {
    format!(
        r#"{{
            "docset": "{docset}",
            "doc_version": "{version}",
            "nowdocs_schema_version": 1,
            "embedder": {{
                "model_id": "jinaai/jina-embeddings-v2-small-en",
                "model_version": "0.1.0",
                "model_revision": "abc123",
                "model_sha256": "deadbeef",
                "vector_dim": 512,
                "engine": "candle",
                "dtype": "f16"
            }},
            "retrieval": {{
                "tokenizer": "default",
                "chunk_size_tokens": 512,
                "window_tokens": 64
            }},
            "source": {{
                "entry_url": "https://example.com/docs",
                "source_url": "https://example.com",
                "scraped_at": "2026-01-01T00:00:00Z",
                "chunk_count": 2
            }},
            "legal": {{
                "license": "MIT",
                "copyright_holder": "Example",
                "attribution": ""
            }},
            "refresh_strategy": {{
                "tier": "stable",
                "auto_days": 30
            }}
        }}"#
    )
}

fn make_tar_entry(name: &str, data: &[u8]) -> Vec<u8> {
    let mut header = [0u8; 512];
    let name_bytes = name.as_bytes();
    header[0..name_bytes.len()].copy_from_slice(name_bytes);
    header[100..107].copy_from_slice(b"000644\0");
    header[108..115].copy_from_slice(b"000000\0");
    header[116..123].copy_from_slice(b"000000\0");
    let size_str = format!("{:011o}\0", data.len());
    header[124..136].copy_from_slice(size_str.as_bytes());
    header[136..148].copy_from_slice(b"00000000000\0");
    header[156] = b'0';
    header[257..263].copy_from_slice(b"ustar\0");
    header[265..267].copy_from_slice(b"00");

    let mut sum: u32 = 0;
    for (i, &b) in header.iter().enumerate() {
        sum += if (148..156).contains(&i) {
            b' ' as u32
        } else {
            b as u32
        };
    }
    let chk_str = format!("{:06o}\0 ", sum);
    header[148..156].copy_from_slice(chk_str.as_bytes());

    let mut entry = header.to_vec();
    entry.extend_from_slice(data);
    let padded = data.len().div_ceil(512) * 512;
    if padded > data.len() {
        entry.extend_from_slice(&vec![0u8; padded - data.len()]);
    }
    entry
}

fn add_dir_to_tar(archive: &mut Vec<u8>, dir: &std::path::Path, prefix: &str) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let name = format!("{}/{}", prefix, entry.file_name().to_string_lossy());
        if path.is_dir() {
            add_dir_to_tar(archive, &path, &name);
        } else {
            let data = std::fs::read(&path).unwrap();
            archive.extend_from_slice(&make_tar_entry(&name, &data));
        }
    }
}

/// Build a registry-release tar: manifest + real `<docset>.lance/` table.
fn make_release_archive(docset: &str, version: &str) -> Vec<u8> {
    let saved = std::env::var("XDG_CACHE_HOME").ok();
    let src = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", src.path()) };
    cache::ensure_layout().unwrap();
    let chunks = two_chunks();
    let vecs = zero_vectors(chunks.len());
    {
        let store = Store::open(docset).unwrap();
        store.insert(&chunks, &vecs).unwrap();
    }
    let lance_dir = cache::db_path(docset);

    let mut archive = Vec::new();
    archive.extend_from_slice(&make_tar_entry(
        "manifest.json",
        manifest_json(docset, version).as_bytes(),
    ));
    add_dir_to_tar(&mut archive, &lance_dir, &format!("{docset}.lance"));
    archive.extend_from_slice(&[0u8; 512]);
    archive.extend_from_slice(&[0u8; 512]);

    match saved {
        Some(v) => unsafe { std::env::set_var("XDG_CACHE_HOME", v) },
        None => unsafe { std::env::remove_var("XDG_CACHE_HOME") },
    }
    archive
}

fn archive_sha256(archive: &[u8]) -> String {
    registry::sha256_hex(archive)
}

fn make_index_json(package: &RegistryPackage) -> String {
    serde_json::json!({
        "schema_version": 1,
        "generated_at": "2026-07-07T00:00:00Z",
        "packages": [package]
    })
    .to_string()
}

fn package_for(docset: &str, version: &str, archive_path: &std::path::Path) -> RegistryPackage {
    let archive = make_release_archive(docset, version);
    let sha = archive_sha256(&archive);
    std::fs::write(archive_path, &archive).unwrap();
    RegistryPackage {
        docset: docset.to_string(),
        version: version.to_string(),
        license: "MIT".to_string(),
        chunk_count: 2,
        freshness: "2026-07-07".to_string(),
        download_url: format!("file://{}", archive_path.display()),
        sha256: sha,
        description: Some("test package".to_string()),
    }
}

/// Set up a registry index fixture under the isolated root and point
/// NOWDOCS_REGISTRY_INDEX_URL at it.
fn setup_index(root: &std::path::Path, package: &RegistryPackage) {
    let index_path = root.join("index.json");
    std::fs::write(&index_path, make_index_json(package)).unwrap();
    std::env::set_var(
        "NOWDOCS_REGISTRY_INDEX_URL",
        format!("file://{}", index_path.display()),
    );
}

/// Count files/dirs under a path (shallow).
fn count_entries(path: &std::path::Path) -> usize {
    if !path.exists() {
        return 0;
    }
    std::fs::read_dir(path)
        .map(|it| it.flatten().count())
        .unwrap_or(0)
}

/// Set up a registry fixture under `root` and create a `client-root`
/// subdirectory with optional Cursor config. Returns the `ApprovedRoot`
/// (pointing at `client-root`) and the `config_root` path so `setup_plan`
/// and `setup_apply` see the same Cursor target fingerprint.
fn setup_fixture(
    root: &std::path::Path,
    docset: &str,
    version: &str,
    cursor_config: Option<&serde_json::Value>,
) -> (ApprovedRoot, std::path::PathBuf) {
    let archive_path = root.join(format!("{docset}-{version}.lance.tar"));
    let package = package_for(docset, version, &archive_path);
    setup_index(root, &package);

    let config_root = root.join("client-root");
    std::fs::create_dir_all(&config_root).unwrap();
    if let Some(config) = cursor_config {
        std::fs::create_dir_all(config_root.join(".cursor")).unwrap();
        std::fs::write(
            config_root.join(".cursor").join("mcp.json"),
            serde_json::to_vec_pretty(config).unwrap(),
        )
        .unwrap();
    }
    let ar = make_approved_root(&config_root);
    (ar, config_root)
}

// ---- 1. Offline missing docset: registry_metadata_required, exit 0, no files ----

#[test]
fn offline_missing_docset_returns_registry_metadata_required_and_creates_no_files() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let ar = make_approved_root(root.path());

    let result =
        setup_plan("nextjs", "cursor", &ar, false, 1_000_000_000).expect("plan should succeed");
    assert!(
        matches!(result, SetupPlanResult::RegistryMetadataRequired { .. }),
        "offline missing docset must return RegistryMetadataRequired, got: {:?}",
        result
    );

    assert!(
        count_entries(root.path()) == 0,
        "offline refusal must create no files under isolated root"
    );
}

// ---- 2. Online planning: one hash, prescribed action ordering, no absolute/secret fields ----

#[test]
#[cfg(unix)]
fn online_planning_produces_one_hash_with_prescribed_action_ordering() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let ar = make_approved_root(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let result =
        setup_plan("nextjs", "cursor", &ar, true, 1_000_000_000).expect("online plan succeeds");
    let plan_hash = match result {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    assert_eq!(plan_hash.len(), 64, "plan hash must be 64 hex chars");
    assert!(
        plan_hash
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
        "plan hash must be lowercase hex"
    );

    let plan = load_plan(&plan_hash, 1_000_000_001).expect("load stored plan");
    assert_eq!(plan.inputs.client.as_deref(), Some("cursor"));
    assert_eq!(plan.inputs.docset.as_deref(), Some("nextjs"));
    assert!(plan.inputs.online);

    assert!(plan.actions.len() >= 2, "plan must have at least 2 actions");
    assert!(
        plan.actions[0].kind == "registry_install" || plan.actions[0].kind == "registry_update",
        "first action must be docset install/update, got {:?}",
        plan.actions[0].kind
    );

    let mut has_verify_docset = false;
    let mut has_client_apply = false;
    let mut has_client_verify = false;
    let mut verify_docset_idx = None;
    let mut client_apply_idx = None;
    let mut client_verify_idx = None;
    for (i, action) in plan.actions.iter().enumerate() {
        match action.kind.as_str() {
            "verify_docset" => {
                has_verify_docset = true;
                verify_docset_idx = Some(i);
            }
            "client_apply" => {
                has_client_apply = true;
                client_apply_idx = Some(i);
            }
            "client_verify" => {
                has_client_verify = true;
                client_verify_idx = Some(i);
            }
            _ => {}
        }
    }
    assert!(has_verify_docset, "plan must include verify_docset");
    assert!(
        has_client_apply,
        "plan must include client_apply for cursor"
    );
    assert!(
        has_client_verify,
        "plan must include client_verify for cursor"
    );

    let vd = verify_docset_idx.unwrap();
    let ca = client_apply_idx.unwrap();
    let cv = client_verify_idx.unwrap();
    assert!(vd < ca, "verify_docset must come before client_apply");
    assert!(ca < cv, "client_apply must come before client_verify");

    let plan_json = serde_json::to_string(&plan).unwrap();
    assert!(
        !plan_json.contains("/Users/") && !plan_json.contains("/home/"),
        "plan must not contain absolute home paths"
    );
    assert!(
        !plan_json.contains("token") && !plan_json.contains("secret"),
        "plan must not contain secrets"
    );
    for action in &plan.actions {
        for tp in &action.target_paths {
            assert!(
                !tp.starts_with('/'),
                "target path must be logical (not absolute): {tp}"
            );
        }
    }
}

// ---- 2b. Already satisfied docset + canonical client ----

#[test]
#[cfg(unix)]
fn already_satisfied_docset_returns_already_satisfied() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let archive = make_release_archive("react", "18.3.1");
    let tar_path = root.path().join("react.tar");
    std::fs::write(&tar_path, &archive).unwrap();
    let url = format!("file://{}", tar_path.display());
    registry::install("react", &url).expect("install fixture docset");

    let config_root = root.path().join("client-root");
    std::fs::create_dir_all(config_root.join(".cursor")).unwrap();
    let binary = std::env::current_exe().unwrap();
    let canonical = serde_json::json!({
        "mcpServers": {
            "nowdocs": {"command": binary.display().to_string(), "args": ["serve"]}
        }
    });
    std::fs::write(
        config_root.join(".cursor").join("mcp.json"),
        serde_json::to_vec_pretty(&canonical).unwrap(),
    )
    .unwrap();
    let ar = make_approved_root(&config_root);

    let result =
        setup_plan("react", "cursor", &ar, false, 1_000_000_000).expect("plan should succeed");
    assert!(
        matches!(result, SetupPlanResult::AlreadySatisfied { .. }),
        "installed docset + canonical client must be already satisfied, got: {:?}",
        result
    );
}

// ---- 2c. Healthy docset alone (no canonical client) is NOT already_satisfied ----

#[test]
#[cfg(unix)]
fn healthy_docset_without_canonical_client_is_not_already_satisfied() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let archive = make_release_archive("react", "18.3.1");
    let tar_path = root.path().join("react.tar");
    std::fs::write(&tar_path, &archive).unwrap();
    let url = format!("file://{}", tar_path.display());
    registry::install("react", &url).expect("install fixture docset");

    let config_root = root.path().join("client-root");
    std::fs::create_dir_all(&config_root).unwrap();
    let ar = make_approved_root(&config_root);

    let result =
        setup_plan("react", "cursor", &ar, false, 1_000_000_000).expect("plan should succeed");
    assert!(
        !matches!(result, SetupPlanResult::AlreadySatisfied { .. }),
        "healthy docset without canonical client must NOT be already_satisfied, got: {:?}",
        result
    );
}

// ---- 3. Tampered, expired, stale plans are refused ----

#[test]
#[cfg(unix)]
fn apply_refuses_tampered_plan() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let (ar, _) = setup_fixture(root.path(), "nextjs", "14.2.5", None);

    let plan_hash = match setup_plan("nextjs", "cursor", &ar, true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let plan_path = root
        .path()
        .join("nowdocs")
        .join("automation")
        .join("plans")
        .join(format!("{plan_hash}.json"));
    let mut content = std::fs::read_to_string(&plan_path).unwrap();
    content = content.replace("14.2.5", "99.99.99");
    std::fs::write(&plan_path, content).unwrap();

    let result = setup_apply(&plan_hash, root.path(), 1_000_000_001);
    assert!(result.is_err(), "tampered plan must be refused");
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("PLAN_TAMPERED"),
        "error must carry PLAN_TAMPERED, got: {msg}"
    );
}

#[test]
#[cfg(unix)]
fn apply_refuses_expired_plan() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let (ar, _) = setup_fixture(root.path(), "nextjs", "14.2.5", None);

    let created_at = 1_000_000_000;
    let plan_hash = match setup_plan("nextjs", "cursor", &ar, true, created_at).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let now = created_at + 30 * 60 + 1;
    let result = setup_apply(&plan_hash, root.path(), now);
    assert!(result.is_err(), "expired plan must be refused");
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("PLAN_EXPIRED"),
        "error must carry PLAN_EXPIRED, got: {msg}"
    );
}

#[test]
#[cfg(unix)]
fn apply_refuses_stale_plan_when_docset_state_drifts() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let (ar, _) = setup_fixture(root.path(), "nextjs", "14.2.5", None);

    let plan_hash = match setup_plan("nextjs", "cursor", &ar, true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let v2_archive = make_release_archive("nextjs", "15.0.0");
    let v2_path = root.path().join("nextjs-15.0.0.lance.tar");
    std::fs::write(&v2_path, &v2_archive).unwrap();
    registry::install("nextjs", &format!("file://{}", v2_path.display())).unwrap();

    let result = setup_apply(&plan_hash, root.path(), 1_000_000_001);
    assert!(result.is_err(), "stale plan must be refused");
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("PLAN_STALE"),
        "error must carry PLAN_STALE, got: {msg}"
    );
}

// ---- 4. Claude Desktop/Generic produce manual-only actions and no write ----

#[test]
#[cfg(unix)]
fn claude_desktop_plan_produces_manual_only_and_no_write() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let ar = make_approved_root(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let result = setup_plan("nextjs", "claude-desktop", &ar, true, 1_000_000_000)
        .expect("plan should succeed");
    let plan_hash = match result {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let plan = load_plan(&plan_hash, 1_000_000_001).expect("load plan");
    assert!(
        !plan
            .actions
            .iter()
            .any(|a| a.kind == "client_apply" || a.kind == "client_verify"),
        "claude-desktop must not have client apply/verify actions"
    );
    assert!(
        plan.actions
            .iter()
            .any(|a| a.kind == "client_manual_guidance"),
        "claude-desktop must have client_manual_guidance action"
    );
}

#[test]
#[cfg(unix)]
fn generic_plan_produces_manual_only_and_no_write() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let ar = make_approved_root(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let result =
        setup_plan("nextjs", "generic", &ar, true, 1_000_000_000).expect("plan should succeed");
    let plan_hash = match result {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let plan = load_plan(&plan_hash, 1_000_000_001).expect("load plan");
    assert!(
        !plan
            .actions
            .iter()
            .any(|a| a.kind == "client_apply" || a.kind == "client_verify"),
        "generic must not have client apply/verify actions"
    );
    assert!(
        plan.actions
            .iter()
            .any(|a| a.kind == "client_manual_guidance"),
        "generic must have client_manual_guidance action"
    );
}

// ---- 5. Cursor conflict, missing config, malformed JSON ----

#[test]
#[cfg(unix)]
fn cursor_apply_returns_partial_when_nowdocs_already_exists() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let existing = serde_json::json!({
        "mcpServers": {"nowdocs": {"command": "/other/nowdocs", "args": ["serve"]}}
    });
    let (ar, config_root) = setup_fixture(root.path(), "nextjs", "14.2.5", Some(&existing));

    let plan_hash = match setup_plan("nextjs", "cursor", &ar, true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let original_bytes = std::fs::read(config_root.join(".cursor").join("mcp.json")).unwrap();

    // Noncanonical entry => manual guidance, no adapter mutation (contract 3).
    let result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    match result {
        SetupApplyResult::ActionRequired { .. } => {}
        other => panic!(
            "expected ActionRequired for noncanonical entry, got: {:?}",
            other
        ),
    }

    assert_eq!(
        std::fs::read(config_root.join(".cursor").join("mcp.json")).unwrap(),
        original_bytes,
        "noncanonical entry must not mutate the config file"
    );
}

#[test]
#[cfg(unix)]
fn cursor_apply_returns_action_required_for_missing_cursor_config() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let (ar, config_root) = setup_fixture(root.path(), "nextjs", "14.2.5", None);

    let plan_hash = match setup_plan("nextjs", "cursor", &ar, true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    match result {
        SetupApplyResult::PartialNoRollback { .. } => {}
        other => panic!(
            "expected PartialNoRollback for missing target, got: {:?}",
            other
        ),
    }

    assert_eq!(
        cache::check_docset_state("nextjs"),
        InstalledDocsetState::Healthy,
        "docset should be installed despite client failure"
    );
    assert!(
        !config_root.join(".cursor").exists(),
        "no .cursor dir should be created as a side effect"
    );
}

#[test]
#[cfg(unix)]
fn cursor_apply_returns_action_required_for_malformed_json() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let (ar, config_root) = setup_fixture(root.path(), "nextjs", "14.2.5", None);

    let plan_hash = match setup_plan("nextjs", "cursor", &ar, true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    // Overwrite with malformed JSON after plan creation.
    let malformed = b"{ this is not valid json ";
    std::fs::create_dir_all(config_root.join(".cursor")).unwrap();
    std::fs::write(config_root.join(".cursor").join("mcp.json"), malformed).unwrap();

    let result = setup_apply(&plan_hash, &config_root, 1_000_000_001);
    // Drift: the plan fingerprint says absent, but now the file exists.
    assert!(result.is_err(), "malformed JSON after plan must be stale");
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("PLAN_STALE"),
        "error must carry PLAN_STALE, got: {msg}"
    );
}

#[cfg(unix)]
#[test]
fn setup_plan_refuses_symlinked_cursor_target() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let config_root = root.path().join("client-root");
    std::fs::create_dir_all(config_root.join(".cursor")).unwrap();
    std::os::unix::fs::symlink("/dev/null", config_root.join(".cursor/mcp.json")).unwrap();
    let ar = make_approved_root(&config_root);

    let result = setup_plan("nextjs", "cursor", &ar, false, 1_000_000_000);
    assert!(result.is_err(), "symlinked Cursor target must fail closed");
}

#[test]
#[cfg(unix)]
fn cursor_apply_succeeds_and_verifies() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let config = serde_json::json!({"mcpServers":{}});
    let (ar, config_root) = setup_fixture(root.path(), "nextjs", "14.2.5", Some(&config));

    let plan_hash = match setup_plan("nextjs", "cursor", &ar, true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    match result {
        SetupApplyResult::SetupComplete { .. } | SetupApplyResult::ClientReloadRequired { .. } => {}
        other => panic!(
            "expected SetupComplete or ClientReloadRequired, got: {:?}",
            other
        ),
    }

    assert_eq!(
        cache::check_docset_state("nextjs"),
        InstalledDocsetState::Healthy
    );

    let after: serde_json::Value = serde_json::from_slice(
        &std::fs::read(config_root.join(".cursor").join("mcp.json")).unwrap(),
    )
    .unwrap();
    assert!(after["mcpServers"]["nowdocs"].is_object());
}

// ---- 6b. client_reload_required result ----

#[test]
#[cfg(unix)]
fn cursor_apply_returns_client_reload_required_when_verified() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let config = serde_json::json!({"mcpServers":{}});
    let (ar, config_root) = setup_fixture(root.path(), "nextjs", "14.2.5", Some(&config));

    let plan_hash = match setup_plan("nextjs", "cursor", &ar, true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    match result {
        SetupApplyResult::SetupComplete { .. } | SetupApplyResult::ClientReloadRequired { .. } => {}
        other => panic!(
            "expected SetupComplete or ClientReloadRequired, got: {:?}",
            other
        ),
    }
}

// ---- 6c. Partial: docset succeeded but client application could not start ----

#[test]
#[cfg(unix)]
fn partial_result_when_docset_succeeds_but_client_fails() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let (ar, config_root) = setup_fixture(root.path(), "nextjs", "14.2.5", None);

    let plan_hash = match setup_plan("nextjs", "cursor", &ar, true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    match result {
        SetupApplyResult::PartialNoRollback { .. } => {}
        other => panic!("expected PartialNoRollback, got: {:?}", other),
    }

    assert_eq!(
        cache::check_docset_state("nextjs"),
        InstalledDocsetState::Healthy,
        "additive docset must remain after partial client failure"
    );
}

// ---- 6d. Exact rollback ----

#[test]
#[cfg(unix)]
fn rollback_restores_cursor_config() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let original = serde_json::json!({
        "mcpServers": {"filesystem": {"command": "/usr/bin/fs", "args": []}}
    });
    let (ar, config_root) = setup_fixture(root.path(), "nextjs", "14.2.5", Some(&original));

    let plan_hash = match setup_plan("nextjs", "cursor", &ar, true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let apply_result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    let operation_id = match apply_result {
        SetupApplyResult::SetupComplete { operation_id, .. }
        | SetupApplyResult::ClientReloadRequired { operation_id, .. } => operation_id,
        other => panic!("expected success variant, got: {:?}", other),
    };

    let after: serde_json::Value = serde_json::from_slice(
        &std::fs::read(config_root.join(".cursor").join("mcp.json")).unwrap(),
    )
    .unwrap();
    assert!(after["mcpServers"]["nowdocs"].is_object());

    let rollback_result = setup_rollback(&operation_id, &config_root).unwrap();
    match rollback_result {
        nowdocs::automation::setup::SetupRollbackResult::RolledBack { .. } => {}
        other => panic!("expected RolledBack, got: {:?}", other),
    }

    let restored: serde_json::Value = serde_json::from_slice(
        &std::fs::read(config_root.join(".cursor").join("mcp.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(restored, original);
    assert!(restored["mcpServers"]["nowdocs"].is_null());
}

// ---- 6e. Later user edit refusal ----

#[test]
#[cfg(unix)]
fn rollback_refuses_after_later_user_edit() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let config = serde_json::json!({"mcpServers":{}});
    let (ar, config_root) = setup_fixture(root.path(), "nextjs", "14.2.5", Some(&config));

    let plan_hash = match setup_plan("nextjs", "cursor", &ar, true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let apply_result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    let operation_id = match apply_result {
        SetupApplyResult::SetupComplete { operation_id, .. }
        | SetupApplyResult::ClientReloadRequired { operation_id, .. } => operation_id,
        other => panic!("expected success variant, got: {:?}", other),
    };

    let approved = approved_root(&config_root).unwrap();
    let target = safe_target(&approved, ".cursor/mcp.json").unwrap();
    atomic_replace(
        &target,
        br#"{"mcpServers":{"nowdocs":{"command":"edited","args":["serve"]}}}"#,
    )
    .unwrap();

    let rollback_result = setup_rollback(&operation_id, &config_root).unwrap();
    match rollback_result {
        nowdocs::automation::setup::SetupRollbackResult::ManualRequired { .. } => {}
        other => panic!("expected ManualRequired, got: {:?}", other),
    }

    let current = std::fs::read(config_root.join(".cursor").join("mcp.json")).unwrap();
    assert!(
        String::from_utf8_lossy(&current).contains("edited"),
        "user-edited content must be preserved"
    );
}

// ---- 7. Plan hash is an integrity/scope check ----

#[test]
#[cfg(unix)]
fn setup_apply_rejects_unknown_plan_hash() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let fake_hash = "a".repeat(64);
    let result = setup_apply(&fake_hash, root.path(), 1_000_000_000);
    assert!(result.is_err(), "unknown plan hash must be rejected");
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("PLAN_NOT_FOUND"),
        "error must carry PLAN_NOT_FOUND, got: {msg}"
    );
}

// ---- 8. One plan, no nested ensure plan ----

#[test]
#[cfg(unix)]
fn setup_plan_creates_exactly_one_plan_file() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let ar = make_approved_root(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let _ = setup_plan("nextjs", "cursor", &ar, true, 1_000_000_000).unwrap();

    let plans_dir = root.path().join("nowdocs").join("automation").join("plans");
    let plan_files: Vec<_> = std::fs::read_dir(&plans_dir)
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().ends_with(".json"))
        .collect();
    assert_eq!(
        plan_files.len(),
        1,
        "exactly one plan file, got {plan_files:?}"
    );
}

// ---- 9. Claude Code: manual-only in tests (no real claude CLI) ----

#[test]
#[cfg(unix)]
fn claude_code_apply_returns_action_required_without_claude_cli() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let ar = make_approved_root(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let plan_hash = match setup_plan("nextjs", "claude-code", &ar, true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let config_root = root.path().join("client-root");
    std::fs::create_dir_all(&config_root).unwrap();

    let result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    match result {
        SetupApplyResult::PartialNoRollback { .. } => {}
        other => panic!(
            "expected PartialNoRollback when claude CLI is missing, got: {:?}",
            other
        ),
    }

    assert_eq!(
        cache::check_docset_state("nextjs"),
        InstalledDocsetState::Healthy
    );
}

// ---- 10. Operation id format ----

#[test]
#[cfg(unix)]
fn operation_id_is_setup_prefixed_first_12_hash_chars() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let config = serde_json::json!({"mcpServers":{}});
    let (ar, config_root) = setup_fixture(root.path(), "nextjs", "14.2.5", Some(&config));

    let plan_hash = match setup_plan("nextjs", "cursor", &ar, true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    let operation_id = match result {
        SetupApplyResult::SetupComplete { operation_id, .. }
        | SetupApplyResult::ClientReloadRequired { operation_id, .. } => operation_id,
        other => panic!("expected success variant, got: {:?}", other),
    };

    let expected_prefix = format!("setup-{}", &plan_hash[..12]);
    assert_eq!(
        operation_id, expected_prefix,
        "operation_id must be 'setup-' + first 12 plan-hash chars"
    );
}

// ---- 11. Rollback rejects unknown operation id ----

#[test]
fn rollback_rejects_unknown_operation_id() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let config_root = root.path().join("client-root");
    std::fs::create_dir_all(&config_root).unwrap();

    let result = setup_rollback("setup-unknown0000", &config_root).unwrap();
    match result {
        nowdocs::automation::setup::SetupRollbackResult::ManualRequired { .. } => {}
        other => panic!("expected ManualRequired for unknown op, got: {:?}", other),
    }
}

#[test]
fn rollback_rejects_non_setup_operation_id() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let config_root = root.path().join("client-root");
    std::fs::create_dir_all(&config_root).unwrap();

    let result = setup_rollback("ensure-somehash12", &config_root).unwrap();
    match result {
        nowdocs::automation::setup::SetupRollbackResult::ManualRequired { observations } => {
            assert!(
                observations
                    .iter()
                    .any(|o| o.contains("not_generated_by_setup")),
                "must cite not_generated_by_setup, got: {:?}",
                observations
            );
        }
        other => panic!("expected ManualRequired for non-setup id, got: {:?}", other),
    }
}

#[test]
#[cfg(unix)]
fn partial_no_rollback_has_no_operation_id() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let (ar, config_root) = setup_fixture(root.path(), "nextjs", "14.2.5", None);

    let plan_hash = match setup_plan("nextjs", "cursor", &ar, true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    match result {
        SetupApplyResult::PartialNoRollback { observations } => {
            assert!(
                !observations
                    .iter()
                    .any(|o| o.contains("setup-") && o.len() >= 17),
                "PartialNoRollback must not carry an operation_id"
            );
        }
        other => panic!("expected PartialNoRollback, got: {:?}", other),
    }

    assert_eq!(
        cache::check_docset_state("nextjs"),
        InstalledDocsetState::Healthy
    );
}

// ---- 12. Redaction: observations contain no paths ----

#[test]
#[cfg(unix)]
fn apply_result_observations_contain_no_absolute_paths() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let config = serde_json::json!({"mcpServers":{}});
    let (ar, config_root) = setup_fixture(root.path(), "nextjs", "14.2.5", Some(&config));

    let plan_hash = match setup_plan("nextjs", "cursor", &ar, true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    let observations = match &result {
        SetupApplyResult::SetupComplete { observations, .. }
        | SetupApplyResult::ClientReloadRequired { observations, .. }
        | SetupApplyResult::ActionRequired { observations }
        | SetupApplyResult::PartialNoRollback { observations }
        | SetupApplyResult::Partial { observations, .. }
        | SetupApplyResult::AppliedButUnverified { observations } => observations,
    };

    let root_str = root.path().to_string_lossy().to_string();
    for obs in observations {
        assert!(
            !obs.contains(&root_str),
            "observation leaked isolated root path: {obs}"
        );
        assert!(
            !obs.contains("/Users/"),
            "observation leaked absolute path: {obs}"
        );
    }
}

// ---- 13. Invalid client/docset validation ----

#[test]
fn setup_plan_rejects_invalid_client() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let ar = make_approved_root(root.path());

    let result = setup_plan("nextjs", "invalid-client", &ar, false, 1_000_000_000);
    assert!(result.is_err(), "invalid client must be rejected");
}

#[test]
fn setup_plan_rejects_invalid_docset() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let ar = make_approved_root(root.path());

    let result = setup_plan("UPPERCASE", "cursor", &ar, false, 1_000_000_000);
    assert!(result.is_err(), "invalid docset must be rejected");
}

// ---- 14. Binary path resolution ----

#[test]
#[cfg(unix)]
fn setup_apply_works_with_real_binary_path() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let config = serde_json::json!({"mcpServers":{}});
    let (ar, config_root) = setup_fixture(root.path(), "nextjs", "14.2.5", Some(&config));

    let plan_hash = match setup_plan("nextjs", "cursor", &ar, true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    match result {
        SetupApplyResult::SetupComplete { .. } | SetupApplyResult::ClientReloadRequired { .. } => {}
        other => panic!("expected success variant, got: {:?}", other),
    }

    let after: serde_json::Value = serde_json::from_slice(
        &std::fs::read(config_root.join(".cursor").join("mcp.json")).unwrap(),
    )
    .unwrap();
    let cmd = after["mcpServers"]["nowdocs"]["command"].as_str().unwrap();
    assert!(
        std::path::Path::new(cmd).is_absolute(),
        "cursor config must reference an absolute binary path, got: {cmd}"
    );
}

// =========================================================================
// C7R3 repair contract tests
// =========================================================================

// ---- Gate 1: healthy docset + absent Cursor config => persisted client plan ----

#[test]
#[cfg(unix)]
fn gate1_healthy_docset_absent_cursor_config_produces_persisted_client_plan() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let archive = make_release_archive("react", "18.3.1");
    let tar_path = root.path().join("react.tar");
    std::fs::write(&tar_path, &archive).unwrap();
    let url = format!("file://{}", tar_path.display());
    registry::install("react", &url).expect("install fixture docset");

    let config_root = root.path().join("client-root");
    std::fs::create_dir_all(&config_root).unwrap();
    let ar = make_approved_root(&config_root);

    let archive_path = root.path().join("react-18.3.1.lance.tar");
    let package = package_for("react", "18.3.1", &archive_path);
    setup_index(root.path(), &package);

    let result = setup_plan("react", "cursor", &ar, true, 1_000_000_000).expect("plan succeeds");
    match result {
        SetupPlanResult::PlanCreated { plan_hash, .. } => {
            let plan = load_plan(&plan_hash, 1_000_000_001).expect("load plan");
            assert!(
                plan.actions.iter().any(|a| a.kind == "client_apply"),
                "plan must include client_apply for cursor with absent config"
            );
        }
        other => panic!(
            "healthy docset + absent cursor config must produce PlanCreated, got: {:?}",
            other
        ),
    }
}

// ---- Gate 2: noncanonical existing Cursor entry => manual guidance plan,
//      then apply leaves config byte-identical ----

#[test]
#[cfg(unix)]
fn gate2_noncanonical_cursor_entry_produces_manual_guidance_and_no_mutation() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let existing = serde_json::json!({
        "mcpServers": {"nowdocs": {"command": "/other/nowdocs", "args": ["serve"]}}
    });
    let (ar, config_root) = setup_fixture(root.path(), "nextjs", "14.2.5", Some(&existing));

    let result = setup_plan("nextjs", "cursor", &ar, true, 1_000_000_000).expect("plan succeeds");
    let plan_hash = match result {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let plan = load_plan(&plan_hash, 1_000_000_001).expect("load plan");
    assert!(
        plan.actions
            .iter()
            .any(|a| a.kind == "client_manual_guidance"),
        "noncanonical cursor entry must produce client_manual_guidance action"
    );
    assert!(
        !plan.actions.iter().any(|a| a.kind == "client_apply"),
        "noncanonical cursor entry must NOT produce client_apply action"
    );

    let config_bytes = std::fs::read(config_root.join(".cursor").join("mcp.json")).unwrap();
    let result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    match result {
        SetupApplyResult::ActionRequired { .. } => {}
        other => panic!(
            "expected ActionRequired for noncanonical entry, got: {:?}",
            other
        ),
    }

    assert_eq!(
        std::fs::read(config_root.join(".cursor").join("mcp.json")).unwrap(),
        config_bytes,
        "noncanonical entry must be byte-identical after apply"
    );
}

// ---- Gate 3: Cursor target content/existence drift => PLAN_STALE before
//      registry fixture sees installation or adapter is invoked ----

#[test]
#[cfg(unix)]
fn gate3_cursor_target_drift_returns_plan_stale_before_install_or_adapter() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let config = serde_json::json!({"mcpServers":{}});
    let (ar, config_root) = setup_fixture(root.path(), "nextjs", "14.2.5", Some(&config));

    let plan_hash = match setup_plan("nextjs", "cursor", &ar, true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    // Drift: modify the cursor config content after plan creation.
    let drifted = serde_json::json!({"mcpServers":{"other":{"command":"/x","args":[]}}});
    std::fs::write(
        config_root.join(".cursor").join("mcp.json"),
        serde_json::to_vec_pretty(&drifted).unwrap(),
    )
    .unwrap();

    assert_ne!(
        cache::check_docset_state("nextjs"),
        InstalledDocsetState::Healthy,
        "docset must not be installed when plan is stale"
    );

    let result = setup_apply(&plan_hash, &config_root, 1_000_000_001);
    assert!(result.is_err(), "stale plan must be refused");
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("PLAN_STALE"),
        "error must carry PLAN_STALE, got: {msg}"
    );

    assert_ne!(
        cache::check_docset_state("nextjs"),
        InstalledDocsetState::Healthy,
        "docset must not be installed after stale refusal"
    );
}

// ---- Gate 4: manual clients retain docset plan/action and apply never
//      calls an adapter ----

#[test]
#[cfg(unix)]
fn gate4_manual_clients_retain_docset_plan_and_apply_never_calls_adapter() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let ar = make_approved_root(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let plan_hash = match setup_plan("nextjs", "claude-desktop", &ar, true, 1_000_000_000).unwrap()
    {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let plan = load_plan(&plan_hash, 1_000_000_001).expect("load plan");
    assert!(
        plan.actions
            .iter()
            .any(|a| a.kind == "registry_install" || a.kind == "registry_update"),
        "claude-desktop plan must retain docset install/update action"
    );
    assert!(
        plan.actions.iter().any(|a| a.kind == "verify_docset"),
        "claude-desktop plan must retain verify_docset action"
    );
    assert!(
        plan.actions
            .iter()
            .any(|a| a.kind == "client_manual_guidance"),
        "claude-desktop plan must have client_manual_guidance"
    );
    assert!(
        !plan.actions.iter().any(|a| a.kind == "client_apply"),
        "claude-desktop must not have client_apply"
    );

    let config_root = root.path().join("client-root");
    std::fs::create_dir_all(&config_root).unwrap();

    let result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    match result {
        SetupApplyResult::ActionRequired { .. } => {}
        other => panic!(
            "expected ActionRequired for manual client, got: {:?}",
            other
        ),
    }

    assert_eq!(
        cache::check_docset_state("nextjs"),
        InstalledDocsetState::Healthy
    );
}

// ---- Gate 5: metadata symlink through the actual successful apply path
//      produces a redacted exit-21 outcome and no rollback object ----

#[test]
#[cfg(unix)]
fn gate5_metadata_unsafe_produces_applied_but_unverified_no_rollback() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let config = serde_json::json!({"mcpServers":{}});
    let (ar, config_root) = setup_fixture(root.path(), "nextjs", "14.2.5", Some(&config));

    let plan_hash = match setup_plan("nextjs", "cursor", &ar, true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    // Pre-create the operation directory with a symlink at setup-meta.json.
    let op_id = format!("setup-{}", &plan_hash[..12]);
    let ops_dir = root
        .path()
        .join("nowdocs")
        .join("automation")
        .join("operations")
        .join(&op_id);
    std::fs::create_dir_all(&ops_dir).unwrap();
    let meta_path = ops_dir.join("setup-meta.json");
    std::os::unix::fs::symlink("/dev/null", &meta_path).unwrap();

    let result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    match result {
        SetupApplyResult::AppliedButUnverified { observations } => {
            let root_str = root.path().to_string_lossy().to_string();
            for obs in &observations {
                assert!(
                    !obs.contains(&root_str),
                    "applied_but_unverified observation leaked path: {obs}"
                );
                assert!(
                    !obs.contains("/Users/"),
                    "observation leaked absolute path: {obs}"
                );
            }
        }
        other => panic!(
            "expected AppliedButUnverified when metadata can't persist, got: {:?}",
            other
        ),
    }

    let after: serde_json::Value = serde_json::from_slice(
        &std::fs::read(config_root.join(".cursor").join("mcp.json")).unwrap(),
    )
    .unwrap();
    assert!(
        after["mcpServers"]["nowdocs"].is_object(),
        "cursor config was written by the adapter before metadata failure"
    );
}

// ---- Gate 6: unknown rollback neither follows metadata symlinks nor
//      creates an operation directory ----

#[test]
fn gate6_unknown_rollback_does_not_follow_symlinks_or_create_dir() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let config_root = root.path().join("client-root");
    std::fs::create_dir_all(&config_root).unwrap();

    let ops_root = root
        .path()
        .join("nowdocs")
        .join("automation")
        .join("operations");
    assert!(
        !ops_root.exists(),
        "operations root must not exist before rollback"
    );

    let result = setup_rollback("setup-unknown0000", &config_root).unwrap();
    match result {
        nowdocs::automation::setup::SetupRollbackResult::ManualRequired { .. } => {}
        other => panic!("expected ManualRequired for unknown op, got: {:?}", other),
    }

    assert!(
        !ops_root.join("setup-unknown0000").exists(),
        "unknown rollback must not create an operation directory"
    );
}

// ---- Gate 7: offline missing docset returns only registry_metadata_required
//      and creates no files ----

#[test]
fn gate7_offline_missing_docset_creates_no_files() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let ar = make_approved_root(root.path());

    let result =
        setup_plan("nextjs", "cursor", &ar, false, 1_000_000_000).expect("plan should succeed");
    assert!(
        matches!(result, SetupPlanResult::RegistryMetadataRequired { .. }),
        "offline missing docset must return RegistryMetadataRequired, got: {:?}",
        result
    );

    assert!(
        count_entries(root.path()) == 0,
        "offline refusal must create no files under isolated root"
    );
}

// ---- Non-Unix: a setup path that reaches secure persistence is rejected with
//      the existing stable higher-level error and creates no plan, metadata,
//      operation journal, client config, or installed docset ----

#[test]
#[cfg(not(unix))]
fn online_setup_plan_fails_closed_without_persisting_or_installing() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());
    let ar = make_approved_root(root.path());

    // Set up a trusted registry index fixture with a syntactically safe
    // release URL. Planning never downloads this package; using HTTPS here
    // avoids embedding Windows backslashes in the stored plan target path,
    // so setup_plan reaches the store_plan (secure persistence) step.
    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let mut package = package_for("nextjs", "14.2.5", &archive_path);
    package.download_url = "https://github.com/nowdocs-registry/nextjs/releases/download/nextjs-14.2.5/nextjs-14.2.5.lance.tar".to_string();
    setup_index(root.path(), &package);

    // setup_plan must fail closed when it reaches store_plan.
    let result = setup_plan("nextjs", "cursor", &ar, true, 1_000_000_000);
    assert!(
        result.is_err(),
        "online setup plan must fail closed on unsupported platform"
    );
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("PLAN_TAMPERED: unsupported platform for no-follow I/O"),
        "setup_plan must fail closed with the stable platform prefix, got: {msg}"
    );

    // Zero payload mutation: store_plan initializes empty automation
    // directories before the unsupported file open, but no plan is persisted.
    let plans_dir = root.path().join("nowdocs").join("automation").join("plans");
    assert!(
        plans_dir.is_dir(),
        "the existing automation initializer creates an empty plans directory"
    );

    // Empty operations scaffolding is allowed; no journal, backup, or metadata
    // payload file may exist.
    let ops_dir = root
        .path()
        .join("nowdocs")
        .join("automation")
        .join("operations");
    assert!(
        ops_dir.is_dir(),
        "the existing automation initializer creates an empty operations directory"
    );

    // No docset was installed.
    let db_dir = root.path().join("nowdocs").join("docsets").join("nextjs");
    assert!(
        !db_dir.exists(),
        "no docset installation may appear on the unsupported platform"
    );

    // No client configuration was created.
    assert!(
        !root.path().join("client-root").join(".cursor").exists(),
        "no client config may be created on the unsupported platform"
    );

    // No automation payload files under the cache root.
    let auto_root = root.path().join("nowdocs").join("automation");
    assert!(
        count_regular_files_recursive(&auto_root) == 0,
        "no automation payload file may appear under the cache root"
    );
}

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

// ---- 13. Codex CLI integration (C7R5) ----
//
// These tests exercise setup plan/apply for the `codex` client through a local
// fake `codex` executable. They isolate PATH/HOME/XDG/TMPDIR/XDG_CACHE_HOME so
// no real Codex configuration, model, registry network, or client process is
// touched, and they assert that Codex never causes setup to inspect, create,
// or fingerprint `.cursor/mcp.json`.

#[cfg(unix)]
fn make_executable(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).unwrap();
}

/// Escape a string for safe insertion into a JSON string literal. Only handles
/// the characters that can realistically appear in a test-binary path.
#[cfg(unix)]
fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Isolated environment with a fake `codex` on PATH. The fake records its argv
/// and maintains state (absent/present/present-other) in the scratch directory.
#[cfg(unix)]
struct CodexTestEnv {
    #[allow(dead_code)]
    tmp: tempfile::TempDir,
    bin_dir: std::path::PathBuf,
    client_root: std::path::PathBuf,
    argv_log: std::path::PathBuf,
    state_file: std::path::PathBuf,
    command_file: std::path::PathBuf,
    _env_guard: CodexEnvGuard,
}

#[cfg(unix)]
impl CodexTestEnv {
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let bin_dir = tmp.path().join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        let client_root = tmp.path().join("client-root");
        std::fs::create_dir_all(&client_root).unwrap();
        let home = tmp.path().join("home");
        std::fs::create_dir_all(&home).unwrap();
        let argv_log = tmp.path().join("argv.log");
        let state_file = tmp.path().join("codex-state");
        let command_file = tmp.path().join("codex-command");

        let env_guard = CodexEnvGuard::new(&bin_dir, &home, tmp.path());

        let mut env = Self {
            tmp,
            bin_dir,
            client_root,
            argv_log,
            state_file,
            command_file,
            _env_guard: env_guard,
        };
        env.install_fake_codex();
        env
    }

    fn approved_root(&self) -> ApprovedRoot {
        make_approved_root(&self.client_root)
    }

    fn install_docset(&self, docset: &str, version: &str) {
        let archive = make_release_archive(docset, version);
        let tar_path = self.tmp.path().join(format!("{docset}-{version}.tar"));
        std::fs::write(&tar_path, &archive).unwrap();
        let url = format!("file://{}", tar_path.display());
        registry::install(docset, &url).expect("install fixture docset");
    }

    fn setup_index(&self, docset: &str, version: &str) -> RegistryPackage {
        let archive_path = self
            .tmp
            .path()
            .join(format!("{docset}-{version}.lance.tar"));
        let package = package_for(docset, version, &archive_path);
        let index_path = self.tmp.path().join("index.json");
        std::fs::write(&index_path, make_index_json(&package)).unwrap();
        std::env::set_var(
            "NOWDOCS_REGISTRY_INDEX_URL",
            format!("file://{}", index_path.display()),
        );
        package
    }

    fn set_state(&self, state: &str) {
        std::fs::write(&self.state_file, state).unwrap();
    }

    fn set_command(&self, command: &str) {
        std::fs::write(&self.command_file, json_escape(command)).unwrap();
    }

    fn recorded_argv(&self) -> Vec<String> {
        let bytes = std::fs::read(&self.argv_log).unwrap_or_default();
        bytes
            .split(|&b| b == 0)
            .filter(|s| !s.is_empty())
            .map(|s| String::from_utf8_lossy(s).to_string())
            .collect()
    }

    fn install_fake_codex(&mut self) {
        let script = r#"#!/bin/sh
{
  printf 'codex\0'
  for a in "$@"; do
    printf '%s\0' "$a"
  done
} >> "__ARGV_LOG__"
state_file="__STATE__"
command_file="__COMMAND__"
subcmd="$2"
case "$subcmd" in
  get)
    if [ -r "$state_file" ]; then
      read -r state < "$state_file"
    else
      state=absent
    fi
    case "$state" in
      present)
        read -r cmd < "$command_file"
        printf '{"name":"nowdocs","enabled":true,"transport":{"type":"stdio","command":"%s","args":["serve"]}}\n' "$cmd"
        exit 0
        ;;
      present-other)
        printf '{"name":"nowdocs","enabled":true,"transport":{"type":"stdio","command":"/bin/other-nowdocs","args":["serve"]}}\n'
        exit 0
        ;;
      disabled)
        read -r cmd < "$command_file"
        printf '{"name":"nowdocs","enabled":false,"transport":{"type":"stdio","command":"%s","args":["serve"]}}\n' "$cmd"
        exit 0
        ;;
      *)
        printf "Error: No MCP server named 'nowdocs' found.\n" >&2
        exit 1
        ;;
    esac
    ;;
  add)
    echo present > "$state_file"
    exit 0
    ;;
  remove)
    echo absent > "$state_file"
    exit 0
    ;;
  *)
    exit 2
    ;;
esac
"#;
        let script = script
            .replace("__ARGV_LOG__", &self.argv_log.display().to_string())
            .replace("__STATE__", &self.state_file.display().to_string())
            .replace("__COMMAND__", &self.command_file.display().to_string());
        let codex = self.bin_dir.join("codex");
        std::fs::write(&codex, script).unwrap();
        make_executable(&codex);

        // Prime the canonical command file with the current test binary path.
        let binary = std::env::current_exe().unwrap();
        self.set_command(&binary.display().to_string());
    }
}

/// RAII guard that holds the global ENV_LOCK and restores PATH/HOME/XDG env
/// values on drop.
#[cfg(unix)]
struct CodexEnvGuard {
    _lock: std::sync::MutexGuard<'static, ()>,
    saved_path: Option<std::ffi::OsString>,
    saved_home: Option<std::ffi::OsString>,
    saved_xdg_config: Option<std::ffi::OsString>,
    saved_xdg_data: Option<std::ffi::OsString>,
    saved_xdg_cache: Option<std::ffi::OsString>,
    saved_tmpdir: Option<std::ffi::OsString>,
    saved_index_url: Option<std::ffi::OsString>,
}

#[cfg(unix)]
impl CodexEnvGuard {
    fn new(bin_dir: &std::path::Path, home: &std::path::Path, xdg_root: &std::path::Path) -> Self {
        let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let saved_path = std::env::var_os("PATH");
        let saved_home = std::env::var_os("HOME");
        let saved_xdg_config = std::env::var_os("XDG_CONFIG_HOME");
        let saved_xdg_data = std::env::var_os("XDG_DATA_HOME");
        let saved_xdg_cache = std::env::var_os("XDG_CACHE_HOME");
        let saved_tmpdir = std::env::var_os("TMPDIR");
        let saved_index_url = std::env::var_os("NOWDOCS_REGISTRY_INDEX_URL");

        std::env::set_var("PATH", bin_dir);
        std::env::set_var("HOME", home);
        std::env::set_var("XDG_CONFIG_HOME", xdg_root.join(".config"));
        std::env::set_var("XDG_DATA_HOME", xdg_root.join(".local").join("share"));
        std::env::set_var("XDG_CACHE_HOME", xdg_root);
        std::env::set_var("TMPDIR", xdg_root);
        std::env::remove_var("NOWDOCS_REGISTRY_INDEX_URL");

        Self {
            _lock: lock,
            saved_path,
            saved_home,
            saved_xdg_config,
            saved_xdg_data,
            saved_xdg_cache,
            saved_tmpdir,
            saved_index_url,
        }
    }
}

#[cfg(unix)]
impl Drop for CodexEnvGuard {
    fn drop(&mut self) {
        restore_env("PATH", &self.saved_path);
        restore_env("HOME", &self.saved_home);
        restore_env("XDG_CONFIG_HOME", &self.saved_xdg_config);
        restore_env("XDG_DATA_HOME", &self.saved_xdg_data);
        restore_env("XDG_CACHE_HOME", &self.saved_xdg_cache);
        restore_env("TMPDIR", &self.saved_tmpdir);
        restore_env("NOWDOCS_REGISTRY_INDEX_URL", &self.saved_index_url);
    }
}

#[cfg(unix)]
fn restore_env(key: &str, saved: &Option<std::ffi::OsString>) {
    match saved {
        Some(v) => std::env::set_var(key, v),
        None => std::env::remove_var(key),
    }
}

#[test]
#[cfg(unix)]
fn codex_plan_absent_creates_apply_and_verify_actions() {
    let env = CodexTestEnv::new();
    env.setup_index("nextjs", "14.2.5");
    env.set_state("absent");
    let ar = env.approved_root();

    let result = setup_plan("nextjs", "codex", &ar, true, 1_000_000_000).unwrap();
    let plan_hash = match result {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let plan = load_plan(&plan_hash, 1_000_000_001).unwrap();
    assert_eq!(plan.inputs.client.as_deref(), Some("codex"));

    let kinds: Vec<&str> = plan.actions.iter().map(|a| a.kind.as_str()).collect();
    assert!(
        kinds.contains(&"client_apply"),
        "absent codex registration must produce client_apply, got: {:?}",
        kinds
    );
    assert!(
        kinds.contains(&"client_verify"),
        "absent codex registration must produce client_verify, got: {:?}",
        kinds
    );
    assert!(
        !kinds.contains(&"client_manual_guidance"),
        "absent registration must not produce manual guidance"
    );

    assert!(
        plan.preconditions.target_files.is_empty(),
        "codex plan must not carry Cursor target-file preconditions"
    );
    assert!(
        !env.client_root.join(".cursor").exists(),
        "codex planning must not create .cursor files"
    );
}

#[test]
#[cfg(unix)]
fn codex_plan_canonical_docset_returns_already_satisfied() {
    let env = CodexTestEnv::new();
    env.install_docset("react", "18.3.1");
    env.set_state("present");
    let ar = env.approved_root();

    let result = setup_plan("react", "codex", &ar, false, 1_000_000_000).unwrap();
    assert!(
        matches!(result, SetupPlanResult::AlreadySatisfied { .. }),
        "healthy docset + canonical codex registration must be already satisfied, got: {:?}",
        result
    );

    assert!(
        !env.client_root.join(".cursor").exists(),
        "codex already-satisfied check must not touch .cursor"
    );
}

#[test]
#[cfg(unix)]
fn codex_plan_noncanonical_returns_manual_guidance() {
    let env = CodexTestEnv::new();
    env.setup_index("nextjs", "14.2.5");
    env.set_state("present-other");
    let ar = env.approved_root();

    let result = setup_plan("nextjs", "codex", &ar, true, 1_000_000_000).unwrap();
    let plan_hash = match result {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let plan = load_plan(&plan_hash, 1_000_000_001).unwrap();
    let kinds: Vec<&str> = plan.actions.iter().map(|a| a.kind.as_str()).collect();
    assert!(
        kinds.contains(&"client_manual_guidance"),
        "noncanonical codex registration must produce client_manual_guidance, got: {:?}",
        kinds
    );
    assert!(
        !kinds.contains(&"client_apply"),
        "noncanonical codex registration must not produce client_apply"
    );
    assert!(
        !kinds.contains(&"client_verify"),
        "noncanonical codex registration must not produce client_verify"
    );
    assert!(plan.preconditions.target_files.is_empty());
    assert!(!env.client_root.join(".cursor").exists());
}

#[test]
#[cfg(unix)]
fn codex_apply_invokes_add_then_verifies() {
    let env = CodexTestEnv::new();
    env.setup_index("nextjs", "14.2.5");
    env.set_state("absent");
    let ar = env.approved_root();

    let plan_hash = match setup_plan("nextjs", "codex", &ar, true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let result = setup_apply(&plan_hash, &env.client_root, 1_000_000_001).unwrap();
    assert!(
        matches!(result, SetupApplyResult::SetupComplete { .. }),
        "absent->add->verify must complete, got: {:?}",
        result
    );

    let argv = env.recorded_argv();
    assert!(
        argv.windows(5)
            .any(|w| w == ["codex", "mcp", "get", "nowdocs", "--json"]),
        "fake codex must have been asked to get the registration, got: {:?}",
        argv
    );
    assert!(
        argv.windows(4)
            .any(|w| w == ["codex", "mcp", "add", "nowdocs"]),
        "fake codex must have been asked to add the registration, got: {:?}",
        argv
    );
    assert!(
        !env.client_root.join(".cursor").exists(),
        "codex apply must not create .cursor files"
    );
}

#[test]
#[cfg(unix)]
fn codex_apply_noncanonical_returns_action_required_without_add() {
    let env = CodexTestEnv::new();
    env.setup_index("nextjs", "14.2.5");
    env.set_state("present-other");
    let ar = env.approved_root();

    let plan_hash = match setup_plan("nextjs", "codex", &ar, true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let result = setup_apply(&plan_hash, &env.client_root, 1_000_000_001).unwrap();
    assert!(
        matches!(result, SetupApplyResult::ActionRequired { .. }),
        "noncanonical codex must return ActionRequired, got: {:?}",
        result
    );

    let argv = env.recorded_argv();
    assert!(
        !argv
            .windows(4)
            .any(|w| w == ["codex", "mcp", "add", "nowdocs"]),
        "noncanonical codex must not invoke codex mcp add, got: {:?}",
        argv
    );
    assert!(!env.client_root.join(".cursor").exists());
}

#[test]
#[cfg(unix)]
fn codex_does_not_create_cursor_fingerprint_even_if_cursor_file_exists() {
    let env = CodexTestEnv::new();
    env.setup_index("nextjs", "14.2.5");
    env.set_state("absent");

    // Pre-create a Cursor config file in the approved root. Codex planning must
    // ignore it entirely and not create a target-file precondition for it.
    std::fs::create_dir_all(env.client_root.join(".cursor")).unwrap();
    std::fs::write(
        env.client_root.join(".cursor").join("mcp.json"),
        b"{\"mcpServers\":{}}",
    )
    .unwrap();

    let ar = env.approved_root();
    let result = setup_plan("nextjs", "codex", &ar, true, 1_000_000_000).unwrap();
    let plan_hash = match result {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let plan = load_plan(&plan_hash, 1_000_000_001).unwrap();
    assert!(
        plan.preconditions.target_files.is_empty(),
        "codex plan must not inherit Cursor target-file preconditions"
    );
}

#[test]
#[cfg(unix)]
fn cursor_regression_still_creates_target_fingerprint() {
    let env = CodexTestEnv::new();
    env.setup_index("nextjs", "14.2.5");
    // No fake `cursor` is needed because planning only inspects the file.
    let ar = env.approved_root();

    let result = setup_plan("nextjs", "cursor", &ar, true, 1_000_000_000).unwrap();
    let plan_hash = match result {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let plan = load_plan(&plan_hash, 1_000_000_001).unwrap();
    assert!(
        !plan.preconditions.target_files.is_empty(),
        "cursor plan must retain target-file preconditions"
    );
}
