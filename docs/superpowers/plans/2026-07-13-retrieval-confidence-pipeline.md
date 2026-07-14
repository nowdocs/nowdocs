# Retrieval Confidence Pipeline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an evidence-preserving, evaluated retrieval pipeline that reports `confident`, `borderline`, or `no_answer` without misdiagnosing healthy no-answer searches as broken installations.

**Architecture:** Keep LanceDB, the pinned local embedder, RRF, MMR, neighbor expansion, token packing, and stdio MCP. Add a versioned evaluation foundation first, then a behavior-compatible retrieval trace and signal-preserving RRF adapter, then binary-equivalent answer-state contracts, and enable `borderline` only after a single global monotonic policy passes development selection and frozen three-docset tests.

**Tech Stack:** Rust 1.97, LanceDB 0.31, Arrow 58.3, Candle/Jina embeddings, serde/serde_json, Clap, GitHub Actions, MCP `2025-11-25` NDJSON.

---

## Scope and stop gates

This is one ordered plan because evaluation, runtime evidence, public contracts,
and calibration depend on each other. Do not parallelize tasks that consume the
same types or evaluation artifacts.

Two gates are mandatory:

1. Do not run policy selection until the Next.js, React, and Vue datasets pass
   schema/count review and a maintainer reviews the relevance targets.
2. Do not enable `borderline` in runtime code unless the frozen test report
   passes every aggregate and per-docset gate. If no policy passes, commit the
   evaluator and binary-equivalent state work, keep `borderline` disabled, and
   open a focused follow-up with the measured failure report.

Do not move MMR, add a second model, expose internal scores through MCP, or
change context assembly in this plan.

## File map

**Create:**

- `src/confidence.rs` — answer-state types, evidence summary, binary-compatible
  decision, monotonic calibrated policy.
- `examples/retrieval_eval.rs` — internal release-build evaluator and policy
  calibration entrypoint; not a public nowdocs CLI subcommand.
- `tests/eval_schema_tests.rs` — fixture validation and split-isolation tests.
- `tests/eval_metrics_tests.rs` — pure ranking/state/Wilson metric tests.
- `tests/eval_report_tests.rs` — versioned JSON report contract tests.
- `tests/confidence_tests.rs` — pure binary compatibility, monotonicity, and
  calibrated decision tests.
- `tests/fixtures/eval/nextjs.json`, `react.json`, `vue.json` — reviewed,
  versioned development/test query families and relevance targets.
- `docs/superpowers/evals/2026-07-13-retrieval-confidence-baseline.md` — corpus
  identity, commands, baseline metrics, and latency.
- `scripts/ci-prepare-react-fixture.sh` and
  `scripts/ci-prepare-vue-fixture.sh` — deterministic real-corpus fixture setup.
- `scripts/rebuild-share-docs.py` — path-safe, docset-agnostic reconstruction
  of Markdown files from a pinned `chunks.jsonl` artifact.

**Modify:**

- `src/lib.rs` — register `confidence`.
- `Cargo.toml` and `Cargo.lock` — add the direct `async-trait` dependency needed
  to implement LanceDB's async `Reranker` trait.
- `src/eval.rs` — fixture schema, validation, metrics, reports, stage evaluator,
  policy search, and release gates.
- `src/store.rs` — signal-preserving RRF adapter and `CandidateEvidence`.
- `src/retrieve.rs` — traceable orchestration, query evidence, answer decision,
  and `SearchResult.answer_state`.
- `src/smoke.rs` and `src/main.rs` — answer-state JSON/human output and exit
  semantics.
- `src/tools.rs` — MCP structured content and text fallback state.
- `tests/eval_tests.rs`, `tests/retrieve_tests.rs`, `tests/store_tests.rs`,
  `tests/smoke_tests.rs`, and `tests/tools_tests.rs` — migration and regression
  coverage.
- `.github/workflows/eval.yml` — three cached docsets, JSON artifact, and
  machine-readable gates.
- `docs/GETTING_STARTED.md`, `docs/TROUBLESHOOTING.md`, and `CHANGELOG.md` —
  smoke/no-answer compatibility notes.

## Task 1: Add the versioned evaluation schema and validator

**Files:**

- Create: `tests/eval_schema_tests.rs`
- Create: `tests/fixtures/eval/schema-smoke.json`
- Modify: `src/eval.rs`
- Modify: `tests/eval_tests.rs`

- [ ] **Step 1: Write failing schema and target-matching tests**

```rust
use nowdocs::eval::{
    hit_matches_target, validate_suite, EvalQuery, EvalSplit, QueryClass,
    QueryForm, RelevanceTarget,
};
use nowdocs::retrieve::ResultChunk;

#[test]
fn heading_target_requires_exact_or_descendant_segment() {
    let hit = ResultChunk {
        chunk_idx: 7,
        heading_path: "Exports > Matcher > Negative matching".into(),
        source_url: "01-app/03-api-reference/03-file-conventions/proxy.md".into(),
        api_version: None,
        chunk_type: nowdocs::chunker::ChunkType::Info,
        text: "matcher docs".into(),
        score: Some(0.03),
    };
    let target = RelevanceTarget {
        source_url: hit.source_url.clone(),
        heading_path_prefix: Some("## Exports > Matcher".into()),
        grade: 2,
    };
    assert!(hit_matches_target(&hit, &target));

    let partial = RelevanceTarget {
        heading_path_prefix: Some("Exports > Match".into()),
        ..target
    };
    assert!(!hit_matches_target(&hit, &partial));
}

#[test]
fn validator_rejects_one_intent_family_across_splits() {
    let dev = eval_query("q-dev", EvalSplit::Development, "middleware-matcher");
    let test = eval_query("q-test", EvalSplit::Test, "middleware-matcher");
    let error = validate_suite(&[dev, test]).unwrap_err().to_string();
    assert!(error.contains("intent family"));
    assert!(error.contains("multiple splits"));
}
```

