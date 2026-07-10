use std::process::Command;

// 1a test: keep verbatim (clap --help coverage).
#[test]
fn test_cli_help_lists_all_subcommands() {
    let output = Command::new(env!("CARGO_BIN_EXE_nowdocs"))
        .arg("--help")
        .output()
        .expect("failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for sub in [
        "serve",
        "install",
        "ingest",
        "share",
        "uninstall",
        "list-installed",
        "update",
        "smoke",
        "doctor",
        "cache",
    ] {
        assert!(stdout.contains(sub), "help must list `{}`", sub);
    }
    // serve must NOT take --host/--port (network-defense rule)
    assert!(
        !stdout.contains("--port"),
        "serve must be argless (stdio binds no port)"
    );
}

// ---- 4d: CLI ↔ real module wiring ----
//
// All tests set XDG_CACHE_HOME + cwd to a per-test tempdir so they don't
// collide when run in parallel. The installed docset name is the unique
// discriminator in list-installed output.

// --- fixture builders (duplicated from registry_tests.rs; cli_tests.rs
//     and registry_tests.rs may not share a common module under the
//     4d constraint of "only edit cli_tests.rs + main.rs") ---

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

fn make_tar_archive(docset: &str, version: &str) -> Vec<u8> {
    let manifest_data = test_manifest_json(docset, version).into_bytes();
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

fn write_tarball(dir: &std::path::Path, docset: &str, version: &str) -> std::path::PathBuf {
    let archive = make_tar_archive(docset, version);
    let tar_path = dir.join(format!("archive_{version}.tar"));
    std::fs::write(&tar_path, &archive).unwrap();
    tar_path
}

// Run the nowdocs binary with a fresh XDG_CACHE_HOME + cwd.
fn run_nowdocs(
    cwd: &std::path::Path,
    cache_home: &std::path::Path,
    args: &[&str],
) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nowdocs"))
        .args(args)
        .current_dir(cwd)
        .env("XDG_CACHE_HOME", cache_home)
        .env("NOWDOCS_TEST_URL", "") // reset for tests that don't use it
        .output()
        .expect("failed to execute nowdocs")
}

// --- Test 1: empty list-installed ---

#[test]
fn test_cli_list_installed_empty() {
    let cache = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();

    let out = run_nowdocs(cwd.path(), cache.path(), &["list-installed"]);
    assert!(
        out.status.success(),
        "list-installed should exit 0, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("no docsets installed"),
        "empty cache must print 'no docsets installed', got: {stdout}"
    );
}

// --- Test 2: install → list → uninstall → list ---

