use nowdocs::chunker::{Chunk, ChunkType};
use nowdocs::manifest;
use nowdocs::store::Store;

// These tests set XDG_CACHE_HOME and must not run in parallel (dirs::cache_dir
// reads the env var, but parallel set_var creates a race).
// Run with: cargo test --test registry_tests -- --test-threads=1

fn test_manifest_json() -> &'static str {
    r#"{
        "docset": "test-docset",
        "doc_version": "1.0.0",
        "nowdocs_schema_version": 1,
        "embedder": {
            "model_id": "jinaai/jina-embeddings-v2-small-en",
            "model_version": "0.1.0",
            "model_revision": "abc123",
            "model_sha256": "deadbeef",
            "vector_dim": 512,
            "engine": "candle",
            "dtype": "f16"
        },
        "retrieval": {
            "tokenizer": "default",
            "chunk_size_tokens": 512,
            "window_tokens": 64
        },
        "source": {
            "entry_url": "https://example.com/docs",
            "source_url": "https://example.com",
            "scraped_at": "2026-01-01T00:00:00Z",
            "chunk_count": 2
        },
        "legal": {
            "license": "MIT",
            "copyright_holder": "Example",
            "attribution": ""
        },
        "refresh_strategy": {
            "tier": "stable",
            "auto_days": 30
        }
    }"#
}

fn test_chunks_jsonl() -> &'static str {
    r#"{"idx":0,"heading_path":"Intro","source_url":"https://example.com/0","api_version":null,"chunk_type":"Info","text":"hello"}
{"idx":1,"heading_path":"API","source_url":"https://example.com/1","api_version":null,"chunk_type":"Info","text":"world"}
"#
}

fn make_tar_archive(_dir: &std::path::Path) -> Vec<u8> {
    let manifest_data = test_manifest_json().as_bytes();
    let chunks_data = test_chunks_jsonl().as_bytes();

    let files: Vec<(&str, &[u8])> = vec![
        ("manifest.json", manifest_data),
        ("chunks.jsonl", chunks_data),
    ];

    let mut archive = Vec::new();
    for (name, data) in &files {
        archive.extend_from_slice(&make_tar_entry(name, data));
    }
    // Two zero blocks to end the archive.
    archive.extend_from_slice(&[0u8; 512]);
    archive.extend_from_slice(&[0u8; 512]);
    archive
}

fn make_tar_entry(name: &str, data: &[u8]) -> Vec<u8> {
    let mut header = [0u8; 512];
    // name (0..100)
    let name_bytes = name.as_bytes();
    header[0..name_bytes.len()].copy_from_slice(name_bytes);
    // mode (100..108) — 8 bytes
    header[100..107].copy_from_slice(b"000644\0");
    // uid (108..116) — 8 bytes
    header[108..115].copy_from_slice(b"000000\0");
    // gid (116..124) — 8 bytes
    header[116..123].copy_from_slice(b"000000\0");
    // size (124..136) — 12 bytes
    let size_str = format!("{:011o}\0", data.len());
    header[124..136].copy_from_slice(size_str.as_bytes());
    // mtime (136..148) — 12 bytes
    header[136..148].copy_from_slice(b"00000000000\0");
    // typeflag (156)
    header[156] = b'0';
    // magic (257..265)
    header[257..263].copy_from_slice(b"ustar\0");
    // version (265..268)
    header[265..267].copy_from_slice(b"00");

    // Compute checksum: sum of header bytes with checksum field (148..156) as spaces.
    let mut sum: u32 = 0;
    for (i, &b) in header.iter().enumerate() {
        sum += if (148..156).contains(&i) { b' ' as u32 } else { b as u32 };
    }
    let chk_str = format!("{:06o}\0 ", sum);
    header[148..156].copy_from_slice(chk_str.as_bytes());

    let mut entry = header.to_vec();
    entry.extend_from_slice(data);
    // Pad to 512-byte boundary.
    let padded = ((data.len() + 511) / 512) * 512;
    if padded > data.len() {
        entry.extend_from_slice(&vec![0u8; padded - data.len()]);
    }
    entry
}

fn write_test_manifest(_dir: &std::path::Path, docset: &str) {
    let manifest_path = nowdocs::cache::manifest_path(docset);
    std::fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
    std::fs::write(&manifest_path, test_manifest_json()).unwrap();
}

fn populate_test_store(docset: &str) {
    let store = Store::open(docset).unwrap();
    let chunks = vec![
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
    ];
    let vectors: Vec<Vec<f32>> = chunks.iter().map(|_| vec![0.0f32; 512]).collect();
    store.insert(&chunks, &vectors).unwrap();
}

// --- Test: uninstall removes db and manifest ---

#[test]
fn test_uninstall_removes_db_and_manifest() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test_uninstall";
    write_test_manifest(dir.path(), docset);

    // Create db directory.
    let db = nowdocs::cache::db_path(docset);
    std::fs::create_dir_all(&db).unwrap();
    std::fs::write(db.join("dummy"), b"data").unwrap();

    let mp = nowdocs::cache::manifest_path(docset);
    assert!(mp.is_file(), "manifest should exist before uninstall");

    nowdocs::registry::uninstall(docset).unwrap();

    assert!(!db.exists(), "db dir should be removed after uninstall");
    assert!(!mp.is_file(), "manifest should be removed after uninstall");
}

