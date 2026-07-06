# nowdocs Robustness + UX hardening spec

> **Status:** design spec for the post-Wave-5 hardening track; product decisions confirmed on 2026-07-06.
> **Date:** 2026-07-06
> **Scope:** product robustness and user experience only. This spec does not change the MCP transport, embedder model, cache root, registry security boundary, licensing policy, or v1 English-only constraint from the project-level AGENTS.md / design review.

---

## 0. Executive summary

nowdocs has crossed the implementation milestone: the single binary can ingest, chunk, embed, store, retrieve, serve MCP tools, and run the registry lifecycle. The next maturity gap is not another engine rewrite; it is making the product **hard to break** and **easy to understand when it does break**.

This track makes two product promises:

1. **Robustness promise:** interrupted installs, corrupt cache state, broken archives, model download failures, invalid manifests, missing docsets, and MCP client mistakes fail safely with actionable diagnostics and no silent data corruption.
2. **UX promise:** a new user can install/ingest one docset, verify the environment, configure an MCP client, run a smoke search, and know what to do next without reading source code.

The design intentionally avoids high-risk architecture churn. We keep:

- MCP stdio NDJSON, no host/port.
- `nowdocs_search(query, docset, max_tokens?, top_k?)` with required `docset`.
- fixed `jinaai/jina-embeddings-v2-small-en` embedder for v1.
- `~/.cache/nowdocs` layout and schema/cache version 1.
- registry downloads restricted to nowdocs-registry-owned domains.
- share text-only; vectors are rebuilt by CI.

---

## 1. Current pain points

### 1.1 Robustness gaps

Observed or high-probability failure modes:

| Area | Failure mode | Current risk | Desired behavior |
|---|---|---|---|
| install/update | process interrupted after manifest write but before chunks/store materialize | partially installed docset appears installed | transactional staging + atomic promote; incomplete staging cleaned or reported |
| install archive | malformed tar, missing `chunks.jsonl`, huge file, duplicate entries, path traversal | confusing error or partial writes | strict archive validation before writes |
| registry archive | vectors accidentally included | local install may accept files CI would reject | local install rejects vector artifacts too |
| cache | missing `db/`, missing manifest, orphan `.lance`, corrupt manifest | list/search errors are hard to interpret | `doctor` classifies and suggests repair |
| model | offline first run, HF failure, SHA mismatch | user sees low-level error | explicit model status + remediation |
| MCP | invalid JSON, unsupported method, client calls search before install | hard to debug client setup | `doctor --mcp` and better error hints |
| CLI | generic `error: ...` | user cannot self-recover | typed error categories and next-step hints |
| eval | regressions in large docsets | quality can silently degrade | smoke and eval commands expose quality/latency |

### 1.2 UX gaps

| Flow | Current status | Gap |
|---|---|---|
| discover docsets | `list-installed` only | no registry discovery or installed status detail |
| verify install | user must infer success from command output | no `doctor`, no smoke query |
| MCP setup | README has one generic JSON snippet | no client-specific templates or validation |
| local ingest | CLI accepts metadata flags | no post-ingest summary with example next command |
| failure recovery | manual cache deletion | no guided repair command |
| sharing | works, but output is terse | no checklist summary for registry PR readiness |

---

## 2. Design principles

1. **Fail closed for safety-critical paths.** If install/update cannot prove the archive is valid, model spec is compatible, and chunks are parseable, it must not modify the active docset.
2. **Never hide a partial state.** Partial installs live under staging paths, not active cache paths.
3. **Diagnostics before mutation.** `doctor` defaults to read-only; destructive repair requires explicit `--repair`.
4. **Actionable errors.** Every user-facing error should answer: what failed, why it matters, and one next command to try.
5. **Machine-readable plus human-readable.** UX commands support stable JSON output for agents/CI and readable tables for humans.
6. **No telemetry.** Diagnostics inspect only local filesystem/process state and do not upload anything.
7. **No network surprise.** Commands that may touch network must say so in help text and have offline-safe modes where practical.
8. **Keep v1 constraints.** Do not add configurable embedders, hosted search, a daemon, or a network server in this track.

---

## 3. New CLI surface

### 3.1 `nowdocs doctor`

Primary command for environment and cache diagnostics.

```bash
nowdocs doctor [--json] [--docset <name>] [--mcp] [--model] [--repair]
```

Default checks:

- binary version and platform.
- cache root exists and is writable.
- cache layout version is supported.
- installed docsets have manifest + store + chunk metadata.
- manifests validate against schema and locked embedder fields.
- no staging directories older than a grace threshold.
- registry URL policy is configured as expected.

Optional checks:

- `--docset <name>`: deep check one docset, including table open and sample retrieval metadata.
- `--model`: check model files, revision pin, SHA-256, tokenizer/config presence; may avoid download unless `--ensure` is added later.
- `--mcp`: run an in-process MCP initialize/tools-list smoke path without requiring an external client.
- `--json`: emit JSON with check IDs, severity, status, message, and remediation. The top-level shape is intentionally small, but the JSON contract is **experimental until v1.0**.
- `--repair`: remove stale staging/rollback directories only in this track. It must never delete an active docset; active deletion remains the job of explicit `uninstall` or a future separately gated repair command.

