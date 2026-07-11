use nowdocs::cache;
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
    // Pad to 512-byte boundary.
    let padded = data.len().div_ceil(512) * 512;
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
    let chunks = two_chunks();
    let vectors: Vec<Vec<f32>> = chunks.iter().map(|_| vec![0.0f32; 512]).collect();
    store.insert(&chunks, &vectors).unwrap();
}

// --- A1.1: RegistryRelease (.lance) archive fixtures ---
//
// `install` now consumes CI-prebuilt `.lance` tables (OQ1 Method A), not
// `chunks.jsonl`. These helpers build a real Lance table via the local `Store`
// and tar it up as a registry release (`manifest.json` + `<docset>.lance/...`).

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

/// Recursively tar every file under `dir`, prefixing entry names with `prefix`.
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

/// Build a registry-release tar: `manifest.json` + a real `<docset>.lance/`
/// table (built from `chunks`/`vectors`), optionally with a `LICENSE` entry.
///
/// The table is materialized under a scratch cache so it never collides with
/// the calling test's cache root; `XDG_CACHE_HOME` is restored afterwards.
fn make_registry_release(
    docset: &str,
    manifest_json: &str,
    chunks: &[Chunk],
    vectors: &[Vec<f32>],
    license: Option<&[u8]>,
) -> Vec<u8> {
    let saved = std::env::var("XDG_CACHE_HOME").ok();
    let src = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", src.path()) };
    nowdocs::cache::ensure_layout().unwrap();
    {
        let store = Store::open(docset).unwrap();
        if !chunks.is_empty() {
            store.insert(chunks, vectors).unwrap();
        }
    }
    let lance_dir = nowdocs::cache::db_path(docset);

    let mut archive = Vec::new();
    archive.extend_from_slice(&make_tar_entry("manifest.json", manifest_json.as_bytes()));
    add_dir_to_tar(&mut archive, &lance_dir, &format!("{docset}.lance"));
    if let Some(lic) = license {
        archive.extend_from_slice(&make_tar_entry("LICENSE", lic));
    }
    archive.extend_from_slice(&[0u8; 512]);
    archive.extend_from_slice(&[0u8; 512]);

    match saved {
        Some(v) => unsafe { std::env::set_var("XDG_CACHE_HOME", v) },
        None => unsafe { std::env::remove_var("XDG_CACHE_HOME") },
    }
    archive
}

/// Convenience: a valid 2-chunk registry release for `docset` using the
/// standard `test_manifest_json` (chunk_count == 2).
fn make_default_release(docset: &str) -> Vec<u8> {
    let chunks = two_chunks();
    let vecs = zero_vectors(chunks.len());
    make_registry_release(docset, test_manifest_json(), &chunks, &vecs, None)
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

// --- Test: install rejects manifest.docset != CLI install name (S7) ---

#[test]
fn test_install_rejects_docset_identity_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    // Archive manifest declares docset "test-docset".
    let archive = make_default_release("test-docset");
    let tar_path = dir.path().join("archive.tar");
    std::fs::write(&tar_path, &archive).unwrap();
    let url = format!("file://{}", tar_path.display());

    // CLI install name differs from the manifest's docset → must be rejected.
    let result = nowdocs::registry::install("cli-name-mismatch", &url);
    assert!(result.is_err(), "docset identity mismatch must be rejected");
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("does not match install name"),
        "error should explain identity mismatch, got: {msg}"
    );
}

// --- Test: install from file:// URL ---

#[test]
fn test_install_from_file_url() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test-docset";
    let archive = make_default_release("test-docset");
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

// --- Test: install persists bundled LICENSE so re-share carries it ---

#[test]
fn test_install_persists_license_for_reshare() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test-docset";
    let license_body = "MIT license body, upstream notice\n";

    // Build a registry release that includes a LICENSE entry alongside manifest
    // + a real `.lance` table.
    let chunks = two_chunks();
    let vecs = zero_vectors(chunks.len());
    let archive = make_registry_release(
        docset,
        test_manifest_json(),
        &chunks,
        &vecs,
        Some(license_body.as_bytes()),
    );

    let tar_path = dir.path().join("archive_lic.tar");
    std::fs::write(&tar_path, &archive).unwrap();
    let url = format!("file://{}", tar_path.display());
    nowdocs::registry::install(docset, &url).unwrap();

    // install should have stashed the LICENSE at license_text_path (same path
    // share reads from), even though this docset was never locally ingested.
    assert!(
        nowdocs::cache::license_text_path(docset).is_file(),
        "install should persist bundled LICENSE to license_text_path"
    );

    // Re-sharing the installed docset must carry the upstream LICENSE forward.
    let out_dir = dir.path().join("share_out_lic");
    let share_path = nowdocs::registry::share(docset, &out_dir).unwrap();
    let shared = std::fs::read_to_string(share_path.join("LICENSE")).unwrap();
    assert_eq!(
        shared, license_body,
        "re-share of installed docset must carry upstream LICENSE verbatim"
    );
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
        assert!(
            v.get("vector").is_none(),
            "chunk should NOT have vector field"
        );
    }
}

// --- Test: share carries upstream LICENSE + synthesized NOTICE ---

fn write_custom_manifest(docset: &str, json: &str) {
    let manifest_path = nowdocs::cache::manifest_path(docset);
    std::fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
    std::fs::write(&manifest_path, json).unwrap();
}

#[test]
fn test_share_carries_license_and_notice() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test_share_lic";
    write_test_manifest(dir.path(), docset);
    populate_test_store(docset);

    // Stash a source LICENSE text, exactly as ingest would.
    let license_body = "MIT License\n\nCopyright (c) Example\n";
    std::fs::write(nowdocs::cache::license_text_path(docset), license_body).unwrap();

    let out_dir = dir.path().join("share_out");
    let share_path = nowdocs::registry::share(docset, &out_dir).unwrap();

    let license = std::fs::read_to_string(share_path.join("LICENSE")).unwrap();
    assert_eq!(
        license, license_body,
        "LICENSE must be the verbatim upstream text"
    );

    let notice = std::fs::read_to_string(share_path.join("NOTICE")).unwrap();
    assert!(notice.contains("MIT"), "NOTICE must state the license");
    assert!(
        notice.contains("https://example.com"),
        "NOTICE must carry the source URL"
    );
}

#[test]
fn test_share_notice_without_license_text() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test_share_nolic";
    write_test_manifest(dir.path(), docset);
    populate_test_store(docset);
    // No license_text_path stashed — source had no LICENSE file.

    let out_dir = dir.path().join("share_out");
    let share_path = nowdocs::registry::share(docset, &out_dir).unwrap();

    assert!(
        !share_path.join("LICENSE").exists(),
        "no LICENSE file when the source had none"
    );
    assert!(
        share_path.join("NOTICE").is_file(),
        "NOTICE is still synthesized from manifest fields"
    );
}

