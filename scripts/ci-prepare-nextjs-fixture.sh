#!/usr/bin/env bash
set -euo pipefail

# Find nowdocs cache directory, respecting XDG_CACHE_HOME and OS defaults.
if [ -n "${XDG_CACHE_HOME:-}" ]; then
    CACHE_ROOT="$XDG_CACHE_HOME/nowdocs"
elif [ "$(uname)" = "Darwin" ]; then
    CACHE_ROOT="$HOME/Library/Caches/nowdocs"
else
    CACHE_ROOT="$HOME/.cache/nowdocs"
fi
DB_PATH="$CACHE_ROOT/db/nextjs_real.lance"
MANIFEST_PATH="$CACHE_ROOT/db/nextjs_real.manifest.json"

echo "=== Preparing Next.js Docs Fixture ==="
echo "Checking cache at: $CACHE_ROOT"

if [ -d "$DB_PATH" ] && [ -f "$MANIFEST_PATH" ]; then
    echo "Next.js real fixture database and manifest are already cached."
    echo "Skipping ingest."
    exit 0
fi

echo "Cache miss. Rebuilding Next.js corpus..."

# 1. Rebuild markdown files if they do not exist
REBUILT_DIR="seed-crates/tmp/nextjs_rebuilt"
if [ ! -d "$REBUILT_DIR" ]; then
    echo "Rebuilt directory '$REBUILT_DIR' not found. Reconstructing from chunks..."
    python3 seed-crates/tmp/rebuild_nextjs.py
else
    echo "Found rebuilt directory '$REBUILT_DIR'."
fi

# 2. Run cargo run --release -- ingest to create the lance database + manifest
echo "Ingesting Next.js corpus to create fixture 'nextjs_real'..."
cargo run --release -- ingest "$REBUILT_DIR" nextjs_real \
    --license MIT \
    --copyright-holder "Vercel, Inc." \
    --source-url "https://github.com/vercel/next.js" \
    --entry-url "https://nextjs.org/docs"

echo "=== Next.js Docs Fixture Prepared Successfully ==="
