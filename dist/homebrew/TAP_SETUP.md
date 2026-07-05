# Homebrew Tap Setup — nowdocs

> **Status (2026-06-30)**: Formula written, tap repo NOT YET created. This document
> is the operator manual for whoever creates the `nowdocs-registry/homebrew-nowdocs`
> repo and wires the release pipeline. See **Open Questions** at the bottom for
> the manual actions still required.

This document covers three things:

1. **Tap repo structure** — where the formula lives, how the tap is named.
2. **One-time setup** — creating the tap repo and seeding it with the formula.
3. **Per-release update flow** — how `version` + `sha256` get bumped on each
   `nowdocs` GitHub Release. Strongly recommend a CI job for this (recipe below);
   manual flow also documented as fallback.

---

## 1. Tap repo structure

Homebrew tap convention: `<owner>/homebrew-<short-name>`. The user-facing tap
command drops the `homebrew-` prefix automatically.

| Item | Value |
|---|---|
| GitHub repo (to create) | `nowdocs-registry/homebrew-nowdocs` |
| Tap command (user-facing) | `brew tap nowdocs-registry/nowdocs` |
| Install command | `brew install nowdocs` |
| Formula path inside repo | `Formula/nowdocs.rb` |
| Formula source (this repo) | `dist/homebrew/nowdocs.rb` |

The `nowdocs-registry` owner is the same one that hosts per-docset repos
(`nowdocs-registry/<docset>`) — see registry task (OQ-2). **The tap repo is the
only `nowdocs-registry/*` repo that ships a binary, not a docset.** All other
`nowdocs-registry/*` repos hold docset tarballs consumed by `nowdocs install`.

> The main `nowdocs/nowdocs` repo (where this `dist/homebrew/` lives) is the
> binary source. The tap repo is a thin mirror of the formula file with
> per-release `version` + `sha256` pins. Don't try to make Homebrew fetch from
> the main repo directly — the tap needs its own copy so `brew audit` and
> `brew bump` can run cleanly.

---

## 2. One-time setup

```bash
# 1. Create the tap repo on GitHub under the nowdocs-registry org.
#    (https://github.com/organizations/nowdocs-registry/repositories/new)
#    Name: homebrew-nowdocs
#    Visibility: public
#    No README, no .gitignore, no license (Homebrew provides its own README template — see below)

# 2. Clone it locally.
gh repo clone nowdocs-registry/homebrew-nowdocs
cd homebrew-nowdocs

# 3. Copy in the formula (from the main repo's dist/homebrew/ nowdocs.rb).
#    At the time of tap creation, the main repo's file has TODO_FILL_AFTER_RELEASE
#    placeholders for sha256 — that's expected; the first real bump happens on
#    the first release (see §3).
mkdir -p Formula
cp /path/to/nowdocs/dist/homebrew/nowdocs.rb Formula/nowdocs.rb

# 4. Add a README so `brew tap` shows something useful.
cat > README.md <<'EOF'
# nowdocs Homebrew Tap

Install nowdocs (local MCP server for third-party dev docs) on macOS:

```sh
brew tap nowdocs-registry/nowdocs
brew install nowdocs
```

Upgrades:

```sh
brew update
brew upgrade nowdocs
```

Issues: https://github.com/nowdocs/nowdocs/issues
EOF

# 5. Commit + push.
git add Formula/nowdocs.rb README.md
git commit -m "feat(tap): seed with nowdocs formula"
git push origin main
```

After this, any macOS user with Homebrew can run:

```sh
brew tap nowdocs-registry/nowdocs
brew install nowdocs
nowdocs --version   # should print 0.1.0 (or whatever version was pinned)
```

> **Why `TODO_FILL_AFTER_RELEASE` placeholders are OK for the seed commit:**
> The formula won't actually `brew install` successfully until real sha256 values
> are filled in. The seed commit establishes the repo layout; the first
> successful install happens only after the first real release + the first
> bump PR (see §3).

---

## 3. Per-release update flow