#[test]
fn test_share_notice_carries_ccby_attribution() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test_share_ccby";
    // CC-BY-4.0 with non-empty attribution (validate enforces this).
    let json = test_manifest_json()
        .replace("\"license\": \"MIT\"", "\"license\": \"CC-BY-4.0\"")
        .replace(
            "\"attribution\": \"\"",
            "\"attribution\": \"Derived from the Example docs, https://example.com\"",
        );
    write_custom_manifest(docset, &json);
    populate_test_store(docset);

    let out_dir = dir.path().join("share_out");
    let share_path = nowdocs::registry::share(docset, &out_dir).unwrap();

    let notice = std::fs::read_to_string(share_path.join("NOTICE")).unwrap();
    assert!(
        notice.contains("CC-BY-4.0"),
        "NOTICE must state the CC-BY-4.0 license"
    );
    assert!(
        notice.contains("Derived from the Example docs"),
        "NOTICE must carry the attribution text (CC-BY-4.0 requirement)"
    );
}

// --- Test: update refreshes manifest ---

#[test]
fn test_update_refreshes_manifest() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test-docset";

    // Write initial manifest.
    write_test_manifest(dir.path(), docset);
    let mp = nowdocs::cache::manifest_path(docset);
    let before = std::fs::read_to_string(&mp).unwrap();
    let m_before = manifest::parse_manifest(&before).unwrap();
    assert_eq!(m_before.doc_version, "1.0.0");

    // Create a v2 registry release with updated doc_version (chunk_count stays
    // 2, so the `.lance` table must carry 2 rows).
    let v2_json = test_manifest_json().replace("1.0.0", "2.0.0");
    let chunks = two_chunks();
    let vecs = zero_vectors(chunks.len());
    let v2_archive = make_registry_release(docset, &v2_json, &chunks, &vecs, None);

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

// ============================================================
// R1: Archive validation tests
// ============================================================

/// Helper: make a tar entry with a custom typeflag (e.g. symlink = b'2').
fn make_tar_entry_with_typeflag(name: &str, data: &[u8], typeflag: u8) -> Vec<u8> {
    let mut header = [0u8; 512];
    let name_bytes = name.as_bytes();
    header[0..name_bytes.len()].copy_from_slice(name_bytes);
    header[100..107].copy_from_slice(b"000644\0");
    header[108..115].copy_from_slice(b"000000\0");
    header[116..123].copy_from_slice(b"000000\0");
    let size_str = format!("{:011o}\0", data.len());
    header[124..136].copy_from_slice(size_str.as_bytes());
    header[136..148].copy_from_slice(b"00000000000\0");
    header[156] = typeflag;
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

fn write_archive_to_tar(dir: &std::path::Path, name: &str, entries: &[(&str, &[u8])]) -> String {
    let mut archive = Vec::new();
    for (entry_name, data) in entries {
        archive.extend_from_slice(&make_tar_entry(entry_name, data));
    }
    archive.extend_from_slice(&[0u8; 512]);
    archive.extend_from_slice(&[0u8; 512]);
    let tar_path = dir.join(name);
    std::fs::write(&tar_path, &archive).unwrap();
    format!("file://{}", tar_path.display())
}

fn write_archive_to_tar_with_flags(
    dir: &std::path::Path,
    name: &str,
    entries: &[(&str, &[u8], u8)],
) -> String {
    let mut archive = Vec::new();
    for (entry_name, data, flag) in entries {
        archive.extend_from_slice(&make_tar_entry_with_typeflag(entry_name, data, *flag));
    }
    archive.extend_from_slice(&[0u8; 512]);
    archive.extend_from_slice(&[0u8; 512]);
    let tar_path = dir.join(name);
    std::fs::write(&tar_path, &archive).unwrap();
    format!("file://{}", tar_path.display())
}

// --- R1: missing manifest rejected ---

#[test]
fn test_r1_missing_manifest_rejected() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let url = write_archive_to_tar(
        dir.path(),
        "no_manifest.tar",
        &[("chunks.jsonl", test_chunks_jsonl().as_bytes())],
    );

    let result = nowdocs::registry::install("r1_no_manifest", &url);
    assert!(result.is_err(), "missing manifest should be rejected");
    let err_str = format!("{}", result.unwrap_err());
    assert!(
        err_str.contains("ARCHIVE_MISSING_MANIFEST"),
        "error should contain ARCHIVE_MISSING_MANIFEST code, got: {}",
        err_str
    );
}

// --- R1: missing chunks rejected ---

#[test]
fn test_r1_manifest_only_rejected_missing_store() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    // A manifest-only archive has no `.lance` table. Under the RegistryRelease
    // contract (Method A) `chunks.jsonl` is optional but a `.lance` dir is
    // required, so this is rejected as ARCHIVE_MISSING_STORE.
    let url = write_archive_to_tar(
        dir.path(),
        "manifest_only.tar",
        &[("manifest.json", test_manifest_json().as_bytes())],
    );

    let result = nowdocs::registry::install("r1_manifest_only", &url);
    assert!(result.is_err(), "manifest-only archive should be rejected");
    let err_str = format!("{}", result.unwrap_err());
    assert!(
        err_str.contains("ARCHIVE_MISSING_STORE"),
        "error should contain ARCHIVE_MISSING_STORE code, got: {}",
        err_str
    );
}

// --- R1: path traversal rejected ---

#[test]
fn test_r1_path_traversal_rejected() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let url = write_archive_to_tar(
        dir.path(),
        "traversal.tar",
        &[
            ("../escape/manifest.json", test_manifest_json().as_bytes()),
            ("chunks.jsonl", test_chunks_jsonl().as_bytes()),
        ],
    );

    let result = nowdocs::registry::install("r1_traversal", &url);
    assert!(result.is_err(), "path traversal should be rejected");
    let err_str = format!("{}", result.unwrap_err());
    assert!(
        err_str.contains("ARCHIVE_UNSAFE_PATH"),
        "error should contain ARCHIVE_UNSAFE_PATH code, got: {}",
        err_str
    );
}

// --- R1: absolute path rejected ---

#[test]
fn test_r1_absolute_path_rejected() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let url = write_archive_to_tar(
        dir.path(),
        "absolute.tar",
        &[
            ("/etc/passwd", b"bad"),
            ("manifest.json", test_manifest_json().as_bytes()),
            ("chunks.jsonl", test_chunks_jsonl().as_bytes()),
        ],
    );

    let result = nowdocs::registry::install("r1_absolute", &url);
    assert!(result.is_err(), "absolute path should be rejected");
    let err_str = format!("{}", result.unwrap_err());
    assert!(
        err_str.contains("ARCHIVE_UNSAFE_PATH"),
        "error should contain ARCHIVE_UNSAFE_PATH code, got: {}",
        err_str
    );
}

