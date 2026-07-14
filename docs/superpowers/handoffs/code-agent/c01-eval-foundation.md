# C01 Evaluation Foundation Acceptance Record

- Status: `needs-review`
- Pull request: https://github.com/nowdocs/nowdocs/pull/57
- Branch: `codex/retrieval-confidence-c01-eval`
- Commits: `5e24e3ed423b68ffd445a77bff1c332977d4fb33`,
  `4677053afa758e2b8ba41eba9ba68b420975be69`
- Worktree: `/Users/kg/Projects/nowdocs/.worktrees/code-agent/c01-eval-foundation`
- Scope: `src/confidence.rs`, `src/eval.rs`, `src/lib.rs`, evaluation tests, and
  schema smoke fixture only.
- Scope respected: yes.

## Codex review

The initial implementation counted duplicate chunks for Precision and divided
by requested `k`. That conflicts with the approved one-gain-per-target rule and
the requirement to divide by returned primary hits. The follow-up commit changes
Precision to count only first gain-bearing hit ranks and divide by actual primary
hits, with duplicate and short-output regression tests.

## Verification evidence

- `cargo test --test eval_schema_tests -- --test-threads=1`: 15 passed.
- `cargo test --test eval_metrics_tests -- --test-threads=1`: 14 passed.
- `cargo test --test eval_tests -- --test-threads=1`: 5 passed, 4 ignored real
  embedder tests.
- `cargo test --test retrieve_tests --no-run`: passed.
- `cargo test -- --test-threads=1`: passed locally after the follow-up fix.
- `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo deny check`, and
  documented `cargo audit`: passed.

## Remaining gate

Wait for PR review and CI. C03 and N01 do not consume this code until the
maintainer approves integration.
