# Changelog

## [0.1.1] - 2026-07-12

- Verified the local stdio MCP flow with protocol `2025-11-25`.
- Added trusted, SHA-256-checked registry artifacts for Next.js, React, and Vue.
- Verified `registry list`, install, update, cache status, list-installed, and Next.js smoke retrieval against the public catalog.
- Added registry artifact building from text-only share bundles, with pinned-model embeddings and canonical HTTPS source URLs.
- Supported distribution targets remain Linux musl (x86_64/aarch64), macOS (arm64/x86_64), and Windows (MSVC); final platform installation checks are still pending.
- Known limitations: English-first retrieval, fixed Candle/Jina embedder, no code signing, and only three curated canonical registry docsets.
