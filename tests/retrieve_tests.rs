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
    // lambda=1 collapses MMR to relevance (the diversity term vanishes). Here
    // all candidates have identical query-cosine, so normalized relevance ties
    // and the picks preserve input (score-descending) order.
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
    let out = mmr_rerank(&[1.0, 0.0, 0.0], hits, &v, 3, 1.0);
    let ids: Vec<u32> = out.iter().map(|h| h.chunk_idx).collect();
    assert_eq!(
        ids,
        vec![0, 1, 2],
        "lambda=1 with tied query-cosine must preserve score-descending order"
    );
}

#[test]
fn test_mmr_lambda_1_url_penalty_vanishes() {
    // Regression for Codex review: when lambda=1.0 (pure-relevance mode), the
    // URL penalty must be zero. Without the (1-lambda) scaling, a higher-cosine
    // same-URL chunk (h1) would be demoted below a lower-cosine cross-URL chunk
    // (h2) after h0 is picked, because the URL penalty fired even in pure-
    // relevance mode, making the diagnostic sweep misleading.
    //
    // Setup: h0 and h1 share "u0"; h2 is at "u1".
    //   query-cosine: h0=1.0, h1=0.9, h2=0.8
    // After picking h0, with correct (1-lambda)*penalty the second pick must be
    // h1 (cos 0.9 > 0.8), NOT h2.
    let hits = vec![
        hit_url(0, "u0", 0.9),
        hit_url(1, "u0", 0.8), // same URL as h0 — must NOT be penalised at lambda=1
        hit_url(2, "u1", 0.7), // different URL, lower cosine
    ];
    let v = vecs(&[
        (0, vec![1.0, 0.0, 0.0]),
        (1, vec![0.9, 0.436, 0.0]), // cos≈0.9 with query
        (2, vec![0.8, 0.6, 0.0]),   // cos≈0.8 with query
    ]);
    let out = mmr_rerank(&[1.0, 0.0, 0.0], hits, &v, 3, 1.0);
    let ids: Vec<u32> = out.iter().map(|h| h.chunk_idx).collect();
    assert_eq!(
        ids,
        vec![0, 1, 2],
        "lambda=1.0: URL penalty must vanish; order must be by query-cosine: {ids:?}"
    );
}

#[test]
fn test_mmr_prefers_diverse_urls_when_scores_similar() {
    // Equal query-cosine relevance: after the first pick, MMR must prefer the
    // diverse chunk over a near-duplicate of the first. All three vectors are
    // unit-norm with cosine 0.8 to the query [1,0,0] (h0/h1 in the x-y plane,
    // h1 rotated slightly around the query axis into z): relevance ties, so
    // the diversity term decides.
    let hits = vec![
        hit_url(0, "u0", 0.5),
        hit_url(1, "u1", 0.5),
        hit_url(2, "u2", 0.5),
    ];
    let eps: f32 = 0.1; // small rotation around the query axis
    let v = vecs(&[
        (0, vec![0.8, 0.6, 0.0]),
        (1, vec![0.8, 0.6 * eps.cos(), 0.6 * eps.sin()]), // ~identical to 0 (sim ≈ 0.998)
        (2, vec![0.8, -0.6, 0.0]), // same query-cosine, sim to h0 only 0.28 -> diverse
    ]);
    let out = mmr_rerank(&[1.0, 0.0, 0.0], hits, &v, 3, 0.7);
    let ids: Vec<u32> = out.iter().map(|h| h.chunk_idx).collect();
    assert_eq!(ids[0], 0, "first pick is the top-scored input (h0)");
    assert_eq!(
        ids[1], 2,
        "second pick must be the diverse chunk, got {ids:?}"
    );
    assert_eq!(ids[2], 1, "near-duplicate is pushed to last, got {ids:?}");
}

