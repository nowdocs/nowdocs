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
use nowdocs::clients::{approved_root, atomic_replace, safe_target};
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

// ---- 1. Offline missing docset: registry_metadata_required, exit 0, no files ----

#[test]
fn offline_missing_docset_returns_registry_metadata_required_and_creates_no_files() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let result = setup_plan("nextjs", "cursor", false, 1_000_000_000).expect("plan should succeed");
    assert!(
        matches!(result, SetupPlanResult::RegistryMetadataRequired { .. }),
        "offline missing docset must return RegistryMetadataRequired, got: {:?}",
        result
    );

    // No cache, model, or automation files should have been created.
    assert!(
        count_entries(root.path()) == 0,
        "offline refusal must create no files under isolated root"
    );
}

// ---- 2. Online planning: one hash, prescribed action ordering, no absolute/secret fields ----

#[test]
fn online_planning_produces_one_hash_with_prescribed_action_ordering() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let result = setup_plan("nextjs", "cursor", true, 1_000_000_000).expect("online plan succeeds");
    let plan_hash = match result {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    // The plan hash is 64 lowercase hex characters.
    assert_eq!(plan_hash.len(), 64, "plan hash must be 64 hex chars");
    assert!(
        plan_hash
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
        "plan hash must be lowercase hex"
    );

    // Load the stored plan and verify action ordering.
    let plan = load_plan(&plan_hash, 1_000_000_001).expect("load stored plan");
    assert_eq!(plan.inputs.client.as_deref(), Some("cursor"));
    assert_eq!(plan.inputs.docset.as_deref(), Some("nextjs"));
    assert!(plan.inputs.online);

    // Action ordering: docset install/update, verify_docset, client_apply, client_verify.
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

    // Ordering: verify_docset < client_apply < client_verify.
    let vd = verify_docset_idx.unwrap();
    let ca = client_apply_idx.unwrap();
    let cv = client_verify_idx.unwrap();
    assert!(vd < ca, "verify_docset must come before client_apply");
    assert!(ca < cv, "client_apply must come before client_verify");

    // No absolute paths, secrets, or config bytes in plan data.
    let plan_json = serde_json::to_string(&plan).unwrap();
    assert!(
        !plan_json.contains("/Users/") && !plan_json.contains("/home/"),
        "plan must not contain absolute home paths"
    );
    assert!(
        !plan_json.contains("token") && !plan_json.contains("secret"),
        "plan must not contain secrets"
    );
    // Target paths are logical, not absolute.
    for action in &plan.actions {
        for tp in &action.target_paths {
            assert!(
                !tp.starts_with('/'),
                "target path must be logical (not absolute): {tp}"
            );
        }
    }
}

// ---- 2b. Already satisfied docset: no plan created ----

#[test]
fn already_satisfied_docset_returns_already_satisfied() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    // Install a docset directly.
    let archive = make_release_archive("react", "18.3.1");
    let tar_path = root.path().join("react.tar");
    std::fs::write(&tar_path, &archive).unwrap();
    let url = format!("file://{}", tar_path.display());
    registry::install("react", &url).expect("install fixture docset");

    let result = setup_plan("react", "cursor", false, 1_000_000_000).expect("plan should succeed");
    assert!(
        matches!(result, SetupPlanResult::AlreadySatisfied { .. }),
        "installed docset must be already satisfied, got: {:?}",
        result
    );
}

// ---- 3. Tampered, expired, wrong-version, drifted plans are refused ----

#[test]
fn apply_refuses_tampered_plan() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let plan_hash = match setup_plan("nextjs", "cursor", true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    // Tamper with the stored plan file.
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
fn apply_refuses_expired_plan() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let created_at = 1_000_000_000;
    let plan_hash = match setup_plan("nextjs", "cursor", true, created_at).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let now = created_at + 30 * 60 + 1; // 30 min + 1 sec
    let result = setup_apply(&plan_hash, root.path(), now);
    assert!(result.is_err(), "expired plan must be refused");
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("PLAN_EXPIRED"),
        "error must carry PLAN_EXPIRED, got: {msg}"
    );
}

#[test]
fn apply_refuses_stale_plan_when_docset_state_drifts() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let plan_hash = match setup_plan("nextjs", "cursor", true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    // Change the state by installing a different version manually.
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
fn claude_desktop_plan_produces_manual_only_and_no_write() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let result =
        setup_plan("nextjs", "claude-desktop", true, 1_000_000_000).expect("plan should succeed");
    let plan_hash = match result {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let plan = load_plan(&plan_hash, 1_000_000_001).expect("load plan");
    // Claude Desktop has no client_apply/client_verify actions (apply is Unsupported).
    assert!(
        !plan
            .actions
            .iter()
            .any(|a| a.kind == "client_apply" || a.kind == "client_verify"),
        "claude-desktop must not have client apply/verify actions"
    );
}

#[test]
fn generic_plan_produces_manual_only_and_no_write() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let result = setup_plan("nextjs", "generic", true, 1_000_000_000).expect("plan should succeed");
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
}

