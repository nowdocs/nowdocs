//! Retrieval pipeline: embed query -> hybrid search -> neighbor-window assembly.

use std::collections::HashMap;

use anyhow::{Context, Result};

use crate::chunker::ChunkType;
use crate::embedder::{self, EmbedderSpec};
use crate::input::{resolve_max_tokens, resolve_top_k, validate_docset, validate_query};
use crate::manifest;
use crate::store::{SearchHit, Store};
use crate::token::count_tokens;

/// MMR lambda: weight of relevance vs. redundancy (0.7 = relevance-leaning).
/// `MMR = lambda * relevance - (1 - lambda) * max_cosine_sim`. (N1/OQ4)
const MMR_LAMBDA: f32 = 0.7;

/// Minimum top-hit relevance to return any results (N4/OQ11). If the best hit
/// is below this floor the docset is treated as having no answer and `search`
/// returns empty rather than the irrelevant top-K.
///
/// The default RRF fusion constant is `k=60`, i.e. `score = 1/(rank + 60)`. A
/// hit that surfaces in only ONE channel (vector-only or FTS-only) tops out at
/// about `1/60 ≈ 0.0167` for rank 1, while a hit in BOTH channels can reach
/// `2/61 ≈ 0.0328`. The gate must therefore stay BELOW the single-channel
/// rank-1 floor (~0.0164) or it would discard every vector-only / FTS-only
/// answer. 0.015 sits just under that floor so single-channel top matches pass
/// while deep-noise hits (rank ≳ 10 single-channel, ~0.014) are still dropped.
/// Proper calibration against real Next.js eval data is deferred to A1.3.
// TODO(A1.3): calibrate threshold against real eval data.
pub const MIN_RELEVANCE_THRESHOLD: f32 = 0.015;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub chunks: Vec<ResultChunk>,
    pub tokens_returned: u32,
    pub truncated: bool,
}

#[derive(Debug, Clone)]
pub struct ResultChunk {
    pub chunk_idx: u32,
    pub heading_path: String,
    pub source_url: String,
    pub api_version: Option<String>,
    pub chunk_type: ChunkType,
    pub text: String,
    pub score: Option<f32>,
}

pub fn search(
    docset: &str,
    query: &str,
    max_tokens: Option<u32>,
    top_k: Option<u32>,
) -> Result<SearchResult> {
    let docset = validate_docset(docset)?;
    let query = validate_query(query)?;
    let max_tokens = resolve_max_tokens(max_tokens);
    let top_k = resolve_top_k(top_k);

    // Load and validate manifest.
    let manifest_path = crate::cache::manifest_path(&docset);
    let manifest_json = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read manifest for docset {docset:?}"))?;
    let manifest: manifest::Manifest = manifest::parse_manifest(&manifest_json)?;
    manifest::validate(&manifest)?;

    // Build embedder spec from manifest (model-version lock).
    let embedder_spec = EmbedderSpec {
        model_id: manifest.embedder.model_id.clone(),
        model_revision: manifest.embedder.model_revision.clone(),
        model_sha256: manifest.embedder.model_sha256.clone(),
    };
    let embedder = embedder::Embedder::load_for(&embedder_spec)?;

    // Embed query and run hybrid search.
    let query_vector = embedder.embed(&query)?;
    let store = Store::open(&docset)?;
    // Over-fetch (top_k*3, min 15) then diversity-rerank with Maximal Marginal
    // Relevance (N1/OQ4). MMR keeps multiple chunks from the same source_url
    // when their vectors differ (e.g. distinct APIs on one reference page),
    // fixing the old `dedup_by_source_url` heuristic that collapsed single-page
    // API references. Over-fetching gives MMR a candidate pool to trade
    // relevance off against redundancy before the window pass re-attaches
    // same-file neighbors as context.
    let raw_k = (top_k * 3).max(15) as usize;
    let candidates = store.hybrid_search(&query_vector, &query, raw_k)?;
    let cand_ids: Vec<u32> = candidates.iter().map(|h| h.chunk_idx).collect();
    let vectors = store.fetch_vectors(&cand_ids)?;
    let hits = mmr_rerank(candidates, &vectors, top_k as usize, MMR_LAMBDA);

    // N4/OQ11: if the best hit is still below the relevance floor, the docset
    // likely has no answer (e.g. Vue syntax queried against React docs) — return
    // empty rather than the irrelevant top-K.
    let hits = apply_relevance_gate(hits);
    if hits.is_empty() {
        return Ok(SearchResult {
            chunks: vec![],
            tokens_returned: 0,
            truncated: false,
        });
    }

    // Build the neighbor window hit-first: all hybrid hits lead (rank 1..N),
    // then their `[hit-1, hit+1]` neighbors follow as context. This keeps a
    // hit recalled by hybrid inside the top-K — the earlier per-hit interleave
    // (`[hit, hit-1, hit+1]`) let hit1..4's neighbors push hit5 from hybrid
    // rank 5 to retrieve rank ~7, squeezing it out of recall@5. See
    // `window_ids_for`.
    let chunk_count = manifest.source.chunk_count as u32;
    let window_ids = window_ids_for(&hits, chunk_count);

    // Build score map from hybrid hits so neighbors can carry None.
    let score_map: std::collections::HashMap<u32, f32> =
        hits.iter().map(|h| (h.chunk_idx, h.score)).collect();

    // Fetch window chunks by chunk_idx (store returns them idx-ascending), then
    // restore the relevance-first window order.
    let window_hits = store.fetch_by_idx(&window_ids)?;
    let window_chunks: Vec<ResultChunk> = window_hits
        .into_iter()
        .map(|h| ResultChunk {
            score: score_map.get(&h.chunk_idx).copied(),
            chunk_idx: h.chunk_idx,
            heading_path: h.heading_path,
            source_url: h.source_url,
            api_version: h.api_version,
            chunk_type: h.chunk_type,
            text: h.text,
        })
        .collect();
    let window_chunks = reorder_to_window(window_chunks, &window_ids);

    // Assemble within max_tokens budget.
    assemble_result(window_chunks, max_tokens)
}

