//! Integration tests for registry provenance receipts.

use std::sync::Mutex;

use nowdocs::cache;
use nowdocs::registry::RegistryPackage;
use nowdocs::registry_receipt;

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
