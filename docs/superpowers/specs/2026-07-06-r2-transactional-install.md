# R2 spec — transactional install/update with rollback

> **Parent track:** `docs/superpowers/specs/2026-07-06-robustness-ux.md`
> **Implementation plan:** `docs/superpowers/plans/2026-07-06-robustness-ux-plan.md`
> **Depends on:** R1 archive validation + error taxonomy
> **Stage:** R2 — fail-safe install/update state
> **Difficulty:** 6/10 (medium-high). Main review risk is preserving valid active state across failures and platforms.
> **Worktree rule:** implement from a dedicated git worktree and branch for R2, e.g. `../nowdocs-r2-transactional-install` on `fix/r2-transactional-install`.

---

## 0. Goal

R2 prevents partial active cache state during `install` and `update`. It builds on R1 validation by writing a complete candidate docset into staging, verifying it, then promoting it to the active cache only after validation succeeds.

The product requirement is fail-safe active state: bad archives, interrupted writes, or failed updates must not publish an invalid active docset.

---

## 1. Non-goals

- Do not change MCP protocol or transport.
- Do not implement `doctor`, `smoke`, cache CLI, or registry discovery.
- Do not change embedder fields or model.
- Do not change registry URL allowlist.
- Do not implement broad cache repair; only add staging/rollback primitives needed by R2.
- Do not require perfect cross-platform atomic directory replacement if fail-safe semantics are preserved.

---

## 2. Dispatch metadata

- **Estimated difficulty:** medium-high, 6/10.
- **Expected implementation time:** 1-2 days for a Rust maintainer; possibly 2-3 days for an external coding agent including review fixes.
- **Review risk:** high around destructive filesystem operations, rollback paths, Windows rename behavior, and not deleting valid active docsets.
- **Required isolation:** independent git worktree and stage-specific branch. Do not share a dirty worktree with R1/R3/U1.

---

## 3. Files in scope

Expected:

- `src/registry.rs`
- `src/cache.rs`
- `tests/registry_tests.rs`
- `tests/cli_tests.rs`

Allowed if helpful:

- `src/errors.rs` / `src/diagnostics.rs` if introduced by R1
- helper tests dedicated to staging/rollback

Out of scope:

- `src/mcp.rs`
- `src/embedder.rs`
- retrieval scoring changes
- README/user docs beyond tiny command output expectation fixes if unavoidable

---

## 4. Staging model

Add a staging root under the nowdocs cache root:

```text
~/.cache/nowdocs/staging/<docset>-<pid>-<timestamp>/
```

R2 should add helpers such as:

- `cache::staging_root()`
- `cache::new_staging_path(docset)` or equivalent
- internal rollback path helper for active replacement

Staging must always remain under `~/.cache/nowdocs`. Tests must assert invalid docset names cannot escape staging root.

---

## 5. Install-to-staging flow

R2 install flow:

1. validate docset name.
2. ensure cache layout.
3. download/open archive.
4. run R1 archive validation.
5. create unique staging directory.
6. materialize manifest/license/notice and LanceDB store under staging, not active paths.
7. reopen/verify staged manifest and store.
8. promote staged docset to active.
9. clean staging/rollback after success.

If any step before promote fails, active cache must remain untouched.

---

## 6. Promote/rollback contract

Promotion should prefer same-filesystem rename where possible. If replacing existing active data:

1. move active to rollback path.
2. move staged candidate to active.
3. verify active candidate.
4. remove rollback.

If promotion fails, restore rollback if possible and leave staging/rollback for later diagnostics. On Windows or platforms where replacing non-empty directories is not reliably atomic, use conservative copy-verify-swap while preserving the same product semantics.

---

## 7. Update contract

`update` must call the same staged install path as install. A failed update must preserve the previous working docset manifest/store.

Test this explicitly:

1. install or create a good active docset.
2. set update source to a bad archive.
3. run update and assert it fails.
4. assert previous manifest/store remains present and readable.

---

## 8. Required tests

- staging path stays under cache root.
- failed install with invalid archive creates no active manifest/store.
- successful install promotes manifest/store to active.
- failed update preserves old active manifest/store.
- rollback cleanup happens on successful replacement.
- stale staging detection primitive identifies leftover staging dirs.
- existing valid file-url install test still passes.
- registry URL allowlist behavior is unchanged.

---

## 9. Verification commands

```bash
cargo test registry_tests -- --test-threads=1
cargo test cli_tests -- --test-threads=1
cargo fmt --check
git diff --check
```

Manual check if feasible: install a valid fixture, attempt update from corrupt fixture, confirm old docset remains readable.

---

## 10. Done definition

R2 is done when:

- install writes no active files until the candidate passes validation and staging verification.
- failed install leaves no active docset.
- failed update preserves the previous active docset.
- rollback/staging paths are nowdocs-owned and discoverable by later `doctor` work.
- R3 can rely on stable staging/rollback conventions.
