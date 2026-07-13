# Contributing to nowdocs

Thank you for your interest in contributing to nowdocs. This guide covers code, documentation, and registry-docset contributions.

## Developer Certificate of Origin (DCO), not a CLA

nowdocs uses the [Developer Certificate of Origin](https://developercertificate.org/) (DCO) and does not require a contributor license agreement (CLA).

- Every commit must include a `Signed-off-by:` line; `git commit -s` adds it automatically.
- By signing off, you certify that you have the right to submit the contribution.
- CI enforces this with `scripts/ci-check-dco.sh` and rejects unsigned commits.
- Project commit messages use the DCO sign-off only; do not add `Co-Authored-By` trailers.

## Code contribution workflow

1. Fork the repository.
2. Create a focused branch. Use a short description with a `feat/`, `fix/`, `docs/`, or `chore/` prefix.
3. Commit with `git commit -s`.
4. Push the branch to your fork.
5. Open a pull request against `main`.

### Local quality-gate setup

Install the local tools and hooks before making a code contribution:

```bash
cargo install cargo-deny --locked
cargo install cargo-audit --locked
pre-commit install
```

The two `cargo-audit` exceptions are documented in `deny.toml` and CI, each with a `Revisit-date` of 2026-10-10. They are time-limited, individually recorded exceptions. New advisories are not ignored automatically and must fail the gate until they receive a separate assessment.

### Quality gates (L1–L4)

| Level | Trigger | Checks |
|---|---|---|
| L1 Commit | Local `pre-commit`, seconds | `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo deny check`, `cargo audit` with only `RUSTSEC-2026-0194` and `RUSTSEC-2026-0195` ignored, and gitleaks secret scanning. Configure with `pre-commit install`. |
| L2 Push | Local `scripts/pre-push.sh`, minutes | `scripts/check.sh`: formatting, Clippy, and `cargo test -- --test-threads=1`, matching CI. Code changes of 15 or more lines on feature branches are required to run this gate; Markdown-only pushes are exempt. |
| L3 Pull request and CI | GitHub Actions `gates.yml` | DCO, formatting, Clippy, tests, release build, cargo-deny, manifest-schema validation, vector-artifact scanning, and license-audit unit tests. |
| L4 Weekly | `weekly-audit.yml`, Friday | `cargo audit` vulnerability scan and `cargo-udeps` dead-dependency scan. |

Run L1 and L2 before pushing. Bypassing hooks with `--no-verify` requires explicit maintainer approval.

## Documentation and docset contributions

To contribute a new registry docset:

1. Import it locally with `nowdocs ingest <dir> <name>`.
2. Create a share bundle with `nowdocs share <docset>`. The bundle contains **text and a manifest, never vectors**.
3. Submit it to the corresponding `nowdocs-registry/<docset>` repository.
4. A GWMM LLC curator reviews it using the checklist below.
5. Registry CI rebuilds vectors from the pinned model to prevent vector injection and model drift.

### Registry curation checklist

Before a docset is admitted, a GWMM LLC curator verifies:

- **Allowed licenses:** MIT, Apache-2.0, CC-BY-4.0, CC0, BSD, and ISC.
- **Disallowed sources:** proprietary documentation such as Clerk or Tailwind, sites whose terms prohibit scraping or redistribution, unauthorized scraped content, and ShareAlike licenses such as CC-BY-SA or GFDL.
- **Required manifest metadata:** `license` (SPDX), `copyright_holder`, and `attribution`.

See [DMCA.md](docs/DMCA.md) and [AUP.md](docs/AUP.md) for the full policy.

## Unsigned distribution

nowdocs release binaries are intentionally not code-signed. Integrity is provided by SHA-256 checksums attached to GitHub Releases and by `cargo-binstall` verification. Distribution uses `cargo-binstall` and Homebrew; contributors do not need a signing workflow.

## Code of Conduct and contact

By participating, you agree to follow the [Code of Conduct](CODE_OF_CONDUCT.md).

- Security vulnerabilities: see [.github/SECURITY.md](.github/SECURITY.md); do not open a public issue.
- General discussion: GitHub Discussions or Issues.
- Legal contact: `legal@gwmmai.com`.