Each time a new `nowdocs` GitHub Release is published (e.g. `v0.2.0`), the
formula's `version` and the two `sha256` lines (arm64 + x86_64 macOS assets)
need to be updated in the tap repo. **Two options — pick one.**

### Option A: CI auto-bump (recommended)

A workflow in the `nowdocs-registry/homebrew-nowdocs` repo, triggered on
release:

```yaml
# .github/workflows/bump.yml (lives in the tap repo, not the main repo)
name: Bump formula on release

on:
  repository_dispatch:
    types: [nowdocs-release-published]
  workflow_dispatch:
    inputs:
      version:
        description: "nowdocs version to bump to (e.g. 0.2.0)"
        required: true

jobs:
  bump:
    runs-on: ubuntu-latest
    permissions:
      contents: write
      pull-requests: write
    steps:
      - uses: actions/checkout@v4

      - name: Set up Ruby
        uses: ruby/setup-ruby@v1
        with:
          ruby-version: "3.3"
          bundler-cache: true

      # Triggered by main-repo release workflow via repository_dispatch.
      # Computes sha256 for both macOS assets and opens a PR.
      - name: Compute sha256 and open PR
        env:
          VERSION: ${{ github.event.inputs.version || github.event.client_payload.version }}
        run: |
          set -euo pipefail
          VERSION="${VERSION:?version required}"
          # github.ref_name arrives as 'v0.2.0'; asset URLs use the bare
          # version (nowdocs-v0.2.0-...), so strip the leading 'v'.
          VERSION="${VERSION#v}"
          BASE="https://github.com/nowdocs/nowdocs/releases/download/v${VERSION}"
          ARM_TGZ="nowdocs-v${VERSION}-aarch64-apple-darwin.tar.gz"
          X86_TGZ="nowdocs-v${VERSION}-x86_64-apple-darwin.tar.gz"
          ARM_SHA=$(curl -fsSL "${BASE}/${ARM_TGZ}" | sha256sum | awk '{print $1}')
          X86_SHA=$(curl -fsSL "${BASE}/${X86_TGZ}" | sha256sum | awk '{print $1}')
          sed -i "s/^  version \".*\"/  version \"${VERSION}\"/" Formula/nowdocs.rb
          # Replace the sha256 lines in each block: first arm64, then x86_64.
          # Match any existing value (placeholder OR a real 64-hex hash from a
          # prior bump) so every release refreshes both hashes, not just the first.
          python3 -c "
          import re, pathlib
          p = pathlib.Path('Formula/nowdocs.rb')
          s = p.read_text()
          s = re.sub(r'(on_arm do\n.*?sha256 \")[^\"]*(\")',
                     r'\g<1>${ARM_SHA}\g<2>', s, count=1, flags=re.DOTALL)
          s = re.sub(r'(on_intel do\n.*?sha256 \")[^\"]*(\")',
                     r'\g<1>${X86_SHA}\g<2>', s, count=1, flags=re.DOTALL)
          p.write_text(s)
          "
          git config user.name "nowdocs-bump-bot"
          git config user.email "bump@nowdocs.dev"
          git checkout -b "bump-v${VERSION}"
          git add Formula/nowdocs.rb
          git commit -m "build(tap): bump nowdocs to v${VERSION}"
          git push origin "bump-v${VERSION}"
          gh pr create \
            --title "build(tap): bump nowdocs to v${VERSION}" \
            --body "Auto-bumped on release. Verifies via \`brew audit\` + \`brew install --build-from-source\` in CI." \
            --base main

      - name: brew audit
        run: brew audit --strict --online Formula/nowdocs.rb

      - name: brew install (smoke)
        run: |
          brew install --quiet nowdocs-registry/nowdocs/nowdocs
          nowdocs --version
          brew uninstall --quiet nowdocs
```

Wire the main repo's `release.yml` to dispatch this workflow after upload:

