# R1 spec — error taxonomy + registry archive validation

> **Parent track:** `docs/superpowers/specs/2026-07-06-robustness-ux.md`
> **Implementation plan:** `docs/superpowers/plans/2026-07-06-robustness-ux-plan.md`
> **Stage:** R1 — foundation for robustness first slice
> **Owner model:** intended for a single implementation agent branch; do not combine with R2/R3/U1 in the same implementation PR.
> **Difficulty:** 3/10 to 4/10 (medium-low). Main review risk is tar edge cases, not architecture.
> **Worktree rule:** implement from a dedicated git worktree and branch for R1 to avoid synchronization conflicts with other agents.

---

## 0. Goal

R1 makes registry installs fail safely **before** any transactional install rewrite. It introduces a small user-facing error taxonomy and validates registry archives before install code writes active cache state.

R1 is intentionally narrow. It does **not** implement staging, rollback, `doctor`, `smoke`, cache repair, registry discovery, or MCP changes.

---

## 1. Non-negotiable constraints

- Keep MCP stdio-only; no host/port flags.
- Do not change embedder model or embedder manifest field names.
- Keep registry URL allowlist behavior: production install URLs must stay under `github.com/nowdocs-registry/*` or `registry.nowdocs.dev/*`; test `file://` URLs may remain allowed.
- Keep share text-only; no contributor vectors accepted.
- Do not change active cache layout in R1.
- Do not implement staging/rollback in R1; that is R2.
- Do not add large model/download tests to default test runs.

---

## 2. Dispatch metadata

- **Estimated difficulty:** medium-low, 3/10 to 4/10.
- **Expected implementation time:** 2-4 hours for a Rust maintainer; up to 0.5-1 day for an external coding agent including tests and review fixes.
- **Review risk:** medium-low. Focus review on tar typeflags, path normalization, size guardrails before allocation, duplicate basename semantics, and not accidentally starting R2 staging work.
- **Required isolation:** use an independent git worktree and stage-specific branch, e.g. `../nowdocs-r1-archive-validation` on `fix/r1-archive-validation`. Do not reuse a dirty worktree or another agent's branch.

---

## 3. Files in scope

Expected files:

- `src/registry.rs`
- `tests/registry_tests.rs`

Allowed if useful:

- `src/errors.rs` or `src/diagnostics.rs`
- `src/lib.rs`
- `tests/diagnostics_tests.rs`

Out of scope:

- `src/cli.rs`
- `src/main.rs`, except only if a tiny error formatting hook is impossible elsewhere; avoid this in R1.
- `src/mcp.rs`
- `src/embedder.rs`
- README or user docs.

---

## 4. Error taxonomy contract

Add the smallest internal representation needed to make registry install failures actionable.

Minimum fields:

- `code`: stable-ish uppercase snake case, e.g. `ARCHIVE_MISSING_CHUNKS`.
- `category`: one of `input`, `network`, `archive`, `manifest`, `cache`, `model`, `retrieval`, `mcp`, `internal`.
- `message`: concise human message.
- `hint`: one next action.

R1 may keep `anyhow::Result` at public function boundaries. The error type can implement `Display` so `anyhow` prints something like:

```text
error[ARCHIVE_MISSING_CHUNKS]: registry archive is missing chunks.jsonl
next: retry install, or report the broken registry release
```

Required R1 codes:

| Code | Category | Trigger |
|---|---|---|
| `ARCHIVE_MISSING_MANIFEST` | archive | archive has no `manifest.json` |
| `ARCHIVE_MISSING_CHUNKS` | archive | archive has no `chunks.jsonl` |
| `ARCHIVE_UNSAFE_PATH` | archive | absolute path or `..` component |
| `ARCHIVE_UNSUPPORTED_ENTRY` | archive | symlink, hardlink, device, or other unsupported non-regular entry with unsafe semantics |
| `ARCHIVE_VECTOR_ARTIFACT` | archive | `.lance`, `.faiss`, `vectors.*`, or `embeddings.*` entry |
| `ARCHIVE_DUPLICATE_ENTRY` | archive | duplicate `manifest.json`, `chunks.jsonl`, `LICENSE`, or `NOTICES` |
| `ARCHIVE_TOO_LARGE` | archive | entry or total size exceeds configured guardrail |

Names may vary slightly if tests document the exact chosen names, but keep them clear and uppercase.

---

## 5. Archive validation contract

Validate archive entries before install writes active cache paths.

Reject archives with:

1. Missing `manifest.json`.
2. Missing `chunks.jsonl`.
3. Absolute paths.
4. Any `..` component after path normalization.
5. Duplicate security-sensitive basename entries:
   - `manifest.json`
   - `chunks.jsonl`
   - `LICENSE`
   - `NOTICES`
6. Symlink, hardlink, device, or other non-regular entry that could affect extraction semantics.
7. Vector artifacts:
   - any path ending in `.lance`
   - any path ending in `.faiss`
   - basename matching `vectors.*`
   - basename matching `embeddings.*`
8. Invalid UTF-8 for metadata entries that must be decoded as text.
9. Files exceeding size/count guardrails.

Accept:

- regular files needed by current share artifacts.
- nested directory prefixes, as long as basename validation finds required entries.
- optional `LICENSE` and `NOTICES` once each.

---

## 6. Size/count guardrails

R1 must add named constants and tests. Choose conservative defaults that are high enough for current seed crates and large docsets, but low enough to prevent accidental huge archives.

Suggested starting constants:

- max total archive bytes: `512 MiB`
- max single entry bytes: `256 MiB`
- max entry count: `100_000`

If implementation finds these too restrictive for existing fixtures, document the chosen values in code comments and tests.

---

## 7. Install integration

R1 should preserve the current install flow, except:

1. Read/extract archive entries.
2. Validate archive structure and safety.
3. Only after validation, parse manifest/chunks and write active cache as current code does.

R1 does not need to solve partial active writes. That is R2. R1 only ensures known-bad archive shapes fail before active writes begin.

---

## 8. Required tests

Add tests for:

- missing `manifest.json` rejected with `ARCHIVE_MISSING_MANIFEST` or equivalent.
- missing `chunks.jsonl` rejected with `ARCHIVE_MISSING_CHUNKS` or equivalent.
- path traversal rejected.
- absolute path rejected.
- duplicate manifest/chunks rejected.
- vector artifact rejected.
- unsupported tar entry rejected if the test tar helper can produce it.
- oversized entry or archive rejected.
- good existing archive fixture still installs.
- error display includes a code and a hint/next action.

Keep tests deterministic and offline. Prefer synthetic tar fixtures already used by `registry_tests`.

---

## 9. Verification commands

Required before commit:

```bash
cargo test registry_tests -- --test-threads=1
cargo test diagnostics_tests -- --test-threads=1  # only if diagnostics_tests.rs is added
cargo fmt --check
git diff --check
```

If no diagnostics test file is added, do not run the second command; mention it as not applicable in the final report.

---

## 10. Done definition

R1 is done when:

- bad archives fail before active cache writes.
- errors for archive validation have actionable codes/hints.
- existing valid install tests still pass.
- R2 can rely on a validation function before implementing staging/rollback.
- no behavior outside registry install error handling is changed.
