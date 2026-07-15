//! Integration tests for registry provenance receipts.

use std::sync::Mutex;

use nowdocs::cache;
use nowdocs::chunker::{Chunk, ChunkType};
use nowdocs::registry::RegistryPackage;
use nowdocs::registry_receipt;
use nowdocs::store::Store;

static ENV_LOCK: Mutex<()> = Mutex::new(());

struct MultiEnvGuard {
    pairs: Vec<(&'static str, Option<String>)>,
    _g: std::sync::MutexGuard<'static, ()>,
}

impl MultiEnvGuard {
    fn set(pairs: &[(&'static str, &str)]) -> Self {
        let g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let saved: Vec<_> = pairs
            .iter()
            .map(|(k, v)| {
                let old = std::env::var(k).ok();
                std::env::set_var(k, v);
                (*k, old)
            })
            .collect();
        Self {
            pairs: saved,
            _g: g,
        }
    }
}

impl Drop for MultiEnvGuard {
    fn drop(&mut self) {
        for (k, old) in self.pairs.iter().rev() {
            match old {
                Some(v) => std::env::set_var(k, v),
                None => std::env::remove_var(k),
            }
        }
    }
}

fn isolated_cache() -> (tempfile::TempDir, MultiEnvGuard) {
    let dir = tempfile::tempdir().unwrap();
    let cache_path = dir.path().to_str().unwrap().to_string();
    let g = MultiEnvGuard::set(&[
        ("XDG_CACHE_HOME", &cache_path),
        ("http_proxy", "http://127.0.0.1:9"),
        ("https_proxy", "http://127.0.0.1:9"),
        ("HTTP_PROXY", "http://127.0.0.1:9"),
        ("HTTPS_PROXY", "http://127.0.0.1:9"),
        ("ALL_PROXY", "http://127.0.0.1:9"),
        ("no_proxy", ""),
        ("NO_PROXY", ""),
    ]);
    (dir, g)
}

fn test_manifest_json(docset: &str, version: &str) -> String {
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

fn write_healthy_manifest(docset: &str, version: &str) {
    let path = cache::manifest_path(docset);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, test_manifest_json(docset, version)).unwrap();
}

fn write_receipt_bytes(docset: &str, bytes: &[u8]) {
    let root = cache::registry_receipts_root();
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join(format!("{docset}.json")), bytes).unwrap();
}

fn package(docset: &str, version: &str, sha256: &str) -> RegistryPackage {
    RegistryPackage {
        docset: docset.to_string(),
        version: version.to_string(),
        license: "MIT".to_string(),
        chunk_count: 2,
        freshness: "2026-01-01".to_string(),
        download_url: format!(
            "https://github.com/nowdocs-registry/{docset}/releases/latest/download/{docset}.tar"
        ),
        sha256: sha256.to_string(),
        description: None,
    }
}

#[test]
fn receipt_requires_the_current_promoted_manifest() {
    let (_dir, _g) = isolated_cache();
    write_healthy_manifest("nextjs", "15.0.0");
    let sha = "a".repeat(64);
    let pkg = package("nextjs", "15.0.0", &sha);
    registry_receipt::record_after_promotion(&pkg).unwrap();

    assert_eq!(registry_receipt::load_matching_installed().len(), 1);

    // Update the manifest version — receipt no longer matches.
    write_healthy_manifest("nextjs", "15.0.1");
    assert!(registry_receipt::load_matching_installed().is_empty());
}

#[test]
fn malformed_receipt_and_same_named_local_ingest_are_excluded() {
    let (_dir, _g) = isolated_cache();
    write_receipt_bytes("react", b"not json");

    // A local ingest with the same name but no receipt is excluded.
    write_healthy_manifest("react", "19.0.0");
    assert!(registry_receipt::load_matching_installed().is_empty());
}

#[test]
fn receipt_root_write_failure_is_visible_to_the_install_boundary() {
    let (_dir, _g) = isolated_cache();
    std::fs::create_dir_all(cache::cache_root()).unwrap();
    // Block directory creation by writing a file where the directory should be.
    std::fs::write(cache::registry_receipts_root(), b"block directory creation").unwrap();
    let sha = "a".repeat(64);
    let err =
        registry_receipt::record_after_promotion(&package("nextjs", "15.0.0", &sha)).unwrap_err();
    assert!(err.to_string().contains("registry provenance receipt"));
}

#[test]
fn remove_receipt_is_idempotent() {
    let (_dir, _g) = isolated_cache();
    // Removing a non-existent receipt is not an error.
    registry_receipt::remove("nonexistent").unwrap();
}

#[test]
fn receipt_path_is_under_cache_root() {
    let (_dir, _g) = isolated_cache();
    let path = registry_receipt::receipt_path("test");
    assert!(cache::is_under_cache_root(&path));
    assert!(path.ends_with("test.json"));
}

// ---- Fixture infrastructure for lifecycle tests ----

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

fn make_registry_release(docset: &str, manifest_json: &str) -> Vec<u8> {
    let saved = std::env::var("XDG_CACHE_HOME").ok();
    let src = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", src.path()) };
    nowdocs::cache::ensure_layout().unwrap();
    {
        let store = Store::open(docset).unwrap();
        let chunks = two_chunks();
        let vectors: Vec<Vec<f32>> = chunks.iter().map(|_| vec![0.0f32; 512]).collect();
        store.insert(&chunks, &vectors).unwrap();
    }
    let lance_dir = nowdocs::cache::db_path(docset);

    let mut archive = Vec::new();
    archive.extend_from_slice(&make_tar_entry("manifest.json", manifest_json.as_bytes()));
    add_dir_to_tar(&mut archive, &lance_dir, &format!("{docset}.lance"));
    archive.extend_from_slice(&[0u8; 512]);
    archive.extend_from_slice(&[0u8; 512]);

    match saved {
        Some(v) => unsafe { std::env::set_var("XDG_CACHE_HOME", v) },
        None => unsafe { std::env::remove_var("XDG_CACHE_HOME") },
    }
    archive
}

fn fixture_package(docset: &str, version: &str, tar_path: &std::path::Path) -> RegistryPackage {
    let data = std::fs::read(tar_path).unwrap();
    let sha = nowdocs::registry::sha256_hex(&data);
    RegistryPackage {
        docset: docset.to_string(),
        version: version.to_string(),
        license: "MIT".to_string(),
        chunk_count: 2,
        freshness: "2026-01-01".to_string(),
        download_url: format!("file://{}", tar_path.display()),
        sha256: sha,
        description: None,
    }
}

// ---- Lifecycle tests ----

#[test]
fn verified_package_install_creates_receipt_after_promotion() {
    let (dir, _g) = isolated_cache();
    let docset = "nextjs";
    let manifest = test_manifest_json(docset, "15.0.0");
    let archive = make_registry_release(docset, &manifest);
    let tar_path = dir.path().join("nextjs.tar");
    std::fs::write(&tar_path, &archive).unwrap();
    let pkg = fixture_package(docset, "15.0.0", &tar_path);

    nowdocs::registry::install_verified_package(&pkg).unwrap();
    assert!(
        registry_receipt::receipt_path(docset).is_file(),
        "receipt must be created after verified install"
    );
    assert!(cache::db_path(docset).exists(), "db must be promoted");
}

#[test]
fn verified_install_fails_if_receipt_cannot_be_persisted_and_uninstall_removes_receipt() {
    let (dir, _g) = isolated_cache();
    let docset = "nextjs";
    let manifest = test_manifest_json(docset, "15.0.0");
    let archive = make_registry_release(docset, &manifest);
    let tar_path = dir.path().join("nextjs.tar");
    std::fs::write(&tar_path, &archive).unwrap();
    let pkg = fixture_package(docset, "15.0.0", &tar_path);

    // Block receipt directory creation.
    std::fs::create_dir_all(cache::cache_root()).unwrap();
    std::fs::write(cache::registry_receipts_root(), b"block").unwrap();
    assert!(
        nowdocs::registry::install_verified_package(&pkg).is_err(),
        "install must fail when receipt cannot be persisted"
    );

    // Remove the block and install successfully.
    std::fs::remove_file(cache::registry_receipts_root()).unwrap();
    nowdocs::registry::install_verified_package(&pkg).unwrap();
    assert!(registry_receipt::receipt_path(docset).is_file());

    // Uninstall removes the receipt.
    nowdocs::registry::uninstall(docset).unwrap();
    assert!(
        !registry_receipt::receipt_path(docset).exists(),
        "uninstall must remove the receipt"
    );
}

#[test]
fn already_satisfied_ensure_does_not_backfill_a_legacy_receipt() {
    // This test is in automation_docset_tests.rs since it needs ensure_apply.
    // Placeholder to remind the plan.
}