Severity levels:

| Severity | Meaning | Exit code |
|---|---|---|
| ok | all required checks pass | 0 |
| warn | product works but user should fix something | 0 |
| fail | a core flow will not work | 1 |

### 3.2 `nowdocs smoke`

Single-command confidence test for an installed docset.

```bash
nowdocs smoke <docset> [query] [--json] [--top-k <n>]
```

Behavior:

- validates docset name.
- verifies docset is installed.
- embeds the query.
- runs retrieve pipeline.
- prints top results with score, heading, source URL, chunk index, and elapsed time.
- exits non-zero if there are no results or if the docset is structurally invalid.

Default query should be generic but useful: `installation configuration example`. README examples should prefer docset-specific queries. `smoke` is a **real retrieval smoke test**: it may load the local embedder and, on first run, trigger the existing model download path. If model setup fails, the error must point users to `nowdocs doctor --model`.

### 3.3 `nowdocs cache`

Cache inspection and safe cleanup namespace.

```bash
nowdocs cache status [--json]
nowdocs cache clean-staging [--older-than <duration>]
```

`cache status` reports:

- cache root path.
- total size by category: db, manifests, models, staging.
- installed docset count.
- stale staging count.

`cache clean-staging` removes only staging paths matching nowdocs-owned naming and older than the threshold.

### 3.4 `nowdocs registry list/search` (UX phase, network-aware)

Registry discovery is important but depends on registry index availability. For this track, design it as phase-2 UX:

```bash
nowdocs registry list [--json]
nowdocs registry search <term> [--json]
```

Constraints:

- source of truth is a signed-off, reviewed `index.json` in the nowdocs-registry GitHub organization or `registry.nowdocs.dev` mirror.
- command help must state it may access the network.
- output includes docset, version, license, chunk count, freshness, and install status.

If registry index is not ready, this remains a plan item, not a blocker for `doctor` and `smoke`.

---

## 4. Transactional install/update design

### 4.1 Active and staging paths

Active paths remain unchanged:

- `~/.cache/nowdocs/db/<docset>.lance`
- `~/.cache/nowdocs/manifests/<docset>.json` or current manifest path helper
- any existing nowdocs-owned metadata paths

Staging path:

```text
~/.cache/nowdocs/staging/<docset>-<pid>-<timestamp>/
```

Install/update writes everything to staging first:

1. validate docset name.
2. download archive to staging or open test file URL.
3. parse and validate archive entries in memory or bounded temp files.
4. parse manifest and validate schema/model/legal/source fields.
5. parse `chunks.jsonl`; enforce chunk count match when manifest declares it.
6. reject vector artifacts and unsafe paths.
7. materialize LanceDB store under staging.
8. write manifest/license/notice under staging.
9. final verification: reopen staged store and manifest.
10. promote staging to active with fail-safe semantics:
    - same-filesystem rename when possible.
    - if replacing existing docset, rename old active to rollback path first.
    - on Windows or other platforms where non-empty directory replacement is not reliably atomic, use conservative copy-verify-swap.
    - on failure, restore old active if possible and leave staging for `doctor`.

The product requirement is stronger than the implementation mechanism: a bad or interrupted install/update must not publish an invalid active docset. Perfect cross-platform atomic directory replacement is not required for the first implementation if fail-safe active-state semantics hold.

### 4.2 Archive validation rules

Before active writes, reject archives with:

- missing `manifest.json`.
- missing `chunks.jsonl` for registry docset archives.
- absolute paths.
- `..` path components.
- duplicate security-sensitive entries (`manifest.json`, `chunks.jsonl`, `LICENSE`, `NOTICES`).
- symlinks/hardlinks/device entries.
- vector artifacts: `.lance`, `.faiss`, `vectors.*`, `embeddings.*`.
- files over configured per-entry and total-size limits.
- invalid UTF-8 for manifest/chunks metadata.

### 4.3 Update rollback

`update` must preserve the old docset until the new one passes verification. If promotion fails:

- active old docset remains available, or is restored from rollback.
- command exits non-zero with a remediation message.
- stale staging/rollback path is reported by `doctor`.

### 4.4 Idempotency

Repeated install/update of the same version should be safe:

- if active manifest matches incoming manifest hash, command can short-circuit or replace cleanly.
- no duplicate docset listing.
- stale staging cleanup does not affect active docsets.

---

## 5. Error taxonomy and messages

Introduce an internal error classification layer without forcing a public Rust API commitment.

Suggested categories:

| Category | Examples | User remediation |
|---|---|---|
| Input | invalid docset, empty query, bad flag combination | show valid format/example |
| Network | curl/download failure, DNS, timeout | retry, proxy hint, offline note |
| Archive | malformed tar, missing required file, unsafe path | report bad archive and source URL |
| Manifest | schema/model/license mismatch | show field and expected value |
| Cache | permission denied, corrupt active docset, stale staging | run `nowdocs doctor`, maybe repair |
| Model | missing files, SHA mismatch, config rejected | retry model download / clear model cache |
| Retrieval | no results, store open failure | run `nowdocs doctor --docset` |
| MCP | invalid JSON-RPC, unsupported method, tool args invalid | show client config hint |
| Internal | unexpected bug | include concise debug context and ask for issue |