#[test]
fn test_mmr_relevance_is_query_cosine_not_rrf_score() {
    // The quality-lift fix: MMR relevance is the query–chunk cosine, not the
    // RRF score. With lambda=1 (pure relevance), a low-RRF-score chunk whose
    // vector aligns with the query must outrank a high-RRF-score chunk whose
    // vector is orthogonal to it.
    let hits = vec![
        hit_url(0, "u0", 0.9), // top RRF score, but orthogonal to the query
        hit_url(1, "u1", 0.1), // low RRF score, but query-aligned
    ];
    let v = vecs(&[(0, vec![0.0, 1.0, 0.0]), (1, vec![1.0, 0.0, 0.0])]);
    let out = mmr_rerank(&[1.0, 0.0, 0.0], hits, &v, 2, 1.0);
    let ids: Vec<u32> = out.iter().map(|h| h.chunk_idx).collect();
    assert_eq!(
        ids,
        vec![1, 0],
        "lambda=1 must order by query-cosine, not RRF score: {ids:?}"
    );
}

#[test]
fn test_mmr_url_penalty_suppresses_hub_chunks_with_diverse_vectors() {
    // hub1 and other2 have IDENTICAL query-cosine (0.70) and IDENTICAL
    // max-similarity to hub0 (also 0.70): the URL penalty
    // ((1-lambda)*0.05 = 0.3*0.05 = 0.015 at lambda=0.7) is the ONLY
    // tie-breaker, so other2 must be picked second over hub1.
    let hits = vec![
        hit_url(0, "hub.md", 0.9),
        hit_url(1, "hub.md", 0.8),
        hit_url(2, "other.md", 0.7),
    ];
    let v = vecs(&[
        (0, vec![1.0, 0.0, 0.0]),      // cos to query = 1.0
        (1, vec![0.7, 0.71414, 0.0]),  // cos to query = 0.7, sim to hub0 = 0.7
        (2, vec![0.7, -0.71414, 0.0]), // cos to query = 0.7, sim to hub0 = 0.7, diff URL
    ]);
    // After picking hub0 (cos=1.0):
    //   hub1:   mmr = 0.7*0.7 - 0.3*0.7 - 0.015*1 = 0.265
    //   other2: mmr = 0.7*0.7 - 0.3*0.7 - 0       = 0.280  → wins
    let out = mmr_rerank(&[1.0, 0.0, 0.0], hits, &v, 3, 0.7);
    let ids: Vec<u32> = out.iter().map(|h| h.chunk_idx).collect();
    assert_eq!(ids[0], 0, "top-cosine hub chunk leads");
    assert_eq!(
        ids[1], 2,
        "URL penalty must let the other page overtake the second hub chunk: {ids:?}"
    );
    assert_eq!(ids[2], 1);
}

#[test]
fn test_mmr_keeps_multiple_chunks_from_same_url_when_relevant() {
    // Same URL but orthogonal vectors + high scores: MMR must keep BOTH (it
    // diversifies by vector similarity, not by source_url). This is the core
    // fix vs the old dedup_by_source_url, which collapsed same-URL API chunks.
    let hits = vec![hit_url(0, "same.md", 0.5), hit_url(1, "same.md", 0.49)];
    let v = vecs(&[(0, vec![1.0, 0.0, 0.0]), (1, vec![0.0, 1.0, 0.0])]);
    let out = mmr_rerank(&[1.0, 0.0, 0.0], hits, &v, 2, 0.7);
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
    let out = mmr_rerank(&[1.0, 0.0, 0.0], hits, &v, 4, 0.7);
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
    // A highly relevant near-duplicate should not be pushed below a much less
    // relevant orthogonal chunk just because raw cosine gaps are small compared
    // to mutual-similarity penalties. Normalizing the query-cosine relevance to
    // [0, 1] within the pool keeps relevance and diversity on the same scale.
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
    let out = mmr_rerank(&[1.0, 0.0, 0.0], hits, &v, 3, 0.7);
    let ids: Vec<u32> = out.iter().map(|h| h.chunk_idx).collect();
    assert_eq!(ids[0], 0, "top hit leads");
    assert_eq!(
        ids[1], 1,
        "normalization must keep the relevant near-duplicate ahead of the low-ranked orthogonal chunk, got {ids:?}"
    );
    assert_eq!(ids[2], 2);
}

