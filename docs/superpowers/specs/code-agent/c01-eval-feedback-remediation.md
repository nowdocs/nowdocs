# C01 Eval Feedback Remediation

## Identity

- Owner: Kimi
- Difficulty: L3
- Wave: feedback remediation wave 1 (parallel with C02R)
- Base commit: `807c98510a25bcc92a619154187ab5ebb9869ca9`
- Worktree: `/Users/kg/Projects/nowdocs/.worktrees/code-agent/c01-eval-foundation`
- Branch: `codex/retrieval-confidence-c01-eval`
- Blocked by: none
- Unblocks: C03 evaluation reporting

## Goal and non-goals

Make C01's ranking metric and Wilson estimate robust against the accepted PR
feedback, and remove the unsigned merge commit that blocks its DCO gate.

Do not alter public schemas, retrieval code, fixture labels, thresholds, or
metric semantics beyond standard nDCG@k truncation and the defined invalid-input
guard for `rate_estimate`.

## Exact scope

### Allowed files

- `src/eval.rs`
- `tests/eval_metrics_tests.rs`

### Forbidden files and behavior

- All other tracked files.
- Do not change `EvalQuery`, relevance matching, gain formula, Precision, MRR,
  Recall, public JSON names, or test fixtures.
- Do not merge, push, or use `--no-verify`.

## Contract

1. `compute_ranking_metrics(..., k)` must calculate nDCG@k with both DCG and
   IDCG limited to the first `k` ranked positions. Sort ideal target gains
   descending and truncate the ideal list to `k` before discounting. A result
   that realizes the top `k` target gains at the top `k` ranks must have
   `ndcg == 1.0`, even when more than `k` targets are labeled.
2. `rate_estimate(count, total)` preserves results for valid inputs
   `count <= total`. For invalid `count > total`, retain a `debug_assert!` for
   the violated invariant and compute the saturated `count = total` rate and
   finite Wilson bounds; `total == 0` keeps its existing zero-valued result.
3. Rebase the PR branch onto current `origin/main` without a merge commit.
   Every resulting PR-only commit must retain a valid DCO sign-off. This is a
   history repair, not permission to merge or push.

## TDD and verification

1. First update only the assigned worktree with `git pull --ff-only`, then
   rebase it onto `origin/main`. Stop if the rebase has conflicts.
2. Add a failing regression test where more than `k` targets exist and the
   returned top `k` hits realize the highest possible gains; assert nDCG@k is
   1.0. Run:

   ```bash
   cargo test --test eval_metrics_tests ndcg -- --test-threads=1
   ```

   Expected red result: the ideal-perfect-at-k case is below 1.0.
3. Add a failing `rate_estimate(total + 1, total)` test asserting finite bounds
   and a saturated rate of 1.0. Run the focused test by name.
4. Implement the minimal changes, then run:

   ```bash
   cargo test --test eval_metrics_tests -- --test-threads=1
   cargo test --test eval_schema_tests -- --test-threads=1
   cargo fmt --check
   cargo clippy --locked --all-targets -- -D warnings
   git diff --check
   ```

Expected green evidence: all named commands exit 0; existing metric behavior
for valid inputs remains covered by the full metrics suite.

## Commit and report

Commit only the allowed file changes using:

```bash
git commit -s -m "fix(eval): correct ndcg at k"
```

Do not push. Return the resulting commit SHA, rebased branch tip, changed-file
list, exact commands and exit statuses, test counts, and any remaining risk.
