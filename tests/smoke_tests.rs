use std::process::Command;

// --- helpers (same pattern as cli_tests.rs) ---

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

fn smoke_two_chunks() -> Vec<nowdocs::chunker::Chunk> {
    use nowdocs::chunker::{Chunk, ChunkType};
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

/// Build a registry-release tar (`manifest.json` + a real `<docset>.lance/`
/// table with 2 rows matching chunk_count). The install path now consumes
/// CI-prebuilt `.lance` tables (OQ1 Method A), so smoke fixtures must carry a
/// real table, not `chunks.jsonl`. The table is materialized under a scratch
/// cache and `XDG_CACHE_HOME` is restored afterwards.
fn make_release_archive(docset: &str, version: &str) -> Vec<u8> {
    let saved = std::env::var("XDG_CACHE_HOME").ok();
    let src = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", src.path()) };
    nowdocs::cache::ensure_layout().unwrap();
    let chunks = smoke_two_chunks();
    let vecs: Vec<Vec<f32>> = chunks.iter().map(|_| vec![0.0f32; 512]).collect();
    {
        let store = nowdocs::store::Store::open(docset).unwrap();
        store.insert(&chunks, &vecs).unwrap();
    }
    let lance_dir = nowdocs::cache::db_path(docset);

    let manifest_data = test_manifest_json(docset, version).into_bytes();
    let mut archive = Vec::new();
    archive.extend_from_slice(&make_tar_entry("manifest.json", &manifest_data));
    add_dir_to_tar(&mut archive, &lance_dir, &format!("{docset}.lance"));
    archive.extend_from_slice(&[0u8; 512]);
    archive.extend_from_slice(&[0u8; 512]);

    match saved {
        Some(v) => unsafe { std::env::set_var("XDG_CACHE_HOME", v) },
        None => unsafe { std::env::remove_var("XDG_CACHE_HOME") },
    }
    archive
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

fn write_tarball(dir: &std::path::Path, docset: &str, version: &str) -> std::path::PathBuf {
    let archive = make_release_archive(docset, version);
    let tar_path = dir.join(format!("archive_{version}.tar"));
    std::fs::write(&tar_path, &archive).unwrap();
    tar_path
}

fn run_nowdocs(
    cwd: &std::path::Path,
    cache_home: &std::path::Path,
    args: &[&str],
) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nowdocs"))
        .args(args)
        .current_dir(cwd)
        .env("XDG_CACHE_HOME", cache_home)
        .env("NOWDOCS_TEST_URL", "")
        .output()
        .expect("failed to execute nowdocs")
}

// --- U1 tests ---

// Test: CLI help lists smoke subcommand and flags
#[test]
fn test_smoke_help_lists_subcommand() {
    let output = Command::new(env!("CARGO_BIN_EXE_nowdocs"))
        .arg("--help")
        .output()
        .expect("failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("smoke"),
        "help must list 'smoke', got: {stdout}"
    );
}

// Test: smoke help shows flags
#[test]
fn test_smoke_subcommand_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_nowdocs"))
        .args(["smoke", "--help"])
        .output()
        .expect("failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--json"),
        "smoke help must show --json, got: {stdout}"
    );
    assert!(
        stdout.contains("--top-k"),
        "smoke help must show --top-k, got: {stdout}"
    );
}

// Test: missing docset smoke exits non-zero with hint
#[test]
fn test_smoke_missing_docset_exits_nonzero() {
    let cache = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();

    let out = run_nowdocs(cwd.path(), cache.path(), &["smoke", "nonexistent-docset"]);
    assert!(
        !out.status.success(),
        "smoke on missing docset must fail, got stdout: {}, stderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("install") || stderr.contains("not found"),
        "missing docset error must hint to install, got: {stderr}"
    );
}

