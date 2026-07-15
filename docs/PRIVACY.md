# Privacy Policy

> Copyright (c) 2026 GWMM LLC.
> Last updated: July 15, 2026.

## Core commitment

nowdocs is local-first. Your query text, embeddings, and document content stay
on your device unless you explicitly enable native Cohere reranking.

## Network activity

nowdocs accesses the network only when the user explicitly initiates one of the following actions:

| Action | Network access | Destination |
|---|---|---|
| `nowdocs ingest` for local documents | No | Reads only a local directory. |
| `nowdocs install` or `nowdocs update` | Yes | Trusted registry sources: `github.com/nowdocs-registry/*` and the reserved, currently inactive `registry.nowdocs.dev/*`. |
| First model-enabled command | One-time download | Hugging Face through `hf-hub`, to download the approximately 66 MB Jina v2 small model; later use reads the local cache. |
| Binary version check after `install`, `update`, `ensure`, `registry`, `smoke`, or `doctor` | Yes, at most once every 24 hours | GitHub's official latest-release metadata at `api.github.com/repos/nowdocs/nowdocs/releases/latest`. The request sends only the standard `User-Agent: nowdocs/<version>` header GitHub requires and no user-specific or product-specific identifier. A failed attempt is rate-limited so repeated commands do not retry during an outage. |
| `nowdocs serve` or `nowdocs smoke` with native Cohere reranking explicitly enabled | Yes, per search | Cohere's native Rerank API. nowdocs sends the query and up to 40 sanitized, size-bounded candidate document strings. The API key is sent only in the required Authorization header, is never persisted by nowdocs, and is never included in logs, MCP output, or evaluator output. Embeddings, cache or filesystem paths, and provider relevance scores are not included in the document payload. |

`nowdocs serve` never performs a binary version check. It may display a previously discovered newer-version reminder from the local cache on stderr, but it never initiates a network request for that purpose.

Set the environment variable `NOWDOCS_UPDATE_CHECK=0` to disable all version checks and reminders. nowdocs never downloads or installs a binary update automatically; the reminder is informational only and directs you to the package manager you used to install nowdocs.

Cloning a source repository from GitHub or another host is a separate user action, not an action performed by `nowdocs ingest`.

When downloads occur, GitHub and Hugging Face receive standard connection metadata such as your IP address and User-Agent under their own privacy policies: [GitHub](https://docs.github.com/en/site-policy/privacy-policies) and [Hugging Face](https://huggingface.co/privacy). nowdocs adds no additional identifier to these requests.

Native Cohere reranking is disabled by default and uses your Cohere account and
key only after explicit configuration. Review Cohere's [Privacy Policy](https://cohere.com/privacy)
and the [native Cohere reranking configuration](../README.md#optional-native-cohere-reranking)
before enabling it.

## Data we do not collect

The nowdocs software collects no data:

- No telemetry.
- No analytics.
- No tracking.
- No user accounts.
- No hosted service.

The code contains no telemetry, analytics, or tracking calls.

If you contact `legal@gwmmai.com`, for example for a DMCA or Code of Conduct report, GWMM LLC processes the contact information in your message only to handle that matter.

## Local storage

| Data | Location |
|---|---|
| Docset data: Lance tables, manifests, and licenses | `~/.cache/nowdocs/db/`, or the platform-equivalent cache path |
| Embedder model cache | `~/.cache/nowdocs/models/` |
| Update check cache (latest version seen and reminder state) | `~/.cache/nowdocs/update-cache.json` |
| Application configuration | None. nowdocs does not create a configuration directory; MCP client configuration lives in the client's own `mcp.json`. |

## GDPR and CCPA

Because the nowdocs software does not collect personal data, GDPR and CCPA do not apply to the software itself. This statement is provided for transparency.

## Registry website privacy

The registry currently uses GitHub and is therefore subject to the [GitHub Privacy Statement](https://docs.github.com/en/site-policy/privacy-policies). If `registry.nowdocs.dev` later provides a website, it will publish a separate privacy notice.

## Changes

Policy changes are recorded in repository history. Material changes are announced in release notes.

## Contact

`legal@gwmmai.com`
