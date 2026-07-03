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
    assert!(
        (rec - 1.0).abs() < 1e-6,
        "all rank-1 hits → recall=1.0, got {rec}"
    );
    assert!(
        (mrr - 1.0).abs() < 1e-6,
        "all rank-1 hits → mrr=1.0, got {mrr}"
    );

    // All hits at rank 5 → recall = 1.0, mrr = 0.2
    let ranks_all_fifth = vec![Some(5usize), Some(5)];
    let (rec, mrr) = compute_metrics(&ranks_all_fifth);
    assert!(
        (rec - 1.0).abs() < 1e-6,
        "all rank-5 hits → recall=1.0, got {rec}"
    );
    assert!(
        (mrr - 0.2).abs() < 1e-6,
        "all rank-5 hits → mrr=0.2, got {mrr}"
    );

    // All misses → recall = 0.0, mrr = 0.0
    let ranks_none = vec![None, None, None];
    let (rec, mrr) = compute_metrics(&ranks_none);
    assert!(
        (rec - 0.0).abs() < 1e-6,
        "all misses → recall=0.0, got {rec}"
    );
    assert!((mrr - 0.0).abs() < 1e-6, "all misses → mrr=0.0, got {mrr}");

    // Mixed: 2 hits at rank 1, 1 hit at rank 3, 1 miss → recall = 0.75, mrr = (1 + 1 + 1/3 + 0) / 4 = 0.5833...
    let ranks_mixed = vec![Some(1usize), Some(1), Some(3), None];
    let (rec, mrr) = compute_metrics(&ranks_mixed);
    assert!(
        (rec - 0.75).abs() < 1e-6,
        "3/4 hits → recall=0.75, got {rec}"
    );
    let expected_mrr = (1.0 + 1.0 + 1.0 / 3.0 + 0.0) / 4.0;
    assert!(
        (mrr - expected_mrr).abs() < 1e-6,
        "mixed → mrr={expected_mrr}, got {mrr}"
    );

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
    use nowdocs::ingest::{ingest_dir, IngestMeta};

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
    let stats =
        ingest_dir(&fixture_dir, docset, &IngestMeta::default()).expect("ingest fixture corpus");
    assert!(
        stats.files >= 3,
        "fixture must have >= 3 md files, got {}",
        stats.files
    );
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
    assert!(
        golden.len() >= 10,
        "golden.json should have >= 10 queries, got {}",
        golden.len()
    );

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

/// Shared Next.js golden query set — concept-level questions whose expected
/// `source_url` is the getting-started / guide page that answers them.
fn golden_nextjs() -> Vec<GoldenQuery> {
    vec![
        GoldenQuery {
            query: "how to install create-next-app CLI setup new project".into(),
            expected_source_url: "01-app/01-getting-started/01-installation.md".into(),
        },
        GoldenQuery {
            query: "linking and navigating between routes Link component prefetch".into(),
            expected_source_url: "01-app/01-getting-started/04-linking-and-navigating.md".into(),
        },
        GoldenQuery {
            query: "server components vs client components use client directive".into(),
            expected_source_url: "01-app/01-getting-started/05-server-and-client-components.md"
                .into(),
        },
        GoldenQuery {
            query: "fetching data in server components async await fetch".into(),
            expected_source_url: "01-app/01-getting-started/06-fetching-data.md".into(),
        },
        GoldenQuery {
            query: "caching fetch requests cache options force-cache no-store".into(),
            expected_source_url: "01-app/01-getting-started/08-caching.md".into(),
        },
        GoldenQuery {
            query: "revalidating data revalidateTag revalidatePath ISR".into(),
            expected_source_url: "01-app/01-getting-started/09-revalidating.md".into(),
        },
        GoldenQuery {
            query: "error handling error.tsx error boundary recovery".into(),
            expected_source_url: "01-app/01-getting-started/10-error-handling.md".into(),
        },
        GoldenQuery {
            query: "route handlers GET POST API endpoints request response".into(),
            expected_source_url: "01-app/01-getting-started/15-route-handlers.md".into(),
        },
        GoldenQuery {
            query: "authentication session strategies auth providers".into(),
            expected_source_url: "01-app/02-guides/authentication.md".into(),
        },
        GoldenQuery {
            query: "environment variables env files NODE_ENV".into(),
            expected_source_url: "01-app/02-guides/environment-variables.md".into(),
        },
    ]
}

