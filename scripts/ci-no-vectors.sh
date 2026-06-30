#!/usr/bin/env bash
# CI vector file rejection — ensures no .lance databases, vector files, or
# embedding artifacts are committed to the registry.
# D10: share publishes text only; CI rebuilds vectors from chunks.jsonl.
#
# Usage: bash scripts/ci-no-vectors.sh <directory>
# Exit 0 = clean, Exit 1 = vector files found.
set -euo pipefail

DIR="${1:?Usage: ci-no-vectors.sh <directory>}"

if [ ! -d "$DIR" ]; then
    echo "ERROR: directory not found: $DIR" >&2
    exit 1
fi

found=0

# Check for .lance directories/files (LanceDB database artifacts)
while IFS= read -r -d '' match; do
    echo "REJECTED: vector file found — $match" >&2
    found=1
done < <(find "$DIR" \( -name "*.lance" -o -name "*.lance.dir" -o -name "*.lance.bin" -o -name "*.lance.tmp" \) -print0 2>/dev/null || true)

# Check for common vector/embedding file patterns
while IFS= read -r -d '' match; do
    echo "REJECTED: vector file found — $match" >&2
    found=1
done < <(find "$DIR" \( -name "vectors.*" -o -name "embeddings.*" -o -name "*.faiss" -o -name "*.hnsw" -o -name "*.ivf" \) -print0 2>/dev/null || true)

if [ "$found" -eq 0 ]; then
    echo "NO VECTORS: $DIR is clean (text+manifest only)"
    exit 0
else
    echo "FAILED: $DIR contains vector files (D10 violation)" >&2
    exit 1
fi