```yaml
# in nowdocs/nowdocs .github/workflows/release.yml, add as final step:
- name: Notify tap bump
  if: success()
  uses: peter-evans/repository-dispatch@v3
  with:
    token: ${{ secrets.TAP_BUMP_TOKEN }}   # PAT scoped to homebrew-nowdocs, with contents + PR perms
    repository: nowdocs-registry/homebrew-nowdocs
    event-type: nowdocs-release-published
    client-payload: '{"version": "${{ github.ref_name }}"}'
```

### Option B: Manual bump (fallback, if Option A is not yet wired)

```bash
# Locally, in a clone of the tap repo:
cd homebrew-nowdocs
NEW="0.2.0"
BASE="https://github.com/nowdocs/nowdocs/releases/download/v${NEW}"
ARM_SHA=$(curl -fsSL "${BASE}/nowdocs-v${NEW}-aarch64-apple-darwin.tar.gz" | shasum -a 256 | awk '{print $1}')
X86_SHA=$(curl -fsSL "${BASE}/nowdocs-v${NEW}-x86_64-apple-darwin.tar.gz" | shasum -a 256 | awk '{print $1}')

# Edit Formula/nowdocs.rb: bump `version "..."` line + replace the two
# TODO_FILL_AFTER_RELEASE values with $ARM_SHA and $X86_SHA respectively.

# Verify locally:
brew audit --strict --new Formula/nowdocs.rb
brew install --build-from-source Formula/nowdocs.rb
nowdocs --version
brew uninstall nowdocs

# Push as PR:
git checkout -b "bump-v${NEW}"
git commit -am "build(tap): bump nowdocs to v${NEW}"
git push origin "bump-v${NEW}"
gh pr create --title "build(tap): bump nowdocs to v${NEW}" --body "Manual bump."
```

---

## 4. What this formula does NOT cover

| Channel | How it gets nowdocs |
|---|---|
| **macOS** (this formula) | `brew install nowdocs` via this tap |
| **Linux** (any arch) | `cargo binstall nowdocs` — uses the same tgz assets, see Task 5a. No Homebrew formula needed. |
| **Windows** | Standalone tgz from the GitHub Release page. No Homebrew formula needed. |
| **Direct download** | `https://github.com/nowdocs/nowdocs/releases/latest` — anyone can grab the binary directly. |

Linux + Windows intentionally excluded: Homebrew is a macOS-first ecosystem and
maintaining per-distro formulae (`aarch64-unknown-linux-musl` etc.) is out of
scope for this task. The unified tgz format (OQ-3) means Linux users can always
`curl | tar -xz` if they don't want cargo-binstall.

---

## 5. Open Questions

These are blockers that **only the human operator can resolve**:

1. **Create the `nowdocs-registry/homebrew-nowdocs` GitHub repo.** (Cannot be
   done by an automated agent.) Suggested visibility: public. Suggested
   description: "Unofficial Homebrew tap for nowdocs — see
   https://github.com/nowdocs/nowdocs for the main project."
2. **Confirm the main repo owner.** This formula hardcodes
   `https://github.com/nowdocs/nowdocs/releases/...`. If the canonical repo URL
   is different (e.g. `kaigetools/nowdocs`, `nowdocs-org/nowdocs`), update the
   four `url` lines in `nowdocs.rb` before the first tap release. The unified
   asset-name format from Task 5a (`nowdocs-v{version}-{target}.tar.gz`) is
   confirmed; only the owner path may change.
3. **Wire Option A (CI auto-bump) or accept Option B (manual) for the first
   few releases.** Option A is recommended for v0.2.0 onward; v0.1.0 was
   released before this tap existed, so the first bump is necessarily manual
   unless we retroactively bump on tap creation.
4. **Decide whether the seed commit should be merged before or after the first
   real `nowdocs` release is published.** If merged first, the `TODO_FILL_AFTER_RELEASE`
   placeholders will be live in the formula and `brew install` will fail until
   the first bump PR lands — this is the safer ordering (forces the bump to
   happen explicitly). If the first release happens first, the seed commit can
   include the real sha256 values directly and skip the bump step for v0.1.0.
