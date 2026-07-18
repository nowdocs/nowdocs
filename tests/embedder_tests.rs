use nowdocs::embedder::{Embedder, EmbedderSpec};
use std::sync::Mutex;

// S0 provenance constants (pinned for reproducibility)
const S0_MODEL_ID: &str = "jinaai/jina-embeddings-v2-small-en";
const S0_REVISION: &str = "44e7d1d6caec8c883c2d4b207588504d519788d0";
const S0_SHA256: &str = "c9a9a7ec012d01efd780474fbb65e25917f3a2aebdff84b5f87daa00f7e90b27";

static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Lock `ENV_LOCK`, recovering the inner guard if a previous test panicked
/// while holding it. The original panic still fails its own test; this keeps
/// unrelated follow-on tests from failing with `PoisonError`.
fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

struct EnvGuard {
    key: &'static str,
    old: Option<String>,
    _g: std::sync::MutexGuard<'static, ()>,
}

impl EnvGuard {
    fn set(key: &'static str, val: &str) -> Self {
        let g = env_lock();
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

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}

// --- Existing E2 tests (S0 regression) ---

#[test]
#[ignore = "needs an isolated writable copy of the pinned model cache or a network-prepared cache"]
fn test_embed_dim_is_512() {
    let _g = env_lock();
    let spec = EmbedderSpec {
        model_id: S0_MODEL_ID.to_string(),
        model_revision: S0_REVISION.to_string(),
        model_sha256: S0_SHA256.to_string(),
    };
    let e = Embedder::load_for(&spec).expect("model load");
    let v = e.embed("hello world").expect("embed");
    assert_eq!(v.len(), 512, "jina-v2-small must produce 512-dim vectors");
}

#[test]
#[ignore = "needs an isolated writable copy of the pinned model cache or a network-prepared cache"]
fn test_embed_semantic_self_consistency() {
    let _g = env_lock();
    let spec = EmbedderSpec {
        model_id: S0_MODEL_ID.to_string(),
        model_revision: S0_REVISION.to_string(),
        model_sha256: S0_SHA256.to_string(),
    };
    let e = Embedder::load_for(&spec).expect("model load");
    let a = e.embed("how to use clerkMiddleware").unwrap();
    let b = e.embed("using clerkMiddleware in middleware").unwrap();
    let c = e.embed("tomato soup recipe").unwrap();
    assert!(cosine(&a, &b) > 0.7, "near queries should be close");
    assert!(cosine(&a, &c) < 0.75, "unrelated query should be far");
}

#[test]
#[ignore] // requires tests/fixtures/jina_ref.json from gen_reference.py
fn test_embed_matches_reference_above_0_99() {
    let _g = env_lock();
    let spec = EmbedderSpec {
        model_id: S0_MODEL_ID.to_string(),
        model_revision: S0_REVISION.to_string(),
        model_sha256: S0_SHA256.to_string(),
    };
    let e = Embedder::load_for(&spec).expect("model load");
    let v = e.embed("how to use clerkMiddleware").unwrap();
    let fixture = std::fs::read_to_string("tests/fixtures/jina_ref.json")
        .expect("run gen_reference.py first");
    let val: serde_json::Value = serde_json::from_str(&fixture).unwrap();
    let ref_vec: Vec<f32> = val["vector"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap() as f32)
        .collect();
    let sim = cosine(&v, &ref_vec);
    assert!(
        sim > 0.99,
        "candle output must match reference embedder (cosine={:.4})",
        sim
    );
}

// --- New 2a tests ---

#[test]
#[ignore = "needs an isolated writable copy of the pinned model cache or a network-prepared cache"]
fn test_load_for_rejects_tampered_sha() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", tmp.path().to_str().unwrap());

    let spec = EmbedderSpec {
        model_id: S0_MODEL_ID.to_string(),
        model_revision: S0_REVISION.to_string(),
        model_sha256: "0000000000000000000000000000000000000000000000000000000000000000"
            .to_string(),
    };
    match Embedder::load_for(&spec) {
        Ok(_) => panic!("load_for must reject tampered sha256"),
        Err(e) => {
            let err_msg = e.to_string();
            assert!(
                err_msg.contains("sha256")
                    || err_msg.contains("integrity")
                    || err_msg.contains("mismatch"),
                "error should mention sha256 mismatch, got: {err_msg}"
            );
        }
    }
}

