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
