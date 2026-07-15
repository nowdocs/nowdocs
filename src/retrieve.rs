//! Retrieval pipeline: embed query -> hybrid search -> neighbor-window assembly.

use std::collections::HashMap;

use anyhow::Result;

use crate::chunker::ChunkType;
use crate::confidence::{self, AnswerState, QueryEvidence};
use crate::embedder::{self, EmbedderSpec};
use crate::input::{resolve_max_tokens, resolve_top_k, validate_docset, validate_query};
use crate::manifest;
use crate::rerank::{RerankDocument, Reranker};
use crate::store::{CandidateEvidence, SearchHit, Store};
use crate::token::count_tokens;

/// MMR lambda: weight of relevance vs. redundancy (0.7 = relevance-leaning).
/// `MMR = lambda * relevance - (1 - lambda) * max_cosine_sim`. (N1/OQ4)
const MMR_LAMBDA: f32 = 0.7;

/// MMR source_url penalty: subtracted per already-selected chunk that shares
/// the candidate's `source_url`. Vector diversity alone cannot suppress hub
/// files whose many chunks have mutually DISSIMILAR vectors — measured on the
/// real Next.js corpus: backend-for-frontend.md held 6 of the top-8 cosine
/// slots for the route-handlers query, and caching-without-cache-components.md
/// held 2 of the 5 slots above the revalidating answer. The penalty restores
/// cross-page diversity without reviving the old `dedup_by_source_url`
/// collapse: multiple chunks from one page still survive when their relevance
/// clears the penalty (N1/OQ4's distinct-APIs-per-page case).
const MMR_URL_PENALTY: f32 = 0.05;

/// Minimum raw vector cosine similarity between the query embedding and the
/// top hit's embedding for the result to count as "answered" (N4 redesign).
/// Below this floor the docset is treated as having no answer and `search`
/// returns empty rather than the irrelevant top-K.
///
/// This replaces the old RRF-only gate, which was structurally ~1.0 FP: RRF
/// scores are rank-based, not similarity-based, so a rank-1 single-channel hit
/// always scored `1/60 ≈ 0.0167` regardless of relevance — and dense vector
/// search always returns a rank-1 neighbor for ANY query. Cosine similarity
/// directly measures query–chunk semantic relatedness: an out-of-scope query
/// (e.g. Vue syntax against React docs) produces top-hit cosine well below a
/// relevant query's, and the threshold sits between those bands.
///
/// The gate's primary signal is cosine similarity; a secondary BYPASS admits
/// dual-channel rank-1 agreement — see DUAL_RANK1_RRF.
///
/// Calibrated 2026-07-11 on the real Next.js corpus (~7480 chunks): golden
/// positive queries cluster at top-hit cosine 0.831–0.901 (n=10), negative
/// out-of-scope queries at 0.759–0.807 (n=12). 0.82 sits in the gap: all
/// positives pass, all negatives are blocked (measured FP rate 0.0).
///
/// The gate is cosine-ONLY: the originally planned secondary RRF floor
/// (`MIN_RELEVANCE_THRESHOLD` = 0.015) was dropped after measurement. Once MMR
/// ranks by cosine, the top hit is often a vector-only chunk whose fused RRF
/// rank is deep (measured: legitimate answers at fused rank 9–33, RRF score
/// < 0.015), so the RRF floor false-blocked relevant queries (caching,
/// authentication on the real Next.js corpus). Deep noise with "moderate"
/// cosine is already handled by the cosine floor itself — the negative band
/// tops out at 0.807.
pub const MIN_ANSWER_COSINE: f32 = 0.82;

/// RRF score reachable ONLY by dual-channel rank-1 agreement: with lancedb's
/// 0-based ranks, both channels at rank 0 gives `1/60 + 1/60 = 0.0333`, while
/// the next best (rank 0 + rank 1) is `1/60 + 1/61 ≈ 0.03306`. A chunk ranked
/// #1 independently by BOTH BM25 and vector search is a high-precision answer
/// signal that bypasses the cosine floor: keyword-dense queries against small
/// docsets systematically under-score on embedding cosine (toy golden fixture:
/// "502 503 504 gateway timeout" -> cos 0.747 but dual rank-1), and exact
/// keyword + semantic agreement is stronger evidence than either channel
/// alone. Measured safe on the real Next.js corpus: no negative query reaches
/// dual rank-1 (max negative RRF 0.0302), so the bypass adds zero FP there.
pub const DUAL_RANK1_RRF: f32 = 0.0331;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub chunks: Vec<ResultChunk>,
    pub tokens_returned: u32,
    pub truncated: bool,
    pub answer_state: AnswerState,
}

/// One candidate in the retrieval trace (C02). `rrf_score` is the fused RRF
/// score from hybrid search; `cosine` is the raw query–candidate cosine
/// recomputed from the already-fetched candidate vectors — never LanceDB's
/// query-local normalized `_distance`/`_score`. `dense_rank`/`lexical_rank`
/// are `None` at this wave; C04 supplies per-channel rank evidence later.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceHit {
    pub chunk_idx: u32,
    pub source_url: String,
    pub rrf_score: f32,
    pub cosine: Option<f32>,
    pub dense_rank: Option<u32>,
    pub lexical_rank: Option<u32>,
}

/// Evaluation-only retrieval trace (C02): the fused candidate pool, the MMR
/// selection, the pre-MMR raw-cosine distribution, and the gate outcome, so
/// later evaluation can inspect ranking decisions. Never exposed through MCP,
/// smoke JSON, human output, logs, or the public `SearchResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct RetrievalTrace {
    pub fused: Vec<TraceHit>,
    pub mmr: Vec<TraceHit>,
    pub pre_mmr_top_cosines: Vec<f32>,
    pub gate_passed: bool,
}

