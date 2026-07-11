# nowdocs architecture review implementation plan

> **Date:** 2026-07-10
> **Spec:** `docs/superpowers/specs/2026-07-10-architecture-review.md`
> **Goal:** resolve the 7 P0 + 31 P1 + 17 P2 findings identified by the architecture review, implementing the 15 confirmed Open Question decisions across 6 phases.

---

## 0. Global rules for this track

- Keep MCP stdio-only; do not add host/port flags.
- Keep embedder model/spec fields frozen (`model_id`/`model_revision`/`model_sha256`).
- Keep registry URL allowlist and share text-only invariant (D10).
- Keep `~/.cache/nowdocs` layout, schema version 1, cache layout version 1.
- Keep v1 English-only, flat exact search (no HNSW/IVF).
- Cargo.toml modifications are authorized for this track (OQ3).
- Every task follows TDD: failing test -> implementation -> passing test -> commit.
- Conventional commits with phase prefix: `feat(registry): ... (A1.1)`, `fix(mcp): ... (A1.0)`.
- Every implementation agent works in an independent git worktree/branch per phase.
- Do not `git push` without explicit user approval.
- Test commands use `--test-threads=1` (registry tests share real cache root, not thread-safe).
- `cargo fmt --check` and `git diff --check` must pass before commit.
- Cross-phase file merge ordering (see track spec §11.3) must be respected when merging worktrees.
- The `NOWDOCS_TEST_URL` env var and `file://` URL acceptance must be `cfg(test)`-gated after Phase 0; tests relying on them must still pass.

---

## 1. Milestone A1.0 - Foundation cleanup

**Purpose:** fix 8 low-risk issues that other phases assume are resolved.

**Difficulty:** 3/10 (easy). All items independent and parallelizable.

**Parallelizable:** yes (all 8 items can be split across agents within this phase).

**Worktree:** `../nowdocs-a1-foundation` on `fix/a1-foundation`.

**Files likely touched:**

- `src/registry.rs` (S3, S7)
- `src/tools.rs` (S6)
- `src/mcp.rs` (M4)
- `src/doctor.rs` (M14)
- `Cargo.toml` (M16)
- `build.rs` (M17)
- `docs/superpowers/specs/2026-06-28-nowdocs-design-review.md` (M18)

**Tasks:**

- [ ] A1.0.1 S3: `cfg(test)` gate `NOWDOCS_TEST_URL` + `file://` acceptance.
  - Wrap env var reads in `#[cfg(test)]`; `file://` rejected in non-test builds.
  - Tests: existing `registry_tests.rs` `file://` tests still pass.

- [ ] A1.0.2 S6: `nowdocs_list` sanitize one-liner.
  - `sanitize::sanitize_metadata(&docsets.join(", "))`.

- [ ] A1.0.3 S7: `manifest.docset == CLI docset` identity binding.
  - Add check in `install_to_staging` after `manifest::validate`.

- [ ] A1.0.4 M4: parse error `-32700`.
  - Add `ERR_PARSE_ERROR: i64 = -32700`; use at `mcp.rs:51`.

- [ ] A1.0.5 M14: doctor `run_model_check` status aggregation.
  - `status: aggregate_status(&checks)` instead of `Severity::Ok`.

- [ ] A1.0.6 M16: delete dead deps (openssl vendored + thiserror).
  - Remove `Cargo.toml:26` (thiserror) and `Cargo.toml:49` (openssl).

- [ ] A1.0.7 M17/OQ15: protoc contradiction verify and fix.
  - Build-test which path works; remove the redundant one.

- [ ] A1.0.8 M18: spec appendix G version update.
  - Update 0.30/7.0 -> 0.31/8.0; add verification note. (non-code)

**Verification:**

```bash
cargo test -- --test-threads=1
cargo build
cargo fmt --check
git diff --check
```

---

## 2. Milestone A1.1 - Registry artifact contract + integrity + atomicity

**Purpose:** resolve the registry artifact route (OQ1 Method A), add sha256 integrity, implement fail-safe atomic promote, and harden install/uninstall/share lifecycle.