// --- R1: Windows absolute/drive-prefixed path rejected ---

#[test]
fn test_r1_windows_absolute_path_rejected() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let url = write_archive_to_tar(
        dir.path(),
        "win_absolute.tar",
        &[
            ("C:\\Windows\\System32\\cmd.exe", b"bad"),
            ("manifest.json", test_manifest_json().as_bytes()),
            ("chunks.jsonl", test_chunks_jsonl().as_bytes()),
        ],
    );

    let result = nowdocs::registry::install("r1_win_absolute", &url);
    assert!(result.is_err(), "Windows absolute path should be rejected");
    let err_str = format!("{}", result.unwrap_err());
    assert!(
        err_str.contains("ARCHIVE_UNSAFE_PATH"),
        "error should contain ARCHIVE_UNSAFE_PATH, got: {}",
        err_str
    );
}

#[test]
fn test_r1_windows_drive_prefix_rejected() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let url = write_archive_to_tar(
        dir.path(),
        "win_drive.tar",
        &[
            ("D:file.txt", b"bad"),
            ("manifest.json", test_manifest_json().as_bytes()),
            ("chunks.jsonl", test_chunks_jsonl().as_bytes()),
        ],
    );

    let result = nowdocs::registry::install("r1_win_drive", &url);
    assert!(
        result.is_err(),
        "Windows drive prefix path should be rejected"
    );
    let err_str = format!("{}", result.unwrap_err());
    assert!(
        err_str.contains("ARCHIVE_UNSAFE_PATH"),
        "error should contain ARCHIVE_UNSAFE_PATH, got: {}",
        err_str
    );
}

// --- R1: duplicate manifest rejected ---

#[test]
fn test_r1_duplicate_manifest_rejected() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let manifest_data = test_manifest_json().as_bytes();
    let url = write_archive_to_tar(
        dir.path(),
        "dup_manifest.tar",
        &[
            ("manifest.json", manifest_data),
            ("subdir/manifest.json", manifest_data),
            ("chunks.jsonl", test_chunks_jsonl().as_bytes()),
        ],
    );

    let result = nowdocs::registry::install("r1_dup_manifest", &url);
    assert!(result.is_err(), "duplicate manifest should be rejected");
    let err_str = format!("{}", result.unwrap_err());
    assert!(
        err_str.contains("ARCHIVE_DUPLICATE_ENTRY"),
        "error should contain ARCHIVE_DUPLICATE_ENTRY code, got: {}",
        err_str
    );
}

// --- R1: duplicate chunks rejected ---

#[test]
fn test_r1_duplicate_chunks_rejected() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let chunks_data = test_chunks_jsonl().as_bytes();
    let url = write_archive_to_tar(
        dir.path(),
        "dup_chunks.tar",
        &[
            ("manifest.json", test_manifest_json().as_bytes()),
            ("chunks.jsonl", chunks_data),
            ("copy/chunks.jsonl", chunks_data),
        ],
    );

    let result = nowdocs::registry::install("r1_dup_chunks", &url);
    assert!(result.is_err(), "duplicate chunks should be rejected");
    let err_str = format!("{}", result.unwrap_err());
    assert!(
        err_str.contains("ARCHIVE_DUPLICATE_ENTRY"),
        "error should contain ARCHIVE_DUPLICATE_ENTRY code, got: {}",
        err_str
    );
}

// --- R1: vector artifact rejected ---

#[test]
fn test_r1_vector_artifact_rejected() {
    // Share bundles (contributor artifacts) must not carry vector data. The
    // install path now uses RegistryRelease mode (which accepts `.lance`), so
    // the share-bundle rejection is exercised through the public
    // `validate_archive` contract (ShareBundle mode).
    let entries = vec![
        (
            "manifest.json".to_string(),
            test_manifest_json().as_bytes().to_vec(),
        ),
        (
            "chunks.jsonl".to_string(),
            test_chunks_jsonl().as_bytes().to_vec(),
        ),
        ("data.lance".to_string(), b"vector data".to_vec()),
    ];
    let result = nowdocs::registry::validate_archive(&entries);
    assert!(result.is_err(), "vector artifact should be rejected");
    assert_eq!(result.unwrap_err().code, "ARCHIVE_VECTOR_ARTIFACT");
}

// --- R1: .faiss artifact rejected ---

#[test]
fn test_r1_faiss_artifact_rejected() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let url = write_archive_to_tar(
        dir.path(),
        "faiss.tar",
        &[
            ("manifest.json", test_manifest_json().as_bytes()),
            ("chunks.jsonl", test_chunks_jsonl().as_bytes()),
            ("index.faiss", b"faiss data"),
        ],
    );

    let result = nowdocs::registry::install("r1_faiss", &url);
    assert!(result.is_err(), ".faiss artifact should be rejected");
    let err_str = format!("{}", result.unwrap_err());
    assert!(
        err_str.contains("ARCHIVE_VECTOR_ARTIFACT"),
        "error should contain ARCHIVE_VECTOR_ARTIFACT code, got: {}",
        err_str
    );
}

// --- R1: file inside .lance directory rejected (P2) ---

#[test]
fn test_r1_file_inside_lance_dir_rejected() {
    // LanceDB stores are directories: `mydb.lance/data.bin`. The child file
    // doesn't end in `.lance`, so the old check missed it. The share-bundle
    // contract (public `validate_archive`) still rejects it.
    let entries = vec![
        (
            "manifest.json".to_string(),
            test_manifest_json().as_bytes().to_vec(),
        ),
        (
            "chunks.jsonl".to_string(),
            test_chunks_jsonl().as_bytes().to_vec(),
        ),
        (
            "mydb.lance/data.bin".to_string(),
            b"vector data inside lance dir".to_vec(),
        ),
    ];
    let result = nowdocs::registry::validate_archive(&entries);
    assert!(
        result.is_err(),
        "file inside .lance directory should be rejected"
    );
    assert_eq!(result.unwrap_err().code, "ARCHIVE_VECTOR_ARTIFACT");
}

// --- R1: vectors.* artifact rejected ---

#[test]
fn test_r1_vectors_star_artifact_rejected() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let url = write_archive_to_tar(
        dir.path(),
        "vectors_star.tar",
        &[
            ("manifest.json", test_manifest_json().as_bytes()),
            ("chunks.jsonl", test_chunks_jsonl().as_bytes()),
            ("vectors.bin", b"vector data"),
        ],
    );

    let result = nowdocs::registry::install("r1_vectors_star", &url);
    assert!(result.is_err(), "vectors.* artifact should be rejected");
    let err_str = format!("{}", result.unwrap_err());
    assert!(
        err_str.contains("ARCHIVE_VECTOR_ARTIFACT"),
        "error should contain ARCHIVE_VECTOR_ARTIFACT code, got: {}",
        err_str
    );
}

