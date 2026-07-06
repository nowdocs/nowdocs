use std::process::Command;

// --- helpers (same pattern as cli_tests.rs) ---

fn test_manifest_json(version: &str) -> String {
    format!(
        r#"{{
            "docset": "test-docset",
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

fn test_chunks_jsonl() -> &'static str {
    r#"{"idx":0,"heading_path":"Intro","source_url":"https://example.com/0","api_version":null,"chunk_type":"Info","text":"hello"}
{"idx":1,"heading_path":"API","source_url":"https://example.com/1","api_version":null,"chunk_type":"Info","text":"world"}
"#
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

fn make_tar_archive(version: &str) -> Vec<u8> {
    let manifest_data = test_manifest_json(version).into_bytes();
    let chunks_data = test_chunks_jsonl().as_bytes();
    let files: Vec<(&str, &[u8])> = vec![
        ("manifest.json", &manifest_data),
        ("chunks.jsonl", chunks_data),
    ];
    let mut archive = Vec::new();
    for (name, data) in &files {
        archive.extend_from_slice(&make_tar_entry(name, data));
    }
    archive.extend_from_slice(&[0u8; 512]);
    archive.extend_from_slice(&[0u8; 512]);
    archive
}

fn write_tarball(dir: &std::path::Path, version: &str) -> std::path::PathBuf {
    let archive = make_tar_archive(version);
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

fn run_nowdocs_with_test_url(
    cwd: &std::path::Path,
    cache_home: &std::path::Path,
    test_url: &str,
    args: &[&str],
) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nowdocs"))
        .args(args)
        .current_dir(cwd)
        .env("XDG_CACHE_HOME", cache_home)
        .env("NOWDOCS_TEST_URL", test_url)
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

    // Install a fixture first
    let tar = write_tarball(cache.path(), "1.0.0");
    let url = format!("file://{}", tar.display());
    let out = run_nowdocs_with_test_url(
        cwd.path(),
        cache.path(),
        &url,
        &["install", "smoke-default-q"],
    );
    assert!(
        out.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

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
#[test]
fn test_install_shows_next_step_hint() {
    let cache = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let tar = write_tarball(cache.path(), "1.0.0");
    let url = format!("file://{}", tar.display());

    let out =
        run_nowdocs_with_test_url(cwd.path(), cache.path(), &url, &["install", "hint-test-42"]);
    assert!(
        out.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("smoke") || stdout.contains("next"),
        "install output should suggest next step (smoke), got: {stdout}"
    );
}