Include helpers that construct a positive query with one grade-2 target. Add
negative tests for duplicate query IDs, grades outside `1..=2`, a positive
query without targets, and a negative query with targets.

Use this exact helper so the test is self-contained:

```rust
fn eval_query(id: &str, split: EvalSplit, family: &str) -> EvalQuery {
    EvalQuery {
        id: id.into(),
        docset: "nextjs".into(),
        query: "middleware matcher".into(),
        split,
        intent_family: family.into(),
        query_form: QueryForm::Short,
        query_class: QueryClass::Positive,
        targets: vec![RelevanceTarget {
            source_url: "01-app/03-api-reference/03-file-conventions/proxy.md".into(),
            heading_path_prefix: Some("Exports > Matcher".into()),
            grade: 2,
        }],
    }
}
```

- [ ] **Step 2: Run the focused test and verify it fails**

Run:

```bash
cargo test --test eval_schema_tests -- --test-threads=1
```

Expected: compilation fails because the new evaluation types and functions do
not exist.

- [ ] **Step 3: Implement the schema and validation**

Add these public shapes to `src/eval.rs` and derive
`Serialize`, `Deserialize`, `Clone`, `Debug`, and equality traits:

```rust
#[serde(rename_all = "snake_case")]
pub enum EvalSplit { Development, Test }

#[serde(rename_all = "snake_case")]
pub enum QueryForm { Short, NaturalLanguage, Verbose, KeywordHeavy }

#[serde(rename_all = "snake_case")]
pub enum QueryClass { Positive, NearDomainNegative, CrossDomainNegative }

pub struct RelevanceTarget {
    pub source_url: String,
    #[serde(default)]
    pub heading_path_prefix: Option<String>,
    pub grade: u8,
}

pub struct EvalQuery {
    pub id: String,
    pub docset: String,
    pub query: String,
    pub split: EvalSplit,
    pub intent_family: String,
    pub query_form: QueryForm,
    pub query_class: QueryClass,
    #[serde(default)]
    pub targets: Vec<RelevanceTarget>,
}
```

Reuse the existing `pub(crate)` `chunker::normalize_heading_path`. Implement
target matching as exact URL equality plus either an exact normalized heading
or a descendant whose suffix starts with `" > "`. `validate_suite` must
enforce unique IDs, one split per `(docset, intent_family)`, valid grades,
positive/negative target rules, and non-empty IDs/docset/query/intent family.

Migrate the toy JSON loader and existing `GoldenQuery` constructions in
`tests/eval_tests.rs` to `EvalQuery`; remove `GoldenQuery` only after all callers
compile.

- [ ] **Step 4: Run schema and legacy eval tests**

```bash
cargo test --test eval_schema_tests -- --test-threads=1
cargo test --test eval_tests --no-run
```

Expected: schema tests pass; the existing eval test target compiles.

- [ ] **Step 5: Commit**

```bash
git add src/eval.rs tests/eval_schema_tests.rs tests/eval_tests.rs tests/fixtures/eval/schema-smoke.json tests/fixtures/golden/golden.json tests/fixtures/golden/negative.json
git commit -s -m "feat(eval): add versioned relevance schema"
```

## Task 2: Implement pure ranking and answer-state metrics

**Files:**

- Create: `src/confidence.rs`
- Modify: `src/lib.rs`
- Create: `tests/eval_metrics_tests.rs`
- Modify: `src/eval.rs`

- [ ] **Step 1: Write failing metric tests**

Cover a ranked list where two chunks match the same grade-2 target and a third
chunk matches a grade-1 target. Assert the duplicate receives zero gain:

```rust
#[test]
fn duplicate_chunks_do_not_inflate_precision_or_ndcg() {
    let report = compute_ranking_metrics(&hits(), &targets(), 5);
    assert_eq!(report.relevant_targets_found, 2);
    assert!((report.recall - 1.0).abs() < 1e-6);
    assert!((report.precision - 0.5).abs() < 1e-6);
    assert!(report.ndcg > 0.0 && report.ndcg <= 1.0);
}

#[test]
fn wilson_interval_contains_observed_rate() {
    let estimate = rate_estimate(1, 20);
    assert!((estimate.rate - 0.05).abs() < 1e-6);
    assert!(estimate.lower <= estimate.rate);
    assert!(estimate.upper >= estimate.rate);
}
```

Also test empty input, MRR at ranks 1 and 5, ideal nDCG ordering, false reject,
false accept, borderline rate, and decisive coverage.

- [ ] **Step 2: Run and verify failure**

```bash
cargo test --test eval_metrics_tests -- --test-threads=1
```

Expected: compilation fails on missing metric/report types.

- [ ] **Step 3: Implement exact metric types and formulas**

Add the shared serialized label in `src/confidence.rs` and register the module
from `src/lib.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswerState { Confident, Borderline, NoAnswer }
```

