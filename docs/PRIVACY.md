# Privacy Policy

> Copyright (c) 2026 GWMM LLC.
> Last updated: July 4, 2026.

## Core commitment

nowdocs runs locally. Your query text, embeddings, and document content do not leave your device through the nowdocs software.

## Network activity

nowdocs accesses the network only when the user explicitly initiates one of the following actions:

| Action | Network access | Destination |
|---|---|---|
| `nowdocs ingest` for local documents | No | Reads only a local directory. |
| `nowdocs install` or `nowdocs update` | Yes | Trusted registry sources: `github.com/nowdocs-registry/*` and the reserved, currently inactive `registry.nowdocs.dev/*`. |
| First model-enabled command | One-time download | Hugging Face through `hf-hub`, to download the approximately 66 MB Jina v2 small model; later use reads the local cache. |

Cloning a source repository from GitHub or another host is a separate user action, not an action performed by `nowdocs ingest`.

When downloads occur, GitHub and Hugging Face receive standard connection metadata such as your IP address and User-Agent under their own privacy policies: [GitHub](https://docs.github.com/en/site-policy/privacy-policies) and [Hugging Face](https://huggingface.co/privacy). nowdocs adds no additional identifier to these requests.

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
| Application configuration | None. nowdocs does not create a configuration directory; MCP client configuration lives in the client's own `mcp.json`. |

## GDPR and CCPA

Because the nowdocs software does not collect personal data, GDPR and CCPA do not apply to the software itself. This statement is provided for transparency.

## Registry website privacy

The registry currently uses GitHub and is therefore subject to the [GitHub Privacy Statement](https://docs.github.com/en/site-policy/privacy-policies). If `registry.nowdocs.dev` later provides a website, it will publish a separate privacy notice.

## Changes

Policy changes are recorded in repository history. Material changes are announced in release notes.

## Contact

`legal@gwmmai.com`