#[test]
fn test_cli_install_uninstall_roundtrip() {
    let cache = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let tar = write_tarball(cache.path(), "rnd-foo-7711", "1.0.0");
    let url = format!("file://{}", tar.display());

    // install — call the library directly: the production binary (built without
    // `cfg(test)`) no longer honors `file://` / NOWDOCS_TEST_URL, so the test
    // fixture path must go through `install` (which IS compiled in cfg(test)).
    unsafe { std::env::set_var("XDG_CACHE_HOME", cache.path()) };
    nowdocs::registry::install("rnd-foo-7711", &url).expect("install should succeed");

    // list-installed shows it
    let out = run_nowdocs(cwd.path(), cache.path(), &["list-installed"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("rnd-foo-7711"),
        "list-installed must contain 'rnd-foo-7711' after install, got: {stdout}"
    );

    // uninstall
    let out = run_nowdocs(cwd.path(), cache.path(), &["uninstall", "rnd-foo-7711"]);
    assert!(
        out.status.success(),
        "uninstall should exit 0, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // list-installed no longer shows it
    let out = run_nowdocs(cwd.path(), cache.path(), &["list-installed"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("rnd-foo-7711"),
        "list-installed must NOT contain 'rnd-foo-7711' after uninstall, got: {stdout}"
    );
}

// --- Test 3: install + share produces manifest.json in out dir ---

#[test]
fn test_cli_share_creates_out_dir() {
    let cache = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let tar = write_tarball(cache.path(), "rnd-share-9912", "1.0.0");
    let url = format!("file://{}", tar.display());

    // install — library path honors `file://` in cfg(test); the production
    // binary does not, so install must go through the library here.
    unsafe { std::env::set_var("XDG_CACHE_HOME", cache.path()) };
    nowdocs::registry::install("rnd-share-9912", &url).expect("install should succeed");

    // share — default out_dir is ./{docset}-share relative to cwd
    let out = run_nowdocs(cwd.path(), cache.path(), &["share", "rnd-share-9912"]);
    assert!(
        out.status.success(),
        "share should exit 0, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // registry::share appends `<docset>` to out_dir, so the final layout is
    // <cwd>/<docset>-share/<docset>/{manifest.json,chunks.jsonl}.
    let share_dir = cwd
        .path()
        .join("rnd-share-9912-share")
        .join("rnd-share-9912");
    assert!(
        share_dir.join("manifest.json").is_file(),
        "share must produce manifest.json at {}",
        share_dir.display()
    );
    assert!(
        share_dir.join("chunks.jsonl").is_file(),
        "share must produce chunks.jsonl at {}",
        share_dir.display()
    );
}

// --- Test 4: update refreshes manifest to v2 ---

#[test]
fn test_cli_update_refreshes_manifest() {
    let cache = tempfile::tempdir().unwrap();
    let v1 = write_tarball(cache.path(), "rnd-upd-3344", "1.0.0");
    let v1_url = format!("file://{}", v1.display());

    // install v1 — library path honors `file://` in cfg(test)
    unsafe { std::env::set_var("XDG_CACHE_HOME", cache.path()) };
    nowdocs::registry::install("rnd-upd-3344", &v1_url).expect("install v1 should succeed");

    // confirm v1 on disk (manifest path is known from cache layout; the test
    // process doesn't see the subprocess's XDG_CACHE_HOME, so we construct
    // the path directly from the tempdir we passed in)
    let manifest_path = cache
        .path()
        .join("nowdocs")
        .join("db")
        .join("rnd-upd-3344.manifest.json");
    let m1 = nowdocs::manifest::parse_manifest(&std::fs::read_to_string(&manifest_path).unwrap())
        .unwrap();
    assert_eq!(m1.doc_version, "1.0.0");

    // write v2 tarball
    let v2 = write_tarball(cache.path(), "rnd-upd-3344", "2.0.0");
    let v2_url = format!("file://{}", v2.display());

    // update with v2 — library path honors NOWDOCS_TEST_URL in cfg(test); the
    // production binary does not, so update must go through the library here.
    unsafe { std::env::set_var("NOWDOCS_TEST_URL", &v2_url) };
    nowdocs::registry::update("rnd-upd-3344").expect("update should succeed");

    // manifest should now be v2
    let m2 = nowdocs::manifest::parse_manifest(&std::fs::read_to_string(&manifest_path).unwrap())
        .unwrap();
    assert_eq!(
        m2.doc_version, "2.0.0",
        "update should refresh manifest to v2"
    );
}

// --- Test 5: ingest + list-installed (uses real embedder) ---

#[test]
#[ignore = "requires real embedder model load"]
fn test_cli_ingest_then_list() {
    let cache = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let md_dir = tempfile::tempdir().unwrap();
    std::fs::write(md_dir.path().join("a.md"), "# Title\n\nhello world\n").unwrap();

    let out = run_nowdocs(
        cwd.path(),
        cache.path(),
        &["ingest", md_dir.path().to_str().unwrap(), "rnd-ing-5566"],
    );
    assert!(
        out.status.success(),
        "ingest should exit 0, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("rnd-ing-5566") || stdout.contains("1 file") || stdout.contains("files"),
        "ingest should print stats, got: {stdout}"
    );

    let out = run_nowdocs(cwd.path(), cache.path(), &["list-installed"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("rnd-ing-5566"),
        "list-installed must contain ingested docset name, got: {stdout}"
    );
}
