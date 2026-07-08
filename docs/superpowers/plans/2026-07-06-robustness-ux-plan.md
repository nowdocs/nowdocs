# nowdocs Robustness + UX hardening implementation plan

> **Date:** 2026-07-06
> **Spec:** `docs/superpowers/specs/2026-07-06-robustness-ux.md`
> **Goal:** turn the completed Wave 0-5 engine into a product that is resilient to partial failure and easy for new users to verify/debug.

---

## 0. Global rules for this track

- Keep MCP stdio-only; do not add host/port flags.
- Keep embedder model/spec fields frozen.
- Keep registry URL allowlist and share text-only invariant.
- Prefer additive CLI subcommands over breaking existing commands.
- Every task follows TDD: failing test → implementation → passing test → commit.
- Every user-facing error added in this track must include a stable code or a clear next-step hint.
- Do not run large model/download E2E tests by default unless already accepted in CI; mark expensive checks ignored or put them behind explicit scripts.
- This PR tracks the remaining implementation slice: R2 → R3 → U1. The archive-validation foundation is handled separately and is a prerequisite for R2.
- `doctor --repair` is staging/rollback cleanup only in this track; it must not delete active docsets.
- `doctor --json` is experimental until v1.0 except for the top-level status/checks shape.
- `smoke` is a real retrieval test and may load/download the model through the existing embedder path.
- Transactional install/update must preserve a valid active state; perfect cross-platform atomic directory replacement is not required in the first implementation if fail-safe semantics hold.
- Every stage handoff prompt must state estimated difficulty and expected review risk.
- Every implementation agent must work in an independent git worktree/branch for that stage; do not implement from another agent's dirty worktree or shared branch.

---

## 1. Prerequisite — external archive-validation foundation

Archive validation + error taxonomy are intentionally not specified in this PR. Treat that separate implementation as an external prerequisite that must be merged before R2 starts. R2 may rely on an existing validation path that rejects unsafe archives before active cache writes.

---

## 2. Milestone R2 — Transactional install/update with rollback

**Purpose:** prevent partial active cache state.

**Files likely touched:**

- `src/registry.rs`
- `src/cache.rs`
- `tests/registry_tests.rs`
- `tests/cli_tests.rs`

**Tasks:**

- [x] R2.1 Add staging path helpers.
  - `cache::staging_root()`.
  - unique staging path for docset + pid + timestamp.
  - tests ensure staging stays under cache root.

- [x] R2.2 Implement install-to-staging.
  - Download/open archive into staging.
  - Validate archive before writing active paths.
  - Materialize manifest/license/chunks/store under staging.
  - Reopen staged manifest/store for final verification.

- [x] R2.3 Implement atomic promote.
  - Promote staged docset into active cache only after verification.
  - If replacing, keep rollback until new active is verified.
  - Use rename where possible; allow copy-verify-swap on Windows if directory replacement is not atomic.
  - Clean rollback on success.

- [x] R2.4 Implement update rollback semantics.
  - Failed update preserves old active manifest/store.
  - Tests simulate bad replacement archive after a good install.

- [x] R2.5 Add stale staging detection primitive.
  - No deletion yet unless R4 implements cache cleanup.

**Verification:**

- `cargo test registry_tests -- --test-threads=1`
- `cargo test cli_tests -- --test-threads=1`
- Manual: install fixture, corrupt update fixture, confirm old docset remains searchable.

---

## 3. Milestone R3 — `nowdocs doctor`

**Purpose:** give users and maintainers a single read-only diagnostic command.

**Files likely touched:**

- `src/cli.rs`
- `src/main.rs`
- new `src/doctor.rs`
- `src/lib.rs`
- `tests/cli_tests.rs`
- new `tests/doctor_tests.rs`
- `README.md`

**Tasks:**

- [x] R3.1 Add `Doctor` CLI subcommand.
  - Flags: `--json`, `--docset <name>`, `--mcp`, `--model`, `--repair`.
  - Initial implementation may reject `--repair` with a clear "not implemented yet" until R4.

- [x] R3.2 Implement read-only check model.
  - Check ID, severity, status, message, remediation.
  - Aggregate exit code: fail if any fail.

- [x] R3.3 Implement default checks.
  - cache root exists/writable.
  - db/manifest directories exist or can be created.
  - installed docsets have matching manifest/store presence.
  - manifest validates.
  - stale staging paths detected.

- [x] R3.4 Implement `doctor --docset`.
  - Validate docset name.
  - Check manifest, store path, license/notice metadata presence where expected.
  - Print repair hint for missing/corrupt pieces.

- [x] R3.5 Implement `doctor --mcp` smoke.
  - In-process initialize/tools-list check or equivalent direct MCP handler check.
  - Must not require a network or an external MCP client.

- [x] R3.6 Implement `doctor --json`.
  - Stable top-level shape: `{ "status": "ok|warn|fail", "checks": [...] }`.
  - Tests parse JSON and assert check IDs.

