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
        let result = retrieve::search(docset, &q.query, Some(4000), Some(5))
            .with_context(|| format!("search failed for query {:?}", q.query))?;
        let rank = result
            .chunks
            .iter()
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