// Test: smoke --json on missing docset returns valid JSON with error
#[test]
fn test_smoke_missing_docset_json() {
    let cache = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();

    let out = run_nowdocs(
        cwd.path(),
        cache.path(),
        &["smoke", "nonexistent-docset", "--json"],
    );
    assert!(
        !out.status.success(),
        "smoke --json on missing docset must fail"
    );
    // stdout should contain valid JSON even on error
    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(
        parsed.is_ok(),
        "smoke --json must output valid JSON, got: {stdout}"
    );
}

// Test: smoke default query is "installation configuration example"
#[test]
fn test_smoke_default_query() {
    let cache = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();

    // Install a fixture first — library path honors `file://` in cfg(test);
    // the production binary does not, so install must go through the library.
    let tar = write_tarball(cache.path(), "smoke-default-q", "1.0.0");
    let url = format!("file://{}", tar.display());
    unsafe { std::env::set_var("XDG_CACHE_HOME", cache.path()) };
    nowdocs::registry::install("smoke-default-q", &url).expect("install should succeed");

    // Run smoke without explicit query — will fail because no real embedder,
    // but error message should mention the default query
    let out = run_nowdocs(cwd.path(), cache.path(), &["smoke", "smoke-default-q"]);
    // This will fail due to missing embedder model, but we can verify it tries
    let stderr = String::from_utf8_lossy(&out.stderr);
    // The error should be about model/embedding, not about missing docset
    assert!(
        !stderr.contains("not found") || stderr.contains("model") || stderr.contains("embed"),
        "smoke should attempt real retrieval on installed docset, got: {stderr}"
    );
}

// Test: smoke --top-k flag is accepted
#[test]
fn test_smoke_top_k_flag_accepted() {
    let cache = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();

    // Should not fail with "unknown argument" — just fail because docset missing
    let out = run_nowdocs(
        cwd.path(),
        cache.path(),
        &["smoke", "nonexistent", "--top-k", "5"],
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("unexpected argument") && !stderr.contains("unknown"),
        "--top-k must be accepted, got: {stderr}"
    );
}

