# Retrieval Confidence Code-Agent Execution Plan

> **For orchestrators:** REQUIRED SKILLS: use `code-agent-orchestration` for Kimi packages and `superpowers:executing-plans` for Codex-plus-maintainer work. Do not use Codex subagents for code work; code changes are assigned to Kimi.

**Goal:** Execute the approved retrieval-confidence pipeline through reviewed Kimi code packages, maintainer-led data and release gates, and only two conflict-free parallel code waves.

**Architecture:** The technical behavior remains specified by the retrieval-confidence design and detailed pipeline plan. This plan adds ownership, wave dependencies, acceptance records, integration discipline, and a just-in-time durable-spec/ephemeral-prompt handoff lifecycle.

**Tech Stack:** Rust 1.97, LanceDB 0.31, Arrow 58.3, serde, Clap, Git worktrees, GitHub Actions, MCP `2025-11-25` NDJSON.

---

## Sources of truth

- Behavioral design: `docs/superpowers/specs/2026-07-13-retrieval-confidence-pipeline-design.md`.
- Orchestration design: `docs/superpowers/specs/2026-07-13-retrieval-confidence-code-agent-orchestration-design.md`.
- Detailed technical tasks: `docs/superpowers/plans/2026-07-13-retrieval-confidence-pipeline.md`.
- Per-package execution contract: `docs/superpowers/specs/code-agent/<package>.md`.

The per-package spec overrides this document only for its exact file scope and
commands. Neither this plan nor a prompt authorizes a behavioral change outside
the approved retrieval-confidence design.

## Shared integration protocol

- [ ] **Step 1: Freeze the wave base commit before drafting its task specs**

Record `git rev-parse HEAD` in every Wave spec. Create Kimi worktrees from that
exact commit, not from a dirty working tree. Commit the specs on the
orchestration branch before rendering prompts; Kimi reads the committed spec
through its absolute orchestration-worktree path.

- [ ] **Step 2: Render prompts only when a reviewed package is ready to dispatch**

Write each prompt to
`.worktrees/code-agent/<package>/tmp/code-agent-prompts/<package>.md`. Verify
with `git check-ignore -q` that it is ignored. Do not stage it. Delete it after
the package is accepted or rejected.

- [ ] **Step 3: Require a Kimi return report and Codex acceptance record**

Kimi returns commit SHA, changed files, commands with exit status, test counts,
and risks. Codex writes the durable acceptance record under
`docs/superpowers/handoffs/code-agent/<package>.md`, then the maintainer
explicitly approves or rejects integration. No `git cherry-pick`, push, merge,
or dependent wave starts before approval.

## Wave 1 — independent foundation packages

### C01: Evaluation foundation (Kimi, L3)

**Spec:** `docs/superpowers/specs/code-agent/c01-eval-foundation.md`

**Files:**

- Create: `src/confidence.rs`, `tests/eval_schema_tests.rs`,
  `tests/eval_metrics_tests.rs`, `tests/fixtures/eval/schema-smoke.json`
- Modify: `src/lib.rs`, `src/eval.rs`, `tests/eval_tests.rs`, golden fixtures

- [ ] **Step 1: Dispatch only after the C01 spec has maintainer approval**

Kimi implements evaluation schema, matching, validation, graded metrics, Wilson
intervals, and the shared serialized answer-state enum. It must not modify
retrieval, store, CLI, MCP, real-corpus fixtures, or policy code.

- [ ] **Step 2: Accept C01 before C03 or N01 consumes its types**

Run the spec's focused schema/metric/eval commands, inspect the changed-file
list and DCO sign-off, and confirm the task's explicit type and target-matching
contracts. After maintainer approval, integrate the accepted commit and record
the integration commit in the C01 handoff report.

### C02: Behavior-preserving retrieval trace (Kimi, L3)

**Spec:** `docs/superpowers/specs/code-agent/c02-retrieval-trace.md`

**Files:**

- Modify: `src/retrieve.rs`, `tests/retrieve_tests.rs`

- [ ] **Step 3: Dispatch C02 in parallel with C01 after its spec is approved**

Kimi adds trace-only evidence capture. The normal search path, MMR ordering,
answer gate, context assembly, and public MCP output must remain unchanged.

- [ ] **Step 4: Accept C02 before C03 or C04 begins**

Run the spec's trace, retrieval, and smoke tests serially. Inspect that raw
cosine is recomputed from existing vectors and that no LanceDB normalized score
is treated as raw evidence. After maintainer approval, integrate its commit.

## Wave 2 — evaluator plus human data curation

### C03: JSON evaluator and report (Kimi, L3)

**Blocked by:** accepted C01 and C02.

- [ ] **Step 5: Create and review the C03 spec from the accepted Wave 1 base**

The spec covers `src/eval.rs`, `examples/retrieval_eval.rs`, evaluator tests,
and no real evaluation labels. It must use C02 trace data and record query IDs,
not raw query/chunk text.

- [ ] **Step 6: Dispatch and accept C03**

Run report tests and `cargo run --example retrieval_eval -- --help`. Integrate
only after L3 acceptance confirms stage names, corpus identity, policy identity,
and machine-readable report behavior.