**Verification:**

- `cargo test doctor_tests -- --test-threads=1`
- `cargo test cli_tests -- --test-threads=1`
- `cargo run -- doctor`
- `cargo run -- doctor --json`

---

## 4. Milestone R4 — Cache status and safe repair

**Purpose:** let users inspect disk usage and clean only nowdocs-owned incomplete state.

**Files likely touched:**

- `src/cli.rs`
- `src/main.rs`
- `src/cache.rs`
- `src/doctor.rs` or new `src/cache_cmd.rs`
- `tests/cache_tests.rs`
- `tests/cli_tests.rs`

**Tasks:**

- [x] R4.1 Add `nowdocs cache status [--json]`.
  - Print cache root, category sizes, installed count, staging count.

- [x] R4.2 Add `nowdocs cache clean-staging [--older-than <duration>]`.
  - Remove only nowdocs staging directories.
  - Default threshold should avoid deleting just-created staging from another process.

- [x] R4.3 Wire `doctor --repair` to staging cleanup only.
  - v1 repair does not delete active docsets.
  - Print exactly what was removed.

- [x] R4.4 Tests for cleanup safety.
  - Does not remove active db paths.
  - Does not remove unrelated directories.
  - Removes old staging dirs.

**Verification:**

- `cargo test cache_tests -- --test-threads=1`
- `cargo test cli_tests -- --test-threads=1`
- Manual: create fake stale staging, run `cargo run -- cache clean-staging`.

---

## 5. Milestone U1 — `nowdocs smoke` and better success output

**Purpose:** provide a simple post-install/post-ingest confidence path.

**Files likely touched:**

- `src/cli.rs`
- `src/main.rs`
- `src/tools.rs` or `src/retrieve.rs` only if reusable formatting is needed
- `tests/cli_tests.rs`
- `tests/retrieve_tests.rs`
- `README.md`

**Tasks:**

- [x] U1.1 Add `Smoke` CLI subcommand.
  - `nowdocs smoke <docset> [query] [--json] [--top-k <n>]`.
  - Default query: `installation configuration example`.

- [x] U1.2 Implement smoke search.
  - Validate installed docset.
  - Run retrieve pipeline.
  - Print top results with score, heading/source/chunk index, elapsed time.
  - Non-zero on missing docset or zero results.

- [x] U1.3 Add JSON output.
  - Include docset, query, elapsed_ms, result count, results array.

- [x] U1.4 Improve install/update/ingest/share success output.
  - Include docset version/chunk count/license where available.
  - Include next-step hints.
  - Preserve tests that depend on existing substrings.

- [x] U1.5 Improve `list-installed` table.
  - Table columns: docset, version, chunks, license, status.
  - Consider `--plain` or keep comma output only if test/script compatibility demands it.

**Verification:**

- `cargo test cli_tests -- --test-threads=1`
- `cargo run -- smoke <fixture-docset> "middleware example"` after fixture install/ingest

---

## 6. Milestone U2 — Documentation and MCP onboarding

**Purpose:** make the improved UX discoverable.

**Files likely touched:**

- `README.md`
- new `docs/GETTING_STARTED.md`
- new `docs/TROUBLESHOOTING.md`
- optional `docs/MCP_CLIENTS.md`
- new `docs/RELEASE_READINESS.md`

**Tasks:**

- [x] U2.1 Add a Getting Started guide.
  - install path.
  - install or ingest first docset.
  - smoke test.
  - serve command.

- [x] U2.2 Add MCP client configuration snippets.
  - Cursor.
  - Claude Code.
  - Claude Desktop.
  - Aider.
  - generic MCP JSON.

- [x] U2.3 Add Troubleshooting guide.
  - model download failures.
  - docset not found.
  - corrupt cache.
  - MCP tools not visible.
  - Windows/macOS/Linux path notes.

- [x] U2.4 Update README quickstart.
  - New happy path includes `doctor` and `smoke`.
  - Link to troubleshooting and MCP clients.

- [x] U2.5 Add release-readiness checklist section.
  - Include robustness/UX manual gates.
  - Added `docs/RELEASE_READINESS.md` for owner-run gates.

**Verification:**

- `rg -n "doctor|smoke|MCP|Troubleshooting" README.md docs`
- Manual review of copy-paste JSON snippets.

---

## 7. Milestone U3 — Registry discovery design/implementation split

**Purpose:** improve discoverability without blocking robustness work on registry-index availability.

**Files likely touched if implemented:**

- `src/cli.rs`
- `src/main.rs`
- `src/registry.rs`
- `tests/registry_tests.rs`
- `tests/cli_tests.rs`
- registry index docs

**Tasks:**

- [x] U3.1 Decide registry index source of truth (decided 2026-07-07: GitHub nowdocs-registry repo; self-hosted mirror deferred post-v1).