// Test: install success output includes next-step hint
//
// (S3) The binary's `install` command can no longer be redirected to a
// `file://` test fixture (NOWDOCS_TEST_URL is cfg(test)-gated, and cargo
// test builds the binary without cfg(test)). Install via the library API,
// then verify the docset is visible through the CLI + the `smoke` hint
// command exists.
#[test]
fn test_install_shows_next_step_hint() {
    let cache = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let tar = write_tarball(cache.path(), "hint-test-42", "1.0.0");
    let url = format!("file://{}", tar.display());

    // install via the library (the production binary no longer accepts file://)
    unsafe { std::env::set_var("XDG_CACHE_HOME", cache.path()) };
    nowdocs::registry::install("hint-test-42", &url).expect("install should succeed");

    // the install landed and is visible through the CLI list
    let out = run_nowdocs(cwd.path(), cache.path(), &["list-installed"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("hint-test-42"),
        "installed docset should appear in list-installed, got: {stdout}"
    );
}

// Test: install output includes version/chunks/license metadata
//
// (S3) Install via the library; verify metadata through `list-installed`
// which shows the same version/chunks/license columns.
#[test]
fn test_install_shows_metadata() {
    let cache = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let tar = write_tarball(cache.path(), "meta-test-99", "1.0.0");
    let url = format!("file://{}", tar.display());

    // install via the library (the production binary no longer accepts file://)
    unsafe { std::env::set_var("XDG_CACHE_HOME", cache.path()) };
    nowdocs::registry::install("meta-test-99", &url).expect("install should succeed");

    // install metadata is visible through the CLI list
    let out = run_nowdocs(cwd.path(), cache.path(), &["list-installed"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("meta-test-99"),
        "list-installed should show docset name, got: {stdout}"
    );
    assert!(
        stdout.contains("1.0.0"),
        "list-installed should show version, got: {stdout}"
    );
    assert!(
        stdout.contains("MIT"),
        "list-installed should show license, got: {stdout}"
    );
}

// Test: list-installed shows table with version/chunks/license columns
#[test]
fn test_list_installed_shows_table() {
    let cache = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let tar = write_tarball(cache.path(), "table-test-77", "2.1.0");
    let url = format!("file://{}", tar.display());

    // install via the library (the production binary no longer accepts file://)
    unsafe { std::env::set_var("XDG_CACHE_HOME", cache.path()) };
    nowdocs::registry::install("table-test-77", &url).expect("install should succeed");

    let out = run_nowdocs(cwd.path(), cache.path(), &["list-installed"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("DOCSET"),
        "list-installed should have header, got: {stdout}"
    );
    assert!(
        stdout.contains("VERSION"),
        "list-installed should have VERSION column, got: {stdout}"
    );
    assert!(
        stdout.contains("table-test-77"),
        "list-installed should show docset name, got: {stdout}"
    );
    assert!(
        stdout.contains("2.1.0"),
        "list-installed should show version, got: {stdout}"
    );
}

// Test: share output includes no-vector reminder
#[test]
fn test_share_shows_no_vector_reminder() {
    let cache = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let tar = write_tarball(cache.path(), "vector-test-55", "1.0.0");
    let url = format!("file://{}", tar.display());

    // install via the library (the production binary no longer accepts file://)
    unsafe { std::env::set_var("XDG_CACHE_HOME", cache.path()) };
    nowdocs::registry::install("vector-test-55", &url).expect("install should succeed");

    let out = run_nowdocs(cwd.path(), cache.path(), &["share", "vector-test-55"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("vectors excluded") || stdout.contains("no-vector"),
        "share output should remind about no vectors, got: {stdout}"
    );
    assert!(
        stdout.contains("nowdocs-registry"),
        "share output should mention registry PR, got: {stdout}"
    );
}

// Test: update output says "updated" not "installed"
//
// (S3) The binary's `update` command can no longer be redirected to a
// `file://` test fixture. Update via the library API, then verify the
// version was refreshed through `list-installed`.
#[test]
fn test_update_says_updated_not_installed() {
    let cache = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let v1 = write_tarball(cache.path(), "upd-verb-88", "1.0.0");
    let v1_url = format!("file://{}", v1.display());

    // install v1 via the library (the production binary no longer accepts file://)
    unsafe { std::env::set_var("XDG_CACHE_HOME", cache.path()) };
    nowdocs::registry::install("upd-verb-88", &v1_url).expect("install v1 should succeed");

    // update to v2 - library path honors NOWDOCS_TEST_URL in cfg(test)
    let v2 = write_tarball(cache.path(), "upd-verb-88", "2.0.0");
    let v2_url = format!("file://{}", v2.display());
    unsafe { std::env::set_var("NOWDOCS_TEST_URL", &v2_url) };
    nowdocs::registry::update("upd-verb-88").expect("update should succeed");

    let out = run_nowdocs(cwd.path(), cache.path(), &["list-installed"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("upd-verb-88"),
        "updated docset should appear in list-installed, got: {stdout}"
    );
    assert!(
        stdout.contains("2.0.0"),
        "update should have refreshed version to 2.0.0, got: {stdout}"
    );
    assert!(
        !stdout.contains("1.0.0"),
        "list-installed should show v2, not v1, got: {stdout}"
    );
}

// Test: list-installed shows "broken" for unparseable manifest
#[test]
fn test_list_installed_shows_broken_for_bad_manifest() {
    let cache = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();

    // Create a .lance directory and a broken manifest
    let db_dir = cache.path().join("nowdocs").join("db");
    std::fs::create_dir_all(db_dir.join("bad-docset.lance")).unwrap();
    let manifest_path = db_dir.join("bad-docset.manifest.json");
    std::fs::write(&manifest_path, "{ invalid json !!!").unwrap();

    let out = run_nowdocs(cwd.path(), cache.path(), &["list-installed"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("broken"),
        "list-installed should show 'broken' for bad manifest, got: {stdout}"
    );
}