// --- A1.2 N4 + gate fix: cosine answer gate ---

use nowdocs::retrieve::{apply_answer_gate, MIN_ANSWER_COSINE};

/// Build a unit vector whose cosine similarity to the query vector `[1, 0]`
/// is exactly `c`: `[c, sqrt(1 - c^2)]`.
fn vec_with_cosine(c: f32) -> Vec<f32> {
    vec![c, (1.0 - c * c).max(0.0).sqrt()]
}

#[test]
fn test_search_returns_empty_when_top_score_below_threshold() {
    // The gate's "score" is now the top-hit query-cosine: below
    // MIN_ANSWER_COSINE must yield empty (no-answer) when the dual-channel
    // rank-1 bypass does not apply (single-channel RRF here).
    let hits = vec![hit(0, 0.016), hit(1, 0.010)];
    let qv = vec![1.0, 0.0];
    let low = MIN_ANSWER_COSINE - 0.10;
    let vectors = vecs(&[(0, vec_with_cosine(low)), (1, vec_with_cosine(low - 0.05))]);
    assert!(
        apply_answer_gate(hits, &qv, &vectors).is_empty(),
        "top cosine below threshold must yield empty (no-answer)"
    );
}

#[test]
fn test_search_returns_results_when_top_score_above_threshold() {
    let hits = vec![hit(0, 0.016), hit(1, 0.010)];
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
    // Single-channel rank-1 RRF score (1/60 ≈ 0.0167) but top-hit cosine below
    // MIN_ANSWER_COSINE must be blocked even though the old RRF-only gate
    // would have passed it.
    let hits = vec![hit(0, 0.0167), hit(1, 0.010)];
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
fn answer_gate_passes_high_cosine_despite_low_rrf_score() {
    // Measured deviation from the original dual-signal design: the secondary
    // RRF floor was dropped because cosine-ranked MMR legitimately surfaces
    // vector-only top hits at deep fused ranks (real Next.js corpus: caching /
    // authentication queries false-blocked at RRF < 0.015). A high-cosine hit
    // must PASS even when its RRF score is deep-rank low.
    let rank50_rrf = 1.0 / (50.0 + 60.0);
    let hits = vec![hit(0, rank50_rrf)];
    let qv = vec![1.0, 0.0];
    let vectors = vecs(&[(0, vec_with_cosine(0.90))]);
    let gated = apply_answer_gate(hits, &qv, &vectors);
    assert_eq!(
        gated.len(),
        1,
        "high-cosine hit must pass regardless of RRF depth (cosine-only gate)"
    );
}

#[test]
fn answer_gate_passes_dual_channel_rank1_despite_low_cosine() {
    // Dual-channel rank-1 agreement (RRF = 2/60 ≈ 0.0333) bypasses the cosine
    // floor: keyword-dense queries on small docsets under-score on embedding
    // cosine even when BOTH retrievers independently rank the right chunk #1.
    let dual_rank1 = 2.0 / 60.0;
    let hits = vec![hit(0, dual_rank1), hit(1, 0.016)];
    let qv = vec![1.0, 0.0];
    let low = MIN_ANSWER_COSINE - 0.15;
    let vectors = vecs(&[(0, vec_with_cosine(low)), (1, vec_with_cosine(low))]);
    let gated = apply_answer_gate(hits, &qv, &vectors);
    assert_eq!(
        gated.len(),
        2,
        "dual-channel rank-1 agreement must bypass the cosine floor"
    );
}

#[test]
fn answer_gate_blocks_low_cosine_single_channel_hit() {
    // A low-cosine hit with only single-channel rank-1 RRF (1/60 ≈ 0.0167 —
    // every query gets one of these from the vector channel) must be blocked:
    // this was the structural FP hole in the original RRF-only gate.
    let single_rank1 = 1.0 / 60.0;
    let hits = vec![hit(0, single_rank1)];
    let qv = vec![1.0, 0.0];
    let low = MIN_ANSWER_COSINE - 0.15;
    let vectors = vecs(&[(0, vec_with_cosine(low))]);
    assert!(
        apply_answer_gate(hits, &qv, &vectors).is_empty(),
        "low-cosine single-channel rank-1 hit must be blocked"
    );
}

// --- N2: downcastable sentinel error types ---

#[test]
fn retrieve_search_returns_docset_not_installed_sentinel() {
    use nowdocs::retrieve::{search, DocsetNotInstalled};
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };
    let err =
        search("no_such_docset", "some query", None, None).expect_err("missing docset must error");
    assert!(
        err.downcast_ref::<DocsetNotInstalled>().is_some(),
        "missing manifest must surface as DocsetNotInstalled sentinel, got: {err:#}"
    );
}