**Agreed `index.json` schema (2026-07-07):**
- Top-level: `schema_version` (int), `generated_at` (RFC3339), `packages` (array).
- Per package: `docset` (id), `version`, `license` (must be on allowlist MIT/Apache-2.0/CC-BY-4.0), `chunk_count` (int), `freshness` (date), `download_url` (must be nowdocs-registry GitHub Releases domain), `sha256` (64-hex package integrity), `description` (optional, human-friendly).
- Sample fixture: `seed-crates/index.json` (nextjs/react/vue).

- [x] U3.2 Add parser tests for registry index.
  - docset, version, license, chunk_count, freshness, download URL.
  - Enforce nowdocs-registry URL policy.

- [x] U3.3 Add `nowdocs registry list/search` behind stable index.
  - Human table and JSON output.
  - Install status column.

- [x] U3.4 Document that command may access network.

**Verification:**

- `cargo test registry_tests -- --test-threads=1`
- `cargo test cli_tests -- --test-threads=1`

### User-owned / manual follow-up

U3.1 (registry index source of truth) decided 2026-07-07: **GitHub raw/release asset in the nowdocs-registry repo** (self-hosted `registry.nowdocs.dev` mirror deferred to post-v1). Rationale: AGENTS.md pins all registry fetch URLs to the nowdocs-registry GitHub Releases domain (external URLs rejected), so index-on-GitHub is compliant and zero-infra.

U3.2–U3.4 done (2026-07-07):

- (U3.1 done) source of truth = GitHub `nowdocs-registry` org (dedicated `registry-index` repo); default `index_url()` overridable via `NOWDOCS_REGISTRY_INDEX_URL`.
- `index.json` schema approved (U3.2) — see "Agreed index.json schema" above.
- Real fixture at `seed-crates/index.json` (nextjs/react/vue) with `nowdocs-registry`-domain `download_url`s (U3.2).
- Implemented `nowdocs registry list` (table + `--json`, with install-status column) and `nowdocs registry search <query>` (U3.3) in `src/registry.rs`, wired via `RegistryCommands` in `cli.rs`/`main.rs`.
- Parser + search + URL-policy-rejection tests in `tests/registry_index_tests.rs` (U3.2).
- Network access documented in code: index is fetched via `curl` through the existing `download_to_temp` + `is_allowed_registry_url` gate (U3.4).

---

## 8. Release readiness gates for this track

Before calling robustness/UX hardening complete:

- [x] `nowdocs doctor` passes on a clean checkout after `cargo build` (verified 2026-07-08: clean build, `doctor` on empty cache → `status: ok`, exit 0).
- [x] `nowdocs doctor --json` is parseable and documented.
- [x] bad/corrupt install does not create an active docset (proven by `test_invalid_archive_install_leaves_no_active_manifest_or_store`, green in full suite). NOTE: "interrupted" mid-process kill is a known limitation, not unit-tested.
- [x] failed update preserves the previous working docset (proven by `test_failed_update_preserves_old_active_manifest_and_store`, green in full suite).
- [ ] `nowdocs smoke` works for a locally ingested fixture — **manual release gate**: requires the 66MB jina embedder model download (not present in CI); covered by the manual-gate clause (G9) above.
- [x] README quickstart includes doctor + smoke + MCP setup verification.
- [x] Troubleshooting guide covers the top 8 expected failures.
- [x] all normal tests pass with `cargo test -- --test-threads=1` (verified 2026-07-08: Exit 0, 0 failures across 21 test binaries).
- [x] expensive model/real-docset checks are either passing in a dedicated script or documented as manual release gates.

### User-owned / manual gates not checked by this agent

The following gates stay unchecked until the owner runs them in a fully provisioned environment with the real model/cache/toolchain setup and real docset assets:

- ~~clean-checkout `cargo build` plus `nowdocs doctor` / `nowdocs doctor --json`~~ — **verified 2026-07-08** (clean build; `doctor` → `status: ok`, exit 0).
- ~~interrupted/bad install and failed-update manual verification against real archives~~ — **bad/corrupt-install and failed-update proven by transactional unit tests 2026-07-08**; "interrupted" mid-kill remains a known limitation, not unit-tested.
- `nowdocs smoke` against a locally ingested fixture with the real embedder available — **still manual** (requires 66MB jina embedder model download; not present in CI).
- ~~full `cargo test -- --test-threads=1`~~ — **verified 2026-07-08** (Exit 0, 0 failures across 21 test binaries).
- expensive model and real-docset checks, including the Next.js large-docset gate.

---

## 9. Suggested task order

1. R2 transactional install/update — done.
2. R3 doctor read-only diagnostics — done.
3. U1 smoke command and output improvements — done.
4. R4 cache status/repair — done.
5. U2 docs/onboarding — done.
6. U3 registry discovery once index source is settled — user-owned decision first.

This order is confirmed: after the archive-validation prerequisite lands, front-load fail-safe install/update, expose diagnostics, then improve the first-run experience. Do not start R4/U2/U3 before the remaining slice has a working implementation and tests unless explicitly reprioritized.
