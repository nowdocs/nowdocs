# Changelog

## [0.2.0] - 2026-07-17

### Added

- Added an agent-first machine contract with deterministic `capabilities`, read-only `status`, one-plan `setup plan` / `setup apply`, offline `verify`, and guarded `setup rollback` commands.
- Added managed setup adapters for Codex CLI, Claude Code, and Cursor. Claude Desktop returns MCPB guidance, while generic MCP clients receive deterministic manual configuration.
- Added registry-aware `ensure` planning plus best-effort, throttled reminders for newer nowdocs binaries and installed registry docsets.
- Added optional native Cohere reranking. It is disabled by default, transmits only disclosed, bounded search inputs when explicitly configured, and falls back to the local ranking on provider failure.
- Added signal-preserving retrieval evidence, calibrated answer states, evaluator reports, and stricter cross-platform automation CI gates.

### Security

- Separated read-only discovery and verification from explicit state-changing CLI actions; MCP remains read-only over stdio.
- Added approved-root permission validation, no-follow file handling, atomic configuration replacement, digest-guarded rollback, and one-shot rollback authorization.
- Client adapters fail closed on missing tools, unsafe paths, malformed or conflicting configuration, unsupported platforms, and ambiguous client state.

### Changed

- Agent setup now reports exact risk, network-access, confirmation, verification, and partial-reversibility boundaries in machine-readable results.
- Update checks are non-blocking, independently throttled per binary and registry channel, and can be disabled with `NOWDOCS_UPDATE_CHECK=0`.

## [0.1.2] - 2026-07-12

- Fix cargo-binstall discovery by excluding the internal registry builder from the default binary set.

## [0.1.1] - 2026-07-12

- Verified the local stdio MCP flow with protocol `2025-11-25`.
- Added trusted, SHA-256-checked registry artifacts for Next.js, React, and Vue.
- Verified `registry list`, install, update, cache status, list-installed, and Next.js smoke retrieval against the public catalog.
- Added registry artifact building from text-only share bundles, with pinned-model embeddings and canonical HTTPS source URLs.
- Supported distribution targets remain Linux musl (x86_64/aarch64), macOS (arm64/x86_64), and Windows (MSVC); final platform installation checks are still pending.
- Known limitations: English-first retrieval, fixed Candle/Jina embedder, no code signing, and only three curated canonical registry docsets.
