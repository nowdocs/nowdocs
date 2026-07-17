# Privacy Policy

> Copyright (c) 2026 GWMM LLC.
> Last updated: July 16, 2026.

## Core commitment

nowdocs is local-first. Your query text, embeddings, and document content stay on your device unless you explicitly enable native Cohere reranking.

## Network activity

nowdocs accesses the network only when the user explicitly initiates one of the following actions:

| Action | Network access | Destination |
|---|---|---|
| `nowdocs ingest` for local documents | No | Reads only a local directory. |
| `nowdocs install` or `nowdocs update` | Yes | Trusted registry sources: `github.com/nowdocs-registry/*` and the reserved, currently inactive `registry.nowdocs.dev/*`. |
| `nowdocs ensure --online`, `nowdocs setup plan --online`, or an approved `setup apply` that installs a docset | Yes | The same trusted registry sources. Offline planning, `status`, and `verify` do not use the network. |
| An explicitly download-enabled model command, such as `nowdocs doctor --model` or `nowdocs smoke` | One-time download | Hugging Face through `hf-hub`, to download the approximately 66 MB Jina v2 small model; later use reads the local cache. |
| Binary version check after `install`, `update`, `ensure`, `registry`, `smoke`, or `doctor` | Yes, at most once every 24 hours | GitHub's official latest-release metadata at `api.github.com/repos/nowdocs/nowdocs/releases/latest`. The request sends only the standard `User-Agent: nowdocs/<version>` header GitHub requires and no user-specific or product-specific identifier. A failed attempt is rate-limited so repeated commands do not retry during an outage. |
| `nowdocs serve` or `nowdocs smoke` with native Cohere reranking explicitly enabled | Yes, per search | Cohere's native Rerank API. nowdocs sends the query and up to 40 sanitized, size-bounded candidate document strings. The API key is sent only in the required Authorization header, is never persisted by nowdocs, and is never included in logs, MCP output, or evaluator output. Embeddings, cache or filesystem paths, and provider relevance scores are not included in the document payload. |

`nowdocs serve` never performs a binary version check. It may display a previously discovered newer-version reminder from the local cache on stderr, but it never initiates a network request for that purpose.

`nowdocs verify` is read-only and cached-only. It never downloads the embedding model; a missing model is reported as an action the user must initiate separately.

Set the environment variable `NOWDOCS_UPDATE_CHECK=0` to disable all version checks and reminders. nowdocs never downloads or installs a binary update automatically; the reminder is informational only and directs you to the package manager you used to install nowdocs.

Cloning a source repository from GitHub or another host is a separate user action, not an action performed by `nowdocs ingest`.

When downloads occur, GitHub and Hugging Face receive standard connection metadata such as your IP address and User-Agent under their own privacy policies: [GitHub](https://docs.github.com/en/site-policy/privacy-policies) and [Hugging Face](https://huggingface.co/privacy). nowdocs adds no additional identifier to these requests.

Native Cohere reranking is disabled by default and uses your Cohere account and key only after explicit configuration. A failed request may already have transmitted its input even though nowdocs falls back to the unchanged local ranking. Review Cohere's [Privacy Policy](https://cohere.com/privacy) and the [native Cohere reranking guide](RERANKING.md) before enabling it.

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
| Agent automation plans, locks, operation journals, and rollback records | `~/.cache/nowdocs/automation/` |
| Update check cache: latest version seen and reminder state | `~/.cache/nowdocs/update-cache.json` |
| MCP client configuration | Owned by the client. An explicitly approved setup may conditionally add a nowdocs entry through a safe client adapter. nowdocs never stores credentials in its automation records. |

Automation plans are private, local, and expiring. Operation records exist to support guarded rollback of an owned client configuration change and contain only the information needed to identify that change. Setup rollback does not uninstall or downgrade a docset. A successful rollback consumes its setup-owned authorization and cannot be replayed against a user-recreated registration.

Codex CLI and Claude Code configuration are managed only through their official MCP CLI commands. Cursor changes stay under the approved user root and use no-follow path checks, operation-owned backup data, and digest-guarded rollback. Claude Desktop and generic adapters do not write client configuration.

## GDPR and CCPA

Because the nowdocs software does not collect personal data, GDPR and CCPA do not apply to the software itself. This statement is provided for transparency.

## Registry website privacy

The registry currently uses GitHub and is therefore subject to the [GitHub Privacy Statement](https://docs.github.com/en/site-policy/privacy-policies). If `registry.nowdocs.dev` later provides a website, it will publish a separate privacy notice.

## Changes

Policy changes are recorded in repository history. Material changes are announced in release notes.

## Contact

`legal@gwmmai.com`
