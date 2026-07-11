use nowdocs::embedder::{Embedder, EmbedderSpec};
use std::sync::{Arc, Mutex};

// S0 provenance constants (pinned for reproducibility)
const S0_MODEL_ID: &str = "jinaai/jina-embeddings-v2-small-en";
const S0_REVISION: &str = "44e7d1d6caec8c883c2d4b207588504d519788d0";
const S0_SHA256: &str = "c9a9a7ec012d01efd780474fbb65e25917f3a2aebdff84b5f87daa00f7e90b27";

static ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvGuard {
    key: &'static str,
    old: Option<String>,
    _g: std::sync::MutexGuard<'static, ()>,
}

impl EnvGuard {
    fn set(key: &'static str, val: &str) -> Self {
        let g = ENV_LOCK.lock().unwrap();
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
fn test_embed_dim_is_512() {
    let _g = ENV_LOCK.lock().unwrap();
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
fn test_embed_semantic_self_consistency() {
    let _g = ENV_LOCK.lock().unwrap();
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
    let _g = ENV_LOCK.lock().unwrap();
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
fn test_load_delegates_to_load_for() {
    let _g = ENV_LOCK.lock().unwrap();
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
fn test_load_for_returns_cached_embedder_on_second_call() {
    let _g = ENV_LOCK.lock().unwrap();
    let spec = EmbedderSpec {
        model_id: S0_MODEL_ID.to_string(),
        model_revision: S0_REVISION.to_string(),
        model_sha256: S0_SHA256.to_string(),
    };
    let e1 = Embedder::load_for(&spec).expect("first load");
    let e2 = Embedder::load_for(&spec).expect("second load");
    assert!(
        Arc::ptr_eq(&e1, &e2),
        "second load_for must return the cached Arc (no model reload)"
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
#[ignore = "slow: runs a full 8191-token forward pass on CPU; cap logic is covered by the embedder::tests::cap_to_max_position_* unit tests"]
fn test_embedder_truncates_oversized_input_without_panic() {
    // N7: an input far beyond the model's 8192 max-position must be truncated
    // and still produce a 512-dim vector — never panic inside candle.
    let _g = ENV_LOCK.lock().unwrap();
    let e = Embedder::load().expect("load");
    let long = "hello ".repeat(20_000);
    let v = e
        .embed(&long)
        .expect("embed of oversized input must not panic");
    assert_eq!(v.len(), 512, "truncated embed must still return 512-dim");
}