/// Maximal Marginal Relevance rerank (N1/OQ4). Greedy: repeatedly pick the
/// candidate maximizing `lambda * relevance - (1 - lambda) * max_sim`, where
/// `max_sim` is the candidate's largest cosine similarity to any already-picked
/// chunk. Replaces the old `dedup_by_source_url`: diversity comes from vector
/// dissimilarity, so multiple chunks from one URL survive when they cover
/// distinct APIs. Candidates without a fetched vector fall back to score order,
/// appended after the MMR-selected hits (should not happen post-S1).
pub fn mmr_rerank(
    hits: Vec<SearchHit>,
    vectors: &HashMap<u32, Vec<f32>>,
    top_k: usize,
    lambda: f32,
) -> Vec<SearchHit> {
    if hits.is_empty() || top_k == 0 {
        return Vec::new();
    }

    let (mut pool, mut fallback): (Vec<SearchHit>, Vec<SearchHit>) = hits
        .into_iter()
        .partition(|h| vectors.contains_key(&h.chunk_idx));
    fallback.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Codex review fix: RRF scores (~0.01-0.03) and cosine similarity (0-1)
    // are on very different scales, so the raw diversity term would dominate
    // after the first pick. Normalize the RRF relevance scores of the vector
    // pool to [0, 1] within this candidate set so lambda=0.7 actually means
    // "relevance-leaning" rather than "diversity-leaning".
    let min_score = pool
        .iter()
        .map(|h| h.score)
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(0.0);
    let max_score = pool
        .iter()
        .map(|h| h.score)
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(0.0);
    let score_range = max_score - min_score;
    let normalize_score = |s: f32| {
        if score_range > 0.0 {
            (s - min_score) / score_range
        } else {
            1.0
        }
    };

    let mut selected: Vec<SearchHit> = Vec::with_capacity(top_k);

    while selected.len() < top_k && !pool.is_empty() {
        let mut best_i = 0;
        let mut best_score = f32::MIN;
        for (i, cand) in pool.iter().enumerate() {
            let max_sim = max_cosine_to_selected(cand, &selected, vectors);
            let rel = normalize_score(cand.score);
            let mmr = lambda * rel - (1.0 - lambda) * max_sim;
            if mmr > best_score {
                best_score = mmr;
                best_i = i;
            }
        }
        selected.push(pool.remove(best_i));
    }

    // Pool exhausted: fill remaining slots from vector-less fallbacks (score order).
    while selected.len() < top_k && !fallback.is_empty() {
        selected.push(fallback.remove(0));
    }

    selected
}