Add these report types to `src/eval.rs` and make state metrics consume
`AnswerState` values:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RankingMetrics {
    pub k: usize,
    pub relevant_targets_found: usize,
    pub recall: f32,
    pub mrr: f32,
    pub ndcg: f32,
    pub precision: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RateEstimate {
    pub count: usize,
    pub total: usize,
    pub rate: f32,
    pub lower: f32,
    pub upper: f32,
}
```

Use gain `(2^grade)-1`, discount `1/log2(rank+1)`, one gain per target, and
Wilson 95% `z = 1.959963984540054`. Empty denominators return zero for all
fields. Keep wrappers for the old `compute_metrics` and `false_positive_rate`
until callers are migrated, then delete them in Task 4.

- [ ] **Step 4: Run tests**

```bash
cargo test --test eval_metrics_tests -- --test-threads=1
cargo test --test eval_tests -- --test-threads=1
```

Expected: all non-ignored tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/confidence.rs src/lib.rs src/eval.rs tests/eval_metrics_tests.rs tests/eval_tests.rs
git commit -s -m "feat(eval): add graded retrieval metrics"
```

## Task 3: Add behavior-preserving retrieval traces

**Files:**

- Modify: `src/retrieve.rs`
- Modify: `tests/retrieve_tests.rs`

- [ ] **Step 1: Write failing pure trace tests**

Add tests that pass synthetic vectors and candidates through the same ranking
helper with tracing disabled and enabled. Assert identical MMR IDs and binary
gate outcome, and assert that the cosine margin is computed from the pre-MMR
pool rather than the MMR runner-up.

```rust
#[test]
fn trace_does_not_change_rank_or_gate() {
    let plain = rank_and_gate_candidates(&qv(), hits(), &vectors(), 3, false);
    let traced = rank_and_gate_candidates(&qv(), hits(), &vectors(), 3, true);
    let plain_ids: Vec<_> = plain.hits.iter().map(|hit| hit.chunk_idx).collect();
    let traced_ids: Vec<_> = traced.hits.iter().map(|hit| hit.chunk_idx).collect();
    assert_eq!(plain_ids, traced_ids);
    assert_eq!(plain.gate_passed, traced.gate_passed);
    assert!(traced.trace.is_some());
}
```

- [ ] **Step 2: Run and verify failure**

```bash
cargo test --test retrieve_tests trace -- --test-threads=1
```

Expected: compilation fails on the new helper and trace fields.

- [ ] **Step 3: Implement one orchestration path with optional trace capture**

Add these internal/public-for-tests types:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct TraceHit {
    pub chunk_idx: u32,
    pub source_url: String,
    pub rrf_score: f32,
    pub cosine: Option<f32>,
    pub dense_rank: Option<u32>,
    pub lexical_rank: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetrievalTrace {
    pub fused: Vec<TraceHit>,
    pub mmr: Vec<TraceHit>,
    pub pre_mmr_top_cosines: Vec<f32>,
    pub gate_passed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RankedGateResult {
    pub hits: Vec<SearchHit>,
    pub gate_passed: bool,
    pub trace: Option<RetrievalTrace>,
}
```

Refactor `search` into `search_impl(..., trace: bool)`. `search` calls it with
`false`; `search_with_trace` calls it with `true`. Clone candidate metadata only
inside the trace branch. Preserve the current `0.82`/dual-rank binary behavior
exactly. Make `rank_and_gate_candidates` return `RankedGateResult`; do not
use `AnswerState` in runtime retrieval yet. Recompute raw cosine from the query
vector and the candidate vectors already fetched for MMR; never treat
LanceDB's query-local normalized `_distance` or `_score` as raw evidence. Read
the pre-MMR cosine top two from the full fused pool, and read the gate cosine
from the selected post-MMR top hit. Leave channel-rank trace fields `None`
until Task 6 supplies them.

- [ ] **Step 4: Run retrieve and smoke tests**

```bash
cargo test --test retrieve_tests -- --test-threads=1
cargo test --test smoke_tests -- --test-threads=1
```

Expected: all non-ignored tests pass with unchanged behavior.

- [ ] **Step 5: Commit**

```bash
git add src/retrieve.rs tests/retrieve_tests.rs
git commit -s -m "refactor(retrieve): add behavior-preserving trace"
```

## Task 4: Build the JSON evaluator and report contract

**Files:**

- Create: `examples/retrieval_eval.rs`
- Create: `tests/eval_report_tests.rs`
- Modify: `src/eval.rs`
- Modify: `tests/eval_tests.rs`

- [ ] **Step 1: Write failing report serialization tests**

Define a fixture report and assert exact top-level fields:

```rust
#[test]
fn report_v1_serializes_machine_readable_contract() {
    let value = serde_json::to_value(report_fixture()).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert!(value["corpora"].is_array());
    assert!(value["stages"]["fused_at_40"].is_object());
    assert!(value["answer_states"]["false_reject"].is_object());
    assert!(value["queries"].is_array());
}
```

- [ ] **Step 2: Run and verify failure**

```bash
cargo test --test eval_report_tests -- --test-threads=1
```

Expected: compilation fails on missing report types.

- [ ] **Step 3: Implement report types and stage evaluation**

Add serializable `EvalReportV1`, `CorpusIdentity`, `StageMetrics`,
`AnswerStateMetrics`, `QueryReport`, and this report-only reason enum:

```rust
#[serde(rename_all = "snake_case")]
pub enum ReportDecisionReason {
    NoCandidates,
    CurrentGatePass,
    CurrentGateReject,
    CalibratedConfident,
    CalibratedBorderline,
    CalibratedNoAnswer,
}
```

`evaluate_suite` must:

1. validate fixtures before model/store access;
2. filter by requested split;
3. use `Store::vector_search` and `Store::fts_search` only in evaluation for
   channel Recall@40;
4. use `search_with_trace` for fused/MMR/output stages;
5. record query IDs, never raw query text, in report rows;
6. include code commit, command, retrieval parameters, OS/architecture,
   manifest SHA-256, `doc_version`, chunk count, model revision, model hash, and
   policy identity (`binary-current-gate-v1` until Task 10 activates the frozen
   policy).

The report's `stages` object has exactly these keys:
`dense_at_40`, `fts_at_40`, `fused_at_40`, `mmr_at_5`, and `output_at_5`.
Each value contains Recall, MRR, nDCG, and Precision with the stage's recorded
`k`; channel reports may additionally expose their native rank but not treat a
normalized LanceDB value as a cross-query score. `answer_states` contains
counts plus Wilson estimates for false reject, false accept, positive
borderline, and negative borderline, together with decisive coverage. Emit the
same stage/state shapes under `by_docset`, `by_query_form`, and
`by_query_class`. Each `QueryReport` contains query ID, docset, split, form,
class, matched target indexes/grades, stage ranks, state, and decision reason; it
must not contain the query text or raw chunk text.

Implement `examples/retrieval_eval.rs` with Clap arguments:

```text
--fixtures-dir tests/fixtures/eval
--split development|test
--output /absolute/path/report.json
--code-commit 40-character-git-commit
--benchmark-runs 30
```

`--benchmark-runs` defaults to `1`; values above one repeat each query in the
same process after one unmeasured warm-up and report median and p95 retrieval
latency.

- [ ] **Step 4: Run unit tests and evaluator help**

```bash
cargo test --test eval_report_tests -- --test-threads=1
cargo run --example retrieval_eval -- --help
```

Expected: tests pass; help lists all arguments and exits zero without loading a
model.

- [ ] **Step 5: Commit**

```bash
git add src/eval.rs examples/retrieval_eval.rs tests/eval_report_tests.rs tests/eval_tests.rs
git commit -s -m "feat(eval): add versioned JSON evaluator"
```

## Task 5: Curate three docsets and capture versioned baseline evidence

**Files:**

- Create: `tests/fixtures/eval/nextjs.json`
- Create: `tests/fixtures/eval/react.json`
- Create: `tests/fixtures/eval/vue.json`
- Create: `docs/superpowers/evals/2026-07-13-retrieval-confidence-baseline.md`
- Modify: `tests/eval_schema_tests.rs`

- [ ] **Step 1: Add the known short-query regression record first**

The Next.js file must contain this frozen test record, with the exact corpus
path and normalized heading target verified against the pinned chunks:

```json
{
  "id": "nextjs.middleware-matcher.short",
  "docset": "nextjs",
  "query": "middleware matcher",
  "split": "test",
  "intent_family": "middleware-matcher",
  "query_form": "short",
  "query_class": "positive",
  "targets": [
    {
      "source_url": "01-app/03-api-reference/03-file-conventions/proxy.md",
      "heading_path_prefix": "Exports > Matcher",
      "grade": 2
    }
  ]
}
```

- [ ] **Step 2: Curate and review the required intent families**

For each docset, create at least 20 positive/15 negative development families
and 20 positive/15 negative test families. At least half of each split's
negatives must be near-domain. Keep every paraphrase of one intent in one
split. Use grade 2 for directly answering sections and grade 1 for supporting
sections. Do not use `chunk_idx` as a target.

This is a human review gate, not a generation task. Record reviewer initials
and the manifest SHA-256 in the baseline document. Do not proceed with
calibration if any target cannot be verified against the pinned chunks.

- [ ] **Step 3: Add count and split assertions**

```rust
#[test]
fn real_suites_meet_minimum_family_counts() {
    for file in ["nextjs.json", "react.json", "vue.json"] {
        let suite = load_eval_file(eval_dir().join(file)).unwrap();
        validate_suite(&suite).unwrap();
        assert_minimum_families(&suite, EvalSplit::Development, 20, 15);
        assert_minimum_families(&suite, EvalSplit::Test, 20, 15);
        assert_near_domain_share(&suite, EvalSplit::Development, 0.5);
        assert_near_domain_share(&suite, EvalSplit::Test, 0.5);
    }
}
```

- [ ] **Step 4: Run validation and the current development baseline**

```bash
cargo test --test eval_schema_tests -- --test-threads=1
cargo run --release --example retrieval_eval -- \
  --fixtures-dir tests/fixtures/eval \
  --split development \
  --output /tmp/nowdocs-confidence-baseline.json \
  --code-commit "$(git rev-parse HEAD)"
```

Expected: validation passes; JSON contains all three corpora and current binary
state/ranking metrics. Copy aggregate, per-docset, risk-group, and latency
values—not raw query text—into the baseline Markdown document.

- [ ] **Step 5: Commit the reviewed data and baseline**

```bash
git add tests/fixtures/eval tests/eval_schema_tests.rs docs/superpowers/evals/2026-07-13-retrieval-confidence-baseline.md
git commit -s -m "test(eval): add reviewed retrieval suites"
```

## Task 6: Preserve channel ranks with a compatible RRF adapter

**Files:**

- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `src/store.rs`
- Modify: `src/retrieve.rs`
- Modify: `tests/store_tests.rs`
- Modify: `tests/retrieve_tests.rs`

- [ ] **Step 1: Write failing Arrow compatibility tests**

Construct vector and FTS `RecordBatch` values with overlapping `_rowid` values.
Run LanceDB `RRFReranker` and `SignalPreservingRrf` with `k = 60`. Assert equal
row IDs, `_relevance_score` values within `1e-7`, and ordering. Assert adapter
columns `_dense_rank` and `_fts_rank` are one-based and nullable.

```rust
assert_eq!(row_ids(&expected), row_ids(&actual));
assert_scores_close(&expected, &actual, 1e-7);
assert_eq!(u32_values(&actual, "_dense_rank"), vec![Some(2), Some(1)]);
assert_eq!(u32_values(&actual, "_fts_rank"), vec![Some(1), None]);
```

- [ ] **Step 2: Run and verify failure**

```bash
cargo test --test store_tests signal_preserving_rrf -- --test-threads=1
```

Expected: compilation fails because the adapter does not exist.

- [ ] **Step 3: Implement the concrete adapter and candidate type**

Add `async-trait = "0.1"` to `[dependencies]`. LanceDB 0.31 annotates
`Reranker` with `#[async_trait]`, so this direct dependency is required for the
downstream implementation; do not add another runtime or reranking crate.

Add:

```rust
pub const DENSE_RANK_COLUMN: &str = "_dense_rank";
pub const FTS_RANK_COLUMN: &str = "_fts_rank";

#[derive(Debug, Clone)]
pub struct CandidateEvidence {
    pub hit: SearchHit,
    pub dense_rank: Option<u32>,
    pub lexical_rank: Option<u32>,
}

#[derive(Debug)]
pub struct SignalPreservingRrf { k: f32 }
```

Implement LanceDB's `Reranker` directly. Build row-ID-to-one-based-rank maps
from input order, reproduce the built-in zero-based `1/(index+k)` RRF score,
merge and sort exactly like the built-in implementation, and append nullable
rank columns. Do not treat normalized `_distance`/`_score` as raw evidence.

Change `hybrid_search`/`hybrid_search_k` to return
`Vec<CandidateEvidence>`. Extend the hybrid result parser to read the nullable
rank columns by row and wrap each parsed `SearchHit`; fail on a non-`UInt32`
rank column rather than silently dropping evidence. Keep fetch methods returning
`SearchHit`. Adapt MMR to preserve evidence while using `candidate.hit` for
existing fields. Populate trace rank fields from `CandidateEvidence`.

- [ ] **Step 4: Prove behavior compatibility**

```bash
cargo test --test store_tests -- --test-threads=1
cargo test --test retrieve_tests -- --test-threads=1
```

Expected: all tests pass, including exact pre-existing MMR ID order and binary
gate behavior.

- [ ] **Step 5: Run the warmed performance comparison**

Before implementing the adapter, run and retain:

```bash
cargo run --release --example retrieval_eval -- \
  --fixtures-dir tests/fixtures/eval \
  --split development \
  --benchmark-runs 30 \
  --output /tmp/nowdocs-confidence-pre-rrf.json \
  --code-commit "$(git rev-parse HEAD)"
```

After Step 4, run the same command with output
`/tmp/nowdocs-confidence-post-rrf.json`. Expected: no extra normal-store query,
identical stage rankings, and median retrieval-time increase no greater than
10%. Append both median and p95 values to the baseline document.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/store.rs src/retrieve.rs tests/store_tests.rs tests/retrieve_tests.rs docs/superpowers/evals/2026-07-13-retrieval-confidence-baseline.md
git commit -s -m "feat(retrieve): preserve hybrid channel ranks"
```

## Task 7: Complete binary-equivalent answer-state decisions

**Files:**

- Modify: `src/confidence.rs`
- Create: `tests/confidence_tests.rs`
- Modify: `src/retrieve.rs`
- Modify: `tests/retrieve_tests.rs`

- [ ] **Step 1: Write failing binary-equivalence tests**

```rust
#[test]
fn binary_policy_matches_current_gate_boundaries() {
    let below = evidence(0.8199, 0.016, Some(1), None);
    assert_eq!(decide_binary(&below).state, AnswerState::NoAnswer);

    let cosine_pass = evidence(0.82, 0.016, Some(1), None);
    assert_eq!(decide_binary(&cosine_pass).state, AnswerState::Confident);

    let dual_rank_pass = evidence(0.75, 0.0332, Some(1), Some(1));
    assert_eq!(decide_binary(&dual_rank_pass).state, AnswerState::Confident);
}
```

Also assert serde values are exactly `confident`, `borderline`, and
`no_answer`.

- [ ] **Step 2: Run and verify failure**

```bash
cargo test --test confidence_tests -- --test-threads=1
```

Expected: compilation fails because `QueryEvidence`, `DecisionReason`, and
`decide_binary` do not exist yet.

- [ ] **Step 3: Implement minimal confidence types**

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct QueryEvidence {
    pub top_selected_cosine: Option<f32>,
    pub top_selected_rrf: Option<f32>,
    pub pre_mmr_top_cosine: Option<f32>,
    pub pre_mmr_second_cosine: Option<f32>,
    pub pre_mmr_cosine_margin: Option<f32>,
    pub dense_rank: Option<u32>,
    pub lexical_rank: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionReason {
    NoCandidates,
    CurrentCosinePass,
    CurrentDualRankPass,
    CurrentGateReject,
    CalibratedConfident,
    CalibratedBorderline,
    CalibratedNoAnswer,
}

pub struct AnswerDecision {
    pub state: AnswerState,
    pub reason: DecisionReason,
}
```

Implement `decide_binary` using the exact current cosine and RRF constants.
Add `answer_state` to `SearchResult`. Route retrieval through `decide_binary`;
make the decision after MMR and before neighbor expansion. `NoAnswer` returns
the current empty result invariant. Update the synthetic `SearchResult`
constructor in `tests/retrieve_tests.rs`. Do not enable calibrated `Borderline`
yet.

- [ ] **Step 4: Run confidence and retrieve tests**

```bash
cargo test --test confidence_tests -- --test-threads=1
cargo test --test retrieve_tests -- --test-threads=1
```

Expected: all pass; existing accept/reject behavior remains identical.

- [ ] **Step 5: Commit**

```bash
git add src/confidence.rs src/retrieve.rs tests/confidence_tests.rs tests/retrieve_tests.rs
git commit -s -m "feat(retrieve): add binary-compatible answer states"
```

## Task 8: Correct MCP and smoke response semantics

**Files:**

- Modify: `src/tools.rs`
- Modify: `src/smoke.rs`
- Modify: `src/main.rs`
- Modify: `tests/tools_tests.rs`
- Modify: `tests/smoke_tests.rs`
- Modify: `docs/GETTING_STARTED.md`
- Modify: `docs/TROUBLESHOOTING.md`
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Write failing MCP contract tests**

Extract a private `format_search_success(&SearchResult) -> Value` in
`src/tools.rs`, call it only after retrieval succeeds, and test it in the
existing in-file `#[cfg(test)]` module. For normal results, assert
`structuredContent.answer_state` and text-fallback state. For a synthetic
no-answer `SearchResult`, assert empty chunks, no `isError`, and no
`doctor`/reinstall hint. Keep the integration assertion that operational
store/model failures retain `isError = true`.

```rust
assert_eq!(result["structuredContent"]["answer_state"], "no_answer");
assert_eq!(result["structuredContent"]["chunks"], json!([]));
assert!(result.get("isError").is_none());
assert!(!result["content"][0]["text"].as_str().unwrap().contains("doctor"));
```

- [ ] **Step 2: Write failing smoke formatter/exit tests**

Assert:

- confident: exit 0 and `smoke ok`;
- borderline fixture formatter: exit semantics 0 and `smoke warning`;
- no-answer: full `SmokeResult` JSON, empty results, exit 1, no repair hint;
- operational error: error object and non-zero exit;
- JSON hit field remains `score`; human hit label is `rrf_score`.

Make exit behavior pure and testable through
`smoke::exit_code(&SmokeResult) -> i32`; `main.rs` must use that helper after it
prints the full result.

- [ ] **Step 3: Run and verify failures**

```bash
cargo test --test tools_tests -- --test-threads=1
cargo test --test smoke_tests -- --test-threads=1
```

Expected: failures show missing answer-state fields and old no-results hint.

- [ ] **Step 4: Implement the additive MCP and intentional CLI changes**

Add `answer_state` beside existing `chunks`, `tokens_returned`, and `truncated`.
Prefix fallback text with `answer_state: confident`, `answer_state: borderline`,
or `answer_state: no_answer`. For `borderline`, include
`Potentially relevant results found with lower confidence.` For `no_answer`,
include `No sufficiently supported match was found in this docset.`

Add `answer_state` to `SmokeResult`. In `main.rs`, always format successful
`SmokeResult` first, then exit 1 only when its state is `NoAnswer`. Delete the
current empty-result `doctor --docset` branch. Keep operational error handling
unchanged. Change only the human label from `score=` to `rrf_score=`; retain
the serialized `SmokeHit.score` field for compatibility.

- [ ] **Step 5: Document migration and run tests**

Document that no-answer JSON changes from an error object to an empty
`SmokeResult`, while operational failures remain errors. Run:

```bash
cargo test --test tools_tests -- --test-threads=1
cargo test --test smoke_tests -- --test-threads=1
cargo test --test doctor_tests -- --test-threads=1
```

Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add src/tools.rs src/smoke.rs src/main.rs tests/tools_tests.rs tests/smoke_tests.rs docs/GETTING_STARTED.md docs/TROUBLESHOOTING.md CHANGELOG.md
git commit -s -m "fix(smoke): distinguish no answer from broken install"
```

## Task 9: Implement deterministic global policy calibration

**Files:**

- Modify: `src/confidence.rs`
- Modify: `src/eval.rs`
- Modify: `examples/retrieval_eval.rs`
- Modify: `tests/confidence_tests.rs`
- Modify: `tests/eval_report_tests.rs`

- [ ] **Step 1: Write failing monotonic policy tests**

Define `ConfidencePolicy` with `confident_cosine`, `no_answer_cosine`,
`margin_min`, and `agreement_rank_max`. Assert that increasing the top-candidate
cosine, improving either channel rank, or adding dual-channel agreement cannot
lower the state. Assert `no_answer_cosine <= confident_cosine` and reject an
invalid policy. The selector must produce one global monotonic policy across
all docsets; it must not accept a docset identifier as a policy parameter.

- [ ] **Step 2: Write failing split-isolation and selection tests**

Build a mixed development/test fixture where the test rows would change the
winner. Assert `select_policy` receives development rows only and returns the
same policy whether the test rows are present or removed.

Candidate thresholds must be deterministic:

- cosine and margin candidates are sorted unique observed development values
  plus adjacent midpoints, using `f32::total_cmp` and bitwise-equal deduplication;
- agreement rank candidates are `[1, 2, 3, 5]`;
- only policies with `no_answer_cosine <= confident_cosine` are evaluated.

Selection applies aggregate and per-docset gates, maximizes decisive coverage,
then minimizes false accept, false reject, and finally maximizes nDCG@5.

- [ ] **Step 3: Run and verify failure**

```bash
cargo test --test confidence_tests calibrated -- --test-threads=1
cargo test --test eval_report_tests policy -- --test-threads=1
```

Expected: failures on missing calibrated policy and selector.

- [ ] **Step 4: Implement policy validation, decision, search, and JSON output**

Add serde-enabled `ConfidencePolicy` and a pure `decide(policy, evidence)`.
Only the top-candidate cosine is monotonic positive evidence; a larger runner-up
may reduce margin and therefore confidence. Implement deterministic grouped
five-fold development reporting by `(docset, intent_family)` and Wilson
interval output.

Assign an entire intent family to fold
`u64::from_be_bytes(SHA256(docset + "\0" + intent_family)[0..8]) % 5`. For
each fold, derive threshold candidates and select the winner from the other
four folds only, then score that winner on the held-out fold. Concatenate the
five held-out prediction sets and require the development gates on their
aggregate and per-docset metrics. Only after this out-of-fold check passes,
derive candidates and select the committed policy from the full development
split. Report all five fold assignments, policies, and held-out metrics; never
pass test records to either operation.

Use these ordered rules, with `dual_agreement` meaning both ranks are present
and each is `<= agreement_rank_max`:

```text
no candidate cosine                                      -> no_answer
dual_agreement OR
  (top cosine >= confident_cosine AND margin >= margin_min) -> confident
top cosine < no_answer_cosine                            -> no_answer
otherwise                                                -> borderline
```

Add these evaluator arguments in this task:

```text
--policy /absolute/path/policy.json
--calibrate-policy-output /absolute/path/policy.json
--enforce-gates
```

Reject `--calibrate-policy-output` with `--split test`; reject supplying both
arguments together. Accept `--enforce-gates` only with `--split test` and
`--policy`; without it, serialize gate outcomes but do not fail solely because
a metric is below threshold. Calibration writes a serde `PolicyFileV1` with
exactly these fields:

```rust
pub struct PolicyFileV1 {
    pub schema_version: u32,
    pub policy_id: String,
    pub dataset_sha256: String,
    pub confident_cosine: f32,
    pub no_answer_cosine: f32,
    pub margin_min: f32,
    pub agreement_rank_max: u32,
}
```

Set `schema_version = 1` and `policy_id = "global-confidence-v1"`. Compute
`dataset_sha256` over the exact bytes
`b"nextjs.json\0" + nextjs_bytes + b"react.json\0" + react_bytes +
b"vue.json\0" + vue_bytes` and encode it as 64 lowercase hexadecimal
characters. If no policy passes, exit non-zero, write no policy file, and print
aggregate/per-docset failures.

- [ ] **Step 5: Run pure tests**

```bash
cargo test --test confidence_tests -- --test-threads=1
cargo test --test eval_report_tests -- --test-threads=1
```

Expected: all pass.

- [ ] **Step 6: Commit calibration machinery without enabling it**

```bash
git add src/confidence.rs src/eval.rs examples/retrieval_eval.rs tests/confidence_tests.rs tests/eval_report_tests.rs
git commit -s -m "feat(eval): add deterministic confidence calibration"
```

## Task 10: Freeze and enable `global-confidence-v1`

**Files:**

- Create after successful calibration: `src/confidence_policy_v1.json`
- Modify: `src/confidence.rs`
- Modify: `src/retrieve.rs`
- Modify: `tests/confidence_tests.rs`
- Modify: `tests/retrieve_tests.rs`
- Modify: `tests/eval_tests.rs`
- Modify: `docs/superpowers/evals/2026-07-13-retrieval-confidence-baseline.md`

- [ ] **Step 1: Generate the policy from development only**

```bash
cargo run --release --example retrieval_eval -- \
  --fixtures-dir tests/fixtures/eval \
  --split development \
  --output /tmp/nowdocs-confidence-development.json \
  --code-commit "$(git rev-parse HEAD)" \
  --calibrate-policy-output src/confidence_policy_v1.json
```

Expected: command exits zero and writes a policy with the SHA-256 of the exact
three fixture files. If it exits non-zero, stop this task and keep binary mode.

- [ ] **Step 2: Evaluate the frozen test without modifying the policy**

```bash
cargo run --release --example retrieval_eval -- \
  --fixtures-dir tests/fixtures/eval \
  --split test \
  --policy src/confidence_policy_v1.json \
  --enforce-gates \
  --output /tmp/nowdocs-confidence-test.json \
  --code-commit "$(git rev-parse HEAD)"
```

Expected gates, combined and per docset:

- false reject `<= 0.05`;
- false accept `<= 0.10`;
- decisive coverage `>= 0.80`;
- output Recall@5 `>= 0.90`;
- output MRR `>= 0.70`;
- `nextjs.middleware-matcher.short` is not `no_answer` and matches a labeled
  target.

If any gate fails, delete the uncommitted policy file, keep binary mode, record
the report in the baseline document, and stop. Do not tune against test rows.

- [ ] **Step 3: Write failing runtime-policy tests**

Assert the embedded policy SHA matches fixture SHA, the short regression is
`Borderline` or `Confident`, clear negatives are not `Confident`, and
`NoAnswer` still produces empty output. Add three ignored real-corpus tests in
`tests/eval_tests.rs` named exactly `test_eval_nextjs_real`,
`test_eval_react_real`, and `test_eval_vue_real`; each loads its corresponding
reviewed fixture file, opens the prepared store (`nextjs_real`, `react_real`,
or `vue_real`), evaluates the frozen test split, and asserts the report has the
correct corpus identity and one result row per selected query. The Next.js test
also asserts `nextjs.middleware-matcher.short` is not `no_answer` and matches a
labeled target. Aggregate release gates remain evaluator-owned.

- [ ] **Step 4: Embed and activate the frozen policy**

Load `include_str!("confidence_policy_v1.json")` once through `OnceLock`.
Validate it at initialization. Route `retrieve.rs` through calibrated `decide`.
Do not expose the policy or evidence through MCP.

- [ ] **Step 5: Run focused and real-corpus tests**

```bash
cargo test --test confidence_tests -- --test-threads=1
cargo test --test retrieve_tests -- --test-threads=1
cargo test --release --test eval_tests test_eval_nextjs_real -- --ignored --test-threads=1 --nocapture
cargo test --release --test eval_tests test_eval_react_real -- --ignored --test-threads=1 --nocapture
cargo test --release --test eval_tests test_eval_vue_real -- --ignored --test-threads=1 --nocapture
```

Expected: focused tests pass; all three corpus/report integration tests pass;
the Next.js short-query regression passes.

- [ ] **Step 6: Re-run performance guardrail and update evidence**

Run:

```bash
cargo run --release --example retrieval_eval -- \
  --fixtures-dir tests/fixtures/eval \
  --split test \
  --policy src/confidence_policy_v1.json \
  --enforce-gates \
  --benchmark-runs 30 \
  --output /tmp/nowdocs-confidence-test-performance.json \
  --code-commit "$(git rev-parse HEAD)"
```

This runs 30 warmed measurements per query and docset in one release process.
Expected: no extra normal query and median retrieval overhead versus the
pre-adapter baseline `<= 10%`; record median/p95 and all frozen metrics in the
baseline document.

- [ ] **Step 7: Commit only after every gate passes**

```bash
git add src/confidence_policy_v1.json src/confidence.rs src/retrieve.rs tests/confidence_tests.rs tests/retrieve_tests.rs tests/eval_tests.rs docs/superpowers/evals/2026-07-13-retrieval-confidence-baseline.md
git commit -s -m "feat(retrieve): enable calibrated answer states"
```

## Task 11: Wire three-docset CI and run final repository gates

**Files:**

- Create: `scripts/rebuild-share-docs.py`
- Create: `scripts/ci-prepare-react-fixture.sh`
- Create: `scripts/ci-prepare-vue-fixture.sh`
- Modify: `.github/workflows/eval.yml`
- Modify: `scripts/ci-prepare-nextjs-fixture.sh` only to share deterministic
  helpers without changing its output
- Modify: `docs/superpowers/evals/2026-07-13-retrieval-confidence-baseline.md`

- [ ] **Step 1: Add shell-level fixture preparation checks**

Implement `scripts/rebuild-share-docs.py` with required `--chunks` and
`--output` arguments. It must reject absolute `source_url` values and any path
containing `..`, group rows by `source_url`, sort each group by integer `idx`,
and write joined chunk text beneath the output directory.

Refactor the Next.js preparer to call it, preserving defaults
`nextjs_real`/`seed-crates/tmp/nextjs_rebuilt`. The new wrappers use these exact
mappings:

| Script | Store docset | Share artifact | Rebuilt directory |
|---|---|---|---|
| `ci-prepare-react-fixture.sh` | `react_real` | `seed-crates/share/react-docs/chunks.jsonl` | `seed-crates/tmp/react_rebuilt` |
| `ci-prepare-vue-fixture.sh` | `vue_real` | `seed-crates/share/vue-docs/chunks.jsonl` | `seed-crates/tmp/vue_rebuilt` |

Each wrapper must verify that the share manifest's `source.chunk_count` equals
the number of non-empty JSONL records before ingest, reuse the existing pinned
model cache, place its LanceDB store in the normal cache under the exact store
docset name, and cache the store plus generated manifest under
`$NOWDOCS_FIXTURE_CACHE`. After restore or ingest, run
`cargo run --locked --release --bin nowdocs -- doctor --docset "$DOCSET"`
and fail unless the state is healthy.

For each wrapper, run once with fresh temporary `XDG_CACHE_HOME` and
`NOWDOCS_FIXTURE_CACHE`, save the generated manifest SHA-256, run it again with
the same directories, assert stdout contains `Fixture restored from cache`,
and assert the manifest hash is unchanged.

- [ ] **Step 2: Replace text-grep CI gating with JSON gating**

Update `eval.yml` to:

1. cache Next.js, React, and Vue fixtures with independent keys;
2. run `cargo run --locked --release --example retrieval_eval` once for the
   development report and once for the frozen test report with
   `src/confidence_policy_v1.json`; add `--enforce-gates` to the test command
   only on `workflow_dispatch`, while nightly still fails on command, fixture,
   schema, or permanent regression-test errors;
3. upload both JSON files and the human summary as artifacts;
4. rely on the evaluator's serde validation and non-zero exit status for
   missing fields or failed gates; do not parse metrics with `grep` or `awk`;
5. enforce thresholds on manual dispatch and fail nightly on evaluator,
   fixture, schema, or regression-test errors;
6. remove stale comments claiming the old RRF-only false-positive behavior.

- [ ] **Step 3: Run YAML/script and focused checks**

```bash
bash -n scripts/ci-prepare-nextjs-fixture.sh
bash -n scripts/ci-prepare-react-fixture.sh
bash -n scripts/ci-prepare-vue-fixture.sh
python3 scripts/rebuild-share-docs.py --help
cargo test --test eval_schema_tests -- --test-threads=1
cargo test --test eval_metrics_tests -- --test-threads=1
cargo test --test eval_report_tests -- --test-threads=1
```

Expected: shell parsing and all evaluator tests pass.

- [ ] **Step 4: Run repository gates serially**

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test -- --test-threads=1
cargo deny check
cargo audit --ignore RUSTSEC-2026-0194 --ignore RUSTSEC-2026-0195
```

Expected: all commands exit zero. Then run the complete ignored real-corpus
eval command from the workflow in release mode and confirm all three corpus
reports and policy gates pass.

- [ ] **Step 5: Review the final diff for product invariants**

Confirm MCP remains `2025-11-25` NDJSON, `serve` remains stdio-only, search
still requires explicit docset, all LLM-visible fields are sanitized, registry
packages still contain no vectors, and no query text is logged by default.

- [ ] **Step 6: Commit**

```bash
git add .github/workflows/eval.yml scripts/rebuild-share-docs.py scripts/ci-prepare-nextjs-fixture.sh scripts/ci-prepare-react-fixture.sh scripts/ci-prepare-vue-fixture.sh docs/superpowers/evals/2026-07-13-retrieval-confidence-baseline.md
git commit -s -m "ci(eval): enforce three-docset confidence gates"
```

## Final completion checklist

- [ ] Every task commit has DCO sign-off and a conventional message.
- [ ] Development selection never receives test rows.
- [ ] Frozen test failures are not used for retuning.
- [ ] Normal retrieval executes one hybrid query plus the existing vector fetch;
      evaluator-only channel diagnostics do not leak into runtime.
- [ ] RRF ordering and scores remain compatible before calibrated decision.
- [ ] `borderline` is enabled only when the frozen policy and evaluation-dataset
      SHA match.
- [ ] MCP exposes only discrete state and sanitized chunks, never raw evidence.
- [ ] Healthy `no_answer` never recommends `doctor` or reinstall.
- [ ] Context assembly, MMR position, embedder, registry format, and product
      boundary remain unchanged.
- [ ] No push or merge occurs without maintainer approval.
