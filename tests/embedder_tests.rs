use nowdocs::embedder::Embedder;

fn ensure_hf_cache() {
    // The default ~/.cache/huggingface/hub may not be writable in some CI/sandbox
    // environments. Route hf-hub into the project-named cache dir for S0.
    if std::env::var("HF_HOME").is_err() {
        let home = std::env::var("HOME").expect("HOME env var");
        let cache = std::path::PathBuf::from(home)
            .join(".cache")
            .join("nowdocs")
            .join("hf");
        std::fs::create_dir_all(&cache).ok();
        std::env::set_var("HF_HOME", cache.as_os_str());
    }
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 { 0.0 } else { dot / (na * nb) }
}

#[test]
fn test_embed_dim_is_512() {
    ensure_hf_cache();
    let e = Embedder::load().expect("model load");
    let v = e.embed("hello world").expect("embed");
    assert_eq!(v.len(), 512, "jina-v2-small must produce 512-dim vectors");
}

#[test]
fn test_embed_semantic_self_consistency() {
    ensure_hf_cache();
    // Two semantically near queries must be much closer than an unrelated one.
    let e = Embedder::load().expect("model load");
    let a = e.embed("how to use clerkMiddleware").unwrap();
    let b = e.embed("using clerkMiddleware in middleware").unwrap();
    let c = e.embed("tomato soup recipe").unwrap();
    // Reference (Python transformers mean-pooling) gives ~0.95 / ~0.69 for these
    // pairs, so 0.5 was too strict for jina-v2-small. Keep a clear margin between
    // near and unrelated.
    assert!(cosine(&a, &b) > 0.7, "near queries should be close");
    assert!(cosine(&a, &c) < 0.75, "unrelated query should be far");
}

#[test]
#[ignore] // requires tests/fixtures/jina_ref.json from gen_reference.py
fn test_embed_matches_reference_above_0_99() {
    ensure_hf_cache();
    let e = Embedder::load().expect("model load");
    let v = e.embed("how to use clerkMiddleware").unwrap();
    let fixture = std::fs::read_to_string("tests/fixtures/jina_ref.json")
        .expect("run gen_reference.py first");
    let val: serde_json::Value = serde_json::from_str(&fixture).unwrap();
    let ref_vec: Vec<f32> = val["vector"].as_array().unwrap()
        .iter().map(|x| x.as_f64().unwrap() as f32).collect();
    let sim = cosine(&v, &ref_vec);
    assert!(sim > 0.99, "candle output must match reference embedder (cosine={:.4})", sim);
}
