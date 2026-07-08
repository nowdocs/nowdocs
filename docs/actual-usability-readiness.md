# nowdocs Actual Usability Readiness Spec

> **Date:** 2026-07-07  
> **Scope:** P0/P1 work required before nowdocs can be described as practically usable by real coding agents and early users.  
> **Parent context:** Wave 0-5 are implemented; the remaining gap is real-world validation, first-run confidence, recovery, onboarding, and registry discoverability.  
> **Non-goals:** no MCP transport change, no network server, no configurable v1 embedder, no registry security relaxation, no release signing requirement.

---

## 0. Product-readiness definition

nowdocs is **actually usable** when a new user can:

1. install or build the binary;
2. install or ingest one real docset;
3. verify the local environment and docset health;
4. run one real retrieval smoke search;
5. connect an MCP client and complete an end-to-end `nowdocs_search` call;
6. recover safely from common partial/corrupt cache states without hand-editing cache directories;
7. understand supported registry docsets and next steps from CLI output and docs.

The current engine already supports ingest, embedding, hybrid retrieval, MCP tools, registry lifecycle, and CLI wiring. The remaining work is therefore **product hardening and proof**, not an engine rewrite.

---

## 1. P0 — must complete before claiming practical usability

### P0.1 Real large-docset retrieval validation

**Problem:** small fixtures and golden tests do not prove the product works on the intended workload: large, fast-moving framework documentation.

**Requirement:** validate retrieval quality and latency on at least one real large docset before release-readiness is declared.

Minimum acceptance:

- Ingest the rebuilt Next.js corpus or equivalent large real docset.
- Record file count, chunk count, ingest time, search latency, recall/MRR or a comparable query-set score.
- Include queries for installation, routing, middleware, caching, server/client component boundaries, metadata, and config APIs.
- Verify returned metadata is usable by LLM agents: source URL, heading path, chunk index, score, and sanitized text.
- Keep this as a dedicated manual or ignored expensive gate if it is too slow for normal CI.

### P0.2 End-to-end MCP client loop

**Problem:** library-level MCP tests do not prove a user can configure a real client and receive useful search results.

**Requirement:** run a full stdio MCP path with `initialize`, `tools/list`, and `nowdocs_search` against an installed or ingested docset.

Minimum acceptance:

- Exercise the actual `nowdocs serve` binary over stdio NDJSON.
- Confirm `nowdocs_search` requires `docset` and returns sanitized text plus metadata.
- Confirm `nowdocs_list` reflects installed docsets.
- Document a reproducible manual command or script for release verification.

### P0.3 First-run confidence command: `nowdocs smoke`

**Problem:** after install/ingest, users currently need to infer whether the docset is searchable.

**Requirement:** provide and document `nowdocs smoke <docset> [query] [--json] [--top-k <n>]` as a real retrieval smoke test.

Minimum acceptance:

- Validates docset name and installed state.
- Loads/uses the real embedder path.
- Runs the real retrieval pipeline.
- Prints result count, elapsed time, score, heading/source URL, chunk index, and preview.
- Exits non-zero for missing docsets, structurally invalid docsets, or zero results.
- JSON mode emits stable top-level keys for agents/CI.
- Install/update/ingest success output points to the next smoke command.

### P0.4 Safe cache inspection and recovery

**Problem:** practical users will hit interrupted installs, stale staging directories, partial updates, or corrupt manifests; manual cache deletion is too risky.

**Requirement:** provide safe cache visibility and cleanup commands.

Minimum acceptance:

- `nowdocs cache status [--json]` reports cache root, db/manifests/models/staging/rollback sizes, installed docset count, and staging count.
- `nowdocs cache clean-staging [--older-than <duration>]` removes only nowdocs-owned staging directories older than the threshold.
- `nowdocs doctor --repair` is wired to the same safe staging cleanup path.
- Repair never deletes active `.lance` stores, manifests, models, or unrelated directories.
- Human output states exactly what was removed; JSON output is available for `cache status`.

### P0.5 Release-readiness gates

**Problem:** product maturity can regress if readiness remains an informal judgment.

**Requirement:** define and execute a release gate checklist before calling the hardening track complete.

Minimum acceptance:

- `nowdocs doctor` passes on a clean checkout after build.
- `nowdocs doctor --json` is parseable.
- Bad/interrupted install does not create an active docset.
- Failed update preserves the previous working docset.
- `nowdocs smoke` works for a locally ingested fixture.
- Real-docset/model checks either pass in a dedicated script or are documented as manual release gates.
- Normal test suite passes with single-threaded tests where environment variables are mutated.

---

## 2. P1 — high-value usability improvements, not first-use blockers

### P1.1 Onboarding docs

Add:

- `docs/GETTING_STARTED.md` — install/build, install or ingest first docset, smoke, serve.
- `docs/TROUBLESHOOTING.md` — model download failures, docset not found, corrupt cache, MCP tools not visible, OS cache paths.
- `docs/MCP_CLIENTS.md` — Cursor, Claude Code, Claude Desktop, Aider, and generic MCP JSON snippets.
- README quickstart update that includes `doctor`, `smoke`, and MCP setup verification.

### P1.2 Registry discovery

Add `nowdocs registry list/search` once the registry index source of truth is settled.

Minimum behavior:

- Network access is explicit in help text.
- Output includes docset, version, license, chunk count, freshness, and install status.
- JSON output is available.
- URL policy still rejects non-nowdocs-registry downloads.

### P1.3 Better human CLI output

Improve CLI output for humans while preserving script compatibility where possible:

- `install`/`update`: show version, chunk count, license, next smoke command.
- `ingest`: show files, chunks, license/source summary, next smoke/share command.
- `share`: show output path, manifest/chunks presence, no-vector reminder, registry PR hint.
- `list-installed`: show docset, version, chunks, license, status.

### P1.4 Distribution polish

Before a broader public announcement:

- Publish prebuilt release artifacts.
- Verify `cargo binstall nowdocs` path.
- Verify Homebrew tap install path.
- Update README from pre-release wording once artifacts are live.

---

## 3. Suggested execution order

1. Confirm/finish `nowdocs smoke` and success-output improvements.
2. Implement cache status / safe staging cleanup / `doctor --repair`.
3. Run large-docset retrieval validation and MCP stdio E2E verification.
4. Update README + Getting Started + Troubleshooting + MCP clients.
5. Add registry discovery after index ownership/schema is settled.
6. Publish release artifacts only after P0 gates pass.

---

## 4. Open questions

1. What query set and threshold should define large-docset pass/fail for Next.js?
2. Should the real MCP E2E be a Rust integration test, a shell script, or both?
3. What is the registry index source of truth: GitHub release asset, raw GitHub file, or `registry.nowdocs.dev` mirror?
4. What is the default safe age threshold for `cache clean-staging` in public releases?

---

## 5. User-owned items the agent cannot finish alone

These are deliberately not marked complete until the project owner supplies decisions or runs manual release gates:

1. **Registry discovery decision:** choose and approve the registry index source of truth (`index.json` in GitHub release/raw file vs `registry.nowdocs.dev` mirror) and schema.
2. **Real large-docset assets:** provide or approve the exact Next.js corpus/query set and pass/fail thresholds.
3. **Real MCP client verification:** run at least one real client configuration outside unit tests, or approve a stdio E2E script as the release gate.
4. **Fully provisioned test environment:** run full build/test with Rust 1.91+ and protobuf includes available.
5. **Release distribution:** publish and verify cargo-binstall/Homebrew artifacts before changing README from pre-release wording.
