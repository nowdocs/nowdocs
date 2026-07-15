# nowdocs

> A local, single-binary MCP server that gives coding agents current third-party documentation.

Coding agents can confidently suggest APIs that have changed since their training data was collected. nowdocs indexes documentation on your machine and exposes it through MCP, so clients such as Claude Code, Cursor, and Aider can search current documentation instead of relying only on model memory.

![Architecture overview: documentation sources are ingested, embedded, indexed locally, retrieved through hybrid search, sanitized, and served to MCP clients over stdio.](docs/assets/architecture.png)

**Current release:** [v0.1.2](https://github.com/nowdocs/nowdocs/releases/tag/v0.1.2). nowdocs is free to run, has no telemetry, and by default keeps queries, embeddings, and indexed documentation on your device.

## Why nowdocs

- **Local-first by default:** query text, embeddings, and document content stay on your machine unless you opt in to native Cohere reranking.
- **Hybrid retrieval:** semantic search, BM25 full-text search, and reciprocal-rank fusion (RRF).
- **MCP over stdio:** no listening port, host, or public service to configure.
- **Curated registry:** start with current Next.js, React, and Vue docsets, or ingest local Markdown documentation.
- **One Rust binary:** prebuilt releases for macOS, Linux musl, and Windows.

## Install

### Prebuilt binary

```bash
# Recommended: Cargo binstall verifies GitHub Release checksums.
cargo binstall nowdocs

# macOS or Linux through the Homebrew tap.
brew tap nowdocs-registry/nowdocs
brew install nowdocs
```

### Build from source

```bash
cargo install nowdocs
# or use the current repository checkout
cargo build --release
```

Source builds require a compatible Rust toolchain, `protoc`, and `curl` on `PATH`.

- macOS: `brew install protobuf`
- Debian/Ubuntu: `sudo apt-get install protobuf-compiler`

The first model-enabled command downloads the Apache-2.0 `jina-embeddings-v2-small-en` model (about 66 MB) from Hugging Face and then caches it locally. Run `nowdocs doctor --model` before your first search to make that download explicit.

## Five-minute quick start

This path installs the curated Next.js docset, verifies retrieval, and starts the MCP server.

```bash
# 1. Check the local environment and download the model if it is missing.
nowdocs doctor --model

# 2. Install a curated docset.
nowdocs install nextjs

# 3. Confirm that retrieval returns useful documentation.
nowdocs smoke nextjs "middleware matcher configuration"

# 4. Start the local MCP server.
nowdocs serve
```

`serve` uses newline-delimited JSON over stdio. It never binds a host or port.

Register the server with an MCP client using this generic configuration:

```json
{
  "mcpServers": {
    "nowdocs": {
      "command": "nowdocs",
      "args": ["serve"]
    }
  }
}
```

Client-specific configuration for Cursor, Claude Code, Claude Desktop, and Aider is in [MCP Clients](docs/MCP_CLIENTS.md).

## Optional native Cohere reranking

Reranking is disabled by default. When you explicitly enable native Cohere
reranking, nowdocs sends each search query and up to 40 sanitized, size-bounded
candidate document strings to Cohere using your account. Disabled mode remains
local-only. `nowdocs smoke` uses the same configured reranker and can also make
a request under your Cohere account. If a remote request fails, nowdocs falls
back to local ranking for that search, although the failed request may already
have transmitted its input.

Configure the environment of the process that starts `nowdocs serve`:

```bash
export NOWDOCS_RERANK_PROVIDER=cohere
export NOWDOCS_RERANK_MODEL=rerank-v3.5
export COHERE_API_KEY='your-cohere-api-key'
# Optional; defaults to 2000 and must be an integer from 100 through 10000.
export NOWDOCS_RERANK_TIMEOUT_MS=2000
nowdocs serve
```

nowdocs does not persist the key. `COHERE_API_KEY` is sent only in Cohere's
required Authorization header; nowdocs never includes it in logs, MCP output,
or evaluator output. A partial or invalid opt-in is a startup error rather than
a silent fallback.

The integration accepts any model identifier that Cohere's native v2 Rerank
endpoint accepts, subject to your account access. These are non-binding
examples:

| Example identifier | Use when |
|---|---|
| `rerank-v3.5` | You already use this Cohere Rerank model or want an established baseline to evaluate. |
| `rerank-v4.0-fast` | Interactive latency and throughput matter most. |
| `rerank-v4.0-pro` | You prioritize ranking quality for more complex workloads. |

See Cohere's [Rerank model documentation](https://docs.cohere.com/docs/rerank)
for current model availability and account terms. OpenRouter credentials and
OpenRouter model slugs are not supported by this native Cohere integration.

Cohere changes only the candidate order before nowdocs applies its local MMR
diversity ranking. nowdocs does not expose or use Cohere relevance scores as an
answer-confidence score, and this optional integration does not promise a
quality improvement.

## Common workflows

| Goal | Command |
|---|---|
| List registry docsets | `nowdocs registry list` |
| Install a curated docset | `nowdocs install <docset>` |
| Import local Markdown | `nowdocs ingest <dir> <name> --license MIT --source-url <url>` |
| Verify retrieval | `nowdocs smoke <docset> [query]` |
| Start the MCP server | `nowdocs serve` |
| List installed docsets | `nowdocs list-installed` |
| Update a docset | `nowdocs update <docset>` |
| Rebuild a local cache | `nowdocs rebuild <docset>` |
| Diagnose or safely repair setup | `nowdocs doctor [--model] [--repair]` |
| Inspect the cache | `nowdocs cache status` |

Use `nowdocs ingest` when you own or are allowed to use the source material. For CC-BY-4.0 documentation, supply the required `--attribution` value. Use `nowdocs share <docset>` to create a text-and-manifest contribution bundle; it intentionally excludes vectors.

## Documentation

- [Getting Started](docs/GETTING_STARTED.md) — installation, ingest, smoke testing, and recovery.
- [MCP Clients](docs/MCP_CLIENTS.md) — client-specific configuration and verification.
- [Troubleshooting](docs/TROUBLESHOOTING.md) — model, cache, registry, MCP, and source-build failures.
- [Architecture](docs/ARCHITECTURE.md) — data flow and security boundaries.
- [Contributing](CONTRIBUTING.md) — code and docset contribution workflow.

## Security and privacy

MCP exposes only the read-only `nowdocs_search` and `nowdocs_list` tools. Commands that modify local state, such as `install`, `ingest`, and `uninstall`, are CLI-only and require an explicit user action.

Before documentation reaches an LLM, nowdocs sanitizes returned text and metadata to reduce prompt-injection content. Registry downloads are restricted to trusted registry releases and verified with SHA-256. Shared docsets contain text and manifests only; registry CI rebuilds vectors with the pinned model.

See the [Privacy Policy](docs/PRIVACY.md), [Threat Model](docs/THREAT_MODEL.md), and [Security Policy](.github/SECURITY.md) for details.

After a successful `install`, `update`, `ensure`, `registry`, `smoke`, or `doctor` command, nowdocs checks GitHub for a newer binary release at most once every 24 hours and prints a reminder to stderr. It never downloads or installs an update automatically. Set `NOWDOCS_UPDATE_CHECK=0` to disable the check.

## Current scope and limitations

- The curated registry currently provides Next.js, React, and Vue docsets.
- Retrieval is English-first and uses the fixed Candle/Jina embedding backend.
- The Next.js real-corpus evaluation gate currently reports recall@5 of 0.900 and MRR of 0.720. It does not represent accuracy for every docset or query.
- Releases are not code-signed. Verify release checksums; `cargo-binstall` does this automatically.
- Five platform release assets are built and checksum-verified. Homebrew CLI installation should still be rechecked on a machine with Homebrew available.

## Contributing and policies

nowdocs is licensed under `MIT OR Apache-2.0`. Contributions use the Developer Certificate of Origin (DCO), not a CLA. See [CONTRIBUTING.md](CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

The public registry is curated and accepts only documentation whose license permits redistribution. Review the [Acceptable Use Policy](docs/AUP.md), [DMCA Policy](docs/DMCA.md), [Trademark Policy](docs/TRADEMARK.md), and [NOTICE](NOTICE).

Do not report security vulnerabilities in a public issue. Use GitHub's private vulnerability-reporting flow or email `legal@gwmmai.com` with `[nowdocs security]` in the subject line.
