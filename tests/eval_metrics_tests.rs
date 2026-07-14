//! Metric-level tests for the evaluation foundation: graded ranking metrics
//! (recall / precision / MRR / nDCG over labeled targets) and Wilson-interval
//! rate estimates. Pure: no embedder, no I/O.

use nowdocs::eval::{
    compute_ranking_metrics, rate_estimate, RankingMetrics, RateEstimate, RelevanceTarget,
};
use nowdocs::retrieve::ResultChunk;

fn hit(source_url: &str) -> ResultChunk {
    ResultChunk {
        chunk_idx: 0,
        heading_path: String::new(),
        source_url: source_url.into(),
        api_version: None,
        chunk_type: nowdocs::chunker::ChunkType::Info,
        text: String::new(),
        score: None,
    }
}

fn target(source_url: &str, grade: u8) -> RelevanceTarget {
    RelevanceTarget {
        source_url: source_url.into(),
        heading_path_prefix: None,
        grade,
    }
}

fn discount(rank: usize) -> f32 {
    1.0 / (rank as f32 + 1.0).log2()
}

#[test]
fn duplicate_chunks_matching_one_target_count_single_gain() {
    // Ranks 1 and 3 both match the same grade-2 target. The gain must be
    // counted once (at the first matching rank), so DCG == IDCG and nDCG == 1.
    let hits = vec![hit("a.md"), hit("x.md"), hit("a.md")];
    let targets = vec![target("a.md", 2)];
    let m = compute_ranking_metrics(&hits, &targets, 5);
    assert_eq!(m.relevant_targets_found, 1);
    assert!(
        (m.ndcg - 1.0).abs() < 1e-6,
        "duplicate chunk matches must not inflate nDCG past 1.0, got {}",
        m.ndcg
    );
}

#[test]
fn recall_counts_found_targets_over_labeled_targets() {
    let hits = vec![hit("a.md"), hit("x.md")];
    let targets = vec![target("a.md", 2), target("missing.md", 2)];
    let m = compute_ranking_metrics(&hits, &targets, 5);
    assert_eq!(m.relevant_targets_found, 1);
    assert!(
        (m.recall - 0.5).abs() < 1e-6,
        "recall 1/2, got {}",
        m.recall
    );
}

#[test]
fn precision_counts_relevant_chunks_over_k() {
    // Two of the five returned chunks match a labeled target (both match the
    // same one): precision@5 = 2/5 even though recall = 1/1.
    let hits = vec![
        hit("a.md"),
        hit("x.md"),
        hit("a.md"),
        hit("y.md"),
        hit("z.md"),
    ];
    let targets = vec![target("a.md", 1)];
    let m = compute_ranking_metrics(&hits, &targets, 5);
    assert!(
        (m.precision - 0.4).abs() < 1e-6,
        "precision 2/5, got {}",
        m.precision
    );
    assert!((m.recall - 1.0).abs() < 1e-6);
}

#[test]
fn mrr_uses_rank_of_first_relevant_hit() {
    let targets = vec![target("a.md", 2)];

    let first = vec![hit("a.md"), hit("x.md")];
    let m = compute_ranking_metrics(&first, &targets, 5);
    assert!(
        (m.mrr - 1.0).abs() < 1e-6,
        "rank 1 → mrr 1.0, got {}",
        m.mrr
    );

    let mut fifth: Vec<ResultChunk> = (0..4).map(|i| hit(&format!("x{i}.md"))).collect();
    fifth.push(hit("a.md"));
    let m = compute_ranking_metrics(&fifth, &targets, 5);
    assert!(
        (m.mrr - 0.2).abs() < 1e-6,
        "rank 5 → mrr 0.2, got {}",
        m.mrr
    );
}

#[test]
fn hits_beyond_k_do_not_count() {
    // The only matching chunk sits at rank 6 with k = 5: it is a miss.
    let mut hits: Vec<ResultChunk> = (0..5).map(|i| hit(&format!("x{i}.md"))).collect();
    hits.push(hit("a.md"));
    let targets = vec![target("a.md", 2)];
    let m = compute_ranking_metrics(&hits, &targets, 5);
    assert_eq!(m.relevant_targets_found, 0);
    assert_eq!(m.recall, 0.0);
    assert_eq!(m.mrr, 0.0);
    assert_eq!(m.ndcg, 0.0);
    assert_eq!(m.precision, 0.0);
}

