# C02 Trace Feedback Remediation

## Identity

- Owner: Kimi
- Difficulty: L2
- Wave: feedback remediation wave 1 (parallel with C01R)
- Base commit: `3033dd52e83863df1591172f2d7a8498e0f3611f`
- Worktree: `/Users/kg/Projects/nowdocs/.worktrees/code-agent/c02-retrieval-trace`
- Branch: `codex/retrieval-confidence-c02-trace`
- Blocked by: none
- Unblocks: C03 evaluation reporting

## Goal and non-goals

Make trace-only pre-MMR cosine ordering deterministic for malformed candidate
vectors, and remove the unsigned merge commit that blocks the PR's DCO gate.

Do not change normal retrieval ranking, the answer gate, trace API shape, or
candidate inclusion rules except to omit non-finite cosine observations from the
trace's numeric distribution.

## Exact scope

### Allowed files

- `src/retrieve.rs`
- `tests/retrieve_tests.rs`

### Forbidden files and behavior

- All other tracked files.
- Do not change MMR ordering, gate decisions, thresholds, `SearchHit`, or
  storage behavior.
- Do not merge, push, or use `--no-verify`.

## Contract

1. `RetrievalTrace.pre_mmr_top_cosines` contains only finite recomputed raw
   cosines, sorted in descending total order with `f32::total_cmp`.
2. A zero-norm or otherwise malformed candidate vector must not make trace
   ordering unspecified. It may remain in normal retrieval's existing fallback
   behavior; only its non-finite cosine is omitted from this diagnostic vector.
3. With finite vectors, trace-enabled and trace-disabled rank/gate outputs stay
   identical, and `pre_mmr_top_cosines` continues to derive from the fused pool
   rather than MMR ordering.
4. Rebase the PR branch onto current `origin/main` without a merge commit.
   Every resulting PR-only commit must retain a valid DCO sign-off. This is a
   history repair, not permission to merge or push.

## TDD and verification

1. First update only the assigned worktree with `git pull --ff-only`, then
   rebase it onto `origin/main`. Stop if the rebase has conflicts.
2. Add a failing regression test using an existing synthetic candidate setup
   containing a zero-norm vector. Assert trace creation completes and its
   `pre_mmr_top_cosines` are finite and descending. Run:

   ```bash
   cargo test --test retrieve_tests trace -- --test-threads=1
   ```

   Expected red result: the trace contains a non-finite value or fails the
   deterministic order assertion.
3. Implement the minimal trace-only filter and `total_cmp` ordering, then run:

   ```bash
   cargo test --test retrieve_tests trace -- --test-threads=1
   cargo test --test retrieve_tests -- --test-threads=1
   cargo test --test smoke_tests -- --test-threads=1
   cargo fmt --check
   cargo clippy --locked --all-targets -- -D warnings
   git diff --check
   ```

Expected green evidence: all named commands exit 0 and the existing
behavior-preservation test still passes.

## Commit and report

Commit only the allowed file changes using:

```bash
git commit -s -m "fix(retrieve): stabilize trace cosine ordering"
```

Do not push. Return the resulting commit SHA, rebased branch tip, changed-file
list, exact commands and exit statuses, test counts, and any remaining risk.
