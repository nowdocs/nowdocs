#!/usr/bin/env bash
set -euo pipefail

# Prepare the Next.js eval fixture for the CI `eval` job (OQ8 / S5).
#
# The real Next.js docset (~7480 chunks) is far too large to commit to git, so
# CI (a) reconstructs the markdown corpus from the vendored share artifact
# (seed-crates/share/nextjs-docs/chunks.jsonl -> rebuild_nextjs.py) and
# (b) builds the `.lance` table + manifest, cached under
# $NOWDOCS_FIXTURE_CACHE (default ~/.cache/nowdocs-ci-fixtures). Subsequent
# runs restore the cache instead of re-ingesting (minutes saved per run).
#
# The corpus is rebuilt on EVERY run (cheap, python only, no model) — it is the
# ingest source on a fixture-cache miss and for the tests' local fallback
# path. The built docset is cached because the full ingest takes minutes; the
# ignored eval tests consume the default-cache fixture directly (no re-ingest).
#
# Deliberately reconstructs from the vendored artifact rather than cloning
# github.com/vercel/next.js: faster, offline-reproducible, and produces
# source_urls identical to the golden query set.
#
# Env overrides:
#   NOWDOCS_FIXTURE_DOCSET  docset name (default: nextjs_real, matches the
#                           ignored eval tests)
#   NOWDOCS_FIXTURE_CACHE   cross-run cache dir (default: ~/.cache/nowdocs-ci-fixtures)
#   XDG_CACHE_HOME          nowdocs cache root base (default: OS cache dir)

DOCSET="${NOWDOCS_FIXTURE_DOCSET:-nextjs_real}"
FIXTURE_CACHE="${NOWDOCS_FIXTURE_CACHE:-$HOME/.cache/nowdocs-ci-fixtures}"
REBUILT_DIR="seed-crates/tmp/nextjs_rebuilt"

# nowdocs cache root (mirrors src/cache.rs: XDG_CACHE_HOME wins, else OS default).
if [ -n "${XDG_CACHE_HOME:-}" ]; then
    CACHE_ROOT="$XDG_CACHE_HOME/nowdocs"
elif [ "$(uname)" = "Darwin" ]; then
    CACHE_ROOT="$HOME/Library/Caches/nowdocs"
else
    CACHE_ROOT="$HOME/.cache/nowdocs"
fi
DB_DIR="$CACHE_ROOT/db"
LANCE_PATH="$DB_DIR/$DOCSET.lance"
MANIFEST_PATH="$DB_DIR/$DOCSET.manifest.json"

echo "=== Preparing Next.js eval fixture (docset=$DOCSET) ==="
echo "cache root: $CACHE_ROOT"
echo "fixture cache: $FIXTURE_CACHE"

# 1. Always ensure the markdown corpus exists — the ignored eval tests ingest
#    from it into isolated temp caches. Cheap (python only, no model).
if [ ! -d "$REBUILT_DIR" ]; then
    echo "Reconstructing markdown corpus from the vendored share artifact..."
    python3 seed-crates/tmp/rebuild_nextjs.py
else
    echo "Corpus already reconstructed at $REBUILT_DIR."
fi

# 2. Restore the pre-built docset from the cross-run cache when available.
if [ -d "$FIXTURE_CACHE/$DOCSET.lance" ] && [ -f "$FIXTURE_CACHE/$DOCSET.manifest.json" ]; then
    echo "Restoring cached docset fixture..."
    mkdir -p "$DB_DIR"
    rm -rf "$LANCE_PATH"
    cp -R "$FIXTURE_CACHE/$DOCSET.lance" "$LANCE_PATH"
    cp "$FIXTURE_CACHE/$DOCSET.manifest.json" "$MANIFEST_PATH"
    echo "=== Fixture restored from cache ==="
    exit 0
fi

# 3. Cache miss: ingest the full corpus (downloads the embedder on first run —
#    warm it via `nowdocs doctor --model`). Release profile: candle in debug
#    is far too slow for ~7480 chunks (would not fit the CI job budget); the
#    eval job builds --release, so this adds no second build.
if [ ! -d "$LANCE_PATH" ] || [ ! -f "$MANIFEST_PATH" ]; then
    echo "Ingesting corpus into docset '$DOCSET' (this takes minutes)..."
    cargo run --locked --release -- ingest "$REBUILT_DIR" "$DOCSET" \
        --license MIT \
        --copyright-holder "Vercel, Inc." \
        --source-url "https://github.com/vercel/next.js" \
        --entry-url "https://nextjs.org/docs"
fi

# 4. Save into the cross-run fixture cache.
echo "Saving docset fixture to the cache..."
mkdir -p "$FIXTURE_CACHE"
rm -rf "$FIXTURE_CACHE/$DOCSET.lance"
cp -R "$LANCE_PATH" "$FIXTURE_CACHE/$DOCSET.lance"
cp "$MANIFEST_PATH" "$FIXTURE_CACHE/$DOCSET.manifest.json"

echo "=== Next.js eval fixture ready (cached at $FIXTURE_CACHE) ==="
