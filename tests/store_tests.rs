use nowdocs::chunker::{Chunk, ChunkType};
use nowdocs::store::Store;

/// Deterministic stub embedding: produces a 512-dim vector from text hash.
/// Same text → same vector, so BM25 + cosine recall can be tested.
fn embed_stub(text: &str) -> Vec<f32> {
    let mut vec = vec![0.0f32; 512];
    for (i, byte) in text.bytes().enumerate() {
        vec[i % 512] += byte as f32;
    }
    // normalize
    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in vec.iter_mut() {
            *v /= norm;
        }
    }
    vec
}

fn make_chunks() -> Vec<Chunk> {
    vec![
        Chunk {
            idx: 0,
            heading_path: "Intro".into(),
            source_url: "https://example.com/0".into(),
            api_version: None,
            chunk_type: ChunkType::Info,
            text: "General documentation about setup and configuration.".into(),
        },
        Chunk {
            idx: 1,
            heading_path: "API > Auth".into(),
            source_url: "https://example.com/1".into(),
            api_version: Some("v2".into()),
            chunk_type: ChunkType::Code,
            text: "Authentication uses zzzunique_token for bearer flow.".into(),
        },
        Chunk {
            idx: 2,
            heading_path: "API > Endpoints".into(),
            source_url: "https://example.com/2".into(),
            api_version: None,
            chunk_type: ChunkType::Info,
            text: "The GET /users endpoint returns a list of all users.".into(),
        },
    ]
}

#[test]
fn test_open_insert_recall() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let store = Store::open("test_recall").unwrap();
    let chunks = make_chunks();
    let vectors: Vec<Vec<f32>> = chunks.iter().map(|c| embed_stub(&c.text)).collect();
    store.insert(&chunks, &vectors).unwrap();

    let qv = embed_stub("zzzunique_token");
    let hits = store.hybrid_search(&qv, "zzzunique_token", 3).unwrap();
    assert!(
        !hits.is_empty(),
        "hybrid_search should return at least one hit"
    );
    // The chunk containing "zzzunique_token" should appear in results.
    // RRF reranking may reorder based on vector similarity, so we check
    // presence rather than exact rank.
    let found = hits.iter().any(|h| h.chunk_idx == 1);
    assert!(
        found,
        "chunk[1] containing zzzunique_token should be in results"
    );
}

#[test]
fn test_insert_len_mismatch_bails() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let store = Store::open("test_mismatch").unwrap();
    let chunks = vec![
        Chunk {
            idx: 0,
            heading_path: "H".into(),
            source_url: "u".into(),
            api_version: None,
            chunk_type: ChunkType::Info,
            text: "a".into(),
        },
        Chunk {
            idx: 1,
            heading_path: "H".into(),
            source_url: "u".into(),
            api_version: None,
            chunk_type: ChunkType::Info,
            text: "b".into(),
        },
    ];
    let vectors = vec![vec![0.0f32; 512]; 3]; // 2 chunks, 3 vectors
    assert!(store.insert(&chunks, &vectors).is_err());
}

#[test]
fn test_insert_rejects_mixed_bad_vector_dimensions() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let store = Store::open("test_bad_vector_dim").unwrap();
    let chunks = vec![
        Chunk {
            idx: 0,
            heading_path: "H".into(),
            source_url: "u".into(),
            api_version: None,
            chunk_type: ChunkType::Info,
            text: "a".into(),
        },
        Chunk {
            idx: 1,
            heading_path: "H".into(),
            source_url: "u".into(),
            api_version: None,
            chunk_type: ChunkType::Info,
            text: "b".into(),
        },
    ];
    // Total length is still 1024, so validating only the flattened vector
    // length would silently corrupt row boundaries. Each row must be checked.
    let vectors = vec![vec![0.0f32; 511], vec![0.0f32; 513]];
    let err = store.insert(&chunks, &vectors).unwrap_err().to_string();
    assert!(err.contains("vector[0] has dimension 511"));
}

#[test]
fn test_open_empty_docset_creates_table() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let store = Store::open("test_empty").unwrap();
    let hits = store.hybrid_search(&[0.0f32; 512], "anything", 5).unwrap();
    assert!(
        hits.is_empty(),
        "empty table should return empty results, not error"
    );
}

#[tokio::test]
async fn test_store_open_rejects_nested_tokio_runtime() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let err = match Store::open("test_nested_runtime") {
        Ok(_) => panic!("Store::open should fail inside an existing Tokio runtime"),
        Err(err) => err,
    };
    let msg = format!("{err:#}");
    assert!(
        msg.contains("existing Tokio runtime"),
        "Store::open should reject nested runtimes gracefully, got: {msg}"
    );
}

#[tokio::test]
async fn test_store_open_allows_spawn_blocking() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let res = tokio::task::spawn_blocking(|| {
        let store = Store::open("test_spawn_blocking")?;
        let hits = store.fetch_by_idx(&[]);
        assert!(hits.is_ok());
        Ok::<(), anyhow::Error>(())
    })
    .await
    .unwrap();

    assert!(
        res.is_ok(),
        "Store::open should succeed inside spawn_blocking, got error: {:?}",
        res.err()
    );
}

// --- A1.2 N1: fetch_vectors (additive; feeds MMR diversity reranking) ---

#[test]
fn test_fetch_vectors_returns_correct_vectors() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    let store = Store::open("test_fetch_vectors").unwrap();
    let chunks = make_chunks();
    let vectors: Vec<Vec<f32>> = chunks.iter().map(|c| embed_stub(&c.text)).collect();
    store.insert(&chunks, &vectors).unwrap();

    let fetched = store.fetch_vectors(&[0, 2]).unwrap();
    assert_eq!(fetched.len(), 2, "should return exactly the requested ids");
    let v0 = fetched.get(&0).expect("chunk 0 vector");
    let v2 = fetched.get(&2).expect("chunk 2 vector");
    assert_eq!(v0.len(), 512);
    assert_eq!(v2.len(), 512);
    // f32 -> f16 -> f32 round-trip loses a little precision; compare loosely.
    for (a, b) in v0.iter().zip(vectors[0].iter()) {
        assert!((a - b).abs() < 1e-3, "vector[0] mismatch: {a} vs {b}");
    }
    for (a, b) in v2.iter().zip(vectors[2].iter()) {
        assert!((a - b).abs() < 1e-3, "vector[2] mismatch: {a} vs {b}");
    }
    assert!(!fetched.contains_key(&1), "unrequested id must be absent");
}

#[test]
fn test_fetch_vectors_empty_input_returns_empty_map() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };
    let store = Store::open("test_fetch_vectors_empty").unwrap();
    let fetched = store.fetch_vectors(&[]).unwrap();
    assert!(fetched.is_empty());
}