#[test]
#[ignore = "needs an isolated writable copy of the pinned model cache or a network-prepared cache"]
fn test_load_delegates_to_load_for() {
    let _g = env_lock();
    // load() should work with DEFAULT_SPEC (S0 provenance constants)
    let e = Embedder::load().expect("load() should succeed with DEFAULT_SPEC");
    let v = e.embed("test").expect("embed");
    assert_eq!(v.len(), 512);
}

// --- A1.2 N3/M13: embedder startup cache + no global HF_HOME ---

#[test]
fn test_no_unsafe_set_var_in_embedder() {
    // N3/M13: the embedder must not mutate global process state (HF_HOME). It
    // routes the hf-hub cache via `ApiBuilder::with_cache_dir` instead so that
    // loading is safe under a tokio runtime and supports dual-model futures.
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/embedder.rs");
    let src = std::fs::read_to_string(&path).expect("read embedder.rs");
    assert!(
        !src.contains("set_var("),
        "embedder.rs must not call std::env::set_var (global HF_HOME); use ApiBuilder::with_cache_dir"
    );
    assert!(
        !src.contains("HF_HOME"),
        "embedder.rs must not reference HF_HOME; cache dir is passed via with_cache_dir"
    );
    assert!(
        src.contains("with_cache_dir"),
        "embedder.rs must route the hf-hub cache through ApiBuilder::with_cache_dir"
    );
}

#[test]
#[ignore = "needs an isolated writable copy of the pinned model cache or a network-prepared cache"]
fn test_load_for_returns_cached_embedder_on_second_call() {
    let _g = env_lock();
    let spec = EmbedderSpec {
        model_id: S0_MODEL_ID.to_string(),
        model_revision: S0_REVISION.to_string(),
        model_sha256: S0_SHA256.to_string(),
    };
    let e1 = Embedder::load_for(&spec).expect("first load");
    let e2 = Embedder::load_for(&spec).expect("second load");
    assert!(
        e1.same_cache_instance(&e2),
        "second load_for must return the cached instance (no model reload)"
    );
}

#[test]
fn test_preload_skips_when_model_uncached() {
    // Cold cache (temp XDG_CACHE_HOME): the default model is absent, so
    // `default_model_cached()` is false and `preload_default_embedder()` must
    // return immediately WITHOUT downloading or panicking. This keeps
    // `nowdocs serve` hermetic/offline-safe in CI and on first run.
    let tmp = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", tmp.path().to_str().unwrap());
    assert!(
        !nowdocs::embedder::default_model_cached(),
        "temp cache must report the default model as absent"
    );
    nowdocs::embedder::preload_default_embedder();
}

#[test]
fn test_preload_uses_cached_only_loader() {
    // `nowdocs serve` promises cache-only startup. When the pinned model is
    // already present, warmup must use the local read-only loader instead of
    // the normal loader, which sanitizes config.json by writing it back.
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/embedder.rs");
    let src = std::fs::read_to_string(&path).expect("read embedder.rs");
    let start = src
        .find("pub fn preload_default_embedder")
        .expect("preload_default_embedder must exist");
    let rest = &src[start..];
    let end = rest[1..]
        .find("\n}")
        .map(|i| start + 1 + i + 2)
        .expect("preload_default_embedder must have a function body");
    let body = &src[start..end];

    assert!(
        body.contains("load_default_cached_only()"),
        "serve-time warmup must use the cached-only loader: {body}"
    );
    assert!(
        !body.contains("Embedder::load()"),
        "serve-time warmup must not use the write-capable normal loader: {body}"
    );
}