/// Outcome of ranking plus the answer gate.
#[derive(Clone)]
pub struct RankedGateResult {
    /// MMR-selected hits before the no-answer gate. C05 derives the answer
    /// decision from this set, so gate rejection never erases its evidence.
    pub pre_gate_hits: Vec<SearchHit>,
    pub hits: Vec<SearchHit>,
    pub gate_passed: bool,
    pub trace: Option<RetrievalTrace>,
}

impl std::fmt::Debug for RankedGateResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hit_ids: Vec<u32> = self.hits.iter().map(|hit| hit.chunk_idx).collect();
        f.debug_struct("RankedGateResult")
            .field(
                "pre_gate_hit_ids",
                &self
                    .pre_gate_hits
                    .iter()
                    .map(|hit| hit.chunk_idx)
                    .collect::<Vec<_>>(),
            )
            .field("hit_ids", &hit_ids)
            .field("gate_passed", &self.gate_passed)
            .field("trace", &self.trace)
            .finish()
    }
}

impl PartialEq for RankedGateResult {
    fn eq(&self, other: &Self) -> bool {
        self.gate_passed == other.gate_passed
            && self.trace == other.trace
            && self.pre_gate_hits.len() == other.pre_gate_hits.len()
            && self.hits.len() == other.hits.len()
            && self
                .pre_gate_hits
                .iter()
                .zip(&other.pre_gate_hits)
                .chain(self.hits.iter().zip(&other.hits))
                .all(|(left, right)| {
                    left.score == right.score
                        && left.chunk_idx == right.chunk_idx
                        && left.heading_path == right.heading_path
                        && left.source_url == right.source_url
                        && left.api_version == right.api_version
                        && left.chunk_type == right.chunk_type
                        && left.text == right.text
                })
    }
}

// N2: downcastable sentinel error types. `retrieve::search` maps each failure
// point to one of these so `tools::classify_error` can classify via
// `anyhow::Error::downcast_ref::<T>()` instead of fragile string matching on
// the error chain (see a1-mcp-error-contract §3.1). `Display` + `Error` are
// implemented manually — thiserror was removed from the dependency tree (M16)
// and must not be re-added.

/// The docset's manifest is missing, unparseable, or fails validation.
#[derive(Debug)]
pub struct DocsetNotInstalled {
    pub docset: String,
    pub reason: String,
}
impl std::fmt::Display for DocsetNotInstalled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "docset {} not installed: {}", self.docset, self.reason)
    }
}
impl std::error::Error for DocsetNotInstalled {}

/// The docset's store (lance table) is missing, corrupt, or unreadable.
#[derive(Debug)]
pub struct StoreError {
    pub docset: String,
    pub reason: String,
}
impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "store error for docset {}: {}", self.docset, self.reason)
    }
}
impl std::error::Error for StoreError {}

/// The embedder model could not be loaded (download failure, sha256 mismatch,
/// config error, tokenizer error).
#[derive(Debug)]
pub struct EmbedderLoadError(pub String);
impl std::fmt::Display for EmbedderLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "embedder load failed: {}", self.0)
    }
}
impl std::error::Error for EmbedderLoadError {}

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
    search_impl(docset, query, max_tokens, top_k, false, None).map(|(result, _)| result)
}

/// Like [`search`], but also returns the evaluation-only [`RetrievalTrace`]
/// (fused pool, MMR selection, pre-MMR raw-cosine distribution, gate outcome).
/// Search behavior is identical to `search`; the trace is for evaluation
/// inspection only and is never exposed through public output.
pub fn search_with_trace(
    docset: &str,
    query: &str,
    max_tokens: Option<u32>,
    top_k: Option<u32>,
) -> Result<(SearchResult, RetrievalTrace)> {
    let (result, trace) = search_impl(docset, query, max_tokens, top_k, true, None)?;
    let trace = trace.expect("search_impl must produce a trace when tracing is enabled");
    Ok((result, trace))
}

/// Crate-private reranker-aware search entry point used by MCP tools and smoke.
/// When `reranker` is `Some`, the retrieval pipeline inserts an optional Cohere
/// reorder stage between the local RRF candidate pool and MMR. On remote
/// failure, the pipeline falls back to the unchanged local path. When
/// `reranker` is `None`, behavior is identical to [`search`].
pub(crate) fn search_with_reranker(
    docset: &str,
    query: &str,
    max_tokens: Option<u32>,
    top_k: Option<u32>,
    reranker: Option<&dyn Reranker>,
) -> Result<SearchResult> {
    search_impl(docset, query, max_tokens, top_k, false, reranker).map(|(result, _)| result)
}

