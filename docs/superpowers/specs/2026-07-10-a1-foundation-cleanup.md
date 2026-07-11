# A1.0 spec - foundation cleanup

> **Parent track:** `docs/superpowers/specs/2026-07-10-architecture-review.md`
> **Implementation plan:** `docs/superpowers/plans/2026-07-10-architecture-review-plan.md`
> **Depends on:** nothing (first phase).
> **Stage:** A1.0 - low-risk foundation fixes.
> **Difficulty:** 3/10 (easy). Main review risk is `cfg(test)` gating correctness and ensuring no test relies on `NOWDOCS_TEST_URL` in non-test builds.
> **Worktree rule:** implement from a dedicated git worktree and branch, e.g. `../nowdocs-a1-foundation` on `fix/a1-foundation`.

---

## 0. Goal

A1.0 fixes 8 low-risk issues that other phases assume are already resolved. All items are independent and can be implemented in parallel within this phase. No architectural changes; each item is a targeted fix.

---

## 1. Non-goals

- Do not change registry install/promote logic (Phase 1).
- Do not change retrieval pipeline (Phase 2).
- Do not change MCP error contract (Phase 4).
- Do not add new dependencies beyond removing dead ones.

---

## 2. Files in scope

Expected:

- `src/registry.rs` (S3, S7)
- `src/tools.rs` (S6)
- `src/mcp.rs` (M4)
- `src/doctor.rs` (M14)
- `Cargo.toml` (M16)
- `build.rs` (M17)
- `docs/superpowers/specs/2026-06-28-nowdocs-design-review.md` (M18 - spec version update only)

Out of scope:

- `src/store.rs`, `src/retrieve.rs`, `src/embedder.rs`, `src/ingest.rs`, `src/chunker.rs`, `src/cache.rs`, `src/manifest.rs`, `src/sanitize.rs`, `src/errors.rs`

---

## 3. Tasks

### 3.1 S3 - `NOWDOCS_TEST_URL` + `file://` cfg(test) gating

**Problem:** `registry.rs:27-29` (`is_test_file_url`) returns `true` for any `file://` URL. `registry.rs:780-783` (`update`) and `main.rs:376-380` (`registry_url_for`) read `NOWDOCS_TEST_URL` env var without `cfg(test)` gating. A production binary with this env var set bypasses the URL gate entirely.

**Fix:**

- `main.rs` `registry_url_for`: remove the `NOWDOCS_TEST_URL` env var read entirely. The binary always returns the canonical GitHub registry URL.
- `main.rs` `Update` handler: call `registry_url_for(&docset)` and pass the result to `nowdocs::registry::install()` (instead of calling `nowdocs::registry::update()` which reads the env var).
- `registry.rs` `update()`, `is_test_file_url`, `is_allowed_registry_url`: keep as-is (no `cfg(test)` gating). These are library-internal APIs. The security boundary is the binary's `registry_url_for` which never reads `NOWDOCS_TEST_URL`.
- Do NOT use `cfg!(test)` (runtime macro) or `std::env::var("CARGO_MANIFEST_DIR")` as a proxy for test detection. These are exploitable at runtime.
- Integration tests that previously spawned the binary with `NOWDOCS_TEST_URL` set must be refactored to call the library API (`nowdocs::registry::install()`) directly.

**Implementation note (verified 2026-07-10):** `cargo test` does NOT set `cfg(test)` for either the library or the binary when compiling integration tests in `tests/`. `#[cfg(test)]` only affects unit tests within `src/` files. Therefore:
- `#[cfg(test)]` in `registry.rs` (lib) does NOT work for integration tests. The `NOWDOCS_TEST_URL` env var read remains in the library for test use.
- `#[cfg(test)]` in `main.rs` (binary) does NOT work either. The binary's `registry_url_for` must simply not read the env var at all.
- The security model: the production binary never reads `NOWDOCS_TEST_URL` because `registry_url_for` doesn't read it. The library API (`update()`, `install()`) still reads it, but the library is only callable from the binary's CLI handlers which use `registry_url_for`.

**Tests:**

- The production binary (built by `cargo build`) does NOT read `NOWDOCS_TEST_URL`. Verified by setting the env var and confirming the binary attempts the canonical GitHub URL.
- No `CARGO_MANIFEST_DIR` string appears in the binary (verified by `strings`).
- Existing `registry_tests.rs` tests using `file://` URLs still pass (they call the library API directly).
- Existing `mcp_tests.rs` and `doctor_tests.rs` tests are NOT deleted - they do not use `file://` URLs and should pass unchanged.

### 3.2 S6 - `nowdocs_list` sanitize

**Problem:** `tools.rs:114-118` (`handle_list`) joins docset names into text without calling `sanitize`.

**Fix:** Add `sanitize::sanitize_metadata` to the joined string:

```rust
let text = sanitize::sanitize_metadata(&docsets.join(", "));
```

**Tests:** Add a test that `handle_list` output passes through `sanitize_metadata`.

### 3.3 S7 - manifest.docset identity binding

**Problem:** `verify_staging` (`registry.rs:492-503`) calls `manifest::parse_manifest` + `manifest::validate` but never compares `manifest.docset` with the CLI-provided `docset` parameter. `install_to_staging` calls `verify_staging` at `registry.rs:486` but does not pass `docset` through.