#[test]
fn test_default_model_cached_requires_all_files() {
    // Cold-cache / interrupted-load guard: weights alone must NOT count as
    // "cached", otherwise serve-time preload would try to fetch the missing
    // config/tokenizer on an offline server. Only when weights + config.json +
    // tokenizer.json are ALL present (and the ref resolves) should preload run.
    //
    // C8-R1: default_model_cached() now resolves through CacheRepo::get, which
    // reads refs/<revision> -> commit hash -> snapshots/<commit_hash>/<file>.
    // The fake cache must mirror this layout for the predicate to resolve.
    let tmp = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", tmp.path().to_str().unwrap());

    let repo_dir = nowdocs::cache::model_path(S0_MODEL_ID)
        .join(format!("models--{}", S0_MODEL_ID.replace('/', "--")));
    let commit = "abc123def4567890abc123def4567890abc123de";
    let snapshots = repo_dir.join("snapshots").join(commit);
    let refs_dir = repo_dir.join("refs");
    std::fs::create_dir_all(&snapshots).unwrap();
    std::fs::create_dir_all(&refs_dir).unwrap();

    // No ref file yet: cold even if files exist (ref cannot resolve).
    std::fs::write(snapshots.join("model.safetensors"), b"").unwrap();
    std::fs::write(snapshots.join("config.json"), b"{}").unwrap();
    std::fs::write(snapshots.join("tokenizer.json"), b"{}").unwrap();
    assert!(
        !nowdocs::embedder::default_model_cached(),
        "snapshot without a ref file must be cold"
    );

    // Write the ref so CacheRepo::get can resolve revision -> commit.
    std::fs::write(refs_dir.join(S0_REVISION), commit).unwrap();

    // Now remove files one at a time to prove each is required.
    // Remove tokenizer: cold.
    std::fs::remove_file(snapshots.join("tokenizer.json")).unwrap();
    assert!(
        !nowdocs::embedder::default_model_cached(),
        "weights+config (no tokenizer) must be cold"
    );
    // Remove config: cold.
    std::fs::remove_file(snapshots.join("config.json")).unwrap();
    assert!(
        !nowdocs::embedder::default_model_cached(),
        "weights-only must be cold"
    );

    // All three required files + ref: warm.
    std::fs::write(snapshots.join("config.json"), b"{}").unwrap();
    std::fs::write(snapshots.join("tokenizer.json"), b"{}").unwrap();
    assert!(
        nowdocs::embedder::default_model_cached(),
        "weights+config+tokenizer with ref must be warm"
    );
}

#[test]
#[ignore = "slow: runs a full 8191-token forward pass on CPU; cap logic is covered by the embedder::tests::cap_to_max_position_* unit tests"]
fn test_embedder_truncates_oversized_input_without_panic() {
    // N7: an input far beyond the model's 8192 max-position must be truncated
    // and still produce a 512-dim vector — never panic inside candle.
    let _g = env_lock();
    let e = Embedder::load().expect("load");
    let long = "hello ".repeat(20_000);
    let v = e
        .embed(&long)
        .expect("embed of oversized input must not panic");
    assert_eq!(v.len(), 512, "truncated embed must still return 512-dim");
}

// ---- C8-R1: cached-only model loading (no network, no writes) ----
//
// These tests build a fake hf-hub cache layout under an isolated XDG_CACHE_HOME
// and exercise the cached-only default-model loader. They prove:
//   - missing refs/files fail locally (no network fallback);
//   - a snapshot without a usable ref is not considered cached;
//   - cached-only failure leaves the entire fake tree byte-for-byte and
//     metadata-stable (no writes).
//
// Blocked proxies are defense-in-depth; the primary no-network proof is the
// call graph (no ApiBuilder/ApiRepo::get/download in the cached-only path).

const FAKE_COMMIT: &str = "abcdef1234567890abcdef1234567890abcdef12";

/// Build the fake hf-hub cache directory structure for the pinned default
/// model under `cache_home`. Returns the snapshots directory path.
fn fake_hub_cache(cache_home: &std::path::Path) -> std::path::PathBuf {
    let model_id = S0_MODEL_ID;
    let model_cache = nowdocs::cache::model_path(model_id);
    // hf-hub routes the hub cache under the nowdocs model_path; the Cache
    // constructor receives the *hub* root (model_path is the hub root here
    // because cache::model_path returns the per-model directory used by
    // ApiBuilder::with_cache_dir, and the cached-only loader mirrors that).
    let _ = model_cache;
    let repo_dir = cache_home
        .join("nowdocs")
        .join("models")
        .join(model_id)
        .join(format!("models--{}", model_id.replace('/', "--")));
    let snapshots = repo_dir.join("snapshots").join(FAKE_COMMIT);
    std::fs::create_dir_all(&snapshots).unwrap();
    snapshots
}

