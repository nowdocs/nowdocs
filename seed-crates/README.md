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

## Resolved follow-ups

1. `ingest::ingest_dir` now takes an `IngestMeta` parameter; the `nowdocs ingest`
   CLI exposes `--license` / `--copyright-holder` / `--attribution` /
   `--source-url` / `--entry-url`. The manifest's legal + source fields are
   correct on first write, so `tmp/patch_manifest.py` is no longer needed
   (kept as a legacy reference only). `scraped_at` is auto-filled with today's
   UTC date via a std-only `civil_from_days` (no chrono dependency).
2. `nowdocs share` takes `--out-dir` to control the output directory; the
   default remains `./{docset}-share`. Passing `--out-dir seed-crates/share`
   produces `seed-crates/share/{docset}/` directly — no `mv`/`rmdir` hack.

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

# 3. Ingest (legal/source metadata baked in) + share
./target/release/nowdocs ingest seed-crates/src/next-docs nextjs-docs \
  --license MIT --copyright-holder "Vercel, Inc." \
  --attribution "Documentation from https://github.com/vercel/next.js, licensed under MIT." \
  --source-url https://github.com/vercel/next.js --entry-url https://nextjs.org
./target/release/nowdocs ingest seed-crates/src/react-docs react-docs \
  --license CC-BY-4.0 --copyright-holder "Meta Platforms, Inc. and React documentation contributors" \
  --attribution "React documentation by Meta Platforms, Inc. ..., licensed CC BY 4.0. Source: https://github.com/reactjs/react.dev." \
  --source-url https://github.com/reactjs/react.dev --entry-url https://react.dev
./target/release/nowdocs ingest seed-crates/src/vue-docs vue-docs \
  --license CC-BY-4.0 --copyright-holder "Yuxi (Evan) You and Vue documentation contributors" \
  --attribution "Vue documentation by Yuxi (Evan) You ..., licensed CC BY 4.0. Source: https://github.com/vuejs/docs." \
  --source-url https://github.com/vuejs/docs --entry-url https://vuejs.org
./target/release/nowdocs share nextjs-docs --out-dir seed-crates/share
# (same `share --out-dir seed-crates/share` for react-docs and vue-docs)
```
