# Security Policy

## Reporting a Vulnerability

**Do NOT open a public GitHub issue for security vulnerabilities.**
Public disclosure before a fix is available puts every deployed nowdocs
user at risk.

Instead, report vulnerabilities through GitHub's private channel:

- Go to the repository **Security** tab → **Report a vulnerability**
  (GitHub Security Advisories). This is a private channel visible only to
  repository maintainers.

If GitHub Security Advisories are unavailable to you, email
**legal@gwmmai.com** with `[nowdocs security]` in the subject line.

## What to Include

- A description of the vulnerability and its impact
- Steps to reproduce (proof-of-concept)
- Affected versions, if known
- Whether you have a proposed fix

## Response Timeline

- We acknowledge receipt within **3 business days**.
- We aim to publish a fix within **30 days** for high-severity issues,
  longer for lower severity.
- We credit reporters in the public advisory unless they prefer to remain
  anonymous.

## Scope

nowdocs is a local single-binary MCP server. Its network surface is limited
(see the Privacy section in the README): registry docset downloads
(`github.com/nowdocs-registry/*`, `registry.nowdocs.rs/*`) and the one-time
embedder model download from HuggingFace. Vulnerabilities in these paths,
in the MCP stdio protocol handling, in path-traversal / docset-name
validation, or in the cache layout are in scope.

## Supported Versions

Only the latest released version receives security fixes.
