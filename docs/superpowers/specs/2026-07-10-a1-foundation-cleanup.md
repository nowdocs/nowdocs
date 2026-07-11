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

### 3.1 S3 - `NOWDOCS_TEST_URL` + `file://` test-mode gating

**Problem:** `registry.rs` (`is_test_file_url`) returns `true` for any `file://` URL. `update()` and the binary's URL derivation read `NOWDOCS_TEST_URL` env var. A production binary with this env var set could bypass the URL gate entirely.

**Fix (two-layer security boundary):**

**Layer 1 - Binary (main.rs):** The production CLI `install` and `update` commands never call `registry::update()` and never read `NOWDOCS_TEST_URL`. They call `catalog_lookup_for()` (which fetches the registry index and validates every package's download URL via `is_allowed_package_url`) then `install_with_sha256(&docset, &url, &sha)` with the catalog-derived URL + hash. The binary's `Update` handler does NOT call `registry_url_for` or `registry::update` -- it uses `catalog_lookup_for` directly.

**Layer 2 - Library API (registry.rs):** `install_to_staging` gates `file://` URLs via `is_test_mode()`, which is a **compile-time** gate using a Cargo feature (`test-fixture`). The feature is activated only during `cargo test` (via a self dev-dependency in `Cargo.toml`). In production builds (`cargo build`, `cargo build --release`), the feature is off, `is_test_mode()` returns `false`, and `file://` URLs are rejected. The `NOWDOCS_TEST_URL` env var read in `update()` is also gated behind `is_test_mode()`. This is not a spoofable runtime check -- the test-only code paths do not exist in the production binary at all (verified by `strings target/release/nowdocs | grep NOWDOCS_TEST_URL` returning 0 matches).

**Why a Cargo feature, not `#[cfg(test)]` alone:** `cargo test` does NOT set `cfg(test)` for integration test builds (tests in `tests/*.rs`). It only sets `cfg(test)` for unit tests within `src/` files. Since 40+ integration tests call `install()` with `file://` URLs, `#[cfg(test)]` alone would break them. The `test-fixture` Cargo feature, activated via the self dev-dependency `nowdocs = { path = ".", features = ["test-fixture"] }`, is automatically enabled during `cargo test` but not during `cargo build` -- giving a compile-time guarantee that test-only code is absent from production builds.

**Tests:**

- The production binary path (`main.rs` `Install`/`Update`) never calls `update()` or reads `NOWDOCS_TEST_URL` (verified by code inspection).
- The library API `install_to_staging` rejects `file://` URLs in production builds (the `test-fixture` feature is not enabled, so `is_test_mode()` returns `false` at compile time).
- `strings target/release/nowdocs | grep NOWDOCS_TEST_URL` returns 0 matches (test-only code is absent from the production binary).
- Existing `registry_tests.rs` tests using `file://` URLs still pass (they run as test binaries where `is_test_mode()` returns `true`).
- Existing `mcp_tests.rs` and `doctor_tests.rs` tests are NOT deleted - they do not use `file://` URLs and pass unchanged.

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

- Production binary path never reads `NOWDOCS_TEST_URL`; library API rejects `file://` when `is_test_mode()` is false.
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
- `cargo test -- --test-threads=1` passes.
- `cargo build` passes from clean.
- No new dependencies added.
- Production binary (`main.rs`) never calls `registry::update()` or reads `NOWDOCS_TEST_URL`; library API gates `file://` via `is_test_mode()`.
- Spec appendix G updated with correct versions.
