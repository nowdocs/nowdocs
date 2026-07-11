use nowdocs::chunker::ChunkType;
use nowdocs::retrieve::{reorder_to_window, window_ids_for, ResultChunk, SearchResult};
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
        score: None,
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
            score: None,
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
        result.chunks[0].text.contains("zzzretrieve_xyz"),
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
    // Pass 1 places hits [2, 1]; pass 2 adds neighbors: hit 2's [1,3] → 3
    // (1 already seen), hit 1's [0,2] → 0 (2 already seen). Result
    // [2, 1, 3, 0] — both hits lead, neighbors follow.
    let hits = vec![hit(2, 0.9), hit(1, 0.8)];
    assert_eq!(window_ids_for(&hits, 10), vec![2, 1, 3, 0]);
}

#[test]
fn test_window_ids_all_hits_lead_before_neighbors() {
    // Three non-adjacent hits: idx 2, 5, 8 (ranks 1, 2, 3). Hit-first ordering
    // places all hits [2, 5, 8] before any neighbor, so hit3 stays at rank 3
    // instead of being pushed to ~7 by hit1/hit2's neighbors — this is the
    // recall@5 squeeze fix. Pass 2 appends neighbors [1,3, 4,6, 7,9].
    let hits = vec![hit(2, 0.9), hit(5, 0.8), hit(8, 0.7)];
    assert_eq!(window_ids_for(&hits, 10), vec![2, 5, 8, 1, 3, 4, 6, 7, 9]);
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

// --- A1.2 N1: true vector MMR (replaces source_url dedup) ---

use nowdocs::retrieve::mmr_rerank;
use std::collections::HashMap;

fn hit_url(chunk_idx: u32, url: &str, score: f32) -> SearchHit {
    SearchHit {
        score,
        chunk_idx,
        heading_path: String::new(),
        source_url: url.to_string(),
        api_version: None,
        chunk_type: ChunkType::Info,
        text: String::new(),
    }
}

fn vecs(pairs: &[(u32, Vec<f32>)]) -> HashMap<u32, Vec<f32>> {
    pairs.iter().cloned().collect()
}

#[test]
fn test_mmr_lambda_1_equals_pure_relevance_ordering() {
    // lambda=1 collapses MMR to relevance (the diversity term vanishes), so the
    // result is strictly score-descending regardless of vector similarity.
    let hits = vec![
        hit_url(0, "u0", 0.5),
        hit_url(1, "u1", 0.4),
        hit_url(2, "u2", 0.3),
    ];
    let v = vecs(&[
        (0, vec![1.0, 0.0, 0.0]),
        (1, vec![1.0, 0.0, 0.0]), // identical to 0 — would be penalized if lambda<1
        (2, vec![1.0, 0.0, 0.0]),
    ]);
    let out = mmr_rerank(hits, &v, 3, 1.0);
    let ids: Vec<u32> = out.iter().map(|h| h.chunk_idx).collect();
    assert_eq!(
        ids,
        vec![0, 1, 2],
        "lambda=1 must yield score-descending order"
    );
}

#[test]
fn test_mmr_prefers_diverse_urls_when_scores_similar() {
    // Equal scores: after the first pick, MMR must prefer the orthogonal
    // (diverse) chunk over a near-duplicate of the first.
    let hits = vec![
        hit_url(0, "u0", 0.5),
        hit_url(1, "u1", 0.5),
        hit_url(2, "u2", 0.5),
    ];
    let v = vecs(&[
        (0, vec![1.0, 0.0, 0.0]),
        (1, vec![0.99, 0.01, 0.0]), // ~identical to 0
        (2, vec![0.0, 1.0, 0.0]),   // orthogonal to 0 -> diverse
    ]);
    let out = mmr_rerank(hits, &v, 3, 0.7);
    let ids: Vec<u32> = out.iter().map(|h| h.chunk_idx).collect();
    assert_eq!(ids[0], 0, "first pick is the top-scored input (h0)");
    assert_eq!(
        ids[1], 2,
        "second pick must be the diverse chunk, got {ids:?}"
    );
    assert_eq!(ids[2], 1, "near-duplicate is pushed to last, got {ids:?}");
}

#[test]
fn test_mmr_keeps_multiple_chunks_from_same_url_when_relevant() {
    // Same URL but orthogonal vectors + high scores: MMR must keep BOTH (it
    // diversifies by vector similarity, not by source_url). This is the core
    // fix vs the old dedup_by_source_url, which collapsed same-URL API chunks.
    let hits = vec![hit_url(0, "same.md", 0.5), hit_url(1, "same.md", 0.49)];
    let v = vecs(&[(0, vec![1.0, 0.0, 0.0]), (1, vec![0.0, 1.0, 0.0])]);
    let out = mmr_rerank(hits, &v, 2, 0.7);
    let ids: Vec<u32> = out.iter().map(|h| h.chunk_idx).collect();
    assert_eq!(out.len(), 2, "both same-URL chunks must be retained");
    assert!(ids.contains(&0) && ids.contains(&1));
}

#[test]
fn test_mmr_with_missing_vector_falls_back_to_score_order() {
    // Hits without a fetched vector fall back to score order, appended after the
    // MMR-selected (vectored) hits.
    let hits = vec![
        hit_url(0, "u0", 0.5),
        hit_url(1, "u1", 0.9), // no vector
        hit_url(2, "u2", 0.4),
        hit_url(3, "u3", 0.8), // no vector
    ];
    let v = vecs(&[(0, vec![1.0, 0.0, 0.0]), (2, vec![0.0, 1.0, 0.0])]);
    let out = mmr_rerank(hits, &v, 4, 0.7);
    let ids: Vec<u32> = out.iter().map(|h| h.chunk_idx).collect();
    // Vectored hits (0, 2) lead; vector-less fallbacks (1, 3) follow in
    // score-descending order (0.9 before 0.8).
    assert_eq!(
        ids,
        vec![0, 2, 1, 3],
        "fallbacks must be score-ordered: {ids:?}"
    );
}

#[test]
fn test_mmr_normalizes_scores_to_avoid_over_penalizing_near_duplicates() {
    // Codex review case: a highly relevant near-duplicate should not be pushed
    // below a much lower-ranked orthogonal chunk just because raw RRF scores
    // are tiny compared to cosine values. Normalization puts relevance and
    // diversity on the same scale.
    let hits = vec![
        hit_url(0, "u0", 0.030), // top hit
        hit_url(1, "u1", 0.029), // almost as relevant, near-duplicate vector
        hit_url(2, "u2", 0.016), // lower ranked, orthogonal vector
    ];
    let v = vecs(&[
        (0, vec![1.0, 0.0, 0.0]),
        (1, vec![0.99, 0.01, 0.0]), // near-duplicate of 0
        (2, vec![0.0, 1.0, 0.0]),   // orthogonal
    ]);
    let out = mmr_rerank(hits, &v, 3, 0.7);
    let ids: Vec<u32> = out.iter().map(|h| h.chunk_idx).collect();
    assert_eq!(ids[0], 0, "top hit leads");
    assert_eq!(
        ids[1], 1,
        "normalization must keep the relevant near-duplicate ahead of the low-ranked orthogonal chunk, got {ids:?}"
    );
    assert_eq!(ids[2], 2);
}

// --- A1.2 N4 + gate fix: dual-signal answer gate (cosine primary, RRF secondary) ---

use nowdocs::retrieve::{apply_answer_gate, MIN_ANSWER_COSINE, MIN_RELEVANCE_THRESHOLD};

/// Build a unit vector whose cosine similarity to the query vector `[1, 0]`
/// is exactly `c`: `[c, sqrt(1 - c^2)]`.
fn vec_with_cosine(c: f32) -> Vec<f32> {
    vec![c, (1.0 - c * c).max(0.0).sqrt()]
}

#[test]
fn test_search_returns_empty_when_top_score_below_threshold() {
    // High cosine but RRF score below the secondary floor (deep-noise rank)
    // must still be blocked.
    let hits = vec![hit(0, MIN_RELEVANCE_THRESHOLD / 2.0), hit(1, 0.001)];
    let qv = vec![1.0, 0.0];
    let vectors = vecs(&[(0, vec_with_cosine(0.90)), (1, vec_with_cosine(0.85))]);
    assert!(
        apply_answer_gate(hits, &qv, &vectors).is_empty(),
        "top score below threshold must yield empty (no-answer)"
    );
}

#[test]
fn test_search_returns_results_when_top_score_above_threshold() {
    let hits = vec![hit(0, MIN_RELEVANCE_THRESHOLD + 1.0), hit(1, 0.0)];
    let qv = vec![1.0, 0.0];
    let vectors = vecs(&[(0, vec_with_cosine(0.90)), (1, vec_with_cosine(0.10))]);
    let gated = apply_answer_gate(hits, &qv, &vectors);
    assert_eq!(gated.len(), 2, "above-threshold top hit must pass through");
}

#[test]
fn test_relevance_gate_empty_stays_empty() {
    let qv = vec![1.0, 0.0];
    assert!(apply_answer_gate(Vec::new(), &qv, &HashMap::new()).is_empty());
}

#[test]
fn answer_gate_blocks_low_cosine_hit() {
    // Strong RRF score (dual-channel rank-1) but top-hit cosine 0.60 — below
    // MIN_ANSWER_COSINE — must be blocked even though the old RRF-only gate
    // would have passed it.
    let hits = vec![hit(0, 0.033), hit(1, 0.016)];
    let qv = vec![1.0, 0.0];
    let low = MIN_ANSWER_COSINE - 0.10;
    let vectors = vecs(&[(0, vec_with_cosine(low)), (1, vec_with_cosine(low - 0.05))]);
    assert!(
        apply_answer_gate(hits, &qv, &vectors).is_empty(),
        "top hit below MIN_ANSWER_COSINE must yield empty (no-answer)"
    );
}

#[test]
fn answer_gate_passes_high_cosine_hit() {
    let hits = vec![hit(0, 0.033), hit(1, 0.016)];
    let qv = vec![1.0, 0.0];
    let high = MIN_ANSWER_COSINE + 0.10;
    let vectors = vecs(&[(0, vec_with_cosine(high)), (1, vec_with_cosine(0.20))]);
    let gated = apply_answer_gate(hits, &qv, &vectors);
    assert_eq!(
        gated.len(),
        2,
        "top hit above MIN_ANSWER_COSINE with a healthy RRF score must pass"
    );
}

#[test]
fn answer_gate_combines_cosine_and_rrf() {
    // A hit with high cosine (0.90) but an extremely low RRF score — rank ~50
    // single-channel, 1/(50+60) ≈ 0.0091 — is deep noise that happens to have
    // moderate vector similarity; the secondary RRF filter must still block it.
    let rank50_rrf = 1.0 / (50.0 + 60.0);
    let hits = vec![hit(0, rank50_rrf)];
    let qv = vec![1.0, 0.0];
    let vectors = vecs(&[(0, vec_with_cosine(0.90))]);
    assert!(
        apply_answer_gate(hits, &qv, &vectors).is_empty(),
        "high-cosine but deep-rank hit must be blocked by the secondary RRF filter"
    );
}
