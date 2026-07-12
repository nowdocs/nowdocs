# Release Readiness Checklist

This checklist is the manual gate before claiming the robustness/UX hardening track is complete.

## Automated or local gates

Run in a fully provisioned environment with Rust 1.91+ and protobuf include files available:

```bash
cargo build
nowdocs doctor
nowdocs doctor --json
cargo test -- --test-threads=1
```

Required outcomes:

- `cargo build` succeeds on a clean checkout.
- `nowdocs doctor` exits successfully on a clean cache.
- `nowdocs doctor --json` emits parseable JSON.
- The full test suite passes single-threaded where environment-mutating tests require it.

## Install/update safety gates

Run with real or fixture archives:

- interrupted or bad install must not create an active docset;
- failed update must preserve the previous working docset;
- stale staging cleanup must not remove active `.lance` directories, manifests, models, or unrelated files.

Useful commands:

```bash
nowdocs cache status
nowdocs cache clean-staging --older-than 1h
nowdocs doctor --repair
```

## Smoke gate

Install or ingest a small local fixture and verify real retrieval:

```bash
nowdocs ingest ./tests/fixtures/golden release-smoke --license MIT --source-url https://example.com/release-smoke
nowdocs smoke release-smoke "authentication token error"
```

## Real-docset gate

Before a public release, run the expensive real-docset gate:

- ingest the approved large Next.js corpus;
- record file count, chunk count, ingest time, search latency, recall/MRR or approved replacement metric;
- run queries covering installation, routing, middleware, caching, server/client component boundaries, metadata, and config APIs;
- verify returned source URLs, headings, chunk indexes, scores, and sanitized text.

## MCP E2E gate

Verify stdio MCP with the real binary:

- initialize;
- list tools;
- call `nowdocs_list`;
- call `nowdocs_search` with an explicit `docset`;
- confirm results include useful sanitized text and metadata.

A real client check is preferred. A checked-in stdio E2E script is acceptable if the owner approves it as the release gate.

## Distribution gate

Only after the above pass:

- publish GitHub release artifacts;
- verify `cargo binstall nowdocs`;
- verify Homebrew tap install;
- update README from pre-release wording.

## v0.1.1 sign-off record

| Gate | Evidence | Owner | Status |
|---|---|---|---|
| Quality Gates | GitHub Quality Gates run 29178406917; local L1 hook green | Kaige Gao | PASS |
| Strict Eval | Next.js real gate: recall@5 0.900, MRR 0.725 | Kaige Gao | PASS |
| 5-target dry run | `.github/workflows/release.yml` workflow_dispatch path | Kaige Gao | PENDING |
| Registry install/update/smoke | Public catalog; Next.js/React/Vue install, Next.js update and smoke verified locally | Kaige Gao | PASS |
| cargo-binstall | v0.1.1 package metadata and archive contract tests | Kaige Gao | PENDING |
| Homebrew install | `nowdocs-registry/homebrew-nowdocs` formula still needs release SHA values | Kaige Gao | PENDING |
| crates.io publication | `cargo package` passes; publication not yet executed | Kaige Gao | PENDING |