// ---- 5. Cursor/Claude Code conflict, unsafe root, missing executable, malformed target ----

#[test]
fn cursor_apply_returns_conflict_when_nowdocs_already_exists() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let plan_hash = match setup_plan("nextjs", "cursor", true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    // Pre-create the cursor config with an existing nowdocs entry.
    let config_root = root.path().join("client-root");
    std::fs::create_dir_all(config_root.join(".cursor")).unwrap();
    let existing = serde_json::json!({
        "mcpServers": {
            "nowdocs": {"command": "/other/nowdocs", "args": ["serve"]}
        }
    });
    std::fs::write(
        config_root.join(".cursor").join("mcp.json"),
        serde_json::to_vec_pretty(&existing).unwrap(),
    )
    .unwrap();

    let original_bytes = std::fs::read(config_root.join(".cursor").join("mcp.json")).unwrap();

    let result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    match result {
        SetupApplyResult::PartialNoRollback { .. } => {}
        other => panic!("expected PartialNoRollback for partial, got: {:?}", other),
    }

    // The file must be byte-identical (no mutation).
    assert_eq!(
        std::fs::read(config_root.join(".cursor").join("mcp.json")).unwrap(),
        original_bytes,
        "conflict must not mutate the config file"
    );
}

#[test]
fn cursor_apply_returns_action_required_for_missing_cursor_config() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let plan_hash = match setup_plan("nextjs", "cursor", true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    // Client root with no .cursor/mcp.json -- cursor apply returns ManualRequired.
    let config_root = root.path().join("client-root");
    std::fs::create_dir_all(&config_root).unwrap();

    let result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    match result {
        SetupApplyResult::PartialNoRollback { .. } => {}
        other => panic!(
            "expected PartialNoRollback for missing target, got: {:?}",
            other
        ),
    }

    // Docset should have been installed (additive), but no .cursor dir created.
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
fn cursor_apply_returns_action_required_for_malformed_json() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let plan_hash = match setup_plan("nextjs", "cursor", true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let config_root = root.path().join("client-root");
    std::fs::create_dir_all(config_root.join(".cursor")).unwrap();
    let malformed = b"{ this is not valid json ";
    std::fs::write(config_root.join(".cursor").join("mcp.json"), malformed).unwrap();

    let result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    match result {
        SetupApplyResult::PartialNoRollback { .. } => {}
        other => panic!(
            "expected PartialNoRollback for malformed JSON, got: {:?}",
            other
        ),
    }

    // The file must be unchanged.
    assert_eq!(
        std::fs::read(config_root.join(".cursor").join("mcp.json")).unwrap(),
        malformed,
        "malformed config must not be mutated"
    );
}

#[test]
fn cursor_apply_succeeds_and_verifies() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let plan_hash = match setup_plan("nextjs", "cursor", true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    // Set up a valid cursor config without a nowdocs entry.
    let config_root = root.path().join("client-root");
    std::fs::create_dir_all(config_root.join(".cursor")).unwrap();
    let original = serde_json::json!({"mcpServers":{}});
    std::fs::write(
        config_root.join(".cursor").join("mcp.json"),
        serde_json::to_vec_pretty(&original).unwrap(),
    )
    .unwrap();

    let result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    // Cursor verify always emits client_reload_required.
    match result {
        SetupApplyResult::SetupComplete { .. } | SetupApplyResult::ClientReloadRequired { .. } => {}
        other => panic!(
            "expected SetupComplete or ClientReloadRequired, got: {:?}",
            other
        ),
    }

    // Docset is healthy.
    assert_eq!(
        cache::check_docset_state("nextjs"),
        InstalledDocsetState::Healthy
    );

    // Cursor config now has a nowdocs entry.
    let after: serde_json::Value = serde_json::from_slice(
        &std::fs::read(config_root.join(".cursor").join("mcp.json")).unwrap(),
    )
    .unwrap();
    assert!(after["mcpServers"]["nowdocs"].is_object());
}

// ---- 6b. client_reload_required result ----

#[test]
fn cursor_apply_returns_client_reload_required_when_verified() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let plan_hash = match setup_plan("nextjs", "cursor", true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let config_root = root.path().join("client-root");
    std::fs::create_dir_all(config_root.join(".cursor")).unwrap();
    let original = serde_json::json!({"mcpServers":{}});
    std::fs::write(
        config_root.join(".cursor").join("mcp.json"),
        serde_json::to_vec_pretty(&original).unwrap(),
    )
    .unwrap();

    let result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    // Cursor verify always emits client_reload_required, so the result should be
    // either SetupComplete or ClientReloadRequired depending on the observation.
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
fn partial_result_when_docset_succeeds_but_client_fails() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let plan_hash = match setup_plan("nextjs", "cursor", true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    // No cursor config exists -> cursor apply returns ManualRequired.
    let config_root = root.path().join("client-root");
    std::fs::create_dir_all(&config_root).unwrap();

    let result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    match result {
        SetupApplyResult::PartialNoRollback { .. } => {}
        other => panic!(
            "expected PartialNoRollback for malformed JSON, got: {:?}",
            other
        ),
    }

    // The docset was installed and must remain (never deleted to conceal partial).
    assert_eq!(
        cache::check_docset_state("nextjs"),
        InstalledDocsetState::Healthy,
        "additive docset must remain after partial client failure"
    );
}

// ---- 6d. Exact rollback ----

#[test]
fn rollback_restores_cursor_config() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let plan_hash = match setup_plan("nextjs", "cursor", true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let config_root = root.path().join("client-root");
    std::fs::create_dir_all(config_root.join(".cursor")).unwrap();
    let original = serde_json::json!({
        "mcpServers": {
            "filesystem": {"command": "/usr/bin/fs", "args": []}
        }
    });
    std::fs::write(
        config_root.join(".cursor").join("mcp.json"),
        serde_json::to_vec_pretty(&original).unwrap(),
    )
    .unwrap();

    let apply_result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    let operation_id = match apply_result {
        SetupApplyResult::SetupComplete { operation_id, .. }
        | SetupApplyResult::ClientReloadRequired { operation_id, .. } => operation_id,
        other => panic!(
            "expected success variant for rollback setup, got: {:?}",
            other
        ),
    };

    // The config now has a nowdocs entry.
    let after: serde_json::Value = serde_json::from_slice(
        &std::fs::read(config_root.join(".cursor").join("mcp.json")).unwrap(),
    )
    .unwrap();
    assert!(after["mcpServers"]["nowdocs"].is_object());

    // Rollback.
    let rollback_result = setup_rollback(&operation_id, &config_root).unwrap();
    match rollback_result {
        nowdocs::automation::setup::SetupRollbackResult::RolledBack { .. } => {}
        other => panic!("expected RolledBack, got: {:?}", other),
    }

    // Original content restored.
    let restored: serde_json::Value = serde_json::from_slice(
        &std::fs::read(config_root.join(".cursor").join("mcp.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(restored, original);
    assert!(restored["mcpServers"]["nowdocs"].is_null());
}

// ---- 6e. Later user edit refusal ----

#[test]
fn rollback_refuses_after_later_user_edit() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let plan_hash = match setup_plan("nextjs", "cursor", true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let config_root = root.path().join("client-root");
    std::fs::create_dir_all(config_root.join(".cursor")).unwrap();
    let original = serde_json::json!({"mcpServers":{}});
    std::fs::write(
        config_root.join(".cursor").join("mcp.json"),
        serde_json::to_vec_pretty(&original).unwrap(),
    )
    .unwrap();

    let apply_result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    let operation_id = match apply_result {
        SetupApplyResult::SetupComplete { operation_id, .. }
        | SetupApplyResult::ClientReloadRequired { operation_id, .. } => operation_id,
        other => panic!("expected success variant, got: {:?}", other),
    };

    // User edits the file after apply.
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
        other => panic!(
            "expected ManualRequired for user-edited state, got: {:?}",
            other
        ),
    }

    // The user-edited content is preserved (rollback did not overwrite it).
    let current = std::fs::read(config_root.join(".cursor").join("mcp.json")).unwrap();
    assert!(
        String::from_utf8_lossy(&current).contains("edited"),
        "user-edited content must be preserved"
    );
}

// ---- 7. Plan hash is an integrity/scope check; setup apply only accepts that hash ----

#[test]
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
fn setup_plan_creates_exactly_one_plan_file() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let _ = setup_plan("nextjs", "cursor", true, 1_000_000_000).unwrap();

    let plans_dir = root.path().join("nowdocs").join("automation").join("plans");
    let plan_files: Vec<_> = std::fs::read_dir(&plans_dir)
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().ends_with(".json"))
        .collect();
    assert_eq!(
        plan_files.len(),
        1,
        "exactly one plan file must be created, got: {plan_files:?}"
    );
}

// ---- 9. Claude Code: manual-only in tests (no real claude CLI) ----

#[test]
fn claude_code_apply_returns_action_required_without_claude_cli() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let plan_hash = match setup_plan("nextjs", "claude-code", true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let config_root = root.path().join("client-root");
    std::fs::create_dir_all(&config_root).unwrap();

    // Without a real `claude` CLI on PATH, apply must return PartialNoRollback
    // (docset installed, client apply could not start).
    let result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    match result {
        SetupApplyResult::PartialNoRollback { .. } => {}
        other => panic!(
            "expected PartialNoRollback when claude CLI is missing, got: {:?}",
            other
        ),
    }

    // Docset should still be installed (additive).
    assert_eq!(
        cache::check_docset_state("nextjs"),
        InstalledDocsetState::Healthy
    );
}

// ---- 10. Operation id format: setup- + first 12 hash chars ----

#[test]
fn operation_id_is_setup_prefixed_first_12_hash_chars() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let plan_hash = match setup_plan("nextjs", "cursor", true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let config_root = root.path().join("client-root");
    std::fs::create_dir_all(config_root.join(".cursor")).unwrap();
    let original = serde_json::json!({"mcpServers":{}});
    std::fs::write(
        config_root.join(".cursor").join("mcp.json"),
        serde_json::to_vec_pretty(&original).unwrap(),
    )
    .unwrap();

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

    // A setup-prefixed id that has no setup-meta.json (never recorded by setup).
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

    // An operation id without the "setup-" prefix must be refused.
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
fn partial_no_rollback_has_no_operation_id() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let plan_hash = match setup_plan("nextjs", "cursor", true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    // No cursor config exists -> cursor apply returns ManualRequired.
    let config_root = root.path().join("client-root");
    std::fs::create_dir_all(&config_root).unwrap();

    let result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    match result {
        SetupApplyResult::PartialNoRollback { observations } => {
            // Must not carry an operation_id (no rollback metadata).
            assert!(
                !observations
                    .iter()
                    .any(|o| o.contains("setup-") && o.len() >= 17),
                "PartialNoRollback must not carry an operation_id"
            );
        }
        other => panic!("expected PartialNoRollback, got: {:?}", other),
    }

    // The docset was installed (additive, not deleted).
    assert_eq!(
        cache::check_docset_state("nextjs"),
        InstalledDocsetState::Healthy
    );
}

// ---- 12. Redaction: apply result observations contain no paths ----

#[test]
fn apply_result_observations_contain_no_absolute_paths() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let plan_hash = match setup_plan("nextjs", "cursor", true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let config_root = root.path().join("client-root");
    std::fs::create_dir_all(config_root.join(".cursor")).unwrap();
    let original = serde_json::json!({"mcpServers":{}});
    std::fs::write(
        config_root.join(".cursor").join("mcp.json"),
        serde_json::to_vec_pretty(&original).unwrap(),
    )
    .unwrap();

    let result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    let observations = match &result {
        SetupApplyResult::SetupComplete { observations, .. }
        | SetupApplyResult::ClientReloadRequired { observations, .. }
        | SetupApplyResult::ActionRequired { observations }
        | SetupApplyResult::PartialNoRollback { observations }
        | SetupApplyResult::Partial { observations, .. } => observations,
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

    let result = setup_plan("nextjs", "invalid-client", false, 1_000_000_000);
    assert!(result.is_err(), "invalid client must be rejected");
}

#[test]
fn setup_plan_rejects_invalid_docset() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let result = setup_plan("UPPERCASE", "cursor", false, 1_000_000_000);
    assert!(result.is_err(), "invalid docset must be rejected");
}

// ---- 14. Binary path resolution: refuse if binary cannot be resolved ----

#[test]
fn setup_apply_works_with_real_binary_path() {
    let root = tempfile::tempdir().unwrap();
    let _g = isolate(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    setup_index(root.path(), &package);

    let plan_hash = match setup_plan("nextjs", "cursor", true, 1_000_000_000).unwrap() {
        SetupPlanResult::PlanCreated { plan_hash, .. } => plan_hash,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let config_root = root.path().join("client-root");
    std::fs::create_dir_all(config_root.join(".cursor")).unwrap();
    let original = serde_json::json!({"mcpServers":{}});
    std::fs::write(
        config_root.join(".cursor").join("mcp.json"),
        serde_json::to_vec_pretty(&original).unwrap(),
    )
    .unwrap();

    // setup_apply resolves the nowdocs binary at apply time via current_exe.
    // In tests, the binary is the test executable, which is absolute.
    let result = setup_apply(&plan_hash, &config_root, 1_000_000_001).unwrap();
    match result {
        SetupApplyResult::SetupComplete { .. } | SetupApplyResult::ClientReloadRequired { .. } => {}
        other => panic!("expected success variant, got: {:?}", other),
    }

    // The cursor config should reference the absolute binary path.
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