// --- R1: embeddings.* artifact rejected ---

#[test]
fn test_r1_embeddings_star_artifact_rejected() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let url = write_archive_to_tar(
        dir.path(),
        "embeddings_star.tar",
        &[
            ("manifest.json", test_manifest_json().as_bytes()),
            ("chunks.jsonl", test_chunks_jsonl().as_bytes()),
            ("embeddings.bin", b"embedding data"),
        ],
    );

    let result = nowdocs::registry::install("r1_embeddings_star", &url);
    assert!(result.is_err(), "embeddings.* artifact should be rejected");
    let err_str = format!("{}", result.unwrap_err());
    assert!(
        err_str.contains("ARCHIVE_VECTOR_ARTIFACT"),
        "error should contain ARCHIVE_VECTOR_ARTIFACT code, got: {}",
        err_str
    );
}

// --- R1: duplicate NOTICES rejected ---

#[test]
fn test_r1_duplicate_notices_rejected() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let notices_data = b"notice text";
    let url = write_archive_to_tar(
        dir.path(),
        "dup_notices.tar",
        &[
            ("manifest.json", test_manifest_json().as_bytes()),
            ("chunks.jsonl", test_chunks_jsonl().as_bytes()),
            ("NOTICES", notices_data),
            ("subdir/NOTICES", notices_data),
        ],
    );

    let result = nowdocs::registry::install("r1_dup_notices", &url);
    assert!(result.is_err(), "duplicate NOTICES should be rejected");
    let err_str = format!("{}", result.unwrap_err());
    assert!(
        err_str.contains("ARCHIVE_DUPLICATE_ENTRY"),
        "error should contain ARCHIVE_DUPLICATE_ENTRY code, got: {}",
        err_str
    );
}

// --- R1: unsupported entry (symlink) rejected ---

#[test]
fn test_r1_symlink_entry_rejected() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let url = write_archive_to_tar_with_flags(
        dir.path(),
        "symlink.tar",
        &[
            ("manifest.json", test_manifest_json().as_bytes(), b'0'),
            ("chunks.jsonl", test_chunks_jsonl().as_bytes(), b'0'),
            // typeflag b'2' = symlink
            ("link_to_manifest", b"manifest.json", b'2'),
        ],
    );

    let result = nowdocs::registry::install("r1_symlink", &url);
    assert!(result.is_err(), "symlink entry should be rejected");
    let err_str = format!("{}", result.unwrap_err());
    assert!(
        err_str.contains("ARCHIVE_UNSUPPORTED_ENTRY"),
        "error should contain ARCHIVE_UNSUPPORTED_ENTRY code, got: {}",
        err_str
    );
}

// --- R1: oversized entry rejected ---

#[test]
fn test_r1_oversized_entry_rejected() {
    // Construct an entry that exceeds MAX_ENTRY_BYTES and verify rejection.
    let oversized = vec![0u8; nowdocs::registry::MAX_ENTRY_BYTES as usize + 1];
    let entries = vec![
        (
            "manifest.json".to_string(),
            test_manifest_json().as_bytes().to_vec(),
        ),
        (
            "chunks.jsonl".to_string(),
            test_chunks_jsonl().as_bytes().to_vec(),
        ),
        ("big.bin".to_string(), oversized),
    ];
    let result = nowdocs::registry::validate_archive(&entries);
    assert!(result.is_err(), "oversized entry should be rejected");
    assert_eq!(result.unwrap_err().code, "ARCHIVE_TOO_LARGE");
}

// --- R1: error display includes code and hint ---

#[test]
fn test_r1_error_display_includes_code_and_hint() {
    let err = nowdocs::errors::NowdocsError {
        code: "TEST_CODE",
        category: nowdocs::errors::ErrorCategory::Archive,
        message: "test message".to_string(),
        hint: "test hint".to_string(),
    };
    let display = format!("{}", err);
    assert!(display.contains("TEST_CODE"), "display should include code");
    assert!(display.contains("test hint"), "display should include hint");
    assert!(
        display.contains("next:"),
        "display should include next: prefix"
    );
}

// --- R1: valid archive still installs (regression) ---

#[test]
fn test_r1_valid_archive_installs() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test-docset";
    let archive = make_default_release("test-docset");
    let tar_path = dir.path().join("valid.tar");
    std::fs::write(&tar_path, &archive).unwrap();

    let url = format!("file://{}", tar_path.display());
    nowdocs::registry::install(docset, &url).unwrap();

    let mp = nowdocs::cache::manifest_path(docset);
    assert!(mp.is_file(), "manifest should be installed");

    let store = Store::open(docset).unwrap();
    let chunks = store.dump_chunks().unwrap();
    assert_eq!(chunks.len(), 2, "store should have 2 chunks");
}

// --- R1: validate_archive public API test ---

#[test]
fn test_r1_validate_archive_accepts_valid() {
    let entries = vec![
        (
            "manifest.json".to_string(),
            test_manifest_json().as_bytes().to_vec(),
        ),
        (
            "chunks.jsonl".to_string(),
            test_chunks_jsonl().as_bytes().to_vec(),
        ),
    ];
    let result = nowdocs::registry::validate_archive(&entries);
    assert!(
        result.is_ok(),
        "valid entries should pass: {:?}",
        result.err()
    );
}

#[test]
fn test_r1_validate_archive_rejects_missing_manifest() {
    let entries = vec![(
        "chunks.jsonl".to_string(),
        test_chunks_jsonl().as_bytes().to_vec(),
    )];
    let result = nowdocs::registry::validate_archive(&entries);
    assert!(result.is_err(), "missing manifest should fail");
    let err = result.unwrap_err();
    assert_eq!(err.code, "ARCHIVE_MISSING_MANIFEST");
}

#[test]
fn test_r1_validate_archive_rejects_missing_chunks() {
    let entries = vec![(
        "manifest.json".to_string(),
        test_manifest_json().as_bytes().to_vec(),
    )];
    let result = nowdocs::registry::validate_archive(&entries);
    assert!(result.is_err(), "missing chunks should fail");
    let err = result.unwrap_err();
    assert_eq!(err.code, "ARCHIVE_MISSING_CHUNKS");
}

// --- R2 Tests: Transactional install/update with rollback ---

#[test]
fn test_staging_path_stays_under_cache_root() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test_staging_path";
    let staging_path = cache::new_staging_path(docset);

    // Staging path should be under cache root
    assert!(
        cache::is_under_cache_root(&staging_path),
        "staging path should be under cache root, got: {:?}",
        staging_path
    );

    // Staging path should contain docset name
    assert!(
        staging_path.to_string_lossy().contains(docset),
        "staging path should contain docset name"
    );
}