Human message format:

```text
error[ARCHIVE_MISSING_CHUNKS]: registry archive is missing chunks.jsonl
why: nowdocs cannot build a searchable local docset without chunk text
next: retry install, or report the broken registry release
```

JSON message format for future commands:

```json
{
  "code": "ARCHIVE_MISSING_CHUNKS",
  "category": "archive",
  "message": "registry archive is missing chunks.jsonl",
  "hint": "retry install, or report the broken registry release"
}
```

---

## 6. UX output contracts

### 6.1 Install/update success output

Human output should include:

- docset name and version.
- chunk count.
- source URL / entry URL.
- license and attribution presence.
- next commands:
  - `nowdocs smoke <docset> "..."`
  - `nowdocs serve`

Example:

```text
installed nextjs-docs 15.3.4
chunks: 7480
license: MIT
next: nowdocs smoke nextjs-docs "How do I configure middleware?"
next: nowdocs serve
```

### 6.2 Ingest success output

Include:

- files and chunks.
- manifest path.
- license summary.
- warning if source URL or attribution is empty.
- next commands: smoke and share.

### 6.3 List-installed output

Current comma-separated output is acceptable for scripts but weak for users. Upgrade to a table:

```text
DOCSET       VERSION   CHUNKS   LICENSE      STATUS
nextjs-docs  15.3.4    7480     MIT          ok
react-docs   19.0.0    4360     CC-BY-4.0   ok
```

Add `--json` later if needed by agents.

### 6.4 Share output

After `share`, print registry PR readiness:

- output path.
- included files.
- vector scan result.
- license/attribution status.
- manifest validation status.

---

## 7. MCP setup UX

Docs must include client-specific snippets for:

- Cursor.
- Claude Code.
- Claude Desktop.
- Aider.
- generic MCP JSON.

Add a “verify MCP setup” section:

1. Run `nowdocs doctor --mcp`.
2. Configure client.
3. Ask the client to list tools.
4. Ask a known docset question.

The server must remain stdio-only. No `--host`, no `--port`, no HTTP listener in this track.

---

## 8. Observability without telemetry

Local-only observability is allowed and encouraged:

- `--verbose` for CLI commands where useful.
- elapsed time in `smoke`.
- optional local logs under cache only if user opts in later.
- no automatic upload, no analytics, no tracking.

`doctor --json` is the primary machine-readable observability interface.

---

## 9. Testing requirements

Minimum test additions for this track:

### 9.1 Unit tests

- archive path traversal rejection.
- duplicate manifest/chunks rejection.
- vector artifact rejection.
- missing chunks rejection.
- error code formatting.
- cache status classification.
- doctor JSON schema shape.

### 9.2 Integration tests

- interrupted/failed install leaves no active docset.
- update rollback preserves old manifest/store.
- stale staging cleanup removes only staging directories.
- smoke returns non-zero for missing docset.
- smoke returns results for fixture docset.
- CLI output includes next-step hints.

### 9.3 E2E/manual gates

- `nowdocs doctor` on clean machine.
- `nowdocs ingest` fixture then `nowdocs smoke`.
- `nowdocs serve` MCP initialize/tools-list/tools-call using stdio.
- real Next.js large docset smoke queries before release.

---

## 10. Non-goals

This track does not include:

- hosted registry service beyond consuming an index.
- HTTP MCP transport.
- daemon/background service.
- configurable embedding model.
- CJK tokenizer work.
- vector indexes beyond v1 flat exact search.
- crawler integration beyond documenting external Markdown ingest.
- telemetry or remote diagnostics.

---

## 11. Confirmed decisions and remaining open questions

### 11.1 Confirmed decisions

1. **First implementation slice:** ship R1 → R2 → R3 → U1 first: archive/error hardening, transactional install/update, read-only doctor diagnostics, and real `smoke` search. Defer cache repair polish, onboarding docs, and registry discovery until that foundation is working.
2. **Repair boundary:** `doctor --repair` only removes stale nowdocs-owned staging/rollback state in this track. It must not delete active docsets.
3. **JSON stability:** `doctor --json` exists early, but its JSON contract is experimental until v1.0 except for the small top-level status/checks shape.
4. **Smoke semantics:** `nowdocs smoke` runs a real retrieval path by default, including embedder load and possible first-run model download.
5. **Transactional install semantics:** prioritize fail-safe active state over perfect cross-platform atomic directory replacement. Unix should use rename where possible; Windows may use copy-verify-swap if necessary.

### 11.2 Remaining open questions

1. **Archive size limits:** choose defaults for max total archive size and max per-entry size. Defaults should fit large docsets like Next.js while preventing accidental huge installs.
2. **Registry index timing:** decide the exact `index.json` hosting and schema before implementing `registry list/search`.