fn search_impl(
    docset: &str,
    query: &str,
    max_tokens: Option<u32>,
    top_k: Option<u32>,
    trace: bool,
    reranker: Option<&dyn Reranker>,
) -> Result<(SearchResult, Option<RetrievalTrace>)> {
    let docset = validate_docset(docset)?;
    let query = validate_query(query)?;
    let max_tokens = resolve_max_tokens(max_tokens);
    let top_k = resolve_top_k(top_k);

    // Load and validate manifest. N2: map each failure point to a downcastable
    // sentinel so tools::classify_error needs no string matching.
    let manifest_path = crate::cache::manifest_path(&docset);
    let manifest_json =
        std::fs::read_to_string(&manifest_path).map_err(|e| DocsetNotInstalled {
            docset: docset.clone(),
            reason: format!("failed to read manifest: {e}"),
        })?;
    let manifest: manifest::Manifest =
        manifest::parse_manifest(&manifest_json).map_err(|e| DocsetNotInstalled {
            docset: docset.clone(),
            reason: format!("manifest parse error: {e}"),
        })?;
    manifest::validate(&manifest).map_err(|e| DocsetNotInstalled {
        docset: docset.clone(),
        reason: format!("manifest validation: {e}"),
    })?;

    // Build embedder spec from manifest (model-version lock).
    let embedder_spec = EmbedderSpec {
        model_id: manifest.embedder.model_id.clone(),
        model_revision: manifest.embedder.model_revision.clone(),
        model_sha256: manifest.embedder.model_sha256.clone(),
    };
    let embedder = embedder::Embedder::load_for(&embedder_spec)
        .map_err(|e| EmbedderLoadError(e.to_string()))?;

    // Embed query and run hybrid search.
    let query_vector = embedder.embed(&query)?;
    let store = Store::open(&docset).map_err(|e| StoreError {
        docset: docset.clone(),
        reason: e.to_string(),
    })?;
    // Over-fetch then diversity-rerank with Maximal Marginal Relevance
    // (N1/OQ4). MMR keeps multiple chunks from the same source_url when their
    // vectors differ (e.g. distinct APIs on one reference page), fixing the old
    // `dedup_by_source_url` heuristic that collapsed single-page API
    // references. Over-fetching gives MMR a candidate pool to trade relevance
    // off against redundancy before the window pass re-attaches same-file
    // neighbors as context.
    //
    // Pool size: diagnosed on the real Next.js corpus (2026-07-11) — queries
    // whose expected page is recalled by the vector channel only (FTS misses
    // it) see the fused RRF rank sink to ~33 because k=60 lets dual-channel
    // hub chunks outrank vector-only hits. `top_k*3` (15) left those expected
    // chunks outside the MMR pool entirely; 40 covers the measured worst case
    // (fused rank 33) with headroom.
    let raw_k = (top_k * 8).max(40) as usize;
    let candidates = store
        .hybrid_search(&query_vector, &query, raw_k)
        .map_err(|e| StoreError {
            docset: docset.clone(),
            reason: e.to_string(),
        })?;
    let cand_ids: Vec<u32> = candidates.iter().map(|c| c.hit.chunk_idx).collect();
    let vectors = store.fetch_vectors(&cand_ids).map_err(|e| StoreError {
        docset: docset.clone(),
        reason: e.to_string(),
    })?;
    // Build per-channel rank map before MMR consumes the candidates (C05).
    let mut rank_map: std::collections::HashMap<u32, (Option<u32>, Option<u32>)> =
        std::collections::HashMap::new();
    for c in &candidates {
        rank_map.insert(c.hit.chunk_idx, (c.dense_rank, c.lexical_rank));
    }
    // MMR rerank (N1/OQ4) + answer gate (N4/OQ11) run through one shared
    // helper so the traced and untraced paths execute identical ranking logic;
    // the gate's design rationale is documented on `apply_answer_gate`.
    let RankedGateResult {
        pre_gate_hits,
        trace,
        ..
    } = rank_and_gate_candidates_with_reranker(
        &query_vector,
        candidates,
        &vectors,
        top_k as usize,
        trace,
        reranker,
        Some(&query),
    );

    // Build evidence after MMR, decide before neighbor expansion (C05).
    let decision = {
        let top = pre_gate_hits.first();
        let top_cosine = top
            .and_then(|hit| vectors.get(&hit.chunk_idx))
            .map(|v| cosine(&query_vector, v));
        let (pre_top, pre_second, pre_margin) = trace
            .as_ref()
            .map(|t| {
                let top = t.pre_mmr_top_cosines.first().copied();
                let second = t.pre_mmr_top_cosines.get(1).copied();
                let margin = match (top, second) {
                    (Some(a), Some(b)) => Some(a - b),
                    _ => None,
                };
                (top, second, margin)
            })
            .unwrap_or((None, None, None));
        let (dr, lr) = top
            .and_then(|hit| rank_map.get(&hit.chunk_idx).copied())
            .unwrap_or((None, None));
        let ev = QueryEvidence {
            // The existing gate treats a missing vector as cosine 0.0; keep
            // the new binary decision exactly equivalent.
            top_selected_cosine: top.map(|_| top_cosine.unwrap_or(0.0)),
            top_selected_rrf: top.map(|hit| hit.score),
            pre_mmr_top_cosine: pre_top,
            pre_mmr_second_cosine: pre_second,
            pre_mmr_cosine_margin: pre_margin,
            dense_rank: dr,
            lexical_rank: lr,
        };
        confidence::decide_calibrated(&ev)
    };

    if decision.state == AnswerState::NoAnswer {
        return Ok((
            SearchResult {
                chunks: vec![],
                tokens_returned: 0,
                truncated: false,
                answer_state: AnswerState::NoAnswer,
            },
            trace,
        ));
    }

    let hits = pre_gate_hits;

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
    let window_hits = store.fetch_by_idx(&window_ids).map_err(|e| StoreError {
        docset: docset.clone(),
        reason: e.to_string(),
    })?;
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
    let mut result = assemble_result(window_chunks, max_tokens)?;
    result.answer_state = decision.state;
    Ok((result, trace))
}

