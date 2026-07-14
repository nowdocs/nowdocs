use std::process::Command;
use std::sync::Mutex;

// Serialize env mutation in this test binary so parallel tests don't race on
// XDG_CACHE_HOME (a pre-existing source of flakiness for install/update/share
// helpers that call the library in-process).
static ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvGuard {
    key: &'static str,
    old: Option<String>,
    _g: std::sync::MutexGuard<'static, ()>,
}

impl EnvGuard {
    fn set(key: &'static str, val: &str) -> Self {
        let g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let old = std::env::var(key).ok();
        std::env::set_var(key, val);
        Self { key, old, _g: g }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.old {
            Some(v) => std::env::set_var(self.key, v),
            None => std::env::remove_var(self.key),
        }
    }
}

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
        "capabilities",
        "ensure",
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

fn cli_two_chunks() -> Vec<nowdocs::chunker::Chunk> {
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

/// Build a registry-release tar (`manifest.json` + a real `<docset>.lance/`
/// table with 2 rows matching chunk_count). The table is materialized under a
/// scratch cache; `XDG_CACHE_HOME` is saved/restored manually. The caller must
/// hold `ENV_LOCK`.
fn make_release_archive_locked(docset: &str, version: &str) -> Vec<u8> {
    let saved = std::env::var("XDG_CACHE_HOME").ok();
    let src = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", src.path()) };
    nowdocs::cache::ensure_layout().unwrap();
    let chunks = cli_two_chunks();
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

fn write_tarball(dir: &std::path::Path, docset: &str, version: &str) -> std::path::PathBuf {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let archive = make_release_archive_locked(docset, version);
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

// --- Test: list-installed shows a STATUS column with the unified state label (M22) ---

#[test]
fn test_cli_list_installed_shows_state_column() {
    let cache = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let tar = write_tarball(cache.path(), "state-col-22", "1.0.0");
    let url = format!("file://{}", tar.display());

    // install via the library (production binary rejects file://); write_tarball
    // ships a 2-row store + manifest chunk_count=2, so the unified state model
    // classifies it as Healthy -> label "ok".
    let _g = EnvGuard::set("XDG_CACHE_HOME", cache.path().to_str().unwrap());
    nowdocs::registry::install("state-col-22", &url).expect("install should succeed");

    let out = run_nowdocs(cwd.path(), cache.path(), &["list-installed"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("STATUS"),
        "list-installed must show a STATUS column header, got: {stdout}"
    );
    assert!(
        stdout.contains("state-col-22"),
        "list-installed must list the docset, got: {stdout}"
    );
    // Healthy install must surface the unified "ok" state label from
    // InstalledDocsetState::label().
    assert!(
        stdout.contains(" ok") || stdout.contains("\tok") || stdout.ends_with("ok"),
        "healthy docset must show state label 'ok', got: {stdout}"
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
    let _g = EnvGuard::set("XDG_CACHE_HOME", cache.path().to_str().unwrap());
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
    let _g = EnvGuard::set("XDG_CACHE_HOME", cache.path().to_str().unwrap());
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
    // Build v2 tarball before acquiring the env guard; write_tarball acquires
    // ENV_LOCK and must not be called while the guard is held on this thread.
    let v2 = write_tarball(cache.path(), "rnd-upd-3344", "2.0.0");
    let v2_url = format!("file://{}", v2.display());

    // install v1 — library path honors `file://` in cfg(test)
    let _g = EnvGuard::set("XDG_CACHE_HOME", cache.path().to_str().unwrap());
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

// ---- C1: agent contract — capabilities command ----
//
// These tests run the binary with per-test temporary HOME, XDG cache,
// XDG config, and working directories so they never touch real user
// configuration. Deliberately unusable proxy variables make any accidental
// network attempt fail fast instead of reaching the network.

fn run_nowdocs_isolated(root: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nowdocs"))
        .args(args)
        .current_dir(root)
        .env("HOME", root)
        .env("XDG_CACHE_HOME", root)
        .env("XDG_CONFIG_HOME", root)
        .env("XDG_DATA_HOME", root)
        .env("TMPDIR", root)
        .env("NOWDOCS_TEST_URL", "") // reset for tests that don't use it
        .env("http_proxy", "http://127.0.0.1:9")
        .env("https_proxy", "http://127.0.0.1:9")
        .env("HTTP_PROXY", "http://127.0.0.1:9")
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .env("ALL_PROXY", "http://127.0.0.1:9")
        .env("no_proxy", "")
        .env("NO_PROXY", "")
        .output()
        .expect("failed to execute nowdocs")
}

#[test]
fn test_capabilities_json_uses_agent_envelope() {
    let root = tempfile::tempdir().unwrap();
    let out = run_nowdocs_isolated(root.path(), &["capabilities", "--json"]);
    assert!(
        out.status.success(),
        "capabilities --json should exit 0, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.stderr.is_empty(),
        "stderr must be empty on success, got: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // stdout parses as exactly one JSON value (from_slice rejects trailing data).
    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("stdout must be one JSON document");
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["nowdocs_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(v["command"], "capabilities");
    assert_eq!(v["status"], "ok");
    assert_eq!(v["code"], "ready");
    assert!(
        v["summary"].as_str().is_some_and(|s| !s.is_empty()),
        "summary must be a non-empty English string"
    );
    assert_eq!(v["next_actions"], serde_json::json!([]));
    assert!(v["rollback"].is_null());

    let data = &v["data"];
    assert_eq!(data["agent_contract_schema_version"], 1);
    assert_eq!(data["mcp_protocol_version"], "2025-11-25");
    assert_eq!(data["transport"], "stdio_ndjson");
    assert_eq!(
        data["mcp_tools"],
        serde_json::json!(["nowdocs_list", "nowdocs_search"])
    );
    assert_eq!(data["commands"][0]["id"], "capabilities");
    assert_eq!(data["commands"][0]["implemented"], true);
    assert_eq!(
        data["commands"].as_array().map(Vec::len),
        Some(8),
        "eight command declarations in the locked order"
    );
    let client_ids: Vec<&str> = data["clients"]
        .as_array()
        .expect("clients must be an array")
        .iter()
        .map(|c| c["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        client_ids,
        ["claude-code", "claude-desktop", "cursor", "generic"]
    );
    let boundaries = &data["security_boundaries"];
    assert_eq!(boundaries["mcp_read_only"], true);
    assert_eq!(boundaries["stdio_only"], true);
    assert_eq!(boundaries["telemetry"], false);
    assert_eq!(boundaries["writable_mcp_tools"], false);
    assert_eq!(boundaries["search_requires_docset"], true);
}

#[test]
fn test_capabilities_human_reports_security_boundaries() {
    let root = tempfile::tempdir().unwrap();
    let out = run_nowdocs_isolated(root.path(), &["capabilities"]);
    assert!(
        out.status.success(),
        "capabilities should exit 0, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("schema version: 1"), "got: {stdout}");
    assert!(stdout.contains("2025-11-25"), "got: {stdout}");
    assert!(stdout.contains("stdio"), "got: {stdout}");
    assert!(stdout.contains("read-only"), "got: {stdout}");
    assert!(
        stdout.contains("capabilities: implemented"),
        "got: {stdout}"
    );
    assert!(stdout.contains("status: implemented"), "got: {stdout}");
    for client in ["claude-code", "claude-desktop", "cursor", "generic"] {
        assert!(
            stdout.contains(client),
            "human output must list {client}, got: {stdout}"
        );
    }
}

#[test]
fn test_capabilities_is_offline_and_creates_no_files() {
    let root = tempfile::tempdir().unwrap();
    for args in [&["capabilities"][..], &["capabilities", "--json"][..]] {
        let out = run_nowdocs_isolated(root.path(), args);
        assert!(
            out.status.success(),
            "capabilities {args:?} should exit 0 offline, stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let mut entries = std::fs::read_dir(root.path()).expect("read isolated root");
    assert!(
        entries.next().is_none(),
        "capabilities must create no files or directories under isolated roots"
    );
}

#[test]
fn test_legacy_doctor_json_is_not_agent_enveloped() {
    let root = tempfile::tempdir().unwrap();
    let out = run_nowdocs_isolated(root.path(), &["doctor", "--json"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!("doctor --json must emit one JSON document: {e}; got: {stdout}")
    });
    assert!(
        v.get("schema_version").is_none(),
        "legacy doctor JSON must not gain schema_version"
    );
    assert!(
        v.get("command").is_none(),
        "legacy doctor JSON must not gain command"
    );
    assert!(
        v.get("next_actions").is_none(),
        "legacy doctor JSON must not gain next_actions"
    );
    assert!(
        v.get("status").is_some(),
        "legacy doctor status field must remain"
    );
    assert!(
        v.get("checks").is_some(),
        "legacy doctor checks field must remain"
    );
}

#[test]
fn test_legacy_cache_status_json_is_not_agent_enveloped() {
    let root = tempfile::tempdir().unwrap();
    let out = run_nowdocs_isolated(root.path(), &["cache", "status", "--json"]);
    assert!(
        out.status.success(),
        "cache status --json should exit 0, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout)
        .expect("cache status --json must emit one JSON document");
    assert!(
        v.get("schema_version").is_none(),
        "legacy cache status JSON must not gain schema_version"
    );
    assert!(
        v.get("command").is_none(),
        "legacy cache status JSON must not gain command"
    );
    assert!(
        v.get("next_actions").is_none(),
        "legacy cache status JSON must not gain next_actions"
    );
    assert!(
        v.get("cache_root").is_some(),
        "legacy cache_root field must remain"
    );
}

// ---- C2: agent contract — status command ----
//
// Same isolation rules as the C1 capabilities tests: per-test temporary
// HOME/XDG/cwd roots plus deliberately unusable proxy variables, so any
// accidental network attempt fails fast and nothing touches real user state.

#[test]
fn test_status_json_uses_agent_envelope_and_is_offline() {
    let root = tempfile::tempdir().unwrap();
    let out = run_nowdocs_isolated(root.path(), &["status", "--json"]);
    assert!(
        out.status.success(),
        "status --json must always exit 0, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.stderr.is_empty(),
        "stderr must be empty on success, got: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // stdout parses as exactly one JSON value (from_slice rejects trailing data).
    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("stdout must be one JSON document");
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["nowdocs_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(v["command"], "status");
    // An absent cache is a degraded observation: warning, never a process error.
    assert_eq!(v["status"], "warning");
    assert_eq!(v["code"], "ready");
    assert!(
        v["summary"].as_str().is_some_and(|s| !s.is_empty()),
        "summary must be a non-empty English string"
    );
    assert_eq!(v["next_actions"], serde_json::json!([]));
    assert!(v["rollback"].is_null());

    let data = &v["data"];
    assert_eq!(data["platform"]["os"], std::env::consts::OS);
    assert_eq!(data["platform"]["arch"], std::env::consts::ARCH);
    assert_eq!(data["cache"]["layout"], "not_initialized");
    assert_eq!(data["cache"]["total_bytes"], 0);
    assert_eq!(data["cache"]["installed_docsets"], 0);
    assert_eq!(data["cache"]["staging_count"], 0);
    assert_eq!(data["model"]["present"], false);
    assert_eq!(data["docsets"], serde_json::json!([]));
    assert_eq!(data["mcp"]["protocol_version"], "2025-11-25");
    assert_eq!(data["mcp"]["transport"], "stdio_ndjson");
    assert_eq!(
        data["mcp"]["tools"],
        serde_json::json!(["nowdocs_list", "nowdocs_search"])
    );
    let automation = &data["automation"];
    assert_eq!(automation["storage_present"], false);
    for field in [
        "plan_count",
        "operation_count",
        "rollback_count",
        "expired_count",
        "total_bytes",
    ] {
        assert_eq!(
            automation[field], 0,
            "automation.{field} must be 0 when storage is absent"
        );
    }

    // Offline and path-free: no absolute path in the output, and the isolated
    // HOME/XDG/cwd root is left completely empty.
    let stdout = String::from_utf8_lossy(&out.stdout);
    let root_str = root.path().to_string_lossy().to_string();
    assert!(
        !stdout.contains(&root_str),
        "status output must not contain the isolated root path"
    );
    let mut entries = std::fs::read_dir(root.path()).expect("read isolated root");
    assert!(
        entries.next().is_none(),
        "status --json must create no files or directories under isolated roots"
    );
}

#[test]
fn test_status_human_output_is_concise_and_path_free() {
    let root = tempfile::tempdir().unwrap();
    let out = run_nowdocs_isolated(root.path(), &["status"]);
    assert!(
        out.status.success(),
        "status should exit 0, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.stderr.is_empty(),
        "stderr must be empty on success, got: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    for needle in [
        "cache layout",
        "not_initialized",
        "model",
        "docsets: 0",
        "2025-11-25",
        "nowdocs_list",
        "nowdocs_search",
        "automation",
    ] {
        assert!(
            stdout.contains(needle),
            "human status must mention {needle}, got: {stdout}"
        );
    }
    let root_str = root.path().to_string_lossy().to_string();
    assert!(
        !stdout.contains(&root_str),
        "human status must not contain absolute paths, got: {stdout}"
    );
    assert!(
        !stdout.contains("/Users/"),
        "human status must not contain user paths, got: {stdout}"
    );
    let lines = stdout.lines().count();
    assert!(
        lines <= 12,
        "human status must be concise (<= 12 lines), got {lines}: {stdout}"
    );
}

#[test]
fn test_status_absent_roots_create_no_files() {
    let root = tempfile::tempdir().unwrap();
    for args in [&["status"][..], &["status", "--json"][..]] {
        let out = run_nowdocs_isolated(root.path(), args);
        assert!(
            out.status.success(),
            "status {args:?} should exit 0 offline, stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let mut entries = std::fs::read_dir(root.path()).expect("read isolated root");
    assert!(
        entries.next().is_none(),
        "both status forms must leave empty isolated HOME/XDG/cwd roots empty"
    );
}

#[test]
fn test_status_updates_capabilities_implementation_state() {
    let root = tempfile::tempdir().unwrap();
    let out = run_nowdocs_isolated(root.path(), &["capabilities", "--json"]);
    assert!(
        out.status.success(),
        "capabilities --json should exit 0, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("capabilities emits one JSON document");
    let commands = v["data"]["commands"]
        .as_array()
        .expect("commands must be an array");
    let status = commands
        .iter()
        .find(|c| c["id"] == "status")
        .expect("status command declaration must exist");
    assert_eq!(
        status["implemented"], true,
        "C2 implements status: capabilities must say so"
    );
    assert_eq!(status["read_only"], true);
    assert_eq!(status["network_access"], false);
}

// ---- C4: agent contract — ensure command ----

#[test]
fn test_capabilities_reports_ensure_implemented() {
    let root = tempfile::tempdir().unwrap();
    let out = run_nowdocs_isolated(root.path(), &["capabilities", "--json"]);
    assert!(
        out.status.success(),
        "capabilities --json should exit 0, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("capabilities emits one JSON document");
    let commands = v["data"]["commands"]
        .as_array()
        .expect("commands must be an array");
    for id in ["ensure.plan", "ensure.apply"] {
        let cmd = commands
            .iter()
            .find(|c| c["id"] == id)
            .unwrap_or_else(|| panic!("{id} command declaration must exist"));
        assert_eq!(
            cmd["implemented"], true,
            "C4 implements {id}: capabilities must say so"
        );
        assert_eq!(cmd["read_only"], false, "{id} must not be read_only");
        assert_eq!(cmd["network_access"], true, "{id} may access network");
    }
}

#[test]
fn test_ensure_offline_missing_docset_returns_registry_metadata_required() {
    let root = tempfile::tempdir().unwrap();
    let out = run_nowdocs_isolated(root.path(), &["ensure", "nextjs", "--json"]);
    assert!(
        out.status.success(),
        "ensure missing docset must exit 0, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("ensure emits one JSON document");
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["command"], "ensure");
    assert_eq!(v["status"], "action_required");
    assert_eq!(v["code"], "registry_metadata_required");
    assert_eq!(v["data"]["docset"], "nextjs");
    assert!(
        v["next_actions"].as_array().is_some_and(|a| !a.is_empty()),
        "ensure must include a next action"
    );
}

#[test]
fn test_ensure_offline_installed_docset_returns_already_satisfied() {
    let root = tempfile::tempdir().unwrap();
    // Build the fixture tarball before acquiring the env guard; write_tarball
    // acquires ENV_LOCK and must not be called while the guard is held.
    let tar = write_tarball(root.path(), "react", "18.3.1");
    let url = format!("file://{}", tar.display());

    // Install a docset via the library (test-mode file:// is allowed in cfg(test)).
    let _g = EnvGuard::set("XDG_CACHE_HOME", root.path().to_str().unwrap());
    nowdocs::registry::install("react", &url).expect("install fixture docset");

    let out = run_nowdocs_isolated(root.path(), &["ensure", "react", "--json"]);
    assert!(
        out.status.success(),
        "ensure installed docset must exit 0, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("ensure emits one JSON document");
    assert_eq!(v["status"], "ok");
    assert_eq!(v["code"], "already_satisfied");
    assert_eq!(v["data"]["docset"], "react");
}

#[test]
fn test_ensure_online_creates_plan_with_hash() {
    let root = tempfile::tempdir().unwrap();

    // Build an index fixture pointing at a syntactically valid release URL.
    // Planning only validates the URL shape; no network download occurs.
    let index_json = serde_json::json!({
        "schema_version": 1,
        "generated_at": "2026-07-07T00:00:00Z",
        "packages": [{
            "docset": "nextjs",
            "version": "14.2.5",
            "license": "MIT",
            "chunk_count": 7480,
            "freshness": "2026-07-01",
            "download_url": "https://github.com/nowdocs-registry/nextjs/releases/download/nextjs-14.2.5/nextjs-14.2.5.lance.tar",
            "sha256": "0000000000000000000000000000000000000000000000000000000000000000",
            "description": "React framework"
        }]
    });
    let index_path = root.path().join("index.json");
    std::fs::write(&index_path, index_json.to_string()).unwrap();

    // Point ensure at the local fixture index so the test is fully offline.
    let index_url = format!("file://{}", index_path.display());
    let _g = EnvGuard::set("NOWDOCS_REGISTRY_INDEX_URL", &index_url);

    let out = run_nowdocs_isolated(root.path(), &["ensure", "nextjs", "--online", "--json"]);
    assert!(
        out.status.success(),
        "ensure --online should exit 0, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("ensure emits one JSON document");
    assert_eq!(v["status"], "action_required");
    assert_eq!(v["code"], "action_required");
    assert!(
        v["data"]["plan_hash"]
            .as_str()
            .is_some_and(|s| !s.is_empty()),
        "online planning must expose a plan_hash, got: {}",
        v["data"]
    );
    assert_eq!(v["data"]["docset"], "nextjs");
    assert!(
        v["next_actions"].as_array().is_some_and(|a| !a.is_empty()),
        "online planning must include an apply next action"
    );
}