### N01: Reviewed relevance suites (Codex + maintainer)

**Blocked by:** accepted C01.

- [ ] **Step 7: Curate three docsets without Kimi delegation**

Add the reviewed Next.js, React, and Vue query families. Each docset has at
least 20 positive and 15 negative development families and the same minimum in
the test split; at least half of negatives in each split are near-domain. Keep
paraphrases of an intent family in one split and verify every source/heading
target against the pinned chunks.

- [ ] **Step 8: Record the maintainer review and run the validator**

Run `cargo test --test eval_schema_tests -- --test-threads=1`. Do not run policy
selection until the maintainer has reviewed targets and the count/split test
passes.

### G01 and N02: Dataset approval and binary baseline (Codex + maintainer)

**Blocked by:** accepted C03 and complete N01.

- [ ] **Step 9: Approve or reject the dataset**

Validate all fixture files, target existence, intent-family split isolation, and
review initials. A rejection returns to N01; it does not authorize Kimi to
invent labels.

- [ ] **Step 10: Capture the binary baseline**

Run the release evaluator on the development split, save the versioned JSON
report outside Git, and add aggregate/per-docset/risk-group/latency values to
the committed baseline document. This baseline is the comparison input for G02.

## Wave 3 and Wave 4 — ranking compatibility then runtime state

### C04: Signal-preserving RRF (Kimi, L4)

**Blocked by:** accepted C02 and G01.

- [ ] **Step 11: Create, review, dispatch, and accept C04 sequentially**

The spec covers `Cargo.toml`, `Cargo.lock`, `src/store.rs`, `src/retrieve.rs`,
and store/retrieve tests. It must exactly reproduce LanceDB RRF row order and
scores while preserving nullable one-based channel ranks. G02 compares the
candidate with the N02 baseline and requires median retrieval overhead at most
10% before integration.

### C05: Binary-compatible answer decisions (Kimi, L3)

**Blocked by:** integrated C04 and passed G02.

- [ ] **Step 12: Create, review, dispatch, and accept C05**

The spec adds `QueryEvidence`, binary decision reasons, and `SearchResult`
answer state after MMR and before neighbor expansion. The state is still binary
at this point; `borderline` remains disabled.

## Wave 5 — parallel public contract and calibration engine

### C06: MCP and smoke contracts (Kimi, L3)

**Blocked by:** accepted C05.

- [ ] **Step 13: Create, review, dispatch, and accept C06**

The code package changes `src/tools.rs`, `src/smoke.rs`, `src/main.rs`, and
their tests. It exposes only discrete sanitized state, treats healthy no-answer
as normal MCP output, and preserves the specified smoke exit behavior.

### C07: Calibration engine (Kimi, L4)

**Blocked by:** accepted C03, C04, C05, and G01.

- [ ] **Step 14: Create, review, dispatch, and accept C07 in parallel with C06**

The spec covers policy types, deterministic five-fold grouped calibration,
development-only selection, dataset hashing, and evaluator policy arguments.
It must not enable runtime calibrated decisions or consume frozen test rows.

### N03 and N04: Documentation and calibration review (Codex + maintainer)

- [ ] **Step 15: Update the user-facing migration text after C06 acceptance**

Document complete smoke JSON for no-answer, no repair hint for a healthy
no-answer, and the retained operational-error path. Do not alter code while
reviewing wording.

- [ ] **Step 16: Run and approve development-only calibration after C07 acceptance**

Review the out-of-fold results, selected global policy, dataset SHA, and
aggregate/per-docset gates. If no policy passes, record binary fallback and stop
C08/C09.

## Wave 6 and Wave 7 — frozen policy and CI

### G03 and C08: Frozen-test approval then runtime activation

- [ ] **Step 17: Run frozen test before dispatching C08**

Evaluate the generated policy only on the frozen test split. Check aggregate and
per-docset false-reject, false-accept, decisive-coverage, Recall@5, MRR, and
the Next.js `middleware matcher` regression. A failure stops the wave and does
not authorize retuning.

- [ ] **Step 18: Create, review, dispatch, and accept C08 only after G03 passes**

The L4 spec embeds the approved policy, validates its evaluation-dataset SHA,
activates calibrated decision after MMR, and keeps raw evidence out of MCP.

### C09: Three-docset JSON-gated CI (Kimi, L4)

**Blocked by:** accepted C03 and C08.

- [ ] **Step 19: Create, review, dispatch, and accept C09**

The spec adds generic safe share-artifact reconstruction, React/Vue fixture
preparers, and JSON-based eval workflow gates. It must not use text-grep metric
parsing and must preserve Next.js fixture output.

### G04: Final maintainer acceptance

- [ ] **Step 20: Run final repository and product-invariant gates**

Run formatter, clippy, serial tests, `cargo deny check`, documented `cargo audit`,
and the complete release evaluator. Confirm stdio-only MCP serving, explicit
docset, sanitizer coverage, no registry vectors, no score exposure, context/MMR
position, and no push or merge without maintainer approval.
