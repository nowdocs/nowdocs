# seed-crates — Task 5d deliverable

Three canonical seed docset crates, license-verified and share-ready.

## Layout

```
seed-crates/
├── share/                      # Final share artifacts (text+manifest+legal only)
│   ├── nextjs-docs/            # Next.js (MIT)
│   │   ├── manifest.json       # Model-locked, source_url + legal filled
│   │   ├── chunks.jsonl        # 5791 text chunks, no vectors
│   │   ├── LICENSE             # MIT license text
│   │   └── NOTICES             # Source + copyright + scope note
│   ├── react-docs/             # React (CC-BY-4.0)
│   │   ├── manifest.json       # attribution filled (required for CC-BY)
│   │   ├── chunks.jsonl        # 4360 text chunks
│   │   ├── LICENSE             # CC-BY-4.0 legal code (393 lines)
│   │   └── NOTICES             # Full attribution
│   └── vue-docs/               # Vue (CC-BY-4.0, image content excluded)
│       ├── manifest.json       # attribution filled, image exclusion noted
│       ├── chunks.jsonl        # 3280 text chunks
│       ├── LICENSE             # CC-BY-4.0 with image carve-out
│       └── NOTICES             # Full attribution + image exclusion clause
├── src/                        # Prepared source markdown (MDX/JSX stripped)
│   ├── next-docs/              # 437 .md files (from vercel/next.js docs/)
│   ├── react-docs/             # 220 .md files (from reactjs/react.dev src/content/)
│   └── vue-docs/               # 118 .md files (from vuejs/docs src/)
└── tmp/                        # Working files (not committed)
    ├── vue-docs/               # Shallow clone of vuejs/docs (gitignored)
    ├── react-docs/             # Shallow clone of reactjs/react.dev (gitignored)
    ├── next-docs/              # Sparse clone of vercel/next.js docs/ (gitignored)
    ├── prep_docs.py            # MDX/JSX stripper
    ├── patch_manifest.py       # Per-docset legal metadata patcher
    └── ingest-*.log, share-*.log
```

## License verification (§6.10)

| Docset | Upstream source | License | Attribution required? | Verified? |
|--------|-----------------|---------|----------------------|-----------|
| `nextjs-docs` | https://github.com/vercel/next.js (docs/) | MIT | No (set for credit) | ✅ manifest passes `ci-check-manifest.sh` |
| `react-docs` | https://github.com/reactjs/react.dev (src/content/) | CC-BY-4.0 | **Yes** (LICENSE-DOCS.md) | ✅ attribution non-empty, manifest passes |
| `vue-docs` | https://github.com/vuejs/docs (src/) | CC-BY-4.0 (text only) | **Yes**, images excluded per upstream LICENSE | ✅ attribution non-empty, image exclusion noted in NOTICES |

Clerk + Tailwind explicitly excluded (per §6.10): Clerk ToS §3/§5 prohibits
copying/scraping; Tailwind docs site is non-OSI ("Tailwind Labs IP").

## CI compliance (Task 5c rules)

All 3 share artifacts pass:

- ✅ `scripts/ci-check-manifest.sh` — schema v1, model lock (jina-v2-small, 512-dim, candle, f16), license allowlist, attribution for CC-BY
- ✅ `scripts/ci-no-vectors.sh` — text+manifest only, no `.lance` / `vectors.*` / `*.faiss` / `*.hnsw` / `*.ivf`
- ✅ DCO — commit signed off (see `git log` for `Signed-off-by:`)
- ✅ Pure text only — no images extracted from any source

## Open questions

1. `ingest::build_manifest` hardcodes `legal.license = "MIT"` (`src/ingest.rs:111`).
   The current `nowdocs ingest` CLI has no `--license` / `--copyright-holder` /
   `--attribution` flag. The CC-BY manifests for React/Vue were produced by
   running `ingest` (which writes the hardcoded MIT manifest), then post-patching
   the manifest via `tmp/patch_manifest.py`. **Suggestion**: add ingest CLI flags
   or a `LegalSpec` parameter to `ingest::ingest_dir` so the source of truth is
   the tool, not a post-step. Files for Task 4d/5d owner.
2. `nowdocs share` hardcodes the output dir to `<cwd>/<docset>-share/<docset>/`
   (`src/main.rs:33`). The redundant `<docset>-share/<docset>/` nesting was
   flattened post-hoc. A `--out-dir` flag would let the caller control the path.

## Reproducibility

```bash
# 1. Clone sources (shallow)
git clone --depth 1 https://github.com/vuejs/docs.git seed-crates/tmp/vue-docs
git clone --depth 1 https://github.com/reactjs/react.dev.git seed-crates/tmp/react-docs
git clone --depth 1 --filter=blob:none --sparse https://github.com/vercel/next.js.git seed-crates/tmp/next-docs
(cd seed-crates/tmp/next-docs && git sparse-checkout set docs)

# 2. Strip MDX/JSX, drop images
uv run python3 seed-crates/tmp/prep_docs.py seed-crates/tmp/vue-docs/src seed-crates/src/vue-docs translations
uv run python3 seed-crates/tmp/prep_docs.py seed-crates/tmp/react-docs/src/content seed-crates/src/react-docs
uv run python3 seed-crates/tmp/prep_docs.py seed-crates/tmp/next-docs/docs seed-crates/src/next-docs

# 3. Ingest + patch manifest + share
./target/release/nowdocs ingest seed-crates/src/next-docs nextjs-docs
./target/release/nowdocs ingest seed-crates/src/react-docs react-docs
./target/release/nowdocs ingest seed-crates/src/vue-docs vue-docs
uv run python3 seed-crates/tmp/patch_manifest.py nextjs-docs MIT "Vercel, Inc." "..." "..." "..."
uv run python3 seed-crates/tmp/patch_manifest.py react-docs CC-BY-4.0 "Meta Platforms, Inc. and React documentation contributors" "..." "..." "..."
uv run python3 seed-crates/tmp/patch_manifest.py vue-docs CC-BY-4.0 "Yuxi (Evan) You and Vue documentation contributors" "..." "..." "..."
cd seed-crates/share && /path/to/nowdocs share nextjs-docs && mv nextjs-docs-share/nextjs-docs . && rmdir nextjs-docs-share
# (same for react-docs and vue-docs)
```