/// Write the ref file so `CacheRepo::get` can resolve the revision to a commit.
fn write_ref(cache_home: &std::path::Path, commit: &str) {
    let repo_dir = cache_home
        .join("nowdocs")
        .join("models")
        .join(S0_MODEL_ID)
        .join(format!("models--{}", S0_MODEL_ID.replace('/', "--")));
    let refs_dir = repo_dir.join("refs");
    std::fs::create_dir_all(&refs_dir).unwrap();
    std::fs::write(refs_dir.join(S0_REVISION), commit).unwrap();
}

/// A cached-only default-model load must fail locally (no network) when the
/// ref file is missing, even if snapshot files exist. The snapshot without a
/// usable ref must not be considered cached.
#[test]
fn cached_only_load_fails_when_ref_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", tmp.path().to_str().unwrap());

    let snapshots = fake_hub_cache(tmp.path());
    // Write the required files but NO ref file.
    std::fs::write(snapshots.join("config.json"), b"{}").unwrap();
    std::fs::write(snapshots.join("tokenizer.json"), b"{}").unwrap();
    std::fs::write(snapshots.join("model.safetensors"), b"not-real-weights").unwrap();

    // The predicate must say not cached (no usable ref).
    assert!(
        !nowdocs::embedder::default_model_cached(),
        "a snapshot without a usable ref must not be considered cached"
    );

    // The cached-only loader must fail locally without network.
    let result = nowdocs::embedder::load_default_cached_only();
    assert!(
        result.is_err(),
        "cached-only load must fail when ref is missing"
    );
}

/// A cached-only default-model load must fail locally when the ref exists but
/// points to a commit whose snapshot directory is missing required files.
#[test]
fn cached_only_load_fails_when_files_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", tmp.path().to_str().unwrap());

    let snapshots = fake_hub_cache(tmp.path());
    write_ref(tmp.path(), FAKE_COMMIT);
    // Only config.json; weights + tokenizer missing.
    std::fs::write(snapshots.join("config.json"), b"{}").unwrap();

    assert!(
        !nowdocs::embedder::default_model_cached(),
        "missing files must not be considered cached"
    );

    let result = nowdocs::embedder::load_default_cached_only();
    assert!(
        result.is_err(),
        "cached-only load must fail when files are missing"
    );
}

/// Cached-only failure must leave the entire fake cache tree byte-for-byte and
/// metadata-stable: no writes, no mode/mtime changes, no directory creation.
#[test]
fn cached_only_failure_leaves_tree_byte_for_byte_stable() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", tmp.path().to_str().unwrap());

    let snapshots = fake_hub_cache(tmp.path());
    write_ref(tmp.path(), FAKE_COMMIT);
    std::fs::write(snapshots.join("config.json"), b"{}").unwrap();
    // weights + tokenizer missing -> load will fail.

    // Snapshot the entire fake tree (paths, contents, mtimes) before the load.
    let before = snapshot_tree(tmp.path());

    let result = nowdocs::embedder::load_default_cached_only();
    assert!(result.is_err(), "expected cached-only failure");

    // Snapshot again after the failed load.
    let after = snapshot_tree(tmp.path());

    assert_eq!(
        before, after,
        "cached-only failure must leave the tree byte-for-byte and metadata-stable"
    );
}

/// A complete cache entry with tampered weights must be rejected by the pinned
/// SHA-256 check before model construction, without deleting or rewriting the
/// cached files. Cached-only verification must preserve the integrity policy
/// enforced by the normal downloader while remaining read-only.
#[test]
fn cached_only_load_rejects_tampered_weights_without_mutation() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", tmp.path().to_str().unwrap());

    let snapshots = fake_hub_cache(tmp.path());
    write_ref(tmp.path(), FAKE_COMMIT);
    std::fs::write(snapshots.join("config.json"), b"{}").unwrap();
    std::fs::write(snapshots.join("tokenizer.json"), b"{}").unwrap();
    std::fs::write(
        snapshots.join("model.safetensors"),
        b"tampered-model-weights",
    )
    .unwrap();
    let before = snapshot_tree(tmp.path());

    let error = match nowdocs::embedder::load_default_cached_only() {
        Ok(_) => panic!("cached-only load must reject weights that fail the pinned digest"),
        Err(error) => error,
    };

    assert!(
        error.to_string().contains("integrity"),
        "tampered weights must fail at the integrity gate, got: {error}"
    );
    assert_eq!(
        before,
        snapshot_tree(tmp.path()),
        "integrity rejection must not mutate or delete cached model files"
    );
}