// --- Test: install rejects external URLs ---

#[test]
fn test_install_rejects_external_url() {
    let result = nowdocs::registry::install("test-evil", "https://evil.com/x.tar");
    assert!(result.is_err(), "external URL should be rejected");
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("not in allowed domains"),
        "error should mention domain restriction, got: {}",
        msg
    );
}

#[test]
fn test_install_rejects_lookalike_domain() {
    // github.com/nowdocs-registry.evil.com should NOT match github.com/nowdocs-registry
    let result = nowdocs::registry::install(
        "test-lookalike",
        "https://github.com/nowdocs-registry.evil.com/x.tar",
    );
    assert!(result.is_err(), "lookalike domain should be rejected");
}

#[test]
fn test_install_rejects_path_traversal_docset() {
    let result = nowdocs::registry::install("../../tmp/victim", "file:///dev/null");
    assert!(result.is_err(), "path traversal docset should be rejected");
}

// --- Test: install from file:// URL ---

#[test]
fn test_install_from_file_url() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test_file_install";
    let archive = make_tar_archive(dir.path());
    let tar_path = dir.path().join("archive.tar");
    std::fs::write(&tar_path, &archive).unwrap();

    let url = format!("file://{}", tar_path.display());
    nowdocs::registry::install(docset, &url).unwrap();

    // Manifest should be written.
    let mp = nowdocs::cache::manifest_path(docset);
    assert!(mp.is_file(), "manifest should be installed");

    let raw = std::fs::read_to_string(&mp).unwrap();
    let m = manifest::parse_manifest(&raw).unwrap();
    assert_eq!(m.docset, "test-docset");

    // Store should be materialized — chunks are searchable.
    let store = Store::open(docset).unwrap();
    let chunks = store.dump_chunks().unwrap();
    assert_eq!(chunks.len(), 2, "store should have 2 chunks after install");
    assert_eq!(chunks[0].text, "hello");
    assert_eq!(chunks[1].text, "world");
}

// --- Test: share produces no vectors ---

#[test]
fn test_share_produces_no_vectors() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test_share";
    write_test_manifest(dir.path(), docset);
    populate_test_store(docset);

    let out_dir = dir.path().join("share_out");
    let share_path = nowdocs::registry::share(docset, &out_dir).unwrap();

    // Verify no .lance files or vector data in share output.
    let entries: Vec<_> = std::fs::read_dir(&share_path)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    for entry in &entries {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        assert!(
            !name_str.contains(".lance"),
            "share output should not contain .lance files, found: {}",
            name_str
        );
    }
    assert!(
        share_path.join("manifest.json").is_file(),
        "share should produce manifest.json"
    );
    assert!(
        share_path.join("chunks.jsonl").is_file(),
        "share should produce chunks.jsonl"
    );

    // Verify chunks.jsonl content: should be JSON lines without vector data.
    let jsonl = std::fs::read_to_string(share_path.join("chunks.jsonl")).unwrap();
    for line in jsonl.lines() {
        let v: serde_json::Value = serde_json::from_str(line).unwrap();
        assert!(v.get("text").is_some(), "chunk should have text field");
        assert!(v.get("idx").is_some(), "chunk should have idx field");
        // No vector field should exist.
        assert!(v.get("vector").is_none(), "chunk should NOT have vector field");
    }
}

// --- Test: update refreshes manifest ---

#[test]
fn test_update_refreshes_manifest() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test_update";

    // Write initial manifest.
    write_test_manifest(dir.path(), docset);
    let mp = nowdocs::cache::manifest_path(docset);
    let before = std::fs::read_to_string(&mp).unwrap();
    let m_before = manifest::parse_manifest(&before).unwrap();
    assert_eq!(m_before.doc_version, "1.0.0");

    // Create a v2 archive with updated doc_version.
    let v2_json = test_manifest_json().replace("1.0.0", "2.0.0");
    let v2_chunks = r#"{"idx":0,"heading_path":"Updated","source_url":"https://example.com/v2","api_version":null,"chunk_type":"Info","text":"updated content"}
"#;
    let v2_archive = {
        let files: Vec<(&str, &[u8])> = vec![
            ("manifest.json", v2_json.as_bytes()),
            ("chunks.jsonl", v2_chunks.as_bytes()),
        ];
        let mut archive = Vec::new();
        for (name, data) in &files {
            archive.extend_from_slice(&make_tar_entry(name, data));
        }
        archive.extend_from_slice(&[0u8; 512]);
        archive.extend_from_slice(&[0u8; 512]);
        archive
    };

    let tar_path = dir.path().join("v2.tar");
    std::fs::write(&tar_path, &v2_archive).unwrap();

    // Set test URL env var for update().
    let url = format!("file://{}", tar_path.display());
    unsafe { std::env::set_var("NOWDOCS_TEST_URL", &url) };

    nowdocs::registry::update(docset).unwrap();

    let after = std::fs::read_to_string(&mp).unwrap();
    let m_after = manifest::parse_manifest(&after).unwrap();
    assert_eq!(
        m_after.doc_version, "2.0.0",
        "manifest should be refreshed to v2.0.0"
    );
}