fn max_cosine_to_selected(
    cand: &SearchHit,
    selected: &[SearchHit],
    vectors: &HashMap<u32, Vec<f32>>,
) -> f32 {
    let Some(cv) = vectors.get(&cand.chunk_idx) else {
        return 0.0;
    };
    let mut best: f32 = 0.0;
    for s in selected {
        if let Some(sv) = vectors.get(&s.chunk_idx) {
            let sim = cosine(cv, sv);
            if sim > best {
                best = sim;
            }
        }
    }
    best
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

/// Apply the no-answer relevance gate (N4/OQ11): return `Vec::new()` when there
/// are no hits or the top hit is below `MIN_RELEVANCE_THRESHOLD`, else return
/// the hits unchanged. Exposed (`pub`) so the gate is unit-testable without a
/// full docset.
pub fn apply_relevance_gate(hits: Vec<SearchHit>) -> Vec<SearchHit> {
    if hits.is_empty() || hits[0].score < MIN_RELEVANCE_THRESHOLD {
        Vec::new()
    } else {
        hits
    }
}

/// Build the relevance-ordered neighbor-window index list from hybrid search
/// hits. `hits` must be in score-descending order (as returned by
/// `Store::hybrid_search`).
///
/// Two passes keep every hybrid hit inside the top-K: (1) all hits in
/// relevance order, then (2) each hit's `[hit-1, hit+1]` neighbors as context.
/// The earlier per-hit interleave (`[hit, hit-1, hit+1]`) let hit1..4's
/// neighbors push hit5 from hybrid rank 5 to retrieve rank ~7 — a hit recalled
/// by hybrid was squeezed out of the top-K window by *other hits'* context.
/// Hit-first ordering keeps the window additive: it only ever appends context
/// below the hits, never displaces them. (Note: `search` MMR-selects hits
/// before this, so same-URL neighbors re-attached here may legitimately extend
/// a page that contributed multiple diverse chunks.) Indices are clamped to
/// `[0, chunk_count)`.
pub fn window_ids_for(hits: &[SearchHit], chunk_count: u32) -> Vec<u32> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Pass 1: hits in relevance order so rank 1..N lead the result.
    for hit in hits {
        let idx = hit.chunk_idx;
        if idx < chunk_count && seen.insert(idx) {
            out.push(idx);
        }
    }

    // Pass 2: neighbor context [hit-1, hit+1] per hit, relevance order.
    for hit in hits {
        for delta in [-1i32, 1] {
            let idx = hit.chunk_idx as i32 + delta;
            if idx >= 0 && (idx as u32) < chunk_count {
                let u = idx as u32;
                if seen.insert(u) {
                    out.push(u);
                }
            }
        }
    }
    out
}

/// Reorder chunks fetched by idx (ascending) back into the relevance-first
/// window order defined by `window_ids`. Chunks whose idx is absent from
/// `window_ids` sort to the end (defensive — `fetch_by_idx` only returns
/// requested ids in practice).
pub fn reorder_to_window(chunks: Vec<ResultChunk>, window_ids: &[u32]) -> Vec<ResultChunk> {
    let mut order: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for (i, &id) in window_ids.iter().enumerate() {
        order.insert(id, i);
    }
    let mut chunks = chunks;
    chunks.sort_by_key(|c| order.get(&c.chunk_idx).copied().unwrap_or(usize::MAX));
    chunks
}

fn assemble_result(chunks: Vec<ResultChunk>, max_tokens: u32) -> Result<SearchResult> {
    let mut selected = Vec::new();
    let mut tokens_used: u32 = 0;
    let mut truncated = false;

    for chunk in chunks {
        let n = count_tokens(&chunk.text) as u32;
        if selected.is_empty() {
            selected.push(chunk);
            tokens_used += n;
            continue;
        }
        if tokens_used + n > max_tokens {
            truncated = true;
            break;
        }
        selected.push(chunk);
        tokens_used += n;
    }

    // If even the first chunk exceeds budget, still return it but mark truncated.
    if selected.len() == 1 && tokens_used > max_tokens {
        truncated = true;
    }

    Ok(SearchResult {
        chunks: selected,
        tokens_returned: tokens_used,
        truncated,
    })
}
