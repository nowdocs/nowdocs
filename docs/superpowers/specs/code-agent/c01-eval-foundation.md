# Code-Agent Task Spec: C01 Evaluation Foundation

## Identity

- Owner: Kimi
- Difficulty: L3
- Wave: 1
- Implementation base commit: `483fe0f47826b9bb3a7c2d1840090f0fd7df884d`
- Worktree: `/Users/kg/Projects/nowdocs/.worktrees/code-agent/c01-eval-foundation`
- Branch: `codex/code-agent/c01-eval-foundation`
- Blocked by: none
- Unblocks: C03 and N01

The implementation branch starts from the implementation base commit. This
spec is committed on the orchestration branch after the worktree is created and
is read through its absolute path:
`/Users/kg/Projects/nowdocs/.worktrees/retrieval-pipeline-confidence-design/docs/superpowers/specs/code-agent/c01-eval-foundation.md`.

## Goal and non-goals

Create the reusable evaluation foundation: versioned labeled-query schema,
target matching, fixture validation, graded ranking/state metrics, Wilson
intervals, and the serialized `AnswerState` label used by later runtime work.

Do not change retrieval, LanceDB queries, RRF/MMR behavior, smoke, MCP, real
three-docset labels, evaluator CLI/report execution, calibration policy, CI,
or public documentation.

## Exact scope

### Allowed files

- `src/confidence.rs`
- `src/lib.rs`
- `src/eval.rs`
- `tests/eval_schema_tests.rs`
- `tests/eval_metrics_tests.rs`
- `tests/eval_tests.rs`
- `tests/fixtures/eval/schema-smoke.json`
- `tests/fixtures/golden/golden.json`
- `tests/fixtures/golden/negative.json`

### Forbidden files and behavior

- `src/retrieve.rs`, `src/store.rs`, `src/smoke.rs`, `src/tools.rs`, and
  `src/main.rs` are forbidden.
- Do not create `tests/fixtures/eval/nextjs.json`, `react.json`, or `vue.json`;
  label curation belongs to N01.
- Do not add a dependency or change `Cargo.toml`/`Cargo.lock`.
- Do not expose scores or state through MCP.
- Do not remove the old `GoldenQuery` API until every existing caller compiles.

## Contract

Add `src/confidence.rs`, registered by `src/lib.rs`, with:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswerState {
    Confident,
    Borderline,
    NoAnswer,
}
```

Add to `src/eval.rs` these serde-enabled, cloneable, debug-printable, equality
types:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvalSplit { Development, Test }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QueryForm { Short, NaturalLanguage, Verbose, KeywordHeavy }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QueryClass { Positive, NearDomainNegative, CrossDomainNegative }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct RelevanceTarget {
    pub source_url: String,
    #[serde(default)]
    pub heading_path_prefix: Option<String>,
    pub grade: u8,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
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

`hit_matches_target(&ResultChunk, &RelevanceTarget) -> bool` requires exact
`source_url` equality. If the target has a heading prefix, normalize it with the
existing `crate::chunker::normalize_heading_path`; it matches only an exact
normalized heading or a descendant separated by `" > "`. `Exports > Match`
must not match `Exports > Matcher`.

`validate_suite(&[EvalQuery]) -> anyhow::Result<()>` rejects duplicate query
IDs, a `(docset, intent_family)` in both splits, grades outside `1..=2`, empty
ID/docset/query/family strings, a positive query without a target, and either
negative class with a target. It accepts multiple targets for a positive query.

Add:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RankingMetrics {
    pub k: usize,
    pub relevant_targets_found: usize,
    pub recall: f32,
    pub mrr: f32,
    pub ndcg: f32,
    pub precision: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RateEstimate {
    pub count: usize,
    pub total: usize,
    pub rate: f32,
    pub lower: f32,
    pub upper: f32,
}
```

Implement `compute_ranking_metrics` with one gain per labeled target even if
several returned chunks match it. Use gain `(2^grade) - 1`, discount
`1 / log2(rank + 1)`, first relevant target for MRR, and zero-valued fields for
empty denominators. Implement `rate_estimate` as a Wilson 95% interval with
`z = 1.959963984540054`.

## TDD and verification

1. Add `tests/eval_schema_tests.rs` with a target-descendant test, partial
   heading rejection, duplicate ID rejection, cross-split intent-family
   rejection, invalid grade rejection, positive-without-target rejection, and
   negative-with-target rejection.

```rust
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
        heading_path_prefix: Some("Exports > Matcher".into()),
        grade: 2,
    };
    assert!(hit_matches_target(&hit, &target));
    assert!(!hit_matches_target(&hit, &RelevanceTarget {
        heading_path_prefix: Some("Exports > Match".into()),
        ..target
    }));
}
```

2. Run:

```bash
cargo test --test eval_schema_tests -- --test-threads=1
```

Expected red result: compilation errors because `EvalQuery`, `RelevanceTarget`,
`hit_matches_target`, and `validate_suite` do not exist.

3. Add `tests/eval_metrics_tests.rs` covering duplicate target gain, Recall,
Precision, MRR ranks 1 and 5, ideal nDCG ordering, empty metrics, false-reject,
false-accept, borderline rate, decisive coverage, and Wilson containment.

```rust
#[test]
fn wilson_interval_contains_observed_rate() {
    let estimate = rate_estimate(1, 20);
    assert!((estimate.rate - 0.05).abs() < 1e-6);
    assert!(estimate.lower <= estimate.rate);
    assert!(estimate.upper >= estimate.rate);
}
```

4. Run:

```bash
cargo test --test eval_metrics_tests -- --test-threads=1
```

Expected red result: compilation errors because the ranking and rate metric
types/functions do not exist.

5. Implement the contract above. Migrate the toy JSON loader and current
`GoldenQuery` test constructions only as needed for compilation; retain
compatibility wrappers until all callers use the new shapes.

6. Run focused and boundary checks:

```bash
cargo test --test eval_schema_tests -- --test-threads=1
cargo test --test eval_metrics_tests -- --test-threads=1
cargo test --test eval_tests -- --test-threads=1
cargo test --test retrieve_tests --no-run
cargo fmt --check
```

Expected green evidence: every command exits zero; no production retrieval file
changes; serialized `AnswerState` values are `confident`, `borderline`, and
`no_answer`.

## Commit and return report

Commit only allowed files:

```bash
git add src/confidence.rs src/lib.rs src/eval.rs tests/eval_schema_tests.rs tests/eval_metrics_tests.rs tests/eval_tests.rs tests/fixtures/eval/schema-smoke.json tests/fixtures/golden/golden.json tests/fixtures/golden/negative.json
git commit -s -m "feat(eval): add evaluation foundation"
```

Return the commit SHA, exact changed-file list, every verification command with
exit status, focused test counts, and any risk to Codex. Do not create a
handoff-report file; Codex records acceptance after review.