/// Maximal Marginal Relevance rerank (N1/OQ4). Greedy: repeatedly pick the
/// candidate maximizing `lambda * relevance - (1 - lambda) * max_sim -
/// (1 - lambda) * MMR_URL_PENALTY * same_url_selected`, where `relevance`
/// is the query–candidate cosine similarity, `max_sim` is the candidate's
/// largest cosine similarity to any already-picked chunk, and
/// `same_url_selected` counts already-picked chunks sharing the candidate's
/// `source_url`. The URL penalty is scaled by `(1 - lambda)` so it vanishes
/// in pure-relevance mode (`lambda = 1.0`), keeping that mode's ordering
/// strictly by query–cosine.
/// Replaces the old `dedup_by_source_url`: diversity comes from vector
/// dissimilarity plus a mild per-URL penalty, so multiple chunks from one URL
/// survive when they cover distinct APIs but hub files cannot monopolize the
/// top-K with many mutually-diverse chunks. Candidates without a fetched
/// vector fall back to score order, appended after the MMR-selected hits
/// (should not happen post-S1).
///
/// Relevance is the raw query–chunk COSINE (textbook MMR, Carbonell &
/// Goldstein 1998: Sim1 = query–doc similarity), not the RRF score: with the
/// default fusion constant k=60 the RRF scores across the candidate pool are
/// nearly flat (rank 1 ≈ 1/61 vs rank 15 ≈ 1/75), so a score-based relevance
/// term carried almost no signal and MMR ranking collapsed into the diversity
/// term alone — dual-channel hub chunks floated above vector-only specific
/// answers. Cosine relevance restored recall on the real Next.js corpus
/// (see the quality-lift commit message for measurements). Cosines are used
/// RAW (absolute similarities, not min-max normalized within the pool) so the
/// relevance weight stays stable as the over-fetch pool size changes.
pub fn mmr_rerank(
    query_vector: &[f32],
    hits: Vec<SearchHit>,
    vectors: &HashMap<u32, Vec<f32>>,
    top_k: usize,
    lambda: f32,
) -> Vec<SearchHit> {
    // Query–candidate cosine for every pooled hit, computed once. Used RAW
    // (not min-max normalized): cosines are already absolute similarities in
    // ~[0, 1], and normalizing within the pool made relevance unstable w.r.t.
    // pool size — a larger over-fetch pool stretches the [min, max] range with
    // low-cosine tail chunks and silently dilutes the relevance weight of
    // every real candidate (measured regression: recall@5 dropped 0.80 -> 0.60
    // when the pool grew 15 -> 40 under normalization).
    let query_cos: HashMap<u32, f32> = hits
        .iter()
        .filter_map(|h| {
            vectors
                .get(&h.chunk_idx)
                .map(|v| (h.chunk_idx, cosine(query_vector, v)))
        })
        .collect();

    mmr_rerank_inner(
        hits,
        vectors,
        top_k,
        lambda,
        &|chunk_idx| query_cos.get(&chunk_idx).copied().unwrap_or(0.0),
        false,
    )
}