/// Recursively snapshot a directory tree: sorted list of (relative_path, content_bytes, mtime_secs).
fn snapshot_tree(
    root: &std::path::Path,
) -> Vec<(std::path::PathBuf, Vec<u8>, std::time::SystemTime)> {
    let mut entries = Vec::new();
    let mut walker = vec![std::path::PathBuf::from("")];
    while let Some(rel) = walker.pop() {
        let abs = root.join(&rel);
        let read = std::fs::read_dir(&abs).unwrap();
        for entry in read.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let rel_child = rel.join(&name);
            let meta = std::fs::symlink_metadata(&path).unwrap();
            if meta.is_dir() {
                walker.push(rel_child.clone());
                entries.push((rel_child, Vec::new(), meta.modified().unwrap()));
            } else if meta.is_file() {
                let content = std::fs::read(&path).unwrap();
                entries.push((rel_child, content, meta.modified().unwrap()));
            }
        }
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries
}

/// The cached-only loader must never construct ApiBuilder, call ApiRepo::get,
/// download, create_dir, write, rename, or remove. This source-level check is
/// the primary no-network/no-write proof (blocked proxies are defense in depth).
#[test]
fn cached_only_path_has_no_network_or_write_calls() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/embedder.rs");
    let src = std::fs::read_to_string(&path).expect("read embedder.rs");

    // The cached-only function must exist.
    assert!(
        src.contains("load_default_cached_only"),
        "embedder.rs must define load_default_cached_only"
    );

    // Extract the cached-only function body. Strip comment lines so the check
    // examines actual code statements, not doc comments that mention the
    // forbidden APIs by name.
    let start = src
        .find("fn load_default_cached_only")
        .expect("cached-only loader must exist");
    let rest = &src[start..];
    let end = rest[1..]
        .find("\nfn ")
        .map(|i| start + 1 + i)
        .unwrap_or(src.len());
    let body_full = &src[start..end];
    let body: String = body_full
        .lines()
        .map(|l| {
            // Strip everything from `//` onward (comments).
            match l.find("//") {
                Some(i) => &l[..i],
                None => l,
            }
        })
        .collect::<Vec<&str>>()
        .join("\n");

    for forbidden in [
        "ApiBuilder",
        "ApiRepo",
        "download",
        "create_dir",
        "std::fs::write",
        "rename",
        "std::fs::remove",
        "sanitize_config",
    ] {
        assert!(
            !body.contains(forbidden),
            "cached-only loader must not call `{forbidden}` (network/write), but it appears in: {body}"
        );
    }
}

/// `default_model_cached()` and the cached-only loader must share the same
/// cache-resolution rules: if the predicate says cached, the loader must be
/// able to resolve the local files (it may still fail on malformed content, but
/// not on a resolution miss).
#[test]
fn predicate_and_loader_share_resolution_rules() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", tmp.path().to_str().unwrap());

    // No cache at all: predicate false, loader errors.
    assert!(!nowdocs::embedder::default_model_cached());
    assert!(nowdocs::embedder::load_default_cached_only().is_err());

    // Fake a complete (ref + files) but deliberately invalid-content cache.
    // The predicate should now be true (files present + ref resolves), and the
    // loader should resolve the paths (even though candle will reject the
    // fake weights). The key invariant: if predicate is true, the loader does
    // NOT fail with a "missing file" error -- it fails at model construction,
    // proving they share resolution rules.
    let snapshots = fake_hub_cache(tmp.path());
    write_ref(tmp.path(), FAKE_COMMIT);
    std::fs::write(snapshots.join("config.json"), b"{}").unwrap();
    std::fs::write(snapshots.join("tokenizer.json"), b"{}").unwrap();
    std::fs::write(snapshots.join("model.safetensors"), b"fake").unwrap();

    assert!(
        nowdocs::embedder::default_model_cached(),
        "complete fake cache must be considered cached by the predicate"
    );
    // The loader resolves locally (no network) but fails on fake content.
    let err = nowdocs::embedder::load_default_cached_only()
        .err()
        .expect("fake-content cached load must fail at construction, not network");
    let msg = format!("{err}");
    assert!(
        !msg.contains("download") && !msg.contains("fetch"),
        "loader must not attempt network; error was: {msg}"
    );
}