#[test]
fn test_invalid_archive_install_leaves_no_active_manifest_or_store() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test_invalid_install";

    // Create an invalid archive (not a real tar)
    let invalid_archive = b"this is not a valid tar archive";
    let tar_path = dir.path().join("invalid.tar");
    std::fs::write(&tar_path, invalid_archive).unwrap();

    let url = format!("file://{}", tar_path.display());
    let result = nowdocs::registry::install(docset, &url);

    // Install should fail
    assert!(result.is_err(), "invalid archive should fail install");

    // No active manifest or store should be created
    let mp = cache::manifest_path(docset);
    let db = cache::db_path(docset);
    assert!(!mp.is_file(), "invalid install should not create manifest");
    assert!(
        !db.exists(),
        "invalid install should not create db directory"
    );
}

#[test]
fn test_successful_install_promotes_active_manifest_and_store() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test-docset";
    let archive = make_default_release("test-docset");
    let tar_path = dir.path().join("archive.tar");
    std::fs::write(&tar_path, &archive).unwrap();

    let url = format!("file://{}", tar_path.display());
    nowdocs::registry::install(docset, &url).unwrap();

    // Active manifest and store should exist
    let mp = cache::manifest_path(docset);
    let db = cache::db_path(docset);
    assert!(mp.is_file(), "successful install should create manifest");
    assert!(db.exists(), "successful install should create db directory");

    // Manifest should be valid
    let raw = std::fs::read_to_string(&mp).unwrap();
    let m = manifest::parse_manifest(&raw).unwrap();
    assert_eq!(m.docset, "test-docset");

    // Store should have chunks
    let store = Store::open(docset).unwrap();
    let chunks = store.dump_chunks().unwrap();
    assert_eq!(chunks.len(), 2, "store should have 2 chunks after install");
}

#[test]
fn test_failed_update_preserves_old_active_manifest_and_store() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test-docset";

    // First, install a valid docset
    let archive = make_default_release("test-docset");
    let tar_path = dir.path().join("good.tar");
    std::fs::write(&tar_path, &archive).unwrap();

    let url = format!("file://{}", tar_path.display());
    nowdocs::registry::install(docset, &url).unwrap();

    // Verify initial install
    let mp = cache::manifest_path(docset);
    let raw = std::fs::read_to_string(&mp).unwrap();
    let m = manifest::parse_manifest(&raw).unwrap();
    assert_eq!(m.doc_version, "1.0.0");

    // Now try to update with an invalid archive
    let invalid_archive = b"this is not a valid tar archive";
    let bad_tar_path = dir.path().join("bad.tar");
    std::fs::write(&bad_tar_path, invalid_archive).unwrap();

    unsafe {
        std::env::set_var(
            "NOWDOCS_TEST_URL",
            format!("file://{}", bad_tar_path.display()),
        )
    };
    let result = nowdocs::registry::update(docset);

    // Update should fail
    assert!(result.is_err(), "update with invalid archive should fail");

    // Old manifest should still be present and valid
    let raw = std::fs::read_to_string(&mp).unwrap();
    let m = manifest::parse_manifest(&raw).unwrap();
    assert_eq!(
        m.doc_version, "1.0.0",
        "old manifest should be preserved after failed update"
    );

    // Old store should still be accessible
    let store = Store::open(docset).unwrap();
    let chunks = store.dump_chunks().unwrap();
    assert_eq!(
        chunks.len(),
        2,
        "old store should be preserved after failed update"
    );
}

#[test]
fn test_successful_update_cleans_rollback() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test-docset";

    // First, install a valid docset
    let archive = make_default_release("test-docset");
    let tar_path = dir.path().join("good.tar");
    std::fs::write(&tar_path, &archive).unwrap();

    let url = format!("file://{}", tar_path.display());
    nowdocs::registry::install(docset, &url).unwrap();

    // Create a v2 registry release (chunk_count stays 2 → 2 rows in the table).
    let v2_json = test_manifest_json().replace("1.0.0", "2.0.0");
    let chunks = two_chunks();
    let vecs = zero_vectors(chunks.len());
    let v2_archive = make_registry_release(docset, &v2_json, &chunks, &vecs, None);

    let v2_tar_path = dir.path().join("v2.tar");
    std::fs::write(&v2_tar_path, &v2_archive).unwrap();

    // Update to v2
    unsafe {
        std::env::set_var(
            "NOWDOCS_TEST_URL",
            format!("file://{}", v2_tar_path.display()),
        )
    };
    nowdocs::registry::update(docset).unwrap();

    // Check that rollback directories are cleaned up
    let rollback_root = cache::cache_root().join("rollback");
    if rollback_root.exists() {
        let rollback_dirs: Vec<_> = std::fs::read_dir(&rollback_root)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(docset))
            .collect();
        assert!(
            rollback_dirs.is_empty(),
            "rollback directories should be cleaned up after successful update"
        );
    }

    // Verify update was successful
    let mp = cache::manifest_path(docset);
    let raw = std::fs::read_to_string(&mp).unwrap();
    let m = manifest::parse_manifest(&raw).unwrap();
    assert_eq!(m.doc_version, "2.0.0", "update should have applied v2.0.0");
}

#[test]
fn test_stale_staging_detection() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    // Create a stale staging directory
    let staging_root = cache::staging_root();
    std::fs::create_dir_all(&staging_root).unwrap();
    let stale_dir = staging_root.join("test-stale-123-456");
    std::fs::create_dir(&stale_dir).unwrap();

    // List staging directories
    let staging_dirs = cache::list_staging_dirs().unwrap();
    assert_eq!(
        staging_dirs.len(),
        1,
        "should detect one stale staging directory"
    );
    assert_eq!(
        staging_dirs[0], stale_dir,
        "should detect the correct stale staging directory"
    );
}

// ============================================================
// A1.1 — registry artifact contract / integrity / atomicity
// (spec: docs/superpowers/specs/2026-07-10-a1-registry-artifact-contract.md)
// ============================================================

fn epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// One JSONL chunk row (chunk_type "Info") for the M8 share-bundle tests.
fn chunk_line(idx: u32, text: &str) -> String {
    format!(
        r#"{{"idx":{idx},"heading_path":"H","source_url":"u","api_version":null,"chunk_type":"Info","text":"{text}"}}"#
    )
}

/// Build ShareBundle entries (`manifest.json` + `chunks.jsonl`) for the
/// contributor-side `validate_archive` contract.
fn share_entries(manifest_json: &str, chunks_jsonl: &str) -> Vec<(String, Vec<u8>)> {
    vec![
        (
            "manifest.json".to_string(),
            manifest_json.as_bytes().to_vec(),
        ),
        ("chunks.jsonl".to_string(), chunks_jsonl.as_bytes().to_vec()),
    ]
}