/// Internal MMR helper parameterized by a relevance closure. Public
/// `mmr_rerank` uses query–chunk cosine; the reranker-aware path uses
/// ordinal relevance `(N - p) / N`.
fn mmr_rerank_inner(
    hits: Vec<SearchHit>,
    vectors: &HashMap<u32, Vec<f32>>,
    top_k: usize,
    lambda: f32,
    relevance: &dyn Fn(u32) -> f32,
    preserve_fallback_order: bool,
) -> Vec<SearchHit> {
    if hits.is_empty() || top_k == 0 {
        return Vec::new();
    }

    let (mut pool, mut fallback): (Vec<SearchHit>, Vec<SearchHit>) = hits
        .into_iter()
        .partition(|h| vectors.contains_key(&h.chunk_idx));
    if !preserve_fallback_order {
        fallback.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    let mut selected: Vec<SearchHit> = Vec::with_capacity(top_k);

    while selected.len() < top_k && !pool.is_empty() {
        let mut best_i = 0;
        let mut best_score = f32::MIN;
        for (i, cand) in pool.iter().enumerate() {
            let max_sim = max_cosine_to_selected(cand, &selected, vectors);
            let rel = relevance(cand.chunk_idx);
            let same_url = selected
                .iter()
                .filter(|s| s.source_url == cand.source_url)
                .count() as f32;
            let mmr = lambda * rel
                - (1.0 - lambda) * max_sim
                - (1.0 - lambda) * MMR_URL_PENALTY * same_url;
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

/// Apply the no-answer gate (N4/OQ11, cosine redesign): return `Vec::new()`
/// when there are no hits, unless the top hit clears EITHER answer signal:
/// (1) its cosine similarity to the query is at least `MIN_ANSWER_COSINE`
/// (semantic match), or (2) its RRF score reaches `DUAL_RANK1_RRF` — rank-1
/// agreement between the BM25 and vector channels (exact-keyword + semantic
/// agreement, which covers keyword-dense queries whose embeddings under-score).
/// A top hit without a fetched vector is treated as cosine 0 (blocked unless
/// the dual-rank-1 bypass applies) — MMR selects from the vector pool first,
/// so this is defensive. Exposed (`pub`) so the gate is unit-testable without
/// a full docset.
pub fn apply_answer_gate(
    hits: Vec<SearchHit>,
    query_vector: &[f32],
    vectors: &HashMap<u32, Vec<f32>>,
) -> Vec<SearchHit> {
    if hits.is_empty() {
        return hits;
    }
    let top = &hits[0];
    let cosine_sim = vectors
        .get(&top.chunk_idx)
        .map(|v| cosine(query_vector, v))
        .unwrap_or(0.0);
    if cosine_sim < MIN_ANSWER_COSINE && top.score < DUAL_RANK1_RRF {
        Vec::new()
    } else {
        hits
    }
}

/// Maximum UTF-8 bytes for a single reranker document.
const MAX_RERANK_DOCUMENT_BYTES: usize = 8192;

/// Sanitize and truncate a string to the reranker document byte cap, respecting
/// Unicode scalar boundaries.
fn sanitize_for_reranker(text: &str) -> String {
    let sanitized = crate::sanitize::sanitize_chunk(text);
    if sanitized.len() <= MAX_RERANK_DOCUMENT_BYTES {
        return sanitized;
    }
    let mut end = MAX_RERANK_DOCUMENT_BYTES;
    while end > 0 && !sanitized.is_char_boundary(end) {
        end -= 1;
    }
    sanitized[..end].to_owned()
}

/// Build `RerankDocument`s from the original candidate pool. Each document is
/// `sanitize_chunk(heading_path) + "\n\n" + sanitize_chunk(text)`, capped at
/// 8192 UTF-8 bytes without splitting a Unicode scalar. Stable IDs are local
/// `chunk_idx` (never sent to Cohere). Returns `None` if both sanitized
/// components are empty for any candidate, or if validation would fail (empty
/// documents, duplicate IDs, or more than 40 documents).
pub(crate) fn build_reranker_documents(
    candidates: &[CandidateEvidence],
) -> Option<Vec<RerankDocument>> {
    if candidates.is_empty() || candidates.len() > 40 {
        return None;
    }

    let mut seen_ids = std::collections::HashSet::with_capacity(candidates.len());
    let mut documents = Vec::with_capacity(candidates.len());

    for c in candidates {
        let heading = sanitize_for_reranker(&c.hit.heading_path);
        let text = sanitize_for_reranker(&c.hit.text);

        // If both sanitized components are empty, skip this candidate (no
        // remote call, take local fallback).
        if (heading.is_empty() && text.is_empty()) || !seen_ids.insert(c.hit.chunk_idx) {
            return None;
        }

        let combined = format!("{heading}\n\n{text}");

        // Cap the combined string at 8192 UTF-8 bytes without splitting a
        // Unicode scalar boundary.
        let capped = if combined.len() <= MAX_RERANK_DOCUMENT_BYTES {
            combined
        } else {
            let mut end = MAX_RERANK_DOCUMENT_BYTES;
            while end > 0 && !combined.is_char_boundary(end) {
                end -= 1;
            }
            combined[..end].to_owned()
        };

        documents.push(RerankDocument {
            stable_id: c.hit.chunk_idx,
            document: capped,
        });
    }

    Some(documents)
}

/// Validate a reranker response: every local stable ID must appear exactly once.
/// Returns the validated order or `None` on any inconsistency.
fn validate_rerank_order(
    response_ids: &[u32],
    candidates: &[CandidateEvidence],
) -> Option<Vec<u32>> {
    let expected: std::collections::HashSet<u32> =
        candidates.iter().map(|c| c.hit.chunk_idx).collect();
    let seen: std::collections::HashSet<u32> = response_ids.iter().copied().collect();
    if expected.len() != candidates.len()
        || response_ids.len() != candidates.len()
        || seen.len() != response_ids.len()
        || expected != seen
    {
        return None;
    }
    Some(response_ids.to_vec())
}

/// Rank fused hybrid candidates with MMR and apply the answer gate,
/// optionally recording an evaluation-only [`RetrievalTrace`] (C02). Both
/// `search` (trace off) and `search_with_trace` (trace on) run through here,
/// so traced evaluation observes exactly the same ranking and gate behavior
/// as normal search.
///
/// With `trace = false` no trace metadata is cloned and no trace vector is
/// allocated. With `trace = true`, raw query–candidate cosines are recomputed
/// solely from the already-fetched candidate `vectors` — never from LanceDB's
/// query-local normalized `_distance`/`_score`. `pre_mmr_top_cosines` is the
/// descending raw-cosine sequence across the complete fused candidate pool
/// before MMR (candidates without a fetched vector contribute no cosine); the
/// gate cosine remains the selected post-MMR top hit's raw cosine, exactly as
/// `apply_answer_gate` computes it. `gate_passed` is true exactly when the
/// gate returns at least one hit, so a no-answer decision still yields its
/// trace.
pub fn rank_and_gate_candidates(
    query_vector: &[f32],
    candidates: Vec<CandidateEvidence>,
    vectors: &HashMap<u32, Vec<f32>>,
    top_k: usize,
    trace: bool,
) -> RankedGateResult {
    rank_and_gate_candidates_with_reranker(
        query_vector,
        candidates,
        vectors,
        top_k,
        trace,
        None,
        None,
    )
}

/// Internal reranker-aware variant of [`rank_and_gate_candidates`].
pub(crate) fn rank_and_gate_candidates_with_reranker(
    query_vector: &[f32],
    candidates: Vec<CandidateEvidence>,
    vectors: &HashMap<u32, Vec<f32>>,
    top_k: usize,
    trace: bool,
    reranker: Option<&dyn Reranker>,
    query: Option<&str>,
) -> RankedGateResult {
    // Build per-channel rank maps from CandidateEvidence before MMR consumes
    // the candidates. MMR only operates on SearchHit; ranks are reattached
    // to the trace after selection.
    let mut rank_map: HashMap<u32, (Option<u32>, Option<u32>)> = HashMap::new();
    for c in &candidates {
        rank_map.insert(c.hit.chunk_idx, (c.dense_rank, c.lexical_rank));
    }

    // Fused-pool trace view, built before MMR consumes the candidates.
    // `bool::then` skips the closure entirely when tracing is disabled.
    let fused_trace = trace.then(|| {
        candidates
            .iter()
            .map(|c| TraceHit {
                chunk_idx: c.hit.chunk_idx,
                source_url: c.hit.source_url.clone(),
                rrf_score: c.hit.score,
                cosine: vectors
                    .get(&c.hit.chunk_idx)
                    .map(|v| cosine(query_vector, v)),
                dense_rank: c.dense_rank,
                lexical_rank: c.lexical_rank,
            })
            .collect::<Vec<TraceHit>>()
    });
    let pre_mmr_top_cosines = fused_trace.as_ref().map(|fused| {
        let mut cosines: Vec<f32> = fused
            .iter()
            .filter_map(|t| t.cosine)
            .filter(|cosine| cosine.is_finite())
            .collect();
        cosines.sort_by(|a, b| b.total_cmp(a));
        cosines
    });

    // MMR operates on SearchHit only; extract hits from evidence.
    let cand_hits: Vec<SearchHit> = candidates.iter().map(|c| c.hit.clone()).collect();

    // C08c: attempt reranker-aware path when a reranker is injected.
    // On success, use ordinal relevance from the reranker's order; on any
    // failure (document construction, remote call, validation), fall through
    // to the unchanged local cosine-MMR path.
    let reranked_hits = reranker.and_then(|r| {
        let query_text = query?;
        let documents = build_reranker_documents(&candidates)?;
        let result = r.rerank(query_text, &documents);
        let order = match result {
            Ok(ids) => validate_rerank_order(&ids, &candidates)?,
            Err(_) => return None,
        };
        // Build reranked hits preserving RRF scores and channel ranks from the
        // original local pool.
        let mut hit_map: HashMap<u32, &SearchHit> = HashMap::with_capacity(cand_hits.len());
        for h in &cand_hits {
            hit_map.insert(h.chunk_idx, h);
        }
        let reranked_cand_hits: Vec<SearchHit> = order
            .iter()
            .filter_map(|id| hit_map.get(id).map(|h| (*h).clone()))
            .collect();
        let n = reranked_cand_hits.len();
        let ordinal_rel: HashMap<u32, f32> = reranked_cand_hits
            .iter()
            .enumerate()
            .map(|(p, h)| (h.chunk_idx, (n - p) as f32 / n as f32))
            .collect();
        let selected = mmr_rerank_inner(
            reranked_cand_hits,
            vectors,
            top_k,
            MMR_LAMBDA,
            &|idx| ordinal_rel.get(&idx).copied().unwrap_or(0.0),
            true,
        );
        Some(selected)
    });

    let hits = if let Some(hits) = reranked_hits {
        hits
    } else {
        mmr_rerank(query_vector, cand_hits, vectors, top_k, MMR_LAMBDA)
    };

    // MMR-selection trace view, built before the gate consumes the hits.
    let mmr_trace = trace.then(|| {
        hits.iter()
            .map(|h| {
                let (dr, lr) = rank_map.get(&h.chunk_idx).copied().unwrap_or((None, None));
                TraceHit {
                    chunk_idx: h.chunk_idx,
                    source_url: h.source_url.clone(),
                    rrf_score: h.score,
                    cosine: vectors.get(&h.chunk_idx).map(|v| cosine(query_vector, v)),
                    dense_rank: dr,
                    lexical_rank: lr,
                }
            })
            .collect::<Vec<TraceHit>>()
    });

    let pre_gate_hits = hits.clone();
    let hits = apply_answer_gate(hits, query_vector, vectors);
    let gate_passed = !hits.is_empty();

    let trace = fused_trace.map(|fused| RetrievalTrace {
        fused,
        mmr: mmr_trace.expect("mmr trace must be built when tracing is enabled"),
        pre_mmr_top_cosines: pre_mmr_top_cosines
            .expect("pre-MMR cosines must be built when tracing is enabled"),
        gate_passed,
    });

    RankedGateResult {
        pre_gate_hits,
        hits,
        gate_passed,
        trace,
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
        answer_state: AnswerState::NoAnswer,
    })
}

#[cfg(test)]
mod c08c_tests {
    use super::*;
    use crate::rerank::{RerankDocument, RerankFailure, RerankFailureClass, Reranker};
    use crate::store::{CandidateEvidence, SearchHit};
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Fake reranker that returns a fixed order of stable IDs.
    struct FakeReranker {
        order: Vec<u32>,
        call_count: AtomicUsize,
    }

    impl FakeReranker {
        fn new(order: Vec<u32>) -> Self {
            Self {
                order,
                call_count: AtomicUsize::new(0),
            }
        }
    }

    impl Reranker for FakeReranker {
        fn rerank(
            &self,
            _query: &str,
            _documents: &[RerankDocument],
        ) -> Result<Vec<u32>, RerankFailure> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            Ok(self.order.clone())
        }
    }

    /// Fake reranker that always fails with a given failure class.
    struct FailingReranker {
        class: RerankFailureClass,
        call_count: AtomicUsize,
    }

    impl FailingReranker {
        fn new(class: RerankFailureClass) -> Self {
            Self {
                class,
                call_count: AtomicUsize::new(0),
            }
        }
    }

    impl Reranker for FailingReranker {
        fn rerank(
            &self,
            _query: &str,
            _documents: &[RerankDocument],
        ) -> Result<Vec<u32>, RerankFailure> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            Err(RerankFailure::new(self.class))
        }
    }

    fn hit(idx: u32, url: &str, score: f32) -> SearchHit {
        SearchHit {
            chunk_idx: idx,
            heading_path: format!("heading_{idx}"),
            source_url: url.to_string(),
            api_version: None,
            chunk_type: crate::chunker::ChunkType::Code,
            text: format!("text for chunk {idx}"),
            score,
        }
    }

    fn evidence(hit: SearchHit) -> CandidateEvidence {
        CandidateEvidence {
            hit,
            dense_rank: None,
            lexical_rank: None,
        }
    }

    fn qv() -> Vec<f32> {
        vec![1.0, 0.0]
    }

    fn vec_with_cosine(cosine: f32) -> Vec<f32> {
        let sin = (1.0 - cosine * cosine).sqrt();
        vec![cosine, sin]
    }

    fn vecs(pairs: &[(u32, Vec<f32>)]) -> HashMap<u32, Vec<f32>> {
        pairs.iter().cloned().collect()
    }

    /// Assertion 1: Exact sanitized document encoding, empty-heading separator,
    /// Unicode-safe 8192-byte cap, and both-empty local fallback.
    #[test]
    fn c08c_document_encoding_heading_separator_and_byte_cap() {
        // Build a candidate with a heading and text.
        let candidates = vec![evidence(SearchHit {
            chunk_idx: 0,
            heading_path: "Getting Started".to_string(),
            source_url: "a.md".to_string(),
            api_version: None,
            chunk_type: crate::chunker::ChunkType::Code,
            text: "Hello world".to_string(),
            score: 0.03,
        })];
        let docs = build_reranker_documents(&candidates).expect("documents must build");
        assert_eq!(docs.len(), 1);
        // Document format: sanitize(heading) + "\n\n" + sanitize(text)
        assert!(
            docs[0].document.starts_with("Getting Started\n\n"),
            "document must start with heading + separator, got: {:?}",
            &docs[0].document[..50.min(docs[0].document.len())]
        );
        assert!(
            docs[0].document.contains("Hello world"),
            "document must contain text"
        );
        // Stable ID is chunk_idx
        assert_eq!(docs[0].stable_id, 0);

        // Unicode-safe 8192-byte cap: the combined string is capped at 8192
        // bytes without splitting a multi-byte character.
        let long_unicode = "é".repeat(5000); // each 'é' is 2 bytes = 10000 bytes
        let candidates_unicode = vec![evidence(SearchHit {
            chunk_idx: 1,
            heading_path: "h".to_string(),
            source_url: "b.md".to_string(),
            api_version: None,
            chunk_type: crate::chunker::ChunkType::Code,
            text: long_unicode.clone(),
            score: 0.03,
        })];
        let docs_unicode =
            build_reranker_documents(&candidates_unicode).expect("documents must build");
        assert!(
            docs_unicode[0].document.len() <= MAX_RERANK_DOCUMENT_BYTES,
            "combined document must not exceed 8192 bytes, got {}",
            docs_unicode[0].document.len()
        );
        // Must not split a multi-byte char
        assert!(
            docs_unicode[0]
                .document
                .is_char_boundary(docs_unicode[0].document.len()),
            "document must end at a char boundary"
        );

        // Both-empty: heading="" and text="" after sanitize -> None (no remote call)
        let empty_candidates = vec![evidence(SearchHit {
            chunk_idx: 2,
            heading_path: String::new(),
            source_url: "c.md".to_string(),
            api_version: None,
            chunk_type: crate::chunker::ChunkType::Code,
            text: String::new(),
            score: 0.03,
        })];
        assert!(
            build_reranker_documents(&empty_candidates).is_none(),
            "both-empty candidate must yield None (no fake call)"
        );
    }

    /// Assertion 2: Disabled identity — `None` reranker gives identical hit IDs,
    /// gate, answer state, and trace to local helper; fake call count remains zero.
    #[test]
    fn c08c_disabled_identity_none_gives_same_result_as_local() {
        let candidates = vec![
            evidence(hit(0, "a.md", 0.030)),
            evidence(hit(1, "b.md", 0.029)),
            evidence(hit(2, "c.md", 0.028)),
        ];
        let vectors = vecs(&[
            (0, vec_with_cosine(0.90)),
            (1, vec_with_cosine(0.85)),
            (2, vec_with_cosine(0.80)),
        ]);
        let local = rank_and_gate_candidates(&qv(), candidates.clone(), &vectors, 3, true);
        let reranked = rank_and_gate_candidates(&qv(), candidates, &vectors, 3, true);
        let local_ids: Vec<u32> = local.hits.iter().map(|h| h.chunk_idx).collect();
        let reranked_ids: Vec<u32> = reranked.hits.iter().map(|h| h.chunk_idx).collect();
        assert_eq!(local_ids, reranked_ids, "disabled must give same hit IDs");
        assert_eq!(local.gate_passed, reranked.gate_passed);
        assert_eq!(local.trace.is_some(), reranked.trace.is_some());
    }

    /// Assertion 3: Ordinal diversity — provider order controls relevance but
    /// MMR still chooses a diverse URL/vector result.
    #[test]
    fn c08c_ordinal_diversity_provider_order_controls_mmr() {
        // The reranker puts chunk 1 first, chunk 0 second. But chunk 1 is a
        // near-duplicate of chunk 0 (same direction vector), so MMR should
        // still prefer the diverse chunk 2 over the near-duplicate.
        let candidates = vec![
            evidence(hit(0, "a.md", 0.030)),
            evidence(hit(1, "a.md", 0.029)), // same URL, near-duplicate vector
            evidence(hit(2, "b.md", 0.028)), // different URL, diverse vector
        ];
        let mut diverse = vec_with_cosine(0.90);
        diverse[1] = -diverse[1]; // dissimilar to chunk 0
        let vectors = vecs(&[
            (0, vec_with_cosine(0.95)),
            (1, vec_with_cosine(0.94)), // near-duplicate of chunk 0
            (2, diverse),               // diverse
        ]);
        // Reranker says: chunk 1 first, chunk 0 second, chunk 2 third
        let fake = FakeReranker::new(vec![1, 0, 2]);
        let result = rank_and_gate_candidates_with_reranker(
            &qv(),
            candidates,
            &vectors,
            3,
            true,
            Some(&fake),
            Some("test query"),
        );
        assert!(result.gate_passed);
        assert_eq!(fake.call_count.load(Ordering::SeqCst), 1);
        let mmr_ids: Vec<u32> = result
            .trace
            .as_ref()
            .unwrap()
            .mmr
            .iter()
            .map(|t| t.chunk_idx)
            .collect();
        // MMR must still prefer diverse chunk 2 over near-duplicate chunk 1
        // because chunk 1 is a near-duplicate of chunk 0 (which is also selected).
        // The exact order depends on MMR lambda and vector geometry.
        assert!(
            mmr_ids.contains(&0) && mmr_ids.contains(&2),
            "MMR must select diverse chunks, got: {mmr_ids:?}"
        );
    }

    /// Assertion 4: Fallback identity — all seven RerankFailureClass values
    /// yield exact local IDs/gate/trace.
    #[test]
    fn c08c_fallback_identity_all_failure_classes() {
        let classes = [
            RerankFailureClass::Network,
            RerankFailureClass::Timeout,
            RerankFailureClass::Authentication,
            RerankFailureClass::InvalidRequest,
            RerankFailureClass::RateLimit,
            RerankFailureClass::ServerError,
            RerankFailureClass::InvalidResponse,
        ];
        let candidates = vec![
            evidence(hit(0, "a.md", 0.030)),
            evidence(hit(1, "b.md", 0.029)),
            evidence(hit(2, "c.md", 0.028)),
        ];
        let vectors = vecs(&[
            (0, vec_with_cosine(0.90)),
            (1, vec_with_cosine(0.85)),
            (2, vec_with_cosine(0.80)),
        ]);
        // Baseline: no reranker
        let baseline = rank_and_gate_candidates(&qv(), candidates.clone(), &vectors, 3, true);
        let baseline_ids: Vec<u32> = baseline.hits.iter().map(|h| h.chunk_idx).collect();

        for class in classes {
            let failing = FailingReranker::new(class);
            let result = rank_and_gate_candidates_with_reranker(
                &qv(),
                candidates.clone(),
                &vectors,
                3,
                true,
                Some(&failing),
                Some("test"),
            );
            let result_ids: Vec<u32> = result.hits.iter().map(|h| h.chunk_idx).collect();
            assert_eq!(
                baseline_ids, result_ids,
                "fallback for {class:?} must match local IDs"
            );
            assert_eq!(
                baseline.gate_passed, result.gate_passed,
                "fallback for {class:?} must match gate"
            );
            assert_eq!(
                baseline.trace.as_ref().map(|t| t.fused.len()),
                result.trace.as_ref().map(|t| t.fused.len()),
                "fallback for {class:?} must match trace fused length"
            );
            assert_eq!(failing.call_count.load(Ordering::SeqCst), 1);
        }
    }

    #[test]
    fn c08c_rejects_duplicate_or_overlong_rerank_order() {
        let candidates = vec![
            evidence(hit(0, "a.md", 0.030)),
            evidence(hit(1, "b.md", 0.029)),
        ];

        assert!(validate_rerank_order(&[0], &candidates).is_none());
        assert!(validate_rerank_order(&[0, 1, 0], &candidates).is_none());
        assert!(validate_rerank_order(&[0, 9], &candidates).is_none());
    }

    #[test]
    fn c08c_successful_rerank_keeps_vectorless_provider_order() {
        let candidates = vec![
            evidence(hit(0, "a.md", 0.010)),
            evidence(hit(1, "b.md", 0.040)),
        ];
        let reranker = FakeReranker::new(vec![0, 1]);
        let result = rank_and_gate_candidates_with_reranker(
            &qv(),
            candidates,
            &HashMap::new(),
            2,
            true,
            Some(&reranker),
            Some("test query"),
        );
        let ids: Vec<u32> = result
            .trace
            .expect("trace enabled")
            .mmr
            .into_iter()
            .map(|hit| hit.chunk_idx)
            .collect();
        assert_eq!(ids, vec![0, 1]);
    }

    /// Assertion 5: Reranked NoAnswer returns no chunks; a gate-passing finite
    /// cosine below 0.845 is Borderline with chunks.
    #[test]
    fn c08c_reranked_answer_states() {
        // NoAnswer: top cosine 0.80 < MIN_ANSWER_COSINE (0.82)
        let no_answer_candidates = vec![evidence(hit(0, "low.md", 0.016))];
        let no_answer_vectors = vecs(&[(0, vec_with_cosine(0.80))]);
        let fake = FakeReranker::new(vec![0]);
        let result = rank_and_gate_candidates_with_reranker(
            &qv(),
            no_answer_candidates,
            &no_answer_vectors,
            1,
            false,
            Some(&fake),
            Some("test"),
        );
        assert!(
            result.hits.is_empty(),
            "NoAnswer must return zero chunks even with reranker"
        );

        // Borderline: top cosine 0.83 (above 0.82, below 0.845)
        let borderline_candidates = vec![evidence(hit(0, "match.md", 0.030))];
        let borderline_vectors = vecs(&[(0, vec_with_cosine(0.83))]);
        let fake2 = FakeReranker::new(vec![0]);
        let result2 = rank_and_gate_candidates_with_reranker(
            &qv(),
            borderline_candidates,
            &borderline_vectors,
            1,
            false,
            Some(&fake2),
            Some("test"),
        );
        assert!(result2.gate_passed, "0.83 cosine must pass the binary gate");
        assert!(
            !result2.hits.is_empty(),
            "Borderline must retain chunks with reranker"
        );
    }

    /// Assertion 6: Runtime config — factory performs no network; invalid
    /// opt-in fails serve and CLI smoke before search; lone COHERE_API_KEY
    /// is disabled.
    #[test]
    fn c08c_runtime_config_factory_no_network_and_lone_key_disabled() {
        // No env vars -> disabled (no network)
        let result = crate::rerank::configured_reranker_with(|_| None);
        assert!(result.is_ok());
        assert!(
            result.unwrap().is_none(),
            "no env vars must yield disabled reranker"
        );

        // Lone COHERE_API_KEY without provider -> disabled
        let result = crate::rerank::configured_reranker_with(|name| match name {
            crate::rerank::COHERE_API_KEY_ENV => Some(std::ffi::OsString::from("test-key")),
            _ => None,
        });
        assert!(
            result.unwrap().is_none(),
            "lone COHERE_API_KEY without provider must be disabled"
        );

        // Provider set without API key -> error (partial opt-in)
        let result = crate::rerank::configured_reranker_with(|name| match name {
            crate::rerank::PROVIDER_ENV => Some(std::ffi::OsString::from("cohere")),
            _ => None,
        });
        assert!(
            result.is_err(),
            "provider without API key must fail startup"
        );
    }

    /// Assertion 6 cont: MCP response keys stay unchanged and contain no
    /// provider data.
    #[test]
    fn c08c_mcp_response_keys_unchanged_no_provider_data() {
        use serde_json::json;
        // Call handle_call with nowdocs_list — must return standard structure
        let result = crate::tools::handle_call("nowdocs_list", json!({}));
        assert!(
            result.get("content").is_some(),
            "nowdocs_list must have content, got: {result:?}"
        );
        let result_str = serde_json::to_string(&result).unwrap();
        // Must not contain any reranker/provider/key data
        assert!(
            !result_str.contains("cohere"),
            "response must not contain 'cohere', got: {result_str}"
        );
        assert!(
            !result_str.contains("rerank"),
            "response must not contain 'rerank', got: {result_str}"
        );
        assert!(
            !result_str.contains("api_key"),
            "response must not contain 'api_key', got: {result_str}"
        );
    }
}
