use nowdocs::embedder::{Embedder, EmbedderSpec};

// S0 provenance constants (pinned for reproducibility)
const S0_MODEL_ID: &str = "jinaai/jina-embeddings-v2-small-en";
const S0_REVISION: &str = "44e7d1d6caec8c883c2d4b207588504d519788d0";
const S0_SHA256: &str = "c9a9a7ec012d01efd780474fbb65e25917f3a2aebdff84b5f87daa00f7e90b27";

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 { 0.0 } else { dot / (na * nb) }
}

// --- Existing E2 tests (S0 regression) ---

#[test]
fn test_embed_dim_is_512() {
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
    let ref_vec: Vec<f32> = val["vector"].as_array().unwrap()
        .iter().map(|x| x.as_f64().unwrap() as f32).collect();
    let sim = cosine(&v, &ref_vec);
    assert!(sim > 0.99, "candle output must match reference embedder (cosine={:.4})", sim);
}

// --- New 2a tests ---

#[test]
fn test_load_for_rejects_tampered_sha() {
    let spec = EmbedderSpec {
        model_id: S0_MODEL_ID.to_string(),
        model_revision: S0_REVISION.to_string(),
        model_sha256: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
    };
    match Embedder::load_for(&spec) {
        Ok(_) => panic!("load_for must reject tampered sha256"),
        Err(e) => {
            let err_msg = e.to_string();
            assert!(
                err_msg.contains("sha256") || err_msg.contains("integrity") || err_msg.contains("mismatch"),
                "error should mention sha256 mismatch, got: {err_msg}"
            );
        }
    }
}

#[test]
fn test_load_delegates_to_load_for() {
    // load() should work with DEFAULT_SPEC (S0 provenance constants)
    let e = Embedder::load().expect("load() should succeed with DEFAULT_SPEC");
    let v = e.embed("test").expect("embed");
    assert_eq!(v.len(), 512);
}
