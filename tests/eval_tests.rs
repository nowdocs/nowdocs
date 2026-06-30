//! Golden evaluation: recall@5 + MRR for retrieval quality gating.

use std::path::PathBuf;

use nowdocs::eval::{compute_metrics, evaluate, GoldenQuery};

const RECALL_GATE: f32 = 0.8;
const MRR_GATE: f32 = 0.6;

/// Pure-function unit test: no embedder, no I/O. Verifies recall@5 + MRR math.
#[test]
fn test_eval_report_math() {
    // All hits at rank 1 → recall = 1.0, mrr = 1.0
    let ranks_all_first = vec![Some(1usize), Some(1), Some(1)];
    let (rec, mrr) = compute_metrics(&ranks_all_first);
    assert!((rec - 1.0).abs() < 1e-6, "all rank-1 hits → recall=1.0, got {rec}");
    assert!((mrr - 1.0).abs() < 1e-6, "all rank-1 hits → mrr=1.0, got {mrr}");

    // All hits at rank 5 → recall = 1.0, mrr = 0.2
    let ranks_all_fifth = vec![Some(5usize), Some(5)];
    let (rec, mrr) = compute_metrics(&ranks_all_fifth);
    assert!((rec - 1.0).abs() < 1e-6, "all rank-5 hits → recall=1.0, got {rec}");
    assert!((mrr - 0.2).abs() < 1e-6, "all rank-5 hits → mrr=0.2, got {mrr}");

    // All misses → recall = 0.0, mrr = 0.0
    let ranks_none = vec![None, None, None];
    let (rec, mrr) = compute_metrics(&ranks_none);
    assert!((rec - 0.0).abs() < 1e-6, "all misses → recall=0.0, got {rec}");
    assert!((mrr - 0.0).abs() < 1e-6, "all misses → mrr=0.0, got {mrr}");

    // Mixed: 2 hits at rank 1, 1 hit at rank 3, 1 miss → recall = 0.75, mrr = (1 + 1 + 1/3 + 0) / 4 = 0.5833...
    let ranks_mixed = vec![Some(1usize), Some(1), Some(3), None];
    let (rec, mrr) = compute_metrics(&ranks_mixed);
    assert!((rec - 0.75).abs() < 1e-6, "3/4 hits → recall=0.75, got {rec}");
    let expected_mrr = (1.0 + 1.0 + 1.0 / 3.0 + 0.0) / 4.0;
    assert!((mrr - expected_mrr).abs() < 1e-6, "mixed → mrr={expected_mrr}, got {mrr}");

    // Empty input → 0/0 safely
    let (rec, mrr) = compute_metrics(&[]);
    assert_eq!(rec, 0.0);
    assert_eq!(mrr, 0.0);

    // Sanity: GoldenQuery shape is what evaluate() will iterate over.
    let _q = GoldenQuery {
        query: "auth".into(),
        expected_source_url: "auth.md".into(),
    };
}

/// End-to-end: ingest the golden fixture, run evaluate(), and assert the
/// quality gate (recall@5 >= 0.8, MRR >= 0.6). Ignored by default because it
/// loads the real embedder (~30s + ~66MB model download on first run).
#[test]
#[ignore = "needs real embedder (~66MB download, ~30s)"]
fn test_evaluate_meets_threshold() {
    use nowdocs::ingest::ingest_dir;

    // Isolated cache so we don't clobber any real docset.
    let cache_dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", cache_dir.path()) };

    // Locate fixture corpus shipped with the crate.
    let fixture_dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "tests", "fixtures", "golden"]
        .iter()
        .collect();
    let golden_json: PathBuf = fixture_dir.join("golden.json");

    // Ingest fixture into a uniquely named docset.
    let docset = "golden_e2e";
    let stats = ingest_dir(&fixture_dir, docset).expect("ingest fixture corpus");
    assert!(stats.files >= 3, "fixture must have >= 3 md files, got {}", stats.files);
    assert!(stats.chunks > 0, "fixture must produce chunks");

    // Load golden.json into a Vec<GoldenQuery>.
    let raw = std::fs::read_to_string(&golden_json).expect("read golden.json");
    let entries: Vec<serde_json::Value> =
        serde_json::from_str(&raw).expect("golden.json must be a JSON array");
    let golden: Vec<GoldenQuery> = entries
        .into_iter()
        .map(|v| GoldenQuery {
            query: v["query"].as_str().unwrap().to_string(),
            expected_source_url: v["expected_source_url"].as_str().unwrap().to_string(),
        })
        .collect();
    assert!(golden.len() >= 10, "golden.json should have >= 10 queries, got {}", golden.len());

    // Run the eval.
    let report = evaluate(docset, &golden).expect("evaluate over golden set");
    eprintln!(
        "golden-eval: n={} recall@5={:.3} mrr={:.3} (gates: recall>={} mrr>={})",
        report.n, report.recall_at_5, report.mrr, RECALL_GATE, MRR_GATE
    );

    assert!(
        report.recall_at_5 >= RECALL_GATE,
        "recall@5 {} below gate {} — retrieval regressed",
        report.recall_at_5,
        RECALL_GATE
    );
    assert!(
        report.mrr >= MRR_GATE,
        "mrr {} below gate {} — retrieval regressed",
        report.mrr,
        MRR_GATE
    );
    assert_eq!(report.n, golden.len(), "report.n must equal golden.len()");
}