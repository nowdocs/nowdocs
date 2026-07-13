# nowdocs contributor guide

nowdocs is a local Rust MCP server that gives coding agents current third-party documentation through hybrid retrieval. This file is intentionally short: detailed implementation plans and agent dispatch material are kept outside the public repository.

## Project invariants

- MCP protocol version is `2025-11-25`; stdio transport uses NDJSON.
- `nowdocs serve` is stdio-only and must not bind a host or port.
- MCP search requires an explicit `docset`.
- All text and metadata returned to an LLM must pass the sanitizer.
- Registry packages contain text and manifests, never vectors; vectors are rebuilt from the pinned model.
- Do not hardcode credentials, tokens, keys, or private endpoints.

## Contributor workflow

- Read [CONTRIBUTING.md](CONTRIBUTING.md) and the public architecture overview in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).
- Keep changes focused and preserve unrelated working-tree files.
- Run the relevant formatter, tests, and security checks before committing.
- Use conventional commits and DCO sign-off (`git commit -s`).
- Do not push or merge without maintainer approval.
