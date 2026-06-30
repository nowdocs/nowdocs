#!/usr/bin/env bash
# CI manifest schema validation — checks a docset manifest.json against
# nowdocs v1 invariants (schema version, model lock, license whitelist,
# domain whitelist for source URLs).
#
# Usage: bash scripts/ci-check-manifest.sh <path/to/manifest.json>
# Exit 0 = all checks passed, Exit 1 = validation errors.
set -euo pipefail

MANIFEST="${1:?Usage: ci-check-manifest.sh <manifest.json>}"

if [ ! -f "$MANIFEST" ]; then
    echo "ERROR: manifest file not found: $MANIFEST" >&2
    exit 1
fi

errors=()

check() {
    local label="$1"
    local condition="$2"
    if ! eval "$condition"; then
        errors+=("$label")
    fi
}

# 1. Parse JSON and check schema version
schema_version=$(jq -r '.nowdocs_schema_version // "missing"' "$MANIFEST")
check "schema version must be 1 (got '$schema_version')" '[ "$schema_version" = "1" ]'

# 2. Model ID must be locked value
model_id=$(jq -r '.embedder.model_id // "missing"' "$MANIFEST")
check "model_id must be jinaai/jina-embeddings-v2-small-en (got '$model_id')" \
    '[ "$model_id" = "jinaai/jina-embeddings-v2-small-en" ]'

# 3. Vector dimension must be 512
vector_dim=$(jq -r '.embedder.vector_dim // "missing"' "$MANIFEST")
check "vector_dim must be 512 (got '$vector_dim')" '[ "$vector_dim" = "512" ]'

# 4. Engine must be candle
engine=$(jq -r '.embedder.engine // "missing"' "$MANIFEST")
check "engine must be candle (got '$engine')" '[ "$engine" = "candle" ]'

# 5. Dtype must be f16
dtype=$(jq -r '.embedder.dtype // "missing"' "$MANIFEST")
check "dtype must be f16 (got '$dtype')" '[ "$dtype" = "f16" ]'

# 6. model_revision must not be empty (A3 integrity pin)
model_revision=$(jq -r '.embedder.model_revision // ""' "$MANIFEST")
check "model_revision must not be empty" '[ -n "$model_revision" ]'

# 7. model_sha256 must not be empty (A3 integrity pin)
model_sha256=$(jq -r '.embedder.model_sha256 // ""' "$MANIFEST")
check "model_sha256 must not be empty" '[ -n "$model_sha256" ]'

# 8. Tokenizer must be "default" for v1
tokenizer=$(jq -r '.retrieval.tokenizer // "missing"' "$MANIFEST")
check "tokenizer must be 'default' for v1 (got '$tokenizer')" '[ "$tokenizer" = "default" ]'

# 9. License must be allowlisted: MIT, Apache-2.0, or CC-BY-4.0
license=$(jq -r '.legal.license // "missing"' "$MANIFEST")
case "$license" in
    MIT|Apache-2.0|CC-BY-4.0) ;;
    *) errors+=("license not allowlisted: '$license' (allowed: MIT, Apache-2.0, CC-BY-4.0)") ;;
esac

# 10. CC-BY-4.0 requires non-empty attribution
if [ "$license" = "CC-BY-4.0" ]; then
    attribution=$(jq -r '.legal.attribution // ""' "$MANIFEST")
    check "CC-BY-4.0 requires non-empty attribution" '[ -n "$attribution" ]'
fi

# Note: source_url is provenance metadata (upstream doc source, e.g.
# github.com/vercel/next.js), NOT the registry download URL. The A2
# download-URL allowlist is enforced at runtime in registry.rs::download_to_temp.
# We deliberately do NOT domain-check source_url here — legal review (§6.10)
# needs the real upstream URL, and the download path is already gated elsewhere.

# Report
if [ ${#errors[@]} -eq 0 ]; then
    echo "MANIFEST CHECK PASSED: $MANIFEST"
    exit 0
else
    echo "MANIFEST CHECK FAILED: $MANIFEST" >&2
    for err in "${errors[@]}"; do
        echo "  - $err" >&2
    done
    exit 1
fi