**Fix:** In `verify_staging`, after `manifest::validate(&m)?` (line 500), add identity check. This requires passing `docset` into `verify_staging` (or performing the check in `install_to_staging` after `verify_staging` returns, using the already-parsed manifest). Recommended: add the check in `install_to_staging` after `verify_staging(&staging_path)?` by re-parsing the staged manifest, or refactor `verify_staging` to accept `expected_docset: &str`:

```rust
fn verify_staging(staging_path: &Path, expected_docset: &str) -> Result<()> {
    // ... existing checks ...
    manifest::validate(&m)?;
    if m.docset != expected_docset {
        return Err(anyhow::anyhow!(
            "manifest docset {:?} does not match install name {:?}",
            m.docset, expected_docset
        ));
    }
    Ok(())
}
```

**Tests:** Add a test that install rejects an archive whose manifest.docset differs from the CLI docset.

### 3.4 M4 - parse error -32700

**Problem:** `mcp.rs` parse error branch (inside `run_loop`, the `Err(e) =>` arm of `serde_json::from_str`) uses `ERR_INVALID_PARAMS` (-32602). JSON-RPC 2.0 spec requires `-32700` for parse errors.

**Fix:** Add `const ERR_PARSE_ERROR: i64 = -32700;` near the existing error constants (`mcp.rs:21-22`). Replace `ERR_INVALID_PARAMS` with `ERR_PARSE_ERROR` in the parse error `err_response` call (the one with message `"parse error: {e}"`).

**Tests:** Add a test that sending malformed JSON returns error code `-32700`.

### 3.5 M14 - doctor status aggregation bug

**Problem:** `doctor.rs:296-297` (`run_model_check`) hardcodes `status: Severity::Ok`, ignoring the `checks` vector content. When the model cache is missing, a `Warn` check is pushed but the overall status remains `Ok`.

**Fix:** Replace `status: Severity::Ok` with `status: aggregate_status(&checks)` (the helper already exists at `doctor.rs:33`).

**Tests:** Add a test that `run_model_check` with a missing model cache returns `status: Warn` (not `Ok`).

### 3.6 M16 - delete dead dependencies

**Problem:** `Cargo.toml:49` declares `openssl = { version = "0.10", features = ["vendored"] }` but `src/` has 0 references to `openssl`. `Cargo.toml:26` declares `thiserror = "2.0"` but `src/` has 0 `#[derive(thiserror::Error)]` or `use thiserror`. `errors.rs` uses hand-written `impl std::error::Error`.

**Fix:** Remove both lines from `Cargo.toml`. Run `cargo build` to confirm no breakage (they were dead deps).

**Tests:** `cargo build` passes. `cargo test -- --test-threads=1` passes.

### 3.7 M17/OQ15 - protoc contradiction

**Problem:** `Cargo.toml:41` gives `lance` the `features = ["protoc"]` (vendored protoc). `build.rs:6-11` detects system `protoc` and panics if missing. These are contradictory: if the feature works, the build.rs panic is redundant; if it does not, the feature declaration is misleading.

**Fix:** Build-verify which one is effective:

1. Remove `build.rs` protoc detection temporarily. Run `cargo build` from clean.
2. If build succeeds: the `protoc` feature works. Delete the `build.rs` protoc panic (or downgrade to a warning). Keep the Cargo.toml feature.
3. If build fails: the `protoc` feature does not work. Restore `build.rs`, remove the `features = ["protoc"]` from Cargo.toml, and document protoc as a build requirement in `README.md`.

**Decision recording:** The agent must record which path was taken (2 or 3) in the commit message and PR description. If path 3, the done definition includes "README.md documents protoc as a build dependency". If path 2, the done definition includes "build.rs no longer panics on missing protoc".

**Tests:** `cargo build` from clean passes without system protoc installed (if path 2) or with a clear error message documenting the requirement (if path 3).

### 3.8 M18 - spec appendix G version update

**Problem:** `docs/superpowers/specs/2026-06-28-nowdocs-design-review.md` appendix G references lancedb 0.30.0 / lance 7.0.0. The lockfile resolves to lancedb 0.31.0 / lance 8.0.0. Build is green on rustc 1.97.0.

**Fix:** Update appendix G version references from 0.30/7.0 to 0.31/8.0. Add a note: "API compatibility verified by successful `cargo check` on rustc 1.97.0, 2026-07-10."

**Tests:** N/A (documentation only).

---

## 4. Required tests

- `NOWDOCS_TEST_URL` env var not read in non-test builds (implicit via `cfg(test)`).
- `handle_list` output is sanitized.
- Install rejects `manifest.docset != CLI docset`.
- Malformed JSON returns `-32700`.
- `run_model_check` with missing model returns `Warn` status.
- `cargo build` passes without openssl/thiserror.
- `cargo build` passes with protoc resolution determined by M17.

---

## 5. Verification commands

```bash
cargo test -- --test-threads=1
cargo fmt --check
git diff --check
cargo build  # confirms dead dep removal + protoc fix
```

---

## 6. Done definition

- All 8 items implemented and committed with conventional commits prefixed `(A1.0)`.
- `cargo test -- --test-threads=1` passes with **zero test deletions** (all pre-existing tests still present and passing).
- `cargo build` passes from clean.
- No new dependencies added.
- `main.rs` `registry_url_for` does NOT read `NOWDOCS_TEST_URL` (production binary cannot be redirected to `file://`).
- No `cfg!(test)` runtime macros or `CARGO_MANIFEST_DIR` env-var proxies used as test detection.
- No `CARGO_MANIFEST_DIR` string in the production binary (verified by `strings`).
- Spec appendix G updated with correct versions.