**Difficulty:** 8/10 (hard). Largest and most critical phase.

**Parallelizable:** no (depends on Phase 0; internal tasks have ordering dependencies).

**Worktree:** `../nowdocs-a1-registry` on `fix/a1-registry-artifact`.

**Files likely touched:**

- `src/registry.rs`, `src/ingest.rs`, `src/manifest.rs`, `src/store.rs`, `src/cache.rs`
- `tests/registry_tests.rs`, `tests/registry_index_tests.rs`, `tests/manifest_tests.rs`

**Tasks:**

- [ ] A1.1.1 S1+S4: two archive types + atomic rename-based promote.
  - `ArchiveType::ShareBundle` vs `ArchiveType::RegistryRelease`.
  - `validate_archive` mode parameter.
  - New `promote_staging`: rename-based, no zero vectors, no `copy`.
  - Delete `zero_vectors` code at `registry.rs:555-557`.

- [ ] A1.1.2 S2: docset sha256 integrity verification.
  - `install()` gains `expected_sha256`; streaming sha256 recompute.

- [ ] A1.1.3 S4+M5+M20: ingest atomicity + manifest fixes.
  - Reorder `ingest_dir`: load embedder before wipe.
  - Manifest atomic write (tmp + rename).
  - `chunk_size_tokens: 384` (not 512).

- [ ] A1.1.4 M8: chunks.jsonl row-level validation.
  - `validate_chunks_jsonl`: idx continuity, duplicates, holes, chunk_count match.

- [ ] A1.1.5 M9: uninstall cleanup (staging/rollback/license).

- [ ] A1.1.6 M10: share output cleanup (reject non-empty dir).

- [ ] A1.1.7 M11: download temp filename (add docset + timestamp).

- [ ] A1.1.8 M12: `validate_manifest_for_docset` business invariants.

- [ ] A1.1.9 M25: `--source-url-base` flag for ingest.

- [ ] A1.1.10 N6/OQ10: lockfile concurrent install protection.

- [ ] A1.1.11 OQ6: URL gate hardening (`registry.nowdocs.dev` path prefix + curl redirect policy).

**Verification:**

```bash
cargo test registry_tests -- --test-threads=1
cargo test registry_index_tests -- --test-threads=1
cargo test manifest_tests -- --test-threads=1
cargo test ingest_tests -- --test-threads=1
cargo fmt --check
git diff --check
```

---

## 3. Milestone A1.2 - Retrieval quality

**Purpose:** embedder startup cache, true vector MMR, no-answer threshold, oversized chunk splitting.

**Difficulty:** 7/10 (medium-hard).

**Parallelizable:** no (depends on Phase 1 store changes).

**Worktree:** `../nowdocs-a1-retrieval` on `fix/a1-retrieval-quality`.

**Files likely touched:**

- `src/embedder.rs`, `src/retrieve.rs`, `src/store.rs`, `src/mcp.rs`, `src/chunker.rs`
- `tests/retrieve_tests.rs`, `tests/embedder_tests.rs`, `tests/chunker_tests.rs`

**Tasks:**

- [ ] A1.2.1 N3+M13: embedder OnceLock cache + HF_HOME -> ApiBuilder::with_cache_dir.
  - Eliminate `unsafe set_var`; cache embedder by spec; `serve` pre-loads.

- [ ] A1.2.2 N1/OQ4: true vector MMR.
  - `store::fetch_vectors(chunk_ids)`; `mmr_rerank` replaces `dedup_by_source_url`; `lambda=0.7`.

- [ ] A1.2.3 N4/OQ11: no-answer threshold.
  - Top score below threshold -> return empty + hint.

- [ ] A1.2.4 N7/OQ12: oversized chunk hard-split + embedder token guard.
  - Chunker splits by function/line windows with fence preservation.

- [ ] A1.2.5 Backlog: query chars vs tokens, neighbor window cross-file guard, chunker torture fixtures.

**Verification:**

```bash
cargo test retrieve_tests -- --test-threads=1
cargo test embedder_tests -- --test-threads=1
cargo test chunker_tests -- --test-threads=1
cargo fmt --check
git diff --check
```

