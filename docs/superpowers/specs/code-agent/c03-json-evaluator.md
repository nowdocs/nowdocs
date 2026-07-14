# C03 — Versioned JSON retrieval evaluator

## Identity

- Owner: Kimi
- Difficulty: L3
- Wave: W2 (parallel with N01 only; no concurrent code package)
- Base commit: `2539729d317def57fee4e30cb6cea8172f1d02aa`
- Worktree: `/Users/kg/Projects/nowdocs/.worktrees/code-agent/c03-json-evaluator`
- Branch: `codex/code-agent/c03-json-evaluator`
- Blocked by: C01, C02 (accepted and merged)
- Unblocks: G01, N02, C07, C09

## Goal and non-goals

Implement an internal example binary which evaluates a versioned `EvalQuery`
suite against installed corpora and writes a stable, machine-readable JSON v1
report.  It must collect retrieval-stage and answer-state metrics without
exposing query text, chunk text, scores, or traces in the report.

This package does **not** create or edit real evaluation data, change retrieval
ranking/gates, add calibration, modify MCP/CLI/smoke output, modify the store,
or add CI.  Keep legacy `evaluate`, `EvalReport`, and golden-fixture callers
behavior-compatible.

## Exact scope

### Allowed files

- `src/eval.rs`
- `examples/retrieval_eval.rs`
- `tests/eval_report_tests.rs`
- `tests/eval_tests.rs` (only migration/regression coverage required by this package)

### Forbidden files and behavior

- No `Cargo.toml`/`Cargo.lock` change; use existing direct dependencies only.
- No changes under `src/store.rs`, `src/retrieve.rs`, `src/confidence.rs`,
  `src/tools.rs`, `src/smoke.rs`, `src/main.rs`, `.github/`, `docs/`, or
  `tests/fixtures/eval/`.
- No change to `MIN_ANSWER_COSINE`, RRF, MMR, context assembly, public CLI/MCP
  protocol, registry/vector invariant, or model selection.
- Do not serialize raw query text, raw chunk text, raw cosine, RRF values, or
  internal retrieval traces into reports.

## Contract

### Report shapes and serialization

In `src/eval.rs`, add serde-serializable public v1 report types:

- `EvalReportV1`
- `CorpusIdentity`
- `StageMetrics`
- `AnswerStateMetrics`
- `QueryReport`
- `ReportDecisionReason` with exactly the snake-case variants:
  `no_candidates`, `current_gate_pass`, `current_gate_reject`,
  `calibrated_confident`, `calibrated_borderline`, and
  `calibrated_no_answer`.

`EvalReportV1` must serialize these required top-level fields with stable
names: `schema_version` (the integer `1`), `corpora`, `stages`,
`answer_states`, `by_docset`, `by_query_form`, `by_query_class`, and `queries`.
`stages` must contain exactly `dense_at_40`, `fts_at_40`, `fused_at_40`,
`mmr_at_5`, and `output_at_5`.  Each stage reports its `k`, Recall, MRR,
nDCG, and Precision.  State reports include counts and Wilson estimates for
false reject, false accept, positive borderline, negative borderline, and
decisive coverage.  The same stage/state structure is emitted by docset,
query form, and query class.

`QueryReport` contains only the query ID, docset, split, form, class, matched
target indexes/grades, per-stage ranks, answer state, and decision reason.
Its type and serialization must have no query text or raw chunk content field.

### Evaluation behavior

Add a public evaluator entry point that:

1. Loads and validates all fixture records before opening an embedder or store.
2. Selects only the requested `EvalSplit`.
3. Uses `Store::vector_search` and `Store::fts_search` **only in evaluator
   code** for native channel Recall@40; it must not compare LanceDB normalized
   values as cross-query relevance scores.
4. Uses `retrieve::search_with_trace` for fused@40, MMR@5, and output@5
   stage evidence.  Compute relevance by existing `hit_matches_target` rules.
5. Records only query IDs in rows and uses the current binary answer behavior
   with policy identity `binary-current-gate-v1`.
6. Captures corpus identity: code commit, command/retrieval parameters,
   OS/architecture, manifest SHA-256, document version, chunk count, model
   revision, and model hash.  If an identity value is unavailable from the
   existing installed metadata, return a contextual error rather than inventing
   it.

The example `retrieval_eval` is internal (not a `nowdocs` subcommand) and must
support:

```text
--fixtures-dir tests/fixtures/eval
--split development|test
--output /absolute/path/report.json
--code-commit <40-character git commit>
--benchmark-runs <positive integer, default 1>
```

Validate `--code-commit` as a 40-character hexadecimal SHA.  With
`--benchmark-runs > 1`, warm each query once unmeasured, repeat it in-process,
and report median/p95 retrieval latency.  `--help` must exit successfully
without loading a model or opening a store.  Write JSON atomically enough that
an existing successful report is not replaced by partial output (write a
temporary sibling then rename).

## TDD and verification

1. Add `tests/eval_report_tests.rs` first.  Include a fixture report test
   named `report_v1_serializes_machine_readable_contract` that asserts
   `schema_version == 1`, arrays for `corpora`/`queries`, objects for
   `stages.fused_at_40` and `answer_states.false_reject`, and asserts no
   serialized query-report field is named `query` or `text`.
2. Add tests for exact required stage-key set, report-only reason snake-case
   serialization, and invalid CLI `--code-commit` rejection without model
   initialization.  Run:

   ```bash
   cargo test --test eval_report_tests -- --test-threads=1
   ```

   Expected RED: compile failure because the new report/evaluator types do not
   exist.
3. Implement the smallest coherent report/evaluator surface in the allowed
   files only.
4. Run focused and boundary checks:

   ```bash
   cargo test --test eval_report_tests -- --test-threads=1
   cargo test --test eval_schema_tests -- --test-threads=1
   cargo test --test eval_metrics_tests -- --test-threads=1
   cargo test --test eval_tests -- --test-threads=1
   cargo test --test retrieve_tests -- --test-threads=1
   cargo run --example retrieval_eval -- --help
   cargo fmt --check
   cargo clippy --locked --all-targets -- -D warnings
   git diff --check
   ```

Expected GREEN: report tests pass, legacy evaluator/retrieval behavior remains
green, example help lists all five arguments and succeeds without model load,
and formatter/clippy/diff checks are clean.

## Commit and report

Commit only allowed files using:

```bash
git commit -s -m "feat(eval): add versioned JSON evaluator"
```

Do not push or merge.  Return the commit SHA, changed-file list, each command
and exit status (including red evidence), test counts, whether any installed
corpus was required, and remaining risks.  The orchestrator writes the durable
acceptance record after review.
