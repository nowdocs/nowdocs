# C10 — Evaluation suite schema gates

## Identity

- Owner: Kimi
- Difficulty: L2
- Wave: W2 gate completion; sequential after C03 and N01 data
- Base commit: `9fcf68d17da78a91a6d14a4012c0f524cdf3ce43`
- Worktree: `/Users/kg/Projects/nowdocs/.worktrees/code-agent/c10-eval-suite-gates`
- Branch: `codex/code-agent/c10-eval-suite-gates`
- Blocked by: C03 accepted; N01 fixture commit `9fcf68d`
- Unblocks: G01 dataset approval and N02 baseline capture

## Goal and non-goals

Make the reviewed Next.js, React, and Vue evaluation suites self-validating in
CI.  Each suite must be loaded and validated independently, then checked for
the minimum development/test positive and negative intent-family counts and
the required near-domain-negative share.

This package does **not** create, revise, or tune labels; change targets,
retrieval, answer-gate behavior, evaluator reports, CI workflows, or public
protocols; or run a corpus-backed baseline.  It must not use the combined
fixture-directory loader for the three real suites because the legacy
`schema-smoke.json` intentionally reuses a Next.js intent family.

## Exact scope

### Allowed files

- `src/eval.rs`
- `tests/eval_schema_tests.rs`

### Forbidden files and behavior

- Do not change `tests/fixtures/eval/*.json`, including `schema-smoke.json`.
- Do not change any other production source, Cargo files, examples, scripts,
  CI, docs, MCP/CLI/smoke contracts, retrieval constants, or model/store code.
- Preserve `load_fixture_suite` behavior: it continues to concatenate sorted
  JSON files and validates the combined suite.
- Do not introduce dependencies or filesystem mutation beyond test reads.

## Contract

Add a public `load_eval_file(path: impl AsRef<Path>) -> Result<Vec<EvalQuery>>`
in `src/eval.rs`.  It reads one JSON array, returns contextual read/parse
errors that name the path, validates that array with `validate_suite`, and
returns the records only when validation succeeds.  Refactor
`load_fixture_suite` to share this single-file parsing behavior where possible,
without changing its sorted-directory or combined-suite validation contract.

In `tests/eval_schema_tests.rs`, add a focused real-suite test which loads
each of `nextjs.json`, `react.json`, and `vue.json` separately through
`load_eval_file`. For every file it must assert:

1. all records use only the corresponding docset name;
2. `validate_suite` succeeds;
3. each split has at least 20 distinct positive `intent_family` values and at
   least 15 distinct negative `intent_family` values, where both negative
   query classes count as negative;
4. at least 50% of the distinct negative intent families in each split are
   `near_domain_negative`; and
5. a `(docset, intent_family)` appears in only one split (the existing
   validator may enforce this; the test must exercise it through the real
   suites).

Counts are by distinct intent family, never by query row.  Test helpers may
stay private to the integration test.  Do not assert an exact record count or
test corpus target existence here; label curation and G01 retain those review
responsibilities.

## TDD and verification

1. First extend `tests/eval_schema_tests.rs` with the real-suite gate and an
   import for `load_eval_file`.  Run:

   ```bash
   cargo test --test eval_schema_tests real_suites_meet_minimum_family_counts -- --test-threads=1
   ```

   Expected RED: compilation fails because `load_eval_file` does not yet
   exist.
2. Implement the smallest allowed loader/refactor in `src/eval.rs`, then make
   the gate green.
3. Run all acceptance checks:

   ```bash
   cargo test --test eval_schema_tests -- --test-threads=1
   cargo test --test eval_metrics_tests -- --test-threads=1
   cargo test --test eval_report_tests -- --test-threads=1
   cargo fmt --check
   cargo clippy --locked --all-targets -- -D warnings
   git diff --check
   ```

Expected GREEN: all focused tests pass, existing schema-smoke validation still
passes, and format/lint/diff are clean.  No installed corpus or embedder is
required.

## Commit and report

Commit only the allowed files using:

```bash
git commit -s -m "test(eval): gate reviewed suite composition"
```

Do not push or merge. Return the commit SHA, changed-file list, each command
and exit status (including RED evidence), test counts, confirmation that no
corpus/model was needed, and remaining risks. The orchestrator will inspect
the diff and run acceptance before proposing integration.