---

## 4. Milestone A1.3 - CI readiness gates

**Purpose:** make CI quality signal truthful; add real-docset eval, thresholds, observability, model pre-warm.

**Difficulty:** 5/10 (medium).

**Parallelizable:** no (depends on Phase 2 retrieval).

**Worktree:** `../nowdocs-a1-ci` on `fix/a1-ci-gates`.

**Files likely touched:**

- `.github/workflows/`, `scripts/`, `src/eval.rs`, `src/smoke.rs`, `src/doctor.rs`, `src/main.rs`
- `tests/eval_tests.rs`, `README.md`, `deny.toml`

**Tasks:**

- [x] A1.3.1 S5: CI job for `--ignored` tests (nightly + manual).
- [x] A1.3.2 OQ8: `scripts/ci-prepare-nextjs-fixture.sh` + cache.
- [x] A1.3.3 OQ9: strict thresholds MRR >= 0.85, Recall@5 >= 0.90.
- [x] A1.3.4 N5/OQ5: model pre-warm hint in install + doctor output.
- [x] A1.3.5 OQ6: declare curl dependency in README + doctor check.
- [x] A1.3.6 M22: `InstalledDocsetState` enum unified across all entry points.
- [x] A1.3.7 M23: performance observability (smoke embed_ms/search_ms; doctor cache sizes).
- [x] A1.3.8 M24: negative queries in eval + false-positive rate.
- [x] A1.3.9 Backlog: deny.toml RUSTSEC ignore expiry, legal CI gate documentation.

**Verification:**

```bash
cargo test eval_tests -- --test-threads=1
cargo test doctor_tests -- --test-threads=1
cargo test smoke_tests -- --test-threads=1
cargo test cli_tests -- --test-threads=1
# Manual: trigger eval CI job via workflow_dispatch
cargo fmt --check
git diff --check
```

---

## 5. Milestone A1.4 - MCP error contract

**Purpose:** business errors return `isError:true` with actionable hints; fix inputSchema types; add NDJSON size cap.

**Difficulty:** 5/10 (medium).

**Parallelizable:** yes (after Phase 0; independent of Phases 1-3). Note: `mcp.rs` merge order is P0 -> P2 -> P4; if Phase 2 is not merged yet, rebase on Phase 0 only.

**Worktree:** `../nowdocs-a1-mcp` on `fix/a1-mcp-contract`.

**Files likely touched:**

- `src/mcp.rs`, `src/tools.rs`
- `tests/mcp_tests.rs`, `tests/tools_tests.rs`

**Tasks:**

- [ ] A1.4.1 N2/OQ2: MCP error contract refactor (isError:true + classification).
- [ ] A1.4.2 M3/OQ14: score field decision documented (no code change, comment only).
- [ ] A1.4.3 M6: inputSchema integer + min/max/default.
- [ ] A1.4.4 M7: NDJSON line size cap (1 MiB).
- [ ] A1.4.5 Backlog: JSON-RPC batch request handling, write_response code-field convention.

**Verification:**

```bash
cargo test mcp_tests -- --test-threads=1
cargo test tools_tests -- --test-threads=1
cargo fmt --check
git diff --check
```

---

## 6. Milestone A1.5 - Hardening misc

**Purpose:** remaining hardening, documentation, and backlog items.

**Difficulty:** 4/10 (easy-medium).

**Parallelizable:** yes (after Phase 0; independent of Phases 1-4). Note: `doctor.rs` merge order is P0 (M14) -> P5 (M15); ensure Phase 0 is merged.

**Worktree:** `../nowdocs-a1-misc` on `fix/a1-hardening-misc`.

**Files likely touched:**

- `src/doctor.rs`, `src/errors.rs`, `src/sanitize.rs`
- `README.md`, `docs/`
- `src/lib.rs`, `src/main.rs`, `src/token.rs`, `src/embedder.rs` (backlog only, avoid Phase 2 conflict)

**Tasks:**