/// Build a corrupt "registry release": a valid manifest plus a `.lance` entry
/// that is NOT a real Lance table. RegistryRelease validation accepts it
/// (a `.lance` component is present), but `Store::open` / `row_count` fails
/// during promote, exercising the rollback path.
fn corrupt_release(docset: &str, manifest_json: &str) -> Vec<u8> {
    let mut archive = Vec::new();
    archive.extend_from_slice(&make_tar_entry("manifest.json", manifest_json.as_bytes()));
    archive.extend_from_slice(&make_tar_entry(
        &format!("{docset}.lance/junk.bin"),
        b"this is not a real lance table",
    ));
    archive.extend_from_slice(&[0u8; 512]);
    archive.extend_from_slice(&[0u8; 512]);
    archive
}

// --- S2: sha256 integrity ---

#[test]
fn install_accepts_tarball_with_correct_sha256() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test-docset";
    let archive = make_default_release(docset);
    let expected = nowdocs::registry::sha256_hex(&archive);

    let tar_path = dir.path().join("good.tar");
    std::fs::write(&tar_path, &archive).unwrap();
    let url = format!("file://{}", tar_path.display());

    nowdocs::registry::install_with_sha256(docset, &url, &expected)
        .expect("correct sha256 must install");

    let store = Store::open(docset).unwrap();
    assert_eq!(store.dump_chunks().unwrap().len(), 2);
}

#[test]
fn install_rejects_tarball_with_wrong_sha256() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test-docset";
    let archive = make_default_release(docset);
    let tar_path = dir.path().join("bad.tar");
    std::fs::write(&tar_path, &archive).unwrap();
    let url = format!("file://{}", tar_path.display());

    let wrong = "0".repeat(64);
    let result = nowdocs::registry::install_with_sha256(docset, &url, &wrong);
    assert!(result.is_err(), "wrong sha256 must be rejected");
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("ARCHIVE_SHA256_MISMATCH"),
        "error must carry ARCHIVE_SHA256_MISMATCH, got: {msg}"
    );
    // Integrity is verified before any active cache path is touched.
    assert!(
        !cache::manifest_path(docset).is_file(),
        "no active manifest on sha256 mismatch"
    );
    assert!(
        !cache::db_path(docset).exists(),
        "no active store on sha256 mismatch"
    );
}

// --- S1+S4: two archive types + atomic promote ---

#[test]
fn install_registry_release_accepts_lance_table() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test-docset";
    let archive = make_default_release(docset);
    let tar_path = dir.path().join("rel.tar");
    std::fs::write(&tar_path, &archive).unwrap();
    let url = format!("file://{}", tar_path.display());

    nowdocs::registry::install(docset, &url).expect("registry release must install");
    assert!(
        cache::db_path(docset).exists(),
        ".lance table promoted to active"
    );
    assert_eq!(Store::open(docset).unwrap().dump_chunks().unwrap().len(), 2);
}

#[test]
fn install_share_bundle_rejects_lance() {
    // The contributor/share contract (public `validate_archive` = ShareBundle
    // mode) must still reject vector artifacts.
    let entries = vec![
        (
            "manifest.json".to_string(),
            test_manifest_json().as_bytes().to_vec(),
        ),
        (
            "chunks.jsonl".to_string(),
            test_chunks_jsonl().as_bytes().to_vec(),
        ),
        ("db.lance".to_string(), b"vector data".to_vec()),
    ];
    let result = nowdocs::registry::validate_archive(&entries);
    assert!(result.is_err(), "share bundle must reject .lance");
    assert_eq!(result.unwrap_err().code, "ARCHIVE_VECTOR_ARTIFACT");
}

#[test]
fn install_after_promote_has_real_vectors() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test-docset";
    let chunks = two_chunks();
    // Distinct, non-zero vectors so the nearest-neighbour of v0 is chunk 0.
    let v0: Vec<f32> = (0..512).map(|i| 0.1 + (i as f32) * 0.001).collect();
    let v1: Vec<f32> = (0..512).map(|i| 2.0 - (i as f32) * 0.002).collect();
    let archive = make_registry_release(
        docset,
        test_manifest_json(),
        &chunks,
        &[v0.clone(), v1],
        None,
    );

    let tar_path = dir.path().join("realvec.tar");
    std::fs::write(&tar_path, &archive).unwrap();
    let url = format!("file://{}", tar_path.display());
    nowdocs::registry::install(docset, &url).unwrap();

    // The promoted store must carry the REAL vectors from the release (not the
    // old zero-vector materialization): querying with v0 returns chunk 0.
    let store = Store::open(docset).unwrap();
    let hits = store.vector_search(&v0, 1).unwrap();
    assert!(!hits.is_empty(), "vector search must return hits");
    assert_eq!(
        hits[0].chunk_idx, 0,
        "nearest neighbour of the inserted v0 must be chunk 0 (real vectors, not zeros)"
    );
}

#[test]
fn promote_uses_rename_not_copy() {
    // Build a real Lance table in a scratch cache, snapshot its ENTIRE file tree
    // (relative path -> bytes), then tar it as a registry release.
    fn snapshot_tree(root: &std::path::Path) -> Vec<(String, Vec<u8>)> {
        let mut out = Vec::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(d) = stack.pop() {
            for entry in std::fs::read_dir(&d).unwrap().flatten() {
                let p = entry.path();
                if p.is_dir() {
                    stack.push(p);
                } else {
                    let rel = p.strip_prefix(root).unwrap().to_string_lossy().into_owned();
                    let bytes = std::fs::read(&p).unwrap();
                    out.push((rel, bytes));
                }
            }
        }
        out.sort();
        out
    }

    let scratch = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", scratch.path()) };
    nowdocs::cache::ensure_layout().unwrap();
    let docset = "rnndoc";
    let src_lance = nowdocs::cache::db_path(docset);
    {
        let store = Store::open(docset).unwrap();
        let chunks = two_chunks();
        let vecs = zero_vectors(chunks.len());
        store.insert(&chunks, &vecs).unwrap();
    }
    let src_tree = snapshot_tree(&src_lance);
    assert!(
        !src_tree.is_empty(),
        "scratch .lance table must contain files"
    );

    // Manifest docset must match the install name (S7 identity binding). The
    // table was built under "rnndoc", so the manifest declares "rnndoc" too.
    let manifest =
        test_manifest_json().replace("\"docset\": \"test-docset\"", "\"docset\": \"rnndoc\"");

    // Tar manifest.json + the real <docset>.lance tree.
    let mut archive = Vec::new();
    archive.extend_from_slice(&make_tar_entry("manifest.json", manifest.as_bytes()));
    add_dir_to_tar(&mut archive, &src_lance, &format!("{docset}.lance"));
    archive.extend_from_slice(&[0u8; 512]);
    archive.extend_from_slice(&[0u8; 512]);

    // Install into a fresh active cache.
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };
    let tar_path = dir.path().join("rnn.tar");
    std::fs::write(&tar_path, &archive).unwrap();
    let url = format!("file://{}", tar_path.display());
    nowdocs::registry::install(docset, &url).unwrap();

    // Every prebuilt file must appear in the active db byte-for-byte. A
    // rebuild-from-zero would mint fresh UUIDs/timestamps; only a rename of the
    // original tree preserves the whole file set verbatim.
    let active_lance = nowdocs::cache::db_path(docset);
    for (rel, bytes) in &src_tree {
        let active_path = active_lance.join(rel);
        assert!(
            active_path.is_file(),
            "active db must carry prebuilt file {rel}"
        );
        let active_bytes = std::fs::read(&active_path).unwrap();
        assert_eq!(
            &active_bytes, bytes,
            "promote must rename (byte-identical), not rebuild, file {rel}"
        );
    }
}