#[test]
#[ignore = "needs real embedder + nextjs_real fixture (scripts/ci-prepare-nextjs-fixture.sh)"]
fn retrieve_search_returns_store_error_sentinel() {
    use nowdocs::cache::manifest_path;
    use nowdocs::retrieve::{search, StoreError};

    // Plant a valid manifest for a docset whose .lance store is CORRUPT (a
    // plain file at the .lance path): manifest read/parse/validate + embedder
    // load all succeed, then the store open/search must fail with the
    // StoreError sentinel. (A merely MISSING store is not an error — lancedb
    // creates an empty table and search returns empty.) Uses the DEFAULT cache
    // (matches ensure_nextjs_real) so the cached model is reused.
    unsafe { std::env::remove_var("XDG_CACHE_HOME") };
    let real_manifest = manifest_path("nextjs_real");
    assert!(
        real_manifest.exists(),
        "run scripts/ci-prepare-nextjs-fixture.sh first"
    );
    let manifest_json = std::fs::read_to_string(&real_manifest).unwrap();

    let fake = "sentinel_nostore";
    let fake_manifest = manifest_path(fake);
    let fake_lance = fake_manifest.with_file_name(format!("{fake}.lance"));
    // A missing store is not an error — lancedb creates an empty table dir and
    // search legitimately returns empty. Plant a CORRUPT store instead: a
    // regular file at the .lance path makes table open fail.
    let _ = std::fs::remove_dir_all(&fake_lance);
    std::fs::write(&fake_manifest, &manifest_json).unwrap();
    std::fs::write(&fake_lance, b"not a lance table").unwrap();

    let result = search(fake, "server components", Some(4000), Some(5));

    // Clean up the planted manifest + corrupt store regardless of outcome.
    let _ = std::fs::remove_file(&fake_manifest);
    let _ = std::fs::remove_file(&fake_lance);
    let _ = std::fs::remove_dir_all(&fake_lance);

    let err = result.expect_err("docset without a store must error");
    assert!(
        err.downcast_ref::<StoreError>().is_some(),
        "missing store must surface as StoreError sentinel, got: {err:#}"
    );
}

// --- C02: behavior-preserving retrieval trace ---

use nowdocs::retrieve::rank_and_gate_candidates;

/// Query vector for the trace tests: unit vector on the x axis, so a
/// candidate's query-cosine is just its first component (unit-normalized).
fn qv() -> Vec<f32> {
    vec![1.0, 0.0]
}

/// Fused candidate pool for `trace_does_not_change_rank_or_gate`: RRF scores
/// sit below DUAL_RANK1_RRF so the gate decision is driven by cosine alone.
fn hits() -> Vec<SearchHit> {
    vec![
        hit_url(0, "a.md", 0.030),
        hit_url(1, "b.md", 0.029),
        hit_url(2, "c.md", 0.028),
        hit_url(3, "d.md", 0.027),
    ]
}

