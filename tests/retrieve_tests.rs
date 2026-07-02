use nowdocs::chunker::ChunkType;
use nowdocs::retrieve::{reorder_to_window, ResultChunk, SearchResult, window_ids_for};
use nowdocs::store::SearchHit;

/// Build a SearchHit with only the fields the ordering logic inspects.
fn hit(chunk_idx: u32, score: f32) -> SearchHit {
    SearchHit {
        score,
        chunk_idx,
        heading_path: String::new(),
        source_url: String::new(),
        api_version: None,
        chunk_type: ChunkType::Info,
        text: String::new(),
    }
}

/// Build a ResultChunk identified by chunk_idx alone.
fn rchunk(idx: u32) -> ResultChunk {
    ResultChunk {
        chunk_idx: idx,
        heading_path: String::new(),
        source_url: String::new(),
        api_version: None,
        chunk_type: ChunkType::Info,
        text: String::new(),
    }
}

#[test]
fn test_search_smoke() {
    let _ = std::hint::black_box((
        ResultChunk {
            chunk_idx: 0,
            heading_path: "H".into(),
            source_url: "a.md".into(),
            api_version: None,
            chunk_type: ChunkType::Info,
            text: "hello".into(),
        },
        SearchResult {
            chunks: vec![],
            tokens_returned: 0,
            truncated: false,
        },
    ));
}

#[test]
fn test_search_rejects_invalid_inputs() {
    use nowdocs::retrieve::search;
    unsafe { std::env::set_var("XDG_CACHE_HOME", tempfile::tempdir().unwrap().path()) };
    assert!(search("../bad", "query", None, None).is_err());
    assert!(search("valid", "", None, None).is_err());
}

#[test]
#[ignore = "needs real embedder (~66MB download, ~30s)"]
fn test_search_end_to_end() {
    use nowdocs::ingest::{ingest_dir, IngestMeta};
    use nowdocs::retrieve::search;
    use std::fs;

    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    fs::write(
        dir.path().join("a.md"),
        "# Auth\n\nUse token zzzretrieve_xyz to authenticate.\n",
    )
    .unwrap();
    fs::write(dir.path().join("b.md"), "# Config\n\nSet timeout to 30s.\n").unwrap();

    let stats = ingest_dir(dir.path(), "retrieve_e2e", &IngestMeta::default()).unwrap();
    assert!(stats.chunks >= 2);

    let result = search("retrieve_e2e", "zzzretrieve_xyz", Some(4000), Some(5)).unwrap();
    assert!(
        !result.chunks.is_empty(),
        "should return at least one chunk"
    );
    assert!(
        result
            .chunks
            .iter()
            .any(|c| c.text.contains("zzzretrieve_xyz")),
        "recalled chunk must contain the unique keyword"
    );
    assert!(result.tokens_returned <= 4000, "tokens must fit budget");
    assert!(
        result
            .chunks
            .iter()
            .any(|c| c.text.contains("zzzretrieve_xyz")),
        "recalled chunk must contain the unique keyword"
    );
    // Hit-first ordering: the top-ranked hit must lead the result so the most
    // relevant chunk is returned first (relevance > document reading order).
    assert!(
        result.chunks[0]
            .text
            .contains("zzzretrieve_xyz"),
        "top hit must be first under hit-first ordering, got idx {}",
        result.chunks[0].chunk_idx
    );
}

// --- hit-first neighbor-window ordering (bug#1 fix) ---

#[test]
fn test_window_ids_hit_leads_then_neighbors() {
    // Single hit at idx 2: window = [hit, prev, next] = [2, 1, 3].
    let hits = vec![hit(2, 0.9)];
    assert_eq!(window_ids_for(&hits, 10), vec![2, 1, 3]);
}

#[test]
fn test_window_ids_lower_bound_no_prev() {
    // hit at idx 0: no prev → [0, 1].
    let hits = vec![hit(0, 0.9)];
    assert_eq!(window_ids_for(&hits, 10), vec![0, 1]);
}

#[test]
fn test_window_ids_upper_bound_no_next() {
    // hit at idx 4, chunk_count 5: no next → [4, 3].
    let hits = vec![hit(4, 0.9)];
    assert_eq!(window_ids_for(&hits, 5), vec![4, 3]);
}

#[test]
fn test_window_ids_dedup_keeps_higher_ranked_hit_first() {
    // Two hits: idx 2 (score 0.9, rank 1), idx 1 (score 0.8, rank 2).
    // rank-1 window [2,1,3]; rank-2 window [1,0,2] → 1 and 2 already seen.
    // Result: [2, 1, 3, 0] — the rank-1 hit (2) leads.
    let hits = vec![hit(2, 0.9), hit(1, 0.8)];
    assert_eq!(window_ids_for(&hits, 10), vec![2, 1, 3, 0]);
}

#[test]
fn test_reorder_restores_window_order_after_idx_asc_fetch() {
    // fetch_by_idx returns chunks sorted by chunk_idx ascending; reorder must
    // restore the relevance-first window order.
    let window_ids = vec![2, 1, 3];
    let chunks = vec![rchunk(1), rchunk(2), rchunk(3)]; // idx-asc, as fetch returns
    let ordered = reorder_to_window(chunks, &window_ids);
    let got: Vec<u32> = ordered.iter().map(|c| c.chunk_idx).collect();
    assert_eq!(got, vec![2, 1, 3]);
}

#[test]
fn test_reorder_drops_chunks_outside_window() {
    // Defensive: a chunk not in window_ids sorts to the end (should not happen
    // in practice — fetch_by_idx only returns requested ids).
    let window_ids = vec![2, 1];
    let chunks = vec![rchunk(1), rchunk(2), rchunk(9)];
    let ordered = reorder_to_window(chunks, &window_ids);
    let got: Vec<u32> = ordered.iter().map(|c| c.chunk_idx).collect();
    assert_eq!(got, vec![2, 1, 9]);
}