#[test]
fn install_corrupt_lance_fails_without_modifying_active() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test-docset";
    // Seed a healthy v1.
    let good = make_default_release(docset);
    let good_path = dir.path().join("good.tar");
    std::fs::write(&good_path, &good).unwrap();
    nowdocs::registry::install(docset, &format!("file://{}", good_path.display())).unwrap();

    // Corrupt release: valid manifest, bogus .lance.
    let bad = corrupt_release(docset, test_manifest_json());
    let bad_path = dir.path().join("bad.tar");
    std::fs::write(&bad_path, &bad).unwrap();
    let result = nowdocs::registry::install(docset, &format!("file://{}", bad_path.display()));
    assert!(result.is_err(), "corrupt .lance must fail promote");

    // Original v1 store is untouched.
    let store = Store::open(docset).unwrap();
    let rows = store.dump_chunks().unwrap();
    assert_eq!(
        rows.len(),
        2,
        "active store must be preserved after corrupt install"
    );
    assert_eq!(rows[0].text, "hello");
    assert_eq!(rows[1].text, "world");
}

#[test]
fn promote_failure_restores_rollback() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test-docset";
    let good = make_default_release(docset);
    let good_path = dir.path().join("good.tar");
    std::fs::write(&good_path, &good).unwrap();
    nowdocs::registry::install(docset, &format!("file://{}", good_path.display())).unwrap();

    let mp = cache::manifest_path(docset);
    assert_eq!(
        manifest::parse_manifest(&std::fs::read_to_string(&mp).unwrap())
            .unwrap()
            .doc_version,
        "1.0.0"
    );

    // Failed promote (corrupt .lance) must restore the v1 manifest from rollback.
    let bad = corrupt_release(docset, test_manifest_json());
    let bad_path = dir.path().join("bad.tar");
    std::fs::write(&bad_path, &bad).unwrap();
    let _ = nowdocs::registry::install(docset, &format!("file://{}", bad_path.display()));

    let m = manifest::parse_manifest(&std::fs::read_to_string(&mp).unwrap()).unwrap();
    assert_eq!(
        m.doc_version, "1.0.0",
        "rollback must restore the previous active manifest after a failed promote"
    );
}

// --- M8: chunks.jsonl row-level validation (ShareBundle path) ---

#[test]
fn rejects_chunks_with_duplicate_idx() {
    let jsonl = format!("{}\n{}\n", chunk_line(0, "hello"), chunk_line(0, "world"));
    let result = nowdocs::registry::validate_archive(&share_entries(test_manifest_json(), &jsonl));
    assert!(result.is_err(), "duplicate idx must be rejected");
    assert_eq!(result.unwrap_err().code, "ARCHIVE_INVALID_CHUNKS");
}

#[test]
fn rejects_chunks_with_gap_in_idx() {
    let jsonl = format!("{}\n{}\n", chunk_line(0, "hello"), chunk_line(2, "world"));
    let result = nowdocs::registry::validate_archive(&share_entries(test_manifest_json(), &jsonl));
    assert!(result.is_err(), "gap in idx must be rejected");
    assert_eq!(result.unwrap_err().code, "ARCHIVE_INVALID_CHUNKS");
}

#[test]
fn rejects_chunks_count_mismatch() {
    // Manifest declares chunk_count 3, but only 2 rows are present.
    let manifest = test_manifest_json().replace("\"chunk_count\": 2", "\"chunk_count\": 3");
    let jsonl = format!("{}\n{}\n", chunk_line(0, "hello"), chunk_line(1, "world"));
    let result = nowdocs::registry::validate_archive(&share_entries(&manifest, &jsonl));
    assert!(result.is_err(), "count mismatch must be rejected");
    assert_eq!(result.unwrap_err().code, "ARCHIVE_INVALID_CHUNKS");
}

#[test]
fn rejects_empty_text_chunk() {
    let jsonl = format!("{}\n{}\n", chunk_line(0, "hello"), chunk_line(1, ""));
    let result = nowdocs::registry::validate_archive(&share_entries(test_manifest_json(), &jsonl));
    assert!(result.is_err(), "empty text must be rejected");
    assert_eq!(result.unwrap_err().code, "ARCHIVE_INVALID_CHUNKS");
}

#[test]
fn accepts_valid_chunks_jsonl() {
    let jsonl = format!("{}\n{}\n", chunk_line(0, "hello"), chunk_line(1, "world"));
    let result = nowdocs::registry::validate_archive(&share_entries(test_manifest_json(), &jsonl));
    assert!(
        result.is_ok(),
        "valid chunks.jsonl must pass: {:?}",
        result.err().map(|e| e.code)
    );
}

// --- M9: uninstall cleans docset-scoped leftovers ---

#[test]
fn uninstall_removes_license() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };
    let docset = "test-docset";
    let archive = make_default_release(docset);
    let tar_path = dir.path().join("a.tar");
    std::fs::write(&tar_path, &archive).unwrap();
    nowdocs::registry::install(docset, &format!("file://{}", tar_path.display())).unwrap();

    std::fs::write(cache::license_text_path(docset), "MIT body\n").unwrap();
    assert!(cache::license_text_path(docset).is_file());

    nowdocs::registry::uninstall(docset).unwrap();
    assert!(
        !cache::license_text_path(docset).exists(),
        "uninstall must remove the stashed license text"
    );
}

#[test]
fn uninstall_removes_staging() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };
    let docset = "test-docset";
    let archive = make_default_release(docset);
    let tar_path = dir.path().join("a.tar");
    std::fs::write(&tar_path, &archive).unwrap();
    nowdocs::registry::install(docset, &format!("file://{}", tar_path.display())).unwrap();

    let stale = cache::staging_root().join(format!("{docset}-123-456"));
    std::fs::create_dir_all(&stale).unwrap();
    std::fs::write(stale.join("leftover"), b"x").unwrap();

    nowdocs::registry::uninstall(docset).unwrap();
    assert!(!stale.exists(), "uninstall must remove docset staging dirs");
}