/// Query-cosines 0.90 / 0.85 / 0.80 / 0.70: the top hit clears
/// MIN_ANSWER_COSINE, so the gate passes.
fn vectors() -> HashMap<u32, Vec<f32>> {
    vecs(&[
        (0, vec_with_cosine(0.90)),
        (1, vec_with_cosine(0.85)),
        (2, vec_with_cosine(0.80)),
        (3, vec_with_cosine(0.70)),
    ])
}

#[test]
fn trace_does_not_change_rank_or_gate() {
    let plain = rank_and_gate_candidates(&qv(), hits(), &vectors(), 3, false);
    let traced = rank_and_gate_candidates(&qv(), hits(), &vectors(), 3, true);
    let plain_ids: Vec<_> = plain.hits.iter().map(|hit| hit.chunk_idx).collect();
    let traced_ids: Vec<_> = traced.hits.iter().map(|hit| hit.chunk_idx).collect();
    assert_eq!(plain_ids, traced_ids);
    assert_eq!(plain.gate_passed, traced.gate_passed);
    assert!(
        plain.trace.is_none(),
        "tracing disabled must not allocate a trace"
    );
    assert!(traced.trace.is_some());
    assert_eq!(plain, plain.clone());
    assert!(format!("{traced:?}").contains("RankedGateResult"));
}

#[test]
fn trace_pre_mmr_cosines_come_from_fused_pool_not_mmr_order() {
    // Chunk 1 is the second-best fused raw cosine (0.94) but a near-duplicate
    // of chunk 0, so MMR demotes it below the diverse chunk 2 (cosine 0.90).
    // pre_mmr_top_cosines must therefore list 0.95, 0.94 — the fused-pool
    // distribution — not the MMR selection's 0.95, 0.90.
    let hits = vec![
        hit_url(0, "a.md", 0.030),
        hit_url(1, "b.md", 0.029),
        hit_url(2, "c.md", 0.028),
    ];
    let mut diverse = vec_with_cosine(0.90);
    diverse[1] = -diverse[1]; // same query-cosine, dissimilar to chunk 0
    let vectors = vecs(&[
        (0, vec_with_cosine(0.95)),
        (1, vec_with_cosine(0.94)), // near-duplicate of chunk 0
        (2, diverse),
    ]);
    let result = rank_and_gate_candidates(&qv(), hits, &vectors, 3, true);
    assert!(result.gate_passed, "top cosine 0.95 must pass the gate");
    let trace = result.trace.expect("trace enabled");

    let mmr_ids: Vec<u32> = trace.mmr.iter().map(|t| t.chunk_idx).collect();
    assert_eq!(
        mmr_ids,
        vec![0, 2, 1],
        "MMR must prefer the diverse chunk over the near-duplicate runner-up"
    );

    assert_eq!(trace.pre_mmr_top_cosines.len(), 3);
    assert!(
        (trace.pre_mmr_top_cosines[0] - 0.95).abs() < 1e-4,
        "got {}",
        trace.pre_mmr_top_cosines[0]
    );
    assert!(
        (trace.pre_mmr_top_cosines[1] - 0.94).abs() < 1e-4,
        "second value must be the fused pool's runner-up cosine 0.94, \
         not the MMR runner-up 0.90: got {}",
        trace.pre_mmr_top_cosines[1]
    );

    // Fused trace view: pool order, RRF scores, channel ranks not yet
    // supplied at this wave (C04 adds rank evidence).
    let fused_ids: Vec<u32> = trace.fused.iter().map(|t| t.chunk_idx).collect();
    assert_eq!(fused_ids, vec![0, 1, 2]);
    assert!((trace.fused[0].rrf_score - 0.030).abs() < 1e-6);
    assert!(trace
        .fused
        .iter()
        .all(|t| t.dense_rank.is_none() && t.lexical_rank.is_none()));
}