#[test]
fn ndcg_rewards_ideal_ordering() {
    // Grade-2 target (gain 3) and grade-1 target (gain 1).
    let targets = vec![target("high.md", 2), target("low.md", 1)];

    // Ideal order: highest grade first → DCG == IDCG → nDCG 1.0.
    let ideal = vec![hit("high.md"), hit("low.md")];
    let m = compute_ranking_metrics(&ideal, &targets, 5);
    assert!(
        (m.ndcg - 1.0).abs() < 1e-6,
        "ideal order → ndcg 1.0, got {}",
        m.ndcg
    );

    // Reversed order: DCG < IDCG → nDCG strictly below 1.
    let reversed = vec![hit("low.md"), hit("high.md")];
    let m = compute_ranking_metrics(&reversed, &targets, 5);
    let dcg = 1.0 * discount(1) + 3.0 * discount(2);
    let idcg = 3.0 * discount(1) + 1.0 * discount(2);
    let expected = dcg / idcg;
    assert!(
        (m.ndcg - expected).abs() < 1e-6,
        "reversed order → ndcg {expected}, got {}",
        m.ndcg
    );
    assert!(m.ndcg < 1.0 - 1e-3, "reversed order must score below ideal");
}

#[test]
fn empty_inputs_yield_zero_valued_fields() {
    let targets = vec![target("a.md", 2)];

    // No hits: every metric is zero.
    let m = compute_ranking_metrics(&[], &targets, 5);
    assert_eq!(
        m,
        RankingMetrics {
            k: 5,
            relevant_targets_found: 0,
            recall: 0.0,
            mrr: 0.0,
            ndcg: 0.0,
            precision: 0.0,
        }
    );

    // No targets: empty denominators → zero-valued fields.
    let m = compute_ranking_metrics(&[hit("a.md")], &[], 5);
    assert_eq!(m.relevant_targets_found, 0);
    assert_eq!(m.recall, 0.0);
    assert_eq!(m.mrr, 0.0);
    assert_eq!(m.ndcg, 0.0);
    assert_eq!(m.precision, 0.0);
}

#[test]
fn false_reject_rate_is_a_rate_estimate() {
    // False rejects: positive queries where retrieval found no labeled target.
    let estimate: RateEstimate = rate_estimate(2, 10);
    assert!((estimate.rate - 0.2).abs() < 1e-6);
    assert_eq!(estimate.count, 2);
    assert_eq!(estimate.total, 10);
}

#[test]
fn false_accept_rate_is_a_rate_estimate() {
    // False accepts: negative queries that retrieval answered anyway.
    let estimate = rate_estimate(1, 12);
    assert!((estimate.rate - (1.0 / 12.0)).abs() < 1e-6);
    assert!(estimate.lower <= estimate.rate);
    assert!(estimate.upper >= estimate.rate);
}

#[test]
fn borderline_rate_is_a_rate_estimate() {
    let estimate = rate_estimate(3, 20);
    assert!((estimate.rate - 0.15).abs() < 1e-6);
    assert!(estimate.lower < estimate.rate);
    assert!(estimate.upper > estimate.rate);
}

#[test]
fn decisive_coverage_is_a_rate_estimate() {
    // Decisive coverage: share of queries with a confident verdict (i.e. not
    // borderline) — 17 decisive out of 20.
    let estimate = rate_estimate(17, 20);
    assert!((estimate.rate - 0.85).abs() < 1e-6);
    assert!(estimate.lower <= estimate.rate);
    assert!(estimate.upper >= estimate.rate);
}

#[test]
fn wilson_interval_contains_observed_rate() {
    let estimate = rate_estimate(1, 20);
    assert!((estimate.rate - 0.05).abs() < 1e-6);
    assert!(estimate.lower <= estimate.rate);
    assert!(estimate.upper >= estimate.rate);
}

#[test]
fn wilson_interval_handles_degenerate_counts() {
    // No observations: zero total → zero-valued fields, no NaN.
    let none = rate_estimate(0, 0);
    assert_eq!(none.rate, 0.0);
    assert_eq!(none.lower, 0.0);
    assert_eq!(none.upper, 0.0);

    // Zero successes: lower bound is exactly 0, upper bound is positive.
    let zero = rate_estimate(0, 20);
    assert_eq!(zero.rate, 0.0);
    assert_eq!(zero.lower, 0.0);
    assert!(zero.upper > 0.0 && zero.upper < 1.0);

    // All successes: upper bound is exactly 1, lower bound is below 1.
    let all = rate_estimate(20, 20);
    assert_eq!(all.rate, 1.0);
    assert_eq!(all.upper, 1.0);
    assert!(all.lower > 0.0 && all.lower < 1.0);
}