- [x] A1.5.1 M15: doctor default checks include model + mcp.
- [x] A1.5.2 M19: errors.rs dead category cleanup.
- [x] A1.5.3 M21: sanitize false-positive fix (`as an ai` narrowing).
- [x] A1.5.4 M26: Store sync facade documentation (comment only).
- [x] A1.5.5 M27: CLI/API naming disambiguation (README docs).
- [x] A1.5.6 OQ7: sanitize markers decision documented (no code change).
- [x] A1.5.7 Backlog: store id dead field, lib.rs visibility, process::exit, list_installed FS scan, pytorch_model.bin fallback, sha256_hex streaming, provenance model_version, token.rs OnceLock, 959-line registry split.


**Verification:**

```bash
cargo test doctor_tests -- --test-threads=1
cargo test sanitize_tests -- --test-threads=1
cargo test -- --test-threads=1
cargo fmt --check
git diff --check
```

---

## 7. Release readiness gates for this track

Before calling the A1 architecture review track complete:

**Worktree-verifiable gates (can be checked with `cargo test`):**

- [ ] `cargo test -- --test-threads=1` passes (all normal tests).
- [ ] `nowdocs_list` output is sanitized.
- [ ] `NOWDOCS_TEST_URL` + `file://` only accessible in `cfg(test)` builds.
- [ ] Dead dependencies (openssl, thiserror) removed.
- [ ] `promote_staging` uses rename, not copy; failed promote restores rollback.
- [ ] `ingest_dir` loads embedder before wiping store.
- [ ] MCP business errors return `isError: true` with actionable hints.
- [ ] Embedder loaded once and cached (no per-search reload); no `unsafe set_var`.
- [ ] `dedup_by_source_url` replaced by MMR.
- [ ] No-answer threshold returns empty for irrelevant queries (placeholder threshold with TODO).
- [ ] Doctor default checks cover model + mcp.
- [ ] `InstalledDocsetState` enum unified across all status entry points.
- [ ] `smoke` outputs embed_ms/search_ms.

**CI-verifiable gates (require CI job to run):**

- [ ] All 7 P0 items resolved (S1-S7).
- [ ] CI `eval` job passes with MRR >= 0.85, Recall@5 >= 0.90 on real Next.js docset.
- [ ] E2 命门 test (cosine > 0.99) passes in nightly CI.

**User-owned / manual gates:**

- [ ] `nowdocs install` downloads and installs a prebuilt `.lance` table with real vectors (no zero vectors). Requires registry CI pipeline to exist.
- [ ] `nowdocs install` verifies sha256 and rejects mismatched archives.
- [ ] `nowdocs install` rejects `manifest.docset != CLI docset`.
- [ ] Registry CI pipeline implementation (produces `.lance` release artifacts). This is a separate infrastructure project; A1.1 defines the install-side contract.
- [ ] Real MCP client compatibility testing (Claude Code, Cursor, Aider) for `isError:true` error shape.
- [ ] MMR lambda tuning against real Next.js + React + Vue eval data.
- [ ] No-answer threshold calibration against real eval data.

---

## 8. Suggested task order

1. **Phase 0 (A1.0)** - foundation cleanup. All items parallelizable. Must complete first.
2. **Phase 1 (A1.1)** - registry artifact contract. Depends on Phase 0. Largest phase; start S1+S4 combined design first.
3. **Phase 2 (A1.2)** - retrieval quality. Depends on Phase 1 (store changes). N3+M13 first (embedder cache), then N1 (MMR), then N4 (threshold).
4. **Phase 3 (A1.3)** - CI readiness gates. Depends on Phase 2 (retrieval stable). S5 + OQ8 first (CI job + fixture), then thresholds.
5. **Phase 4 (A1.4)** - MCP error contract. Independent after Phase 0. Can run in parallel with Phases 1-3. Mind `mcp.rs` merge order.
6. **Phase 5 (A1.5)** - hardening misc. Independent after Phase 0. Can run in parallel with Phases 1-4. Mind `doctor.rs` merge order.

```text
Phase 0 ──┬──> Phase 1 ──> Phase 2 ──> Phase 3
           │
           ├──> Phase 4  (parallel with 1-3)
           └──> Phase 5  (parallel with 1-4)
```
