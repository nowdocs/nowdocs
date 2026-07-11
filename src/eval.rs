//! Golden evaluation: recall@5 + MRR over a docset's golden query set.
//!
//! `evaluate()` is the public entry point; `compute_metrics()` is the pure
//! function exposed for unit testing the recall/MRR math without loading
//! the embedder.

use anyhow::{Context, Result};

use crate::retrieve;

/// A single golden query: the natural-language question and the `source_url`
/// of the chunk we expect to recall.
#[derive(Debug, Clone)]
pub struct GoldenQuery {
    pub query: String,
    pub expected_source_url: String,
}

/// Aggregate quality metrics over a golden query set.
#[derive(Debug, Clone, PartialEq)]
pub struct EvalReport {
    pub recall_at_5: f32,
    pub mrr: f32,
    pub n: usize,
}

/// Pure: compute recall@K and MRR from per-query hit ranks.
///
/// `ranks[i] = Some(r)` if the expected chunk appeared at 1-indexed rank `r`
/// within the top-K results for query `i`. `ranks[i] = None` if it did not.
///
/// - recall@K = (# hits) / n
/// - MRR      = mean of (1 / rank) over hits; misses contribute 0
///
/// An empty input returns `(0.0, 0.0)` to avoid division by zero.
pub fn compute_metrics(ranks: &[Option<usize>]) -> (f32, f32) {
    if ranks.is_empty() {
        return (0.0, 0.0);
    }
    let n = ranks.len();
    let hits = ranks.iter().filter(|r| r.is_some()).count();
    let recall = hits as f32 / n as f32;
    let mrr = ranks
        .iter()
        .map(|r| match r {
            Some(rank) => 1.0 / (*rank as f32),
            None => 0.0,
        })
        .sum::<f32>()
        / n as f32;
    (recall, mrr)
}

/// Recall@K cutoff: a hit beyond this rank counts as a miss.
///
/// `retrieve::search` expands each hybrid hit into a 3-chunk neighbor window
/// (`[hit, hit-1, hit+1]`), so `result.chunks` may hold more than `RECALL_K`
/// chunks. `evaluate()` therefore only looks at the first `RECALL_K` chunks
/// (relevance-first order from `reorder_to_window`) — a hit beyond rank K
/// counts as a miss, so `recall_at_5` measures true recall@5, not recall@N.
const RECALL_K: usize = 5;

/// Release-quality thresholds for the real Next.js golden set (OQ9): the
/// concept-level query set (20-30 queries over ~7480 chunks) must clear
/// `MRR >= 0.85` and `Recall@5 >= 0.90` before a release is cut.
///
/// These are enforced by the CI `eval` job (`.github/workflows/eval.yml`) on
/// manual dispatch — the nightly run reports metrics against them so a
/// regression is visible before release. They deliberately apply to the real
/// docset only: the toy 3-file golden fixture keeps its own looser gates
/// (`RECALL_GATE`/`MRR_GATE` in tests/eval_tests.rs) as an arithmetic
/// regression check that is documented as non-generalizable.
pub const RELEASE_MRR_THRESHOLD: f32 = 0.85;
pub const RELEASE_RECALL_THRESHOLD: f32 = 0.90;

/// Run the golden set against an already-ingested docset and report quality.
pub fn evaluate(docset: &str, golden: &[GoldenQuery]) -> Result<EvalReport> {
    if golden.is_empty() {
        return Ok(EvalReport {
            recall_at_5: 0.0,
            mrr: 0.0,
            n: 0,
        });
    }

    let mut ranks: Vec<Option<usize>> = Vec::with_capacity(golden.len());
    for q in golden {
        let result = retrieve::search(docset, &q.query, Some(4000), Some(RECALL_K as u32))
            .with_context(|| format!("search failed for query {:?}", q.query))?;
        let rank = result
            .chunks
            .iter()
            .take(RECALL_K)
            .position(|c| c.source_url == q.expected_source_url)
            .map(|p| p + 1); // 1-indexed
        ranks.push(rank);
    }

    let (recall_at_5, mrr) = compute_metrics(&ranks);
    Ok(EvalReport {
        recall_at_5,
        mrr,
        n: golden.len(),
    })
}

/// Maximum acceptable false-positive rate over the negative query set (M24).
/// Starts at 10% and is tunable once real eval data accumulates.
///
/// Status (2026-07): NOT yet enforceable. Measured FP rate is ~1.0 on every
/// corpus (toy and the real Next.js ~7480-chunk store) because dense vector
/// search always returns a rank-1 neighbor whose RRF score (1/60 ≈ 0.0167)
/// clears the N4 no-answer gate (`MIN_RELEVANCE_THRESHOLD` = 0.015) — so every
/// query is "answered" regardless of relevance. Until the no-answer gate is
/// recalibrated (e.g. require dual-channel agreement, or threshold the raw
/// vector cosine instead of the RRF score), this constant documents the
/// intent only; the CI eval job reports the rate without gating on it.
pub const MAX_FALSE_POSITIVE_RATE: f32 = 0.10;

/// Negative-eval report (M24): how often out-of-scope queries still returned
/// results above the no-answer relevance gate.
#[derive(Debug, Clone, PartialEq)]
pub struct NegativeReport {
    /// Fraction of negative queries that returned at least one chunk.
    pub false_positive_rate: f32,
    /// Count of negative queries that returned at least one chunk.
    pub answered: usize,
    pub n: usize,
    /// Per-query top-chunk score (`None` when the query returned no chunks),
    /// aligned with the input query order. Lets callers gate on confidence,
    /// not just on non-emptiness.
    pub top_scores: Vec<Option<f32>>,
}

/// Pure: false-positive rate = (# answered) / n; empty input returns 0.0 to
/// avoid division by zero.
pub fn false_positive_rate(answered: &[bool]) -> f32 {
    if answered.is_empty() {
        return 0.0;
    }
    answered.iter().filter(|&&a| a).count() as f32 / answered.len() as f32
}

/// Run negative (out-of-scope) queries against a docset and report how often
/// the pipeline returned results anyway (M24). A query counts as "answered"
/// when `retrieve::search` returns at least one chunk — i.e. its top hit
/// cleared the N4 no-answer relevance gate (`MIN_RELEVANCE_THRESHOLD`).
/// Ideally every negative query returns empty, so `false_positive_rate` is 0.
pub fn evaluate_negatives(docset: &str, queries: &[String]) -> Result<NegativeReport> {
    let mut answered_flags = Vec::with_capacity(queries.len());
    let mut top_scores = Vec::with_capacity(queries.len());
    for q in queries {
        let result = retrieve::search(docset, q, Some(4000), Some(RECALL_K as u32))
            .with_context(|| format!("negative search failed for query {:?}", q))?;
        answered_flags.push(!result.chunks.is_empty());
        top_scores.push(
            result
                .chunks
                .iter()
                .filter_map(|c| c.score)
                .reduce(f32::max),
        );
    }
    let answered = answered_flags.iter().filter(|&&a| a).count();
    Ok(NegativeReport {
        false_positive_rate: false_positive_rate(&answered_flags),
        answered,
        n: queries.len(),
        top_scores,
    })
}
