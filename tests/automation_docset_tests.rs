//! C4 — Registry-aware planning and idempotent docset ensure.
//!
//! All tests run with an isolated XDG_CACHE_HOME/HOME/cwd root so they never
//! touch a real cache, model, client config, or network. Tests that need a
//! registry index use a local `file://` fixture; tests that need a release
//! artifact build it in memory from a real `.lance` table.

use std::sync::Mutex;

use nowdocs::automation::docset::{ensure_apply, ensure_plan, EnsurePlanResult};
use nowdocs::automation::plan::load_plan;
use nowdocs::cache::{self, InstalledDocsetState};
use nowdocs::chunker::{Chunk, ChunkType};
use nowdocs::registry::{self, RegistryPackage};
use nowdocs::store::Store;

// Serialize env mutation so parallel tests don't race on XDG_CACHE_HOME.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Holds the env lock and the prior values for XDG_CACHE_HOME and TMPDIR.
/// A single guard sets both vars atomically under one lock (the std Mutex is
/// not reentrant, so two separate guards would deadlock in the same thread).
struct EnvGuard {
    old_xdg: Option<String>,
    old_tmpdir: Option<String>,
    _g: std::sync::MutexGuard<'static, ()>,
}

impl EnvGuard {
    fn isolate(root: &std::path::Path) -> Self {
        let g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let old_xdg = std::env::var("XDG_CACHE_HOME").ok();
        std::env::set_var("XDG_CACHE_HOME", root.to_str().unwrap());
        let old_tmpdir = std::env::var("TMPDIR").ok();
        std::env::set_var("TMPDIR", root.to_str().unwrap());
        Self {
            old_xdg,
            old_tmpdir,
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
    }
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

/// Compute the SHA-256 of the release archive bytes.
fn archive_sha256(archive: &[u8]) -> String {
    registry::sha256_hex(archive)
}

/// Build an index fixture containing one package pointing at the given archive.
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

/// Set the isolated test root as both XDG cache dir and temp dir so any
/// transient downloads stay inside the test root and are cleaned up with it.
fn isolate(root: &std::path::Path) -> EnvGuard {
    EnvGuard::isolate(root)
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

// ---- RED tests: offline refusal, no files created ----

#[test]
fn offline_missing_docset_returns_registry_metadata_required_and_creates_no_files() {
    let root = tempfile::tempdir().unwrap();
    let _guards = isolate(root.path());

    let result = ensure_plan("nextjs", false, 1_000_000_000);
    assert!(
        result.is_ok(),
        "ensure_plan should succeed with a required-metadata envelope: {}",
        result.unwrap_err()
    );
    let result = result.unwrap();
    assert!(
        matches!(result, EnsurePlanResult::RegistryMetadataRequired { .. }),
        "offline missing docset must return RegistryMetadataRequired, got: {:?}",
        result
    );

    // No cache, model, or automation files should have been created.
    assert!(
        count_entries(root.path()) == 0,
        "offline refusal must create no files under isolated root"
    );
}

// ---- RED tests: online planning persists only selected metadata ----

#[test]
fn online_planning_creates_plan_and_leaves_no_index_bytes() {
    let root = tempfile::tempdir().unwrap();
    let _guards = isolate(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    let index_path = root.path().join("index.json");
    std::fs::write(&index_path, make_index_json(&package)).unwrap();
    std::env::set_var(
        "NOWDOCS_REGISTRY_INDEX_URL",
        format!("file://{}", index_path.display()),
    );

    let result =
        ensure_plan("nextjs", true, 1_000_000_000).expect("online planning should succeed");
    let plan_id = match result {
        EnsurePlanResult::PlanCreated { plan_id, .. } => plan_id,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    // The plan file exists.
    let plan_path = root
        .path()
        .join("nowdocs")
        .join("automation")
        .join("plans")
        .join(format!("{plan_id}.json"));
    assert!(plan_path.is_file(), "plan must be stored at {plan_path:?}");

    // No full-index bytes remain (the fixture file itself is the caller's; the
    // temporary download created by fetch_index_from must be gone).
    let mut index_download_leftovers = 0;
    for entry in std::fs::read_dir(std::env::temp_dir()).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("nowdocs_dl_index_") {
            index_download_leftovers += 1;
        }
    }
    assert_eq!(
        index_download_leftovers, 0,
        "online planning must remove temporary full-index bytes"
    );

    // The plan can be loaded and contains exactly one docset precondition and
    // two deterministic actions (install/update then verify).
    let plan = load_plan(&plan_id, 1_000_000_001).expect("load stored plan");
    assert_eq!(plan.inputs.docset.as_deref(), Some("nextjs"));
    assert_eq!(plan.preconditions.docset_state.len(), 1);
    assert_eq!(plan.actions.len(), 2);
    assert!(
        plan.actions[0].kind == "registry_install" || plan.actions[0].kind == "registry_update",
        "first action must be install/update, got {:?}",
        plan.actions[0].kind
    );
    assert_eq!(plan.actions[1].kind, "verify_docset");
}

// ---- RED tests: already satisfied ----

#[test]
fn installed_docset_is_already_satisfied_offline() {
    let root = tempfile::tempdir().unwrap();
    let _guards = isolate(root.path());

    // Install a docset directly via the registry library.
    let archive = make_release_archive("react", "18.3.1");
    let tar_path = root.path().join("react.tar");
    std::fs::write(&tar_path, &archive).unwrap();
    let url = format!("file://{}", tar_path.display());
    registry::install("react", &url).expect("install fixture docset");

    let result = ensure_plan("react", false, 1_000_000_000).expect("ensure_plan should succeed");
    assert!(
        matches!(result, EnsurePlanResult::AlreadySatisfied { .. }),
        "installed docset must be already satisfied offline, got: {:?}",
        result
    );
}

#[test]
fn installed_docset_is_already_satisfied_online() {
    let root = tempfile::tempdir().unwrap();
    let _guards = isolate(root.path());

    let archive = make_release_archive("vue", "3.4.0");
    let tar_path = root.path().join("vue.tar");
    std::fs::write(&tar_path, &archive).unwrap();
    let url = format!("file://{}", tar_path.display());
    registry::install("vue", &url).expect("install fixture docset");

    // Build an index pointing at the same version.
    let archive_path = root.path().join("vue-3.4.0.lance.tar");
    let package = package_for("vue", "3.4.0", &archive_path);
    let index_path = root.path().join("index.json");
    std::fs::write(&index_path, make_index_json(&package)).unwrap();
    std::env::set_var(
        "NOWDOCS_REGISTRY_INDEX_URL",
        format!("file://{}", index_path.display()),
    );

    let result = ensure_plan("vue", true, 1_000_000_000).expect("online ensure should succeed");
    assert!(
        matches!(result, EnsurePlanResult::AlreadySatisfied { .. }),
        "installed docset matching registry must be already satisfied, got: {:?}",
        result
    );
}

// ---- RED tests: untrusted catalog rejection ----

#[test]
fn online_planning_rejects_untrusted_catalog() {
    let root = tempfile::tempdir().unwrap();
    let _guards = isolate(root.path());

    let bad_index = r#"{
        "schema_version": 1,
        "generated_at": "2026-07-07T00:00:00Z",
        "packages": [{
            "docset": "evil",
            "version": "1.0.0",
            "license": "MIT",
            "chunk_count": 1,
            "freshness": "2026-07-07",
            "download_url": "https://evil.example.com/evil.tar",
            "sha256": "0000000000000000000000000000000000000000000000000000000000000000",
            "description": "disallowed url"
        }]
    }"#;
    let index_path = root.path().join("bad_index.json");
    std::fs::write(&index_path, bad_index).unwrap();
    std::env::set_var(
        "NOWDOCS_REGISTRY_INDEX_URL",
        format!("file://{}", index_path.display()),
    );

    let result = ensure_plan("evil", true, 1_000_000_000);
    assert!(result.is_err(), "untrusted catalog must be rejected");
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("disallowed download_url") || msg.contains("not in allowed domains"),
        "error must cite disallowed URL, got: {msg}"
    );
}

// ---- RED tests: exact apply and repeated apply ----

#[test]
fn apply_plan_installs_docset_exactly() {
    let root = tempfile::tempdir().unwrap();
    let _guards = isolate(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    let index_path = root.path().join("index.json");
    std::fs::write(&index_path, make_index_json(&package)).unwrap();
    std::env::set_var(
        "NOWDOCS_REGISTRY_INDEX_URL",
        format!("file://{}", index_path.display()),
    );

    let result = ensure_plan("nextjs", true, 1_000_000_000).expect("plan should be created");
    let plan_id = match result {
        EnsurePlanResult::PlanCreated { plan_id, .. } => plan_id,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    ensure_apply("nextjs", &plan_id, 1_000_000_001).expect("apply should succeed");

    assert_eq!(
        cache::check_docset_state("nextjs"),
        InstalledDocsetState::Healthy,
        "docset must be healthy after apply"
    );

    // Repeated apply must be already_satisfied and not redownload.
    ensure_apply("nextjs", &plan_id, 1_000_000_002).expect("repeat apply should succeed");
    assert_eq!(
        cache::check_docset_state("nextjs"),
        InstalledDocsetState::Healthy
    );
}

#[test]
fn apply_refuses_archive_whose_installed_version_differs_from_the_plan() {
    let root = tempfile::tempdir().unwrap();
    let _guards = isolate(root.path());

    // The trusted index says 14.2.5, but its SHA points at an archive whose
    // manifest declares 15.0.0. Apply must not claim the requested version was
    // ensured merely because the archive itself is structurally healthy.
    let archive_path = root.path().join("nextjs-mismatched.lance.tar");
    let archive = make_release_archive("nextjs", "15.0.0");
    std::fs::write(&archive_path, &archive).unwrap();
    let package = RegistryPackage {
        docset: "nextjs".to_string(),
        version: "14.2.5".to_string(),
        license: "MIT".to_string(),
        chunk_count: 2,
        freshness: "2026-07-07".to_string(),
        download_url: format!("file://{}", archive_path.display()),
        sha256: archive_sha256(&archive),
        description: None,
    };
    let index_path = root.path().join("index.json");
    std::fs::write(&index_path, make_index_json(&package)).unwrap();
    std::env::set_var(
        "NOWDOCS_REGISTRY_INDEX_URL",
        format!("file://{}", index_path.display()),
    );

    let plan_id = match ensure_plan("nextjs", true, 1_000_000_000).unwrap() {
        EnsurePlanResult::PlanCreated { plan_id, .. } => plan_id,
        other => panic!("expected PlanCreated, got: {other:?}"),
    };

    let err = ensure_apply("nextjs", &plan_id, 1_000_000_001).unwrap_err();
    let msg = format!("{err:#}");
    assert!(
        msg.contains("VERIFICATION_FAILED") || msg.contains("registry provenance receipt"),
        "a manifest/index version mismatch must not report apply success: {msg}"
    );
}

// ---- RED tests: stale/tampered/expired plan refusal ----

#[test]
fn apply_refuses_tampered_plan() {
    let root = tempfile::tempdir().unwrap();
    let _guards = isolate(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    let index_path = root.path().join("index.json");
    std::fs::write(&index_path, make_index_json(&package)).unwrap();
    std::env::set_var(
        "NOWDOCS_REGISTRY_INDEX_URL",
        format!("file://{}", index_path.display()),
    );

    let result = ensure_plan("nextjs", true, 1_000_000_000).expect("plan should be created");
    let plan_id = match result {
        EnsurePlanResult::PlanCreated { plan_id, .. } => plan_id,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    // Tamper with the stored plan file.
    let plan_path = root
        .path()
        .join("nowdocs")
        .join("automation")
        .join("plans")
        .join(format!("{plan_id}.json"));
    let mut content = std::fs::read_to_string(&plan_path).unwrap();
    content = content.replace("14.2.5", "99.99.99");
    std::fs::write(&plan_path, content).unwrap();

    let result = ensure_apply("nextjs", &plan_id, 1_000_000_001);
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
    let _guards = isolate(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    let index_path = root.path().join("index.json");
    std::fs::write(&index_path, make_index_json(&package)).unwrap();
    std::env::set_var(
        "NOWDOCS_REGISTRY_INDEX_URL",
        format!("file://{}", index_path.display()),
    );

    let created_at = 1_000_000_000;
    let result = ensure_plan("nextjs", true, created_at).expect("plan should be created");
    let plan_id = match result {
        EnsurePlanResult::PlanCreated { plan_id, .. } => plan_id,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    let now = created_at + 30 * 60 + 1; // 30 minutes + 1 second
    let result = ensure_apply("nextjs", &plan_id, now);
    assert!(result.is_err(), "expired plan must be refused");
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("PLAN_EXPIRED"),
        "error must carry PLAN_EXPIRED, got: {msg}"
    );
}

#[test]
fn apply_refuses_stale_plan_when_state_changes() {
    let root = tempfile::tempdir().unwrap();
    let _guards = isolate(root.path());

    let archive_path = root.path().join("nextjs-14.2.5.lance.tar");
    let package = package_for("nextjs", "14.2.5", &archive_path);
    let index_path = root.path().join("index.json");
    std::fs::write(&index_path, make_index_json(&package)).unwrap();
    std::env::set_var(
        "NOWDOCS_REGISTRY_INDEX_URL",
        format!("file://{}", index_path.display()),
    );

    let result = ensure_plan("nextjs", true, 1_000_000_000).expect("plan should be created");
    let plan_id = match result {
        EnsurePlanResult::PlanCreated { plan_id, .. } => plan_id,
        other => panic!("expected PlanCreated, got: {:?}", other),
    };

    // Change the state by installing a different version manually.
    let v2_archive = make_release_archive("nextjs", "15.0.0");
    let v2_path = root.path().join("nextjs-15.0.0.lance.tar");
    std::fs::write(&v2_path, &v2_archive).unwrap();
    registry::install("nextjs", &format!("file://{}", v2_path.display())).unwrap();

    let result = ensure_apply("nextjs", &plan_id, 1_000_000_001);
    assert!(result.is_err(), "stale plan must be refused");
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("PLAN_STALE"),
        "error must carry PLAN_STALE, got: {msg}"
    );
}
