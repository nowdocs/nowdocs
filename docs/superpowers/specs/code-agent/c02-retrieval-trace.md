# Code-Agent Task Spec: C02 Behavior-Preserving Retrieval Trace

## Identity

- Owner: Kimi
- Difficulty: L3
- Wave: 1
- Implementation base commit: `483fe0f47826b9bb3a7c2d1840090f0fd7df884d`
- Worktree: `/Users/kg/Projects/nowdocs/.worktrees/code-agent/c02-retrieval-trace`
- Branch: `codex/code-agent/c02-retrieval-trace`
- Blocked by: none
- Unblocks: C03 and C04

The implementation branch starts from the implementation base commit. This
spec is committed on the orchestration branch after the worktree is created and
is read through its absolute path:
`/Users/kg/Projects/nowdocs/.worktrees/retrieval-pipeline-confidence-design/docs/superpowers/specs/code-agent/c02-retrieval-trace.md`.

## Goal and non-goals

Add an evaluation-only retrieval trace without changing normal search results,
the current binary answer gate, MMR behavior, token packing, or MCP/CLI output.
The trace must make later evaluation able to inspect fused and MMR candidates,
pre-MMR raw-cosine distribution, and the current gate outcome.

Do not add channel ranks, RRF adapters, answer states, calibrated policy,
evaluator CLI, fixtures, dependencies, store queries, smoke/MCP changes, or
documentation changes. Those belong to later packages.

## Exact scope

### Allowed files

- `src/retrieve.rs`
- `tests/retrieve_tests.rs`

### Forbidden files and behavior

- `src/store.rs`, `src/eval.rs`, `src/confidence.rs`, `src/smoke.rs`,
  `src/tools.rs`, `src/main.rs`, `Cargo.toml`, and `Cargo.lock` are forbidden.
- Do not change `MIN_ANSWER_COSINE`, `DUAL_RANK1_RRF`, `MMR_LAMBDA`,
  `MMR_URL_PENALTY`, candidate-pool sizing, MMR ordering, neighbor-window order,
  token packing, or `SearchResult` fields.
- Do not make a second vector, FTS, hybrid, or neighbor fetch in normal search.
- Do not expose trace data through MCP, smoke JSON, human output, logs, or
  public `SearchResult`.

## Contract

Add these public types in `src/retrieve.rs`:

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

Refactor the current `search` body into one shared implementation with a trace
flag. Preserve this public function exactly:

```rust
pub fn search(
    docset: &str,
    query: &str,
    max_tokens: Option<u32>,
    top_k: Option<u32>,
) -> Result<SearchResult>
```

Add:

```rust
pub fn search_with_trace(
    docset: &str,
    query: &str,
    max_tokens: Option<u32>,
    top_k: Option<u32>,
) -> Result<(SearchResult, RetrievalTrace)>
```

`search` calls the shared implementation with tracing disabled. When tracing is
disabled, do not clone trace metadata or allocate a trace vector. When tracing
is enabled, use only the candidate vectors already fetched for existing MMR to
recompute raw query-to-candidate cosine. Never use LanceDB's query-local
normalized `_distance` or `_score` as raw evidence.

`pre_mmr_top_cosines` is the descending raw-cosine sequence across the complete
fused candidate pool before MMR. The gate cosine remains the selected post-MMR
top hit's raw cosine, exactly as current behavior. At this wave, every
`dense_rank` and `lexical_rank` is `None`; C04 later supplies rank evidence.

Extract a pure helper named:

```rust
pub fn rank_and_gate_candidates(
    query_vector: &[f32],
    candidates: Vec<SearchHit>,
    vectors: &HashMap<u32, Vec<f32>>,
    top_k: usize,
    trace: bool,
) -> RankedGateResult
```

It must call the current MMR function and current answer-gate logic without
changing their semantics. `gate_passed` is true exactly when the current answer
gate returns at least one hit. A normal no-answer still returns the existing
empty `SearchResult` through `search`; `search_with_trace` still returns the
trace for that no-answer decision.

## TDD and verification

1. Add pure tests in `tests/retrieve_tests.rs` with synthetic candidate vectors.
The test compares selected chunk IDs, not `SearchHit` values, because
`SearchHit` does not implement `PartialEq`.

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

Add a second test whose MMR runner-up differs from the second-best fused raw
cosine, and assert the trace's first two `pre_mmr_top_cosines` values use the
fused pool rather than the MMR result.

2. Run:

```bash
cargo test --test retrieve_tests trace -- --test-threads=1
```

Expected red result: compilation errors because `rank_and_gate_candidates`,
`RetrievalTrace`, and trace fields do not exist.

3. Implement the contract without changing the existing `search` validation,
manifest loading, embedder loading, hybrid query, vector fetch, MMR, answer
gate, neighbor window, or token packing code path.

4. Run focused and boundary checks:

```bash
cargo test --test retrieve_tests trace -- --test-threads=1
cargo test --test retrieve_tests -- --test-threads=1
cargo test --test smoke_tests -- --test-threads=1
cargo fmt --check
```

Expected green evidence: all commands exit zero; the existing MMR order and
binary answer-gate tests still pass; no trace appears in normal public output;
normal search still performs one hybrid query plus the existing vector fetch.

## Commit and return report

Commit only allowed files:

```bash
git add src/retrieve.rs tests/retrieve_tests.rs
git commit -s -m "refactor(retrieve): add behavior-preserving trace"
```

Return the commit SHA, exact changed-file list, every verification command with
exit status, focused test counts, and any risk to Codex. Do not create a
handoff-report file; Codex records acceptance after review.