#[test]
fn uninstall_removes_rollback() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };
    let docset = "test-docset";
    let archive = make_default_release(docset);
    let tar_path = dir.path().join("a.tar");
    std::fs::write(&tar_path, &archive).unwrap();
    nowdocs::registry::install(docset, &format!("file://{}", tar_path.display())).unwrap();

    let rb = cache::rollback_root().join(format!("{docset}-123-456"));
    std::fs::create_dir_all(&rb).unwrap();
    std::fs::write(rb.join("db.lance"), b"x").unwrap();

    nowdocs::registry::uninstall(docset).unwrap();
    assert!(!rb.exists(), "uninstall must remove docset rollback dirs");
}

#[test]
fn uninstall_preserves_other_docsets() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    // Install docset A.
    let a = "test-docset";
    let archive = make_default_release(a);
    let tar_path = dir.path().join("a.tar");
    std::fs::write(&tar_path, &archive).unwrap();
    nowdocs::registry::install(a, &format!("file://{}", tar_path.display())).unwrap();

    // Staging leftovers for A and for an unrelated docset B.
    let a_staging = cache::staging_root().join(format!("{a}-1-2"));
    let b_staging = cache::staging_root().join("other-doc-1-2");
    std::fs::create_dir_all(&a_staging).unwrap();
    std::fs::create_dir_all(&b_staging).unwrap();

    nowdocs::registry::uninstall(a).unwrap();
    assert!(!a_staging.exists(), "A's staging dir must be removed");
    assert!(
        b_staging.exists(),
        "uninstall must NOT touch other docsets' staging dirs"
    );
}

// --- M10: share output must be a clean dir ---

#[test]
fn share_rejects_non_empty_output_dir() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test_share_ne";
    write_test_manifest(dir.path(), docset);
    populate_test_store(docset);

    let out_dir = dir.path().join("share_out");
    // First share populates <out_dir>/<docset>.
    nowdocs::registry::share(docset, &out_dir).unwrap();
    // Second share to the same (now non-empty) dir must be rejected.
    let result = nowdocs::registry::share(docset, &out_dir);
    assert!(
        result.is_err(),
        "share into a non-empty dir must be rejected"
    );
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("non-empty"),
        "error must explain non-empty output dir, got: {msg}"
    );
}

#[test]
fn share_creates_clean_output_in_empty_dir() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test_share_clean";
    write_test_manifest(dir.path(), docset);
    populate_test_store(docset);

    let out_dir = dir.path().join("share_out_empty");
    let share_path = nowdocs::registry::share(docset, &out_dir).unwrap();
    assert!(share_path.join("manifest.json").is_file());
    assert!(share_path.join("chunks.jsonl").is_file());
}

// --- N6: per-docset install lock ---

#[test]
fn concurrent_install_same_docset_second_fails() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };
    cache::ensure_layout().unwrap();

    let docset = "test-docset";
    // Fresh lock (current epoch) → not stale → busy.
    let lock_path = cache::staging_root().join(format!("{docset}.lock"));
    std::fs::write(&lock_path, format!("{}\n", epoch_secs())).unwrap();

    let archive = make_default_release(docset);
    let tar_path = dir.path().join("a.tar");
    std::fs::write(&tar_path, &archive).unwrap();
    let url = format!("file://{}", tar_path.display());

    let result = nowdocs::registry::install(docset, &url);
    assert!(result.is_err(), "concurrent install must be rejected");
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("currently being installed"),
        "error must explain the busy lock, got: {msg}"
    );
}

#[test]
fn stale_lockfile_older_than_1hr_is_ignored() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };
    cache::ensure_layout().unwrap();

    let docset = "test-docset";
    // Epoch 0 → far older than 1 hour → stale → replaced, install proceeds.
    let lock_path = cache::staging_root().join(format!("{docset}.lock"));
    std::fs::write(&lock_path, "0\n").unwrap();

    // Manifest-only archive has no .lance → fails with ARCHIVE_MISSING_STORE,
    // but crucially NOT with the busy-lock error.
    let mut archive = Vec::new();
    archive.extend_from_slice(&make_tar_entry(
        "manifest.json",
        test_manifest_json().as_bytes(),
    ));
    archive.extend_from_slice(&[0u8; 512]);
    archive.extend_from_slice(&[0u8; 512]);
    let tar_path = dir.path().join("mo.tar");
    std::fs::write(&tar_path, &archive).unwrap();
    let url = format!("file://{}", tar_path.display());

    let result = nowdocs::registry::install(docset, &url);
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(
        !msg.contains("currently being installed"),
        "stale lock must not trigger the busy error, got: {msg}"
    );
}

#[test]
fn lockfile_removed_after_install_success() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test-docset";
    let archive = make_default_release(docset);
    let tar_path = dir.path().join("a.tar");
    std::fs::write(&tar_path, &archive).unwrap();
    nowdocs::registry::install(docset, &format!("file://{}", tar_path.display())).unwrap();

    let lock_path = cache::staging_root().join(format!("{docset}.lock"));
    assert!(
        !lock_path.exists(),
        "lockfile must be removed after a successful install"
    );
}

#[test]
fn lockfile_removed_after_install_failure() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let docset = "test-docset";
    // Manifest-only archive → install fails (ARCHIVE_MISSING_STORE).
    let mut archive = Vec::new();
    archive.extend_from_slice(&make_tar_entry(
        "manifest.json",
        test_manifest_json().as_bytes(),
    ));
    archive.extend_from_slice(&[0u8; 512]);
    archive.extend_from_slice(&[0u8; 512]);
    let tar_path = dir.path().join("mo.tar");
    std::fs::write(&tar_path, &archive).unwrap();
    let _ = nowdocs::registry::install(docset, &format!("file://{}", tar_path.display()));

    let lock_path = cache::staging_root().join(format!("{docset}.lock"));
    assert!(
        !lock_path.exists(),
        "lockfile must be removed after a failed install (Drop on the guard)"
    );
}

// --- OQ6: URL gate hardening ---

#[test]
fn registry_nowdocs_dev_requires_path_prefix() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    // registry.nowdocs.dev WITHOUT the required /releases/ prefix → rejected
    // at the URL gate (offline, before any network call).
    let result = nowdocs::registry::install("mypkg", "https://registry.nowdocs.dev/mypkg.tar");
    assert!(
        result.is_err(),
        "missing /releases/ prefix must be rejected"
    );
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("not in allowed domains"),
        "error must cite the allowlist, got: {msg}"
    );
}