/// Exploratory: ingest the rebuilt Next.js corpus (437 files / ~7480 chunks)
/// and run concept-level golden queries to probe retrieval quality on a real
/// large docset — the 3-file synthetic fixture's MRR 1.0 does not generalize
/// by itself. Prints per-query rank + recall/MRR. No hard gate (exploratory);
/// only asserts recall stays reasonable for a real corpus.
#[test]
#[ignore = "needs real embedder + rebuilt nextjs corpus (~minutes)"]
fn test_eval_nextjs_real() {
    use nowdocs::ingest::{ingest_dir, IngestMeta};
    use nowdocs::{eval::compute_metrics, retrieve};

    let cache_dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", cache_dir.path()) };

    let corpus: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "seed-crates",
        "tmp",
        "nextjs_rebuilt",
    ]
    .iter()
    .collect();
    assert!(
        corpus.exists(),
        "run `uv run python3 seed-crates/tmp/rebuild_nextjs.py` first"
    );

    let meta = IngestMeta {
        license: "MIT".into(),
        copyright_holder: "Vercel, Inc.".into(),
        source_url: "https://github.com/vercel/next.js".into(),
        entry_url: "https://nextjs.org/docs".into(),
        attribution: String::new(),
    };
    let stats = ingest_dir(&corpus, "nextjs_real", &meta).expect("ingest nextjs corpus");
    eprintln!(
        "nextjs-real ingest: {} files, {} chunks",
        stats.files, stats.chunks
    );

    let golden = golden_nextjs();

    let mut ranks: Vec<Option<usize>> = Vec::with_capacity(golden.len());
    for q in &golden {
        let result = retrieve::search("nextjs_real", &q.query, Some(4000), Some(5))
            .expect("search nextjs_real");
        let rank = result
            .chunks
            .iter()
            .take(5)
            .position(|c| c.source_url == q.expected_source_url)
            .map(|p| p + 1);
        eprintln!(
            "  q={:?} expected={:?} rank={:?} hits={}",
            q.query,
            q.expected_source_url,
            rank,
            result
                .chunks
                .iter()
                .map(|c| c.source_url.clone())
                .collect::<Vec<_>>()
                .join(",")
        );
        ranks.push(rank);
    }
    let (recall, mrr) = compute_metrics(&ranks);
    eprintln!(
        "nextjs-real eval: n={} recall@5={:.3} mrr={:.3}",
        golden.len(),
        recall,
        mrr
    );
    assert!(
        recall >= 0.5,
        "recall@5 {recall} too low on real nextjs corpus"
    );
}

/// Diagnostic: bypass `retrieve::search`'s window expansion and probe the raw
/// hybrid search (FTS + vector + RRF) top-15 for each golden query. This
/// separates two miss root causes:
/// - "hybrid never recalled" — expected absent from raw top-15 → fix
///   FTS/vector/RRF recall.
/// - "window squeezed out" — expected present in raw top-5 but pushed beyond
///   rank 5 by hub-chunk neighbor windows → fix window assembly.
#[test]
#[ignore = "needs real embedder + rebuilt nextjs corpus (~minutes)"]
fn test_eval_nextjs_diagnose() {
    use nowdocs::cache::manifest_path;
    use nowdocs::embedder::{Embedder, EmbedderSpec};
    use nowdocs::ingest::{ingest_dir, IngestMeta};
    use nowdocs::manifest::{parse_manifest, validate};
    use nowdocs::store::Store;

    let cache_dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", cache_dir.path()) };

    let corpus: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "seed-crates",
        "tmp",
        "nextjs_rebuilt",
    ]
    .iter()
    .collect();
    assert!(
        corpus.exists(),
        "run `uv run python3 seed-crates/tmp/rebuild_nextjs.py` first"
    );

    let meta = IngestMeta {
        license: "MIT".into(),
        copyright_holder: "Vercel, Inc.".into(),
        source_url: "https://github.com/vercel/next.js".into(),
        entry_url: "https://nextjs.org/docs".into(),
        attribution: String::new(),
    };
    let stats = ingest_dir(&corpus, "nextjs_real", &meta).expect("ingest nextjs corpus");
    eprintln!(
        "nextjs-diagnose ingest: {} files, {} chunks",
        stats.files, stats.chunks
    );

    // Load manifest + embedder once (mirror retrieve::search's setup).
    let manifest_json = std::fs::read_to_string(manifest_path("nextjs_real")).unwrap();
    let manifest = parse_manifest(&manifest_json).unwrap();
    validate(&manifest).unwrap();
    let spec = EmbedderSpec {
        model_id: manifest.embedder.model_id.clone(),
        model_revision: manifest.embedder.model_revision.clone(),
        model_sha256: manifest.embedder.model_sha256.clone(),
    };
    let embedder = Embedder::load_for(&spec).expect("load embedder");
    let store = Store::open("nextjs_real").expect("open store");

    let raw_topn = 15usize;
    let golden = golden_nextjs();
    for q in &golden {
        let qv = embedder.embed(&q.query).expect("embed query");
        let hits = store
            .hybrid_search(&qv, &q.query, raw_topn)
            .expect("hybrid search");
        let raw_rank = hits
            .iter()
            .position(|h| h.source_url == q.expected_source_url)
            .map(|p| p + 1);
        eprintln!(
            "  q={:?} expected={:?} raw_hybrid_rank={:?} (of top {})",
            q.query, q.expected_source_url, raw_rank, raw_topn
        );
        eprintln!(
            "    top-{}: {}",
            raw_topn,
            hits.iter()
                .map(|h| format!("{}@{}({:.3})", h.source_url, h.chunk_idx, h.score))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
}
