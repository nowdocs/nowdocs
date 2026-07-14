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
    let found = hits.iter().any(|c| c.hit.chunk_idx == 1);
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

// --- C04: signal-preserving RRF ---

#[tokio::test]
async fn signal_preserving_rrf() {
    use std::sync::Arc;

    use arrow::array::downcast_array;
    use arrow_array::{Array, RecordBatch, StringArray, UInt64Array};
    use arrow_schema::{DataType, Field, Schema};
    use lancedb::rerankers::rrf::RRFReranker;
    use lancedb::rerankers::Reranker;
    use nowdocs::store::{SignalPreservingRrf, DENSE_RANK_COLUMN, FTS_RANK_COLUMN};

    let k: f32 = 60.0;

    // Construct mock vector and FTS result batches as LanceDB's hybrid
    // pipeline would pass them to the reranker: each has a `_rowid` column
    // (UInt64) and a data column ("name"). Overlapping row IDs test that
    // both rerankers merge and score identically.
    let schema = Arc::new(Schema::new(vec![
        Field::new("name", DataType::Utf8, false),
        Field::new("_rowid", DataType::UInt64, false),
    ]));

    let vec_results = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(StringArray::from(vec!["foo", "bar", "baz", "bean", "dog"])),
            Arc::new(UInt64Array::from(vec![1, 4, 2, 5, 3])),
        ],
    )
    .unwrap();

    let fts_results = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(StringArray::from(vec!["bar", "bean", "dog"])),
            Arc::new(UInt64Array::from(vec![4, 5, 3])),
        ],
    )
    .unwrap();

    // Reference: LanceDB's built-in RRFReranker.
    let built_in = RRFReranker::new(k);
    let reference = built_in
        .rerank_hybrid("", vec_results.clone(), fts_results.clone())
        .await
        .unwrap();

    // Under test: our signal-preserving adapter.
    let adapter = SignalPreservingRrf::new(k);
    let result = adapter
        .rerank_hybrid("", vec_results, fts_results)
        .await
        .unwrap();

    // 1. Byte-for-byte compatible row IDs and ordering.
    let ref_ids: UInt64Array = downcast_array(reference.column_by_name("_rowid").unwrap());
    let res_ids: UInt64Array = downcast_array(result.column_by_name("_rowid").unwrap());
    assert_eq!(
        ref_ids.values(),
        res_ids.values(),
        "row IDs and order must match RRFReranker"
    );

    // 2. RRF scores within 1e-7 tolerance.
    let ref_scores: arrow_array::Float32Array =
        downcast_array(reference.column_by_name("_relevance_score").unwrap());
    let res_scores: arrow_array::Float32Array =
        downcast_array(result.column_by_name("_relevance_score").unwrap());
    for i in 0..reference.num_rows() {
        assert!(
            (ref_scores.value(i) - res_scores.value(i)).abs() < 1e-7,
            "RRF score mismatch at row {i}: built-in={}, adapter={}",
            ref_scores.value(i),
            res_scores.value(i)
        );
    }

    // 3. Nullable per-channel rank columns are present and correct.
    //    One-based ranks: position in the original batch + 1.
    //    After sort by RRF score descending the order is:
    //      bar(row4): dense=2, fts=1
    //      bean(row5): dense=4, fts=2
    //      dog(row3): dense=5, fts=3
    //      foo(row1): dense=1, fts=None
    //      baz(row2): dense=3, fts=None
    let dense_col = result
        .column_by_name(DENSE_RANK_COLUMN)
        .expect("_dense_rank column must exist");
    let fts_col = result
        .column_by_name(FTS_RANK_COLUMN)
        .expect("_fts_rank column must exist");
    let dense: arrow_array::UInt32Array = downcast_array(dense_col);
    let fts: arrow_array::UInt32Array = downcast_array(fts_col);

    let expected_dense: Vec<Option<u32>> = vec![Some(2), Some(4), Some(5), Some(1), Some(3)];
    let expected_fts: Vec<Option<u32>> = vec![Some(1), Some(2), Some(3), None, None];

    for i in 0..result.num_rows() {
        assert_eq!(
            dense.is_null(i),
            expected_dense[i].is_none(),
            "dense null mismatch at row {i}"
        );
        if let Some(rank) = expected_dense[i] {
            assert_eq!(dense.value(i), rank, "dense rank mismatch at row {i}");
        }
        assert_eq!(
            fts.is_null(i),
            expected_fts[i].is_none(),
            "fts null mismatch at row {i}"
        );
        if let Some(rank) = expected_fts[i] {
            assert_eq!(fts.value(i), rank, "fts rank mismatch at row {i}");
        }
    }
}
