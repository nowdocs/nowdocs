#!/usr/bin/env bash
# Tests for scripts/ci-check-manifest.sh — CI manifest schema validation.
# Run: bash tests/ci/test_manifest_check.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
CHECK="$REPO_ROOT/scripts/ci-check-manifest.sh"
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

pass=0
fail=0

run_check() {
    local manifest_path="$1"
    bash "$CHECK" "$manifest_path" 2>&1
}

assert_pass() {
    local label="$1"
    shift
    if "$@" > /dev/null 2>&1; then
        echo "  PASS: $label"
        pass=$((pass + 1))
    else
        echo "  FAIL (expected pass): $label"
        fail=$((fail + 1))
    fi
}

assert_fail() {
    local label="$1"
    shift
    if "$@" > /dev/null 2>&1; then
        echo "  FAIL (expected fail): $label"
        fail=$((fail + 1))
    else
        echo "  PASS: $label"
        pass=$((pass + 1))
    fi
}

# --- Valid manifest ---
cat > "$TMPDIR/valid.json" <<'EOF'
{
  "docset": "nextjs",
  "doc_version": "15.0.0",
  "nowdocs_schema_version": 1,
  "embedder": {
    "model_id": "jinaai/jina-embeddings-v2-small-en",
    "model_version": "512",
    "model_revision": "abc123def456",
    "model_sha256": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2",
    "vector_dim": 512,
    "engine": "candle",
    "dtype": "f16"
  },
  "retrieval": {
    "tokenizer": "default",
    "chunk_size_tokens": 384,
    "window_tokens": 2048
  },
  "source": {
    "entry_url": "https://nextjs.org/docs",
    "source_url": "https://github.com/nowdocs-registry/nextjs/releases/latest/download/nextjs.tar",
    "scraped_at": "2026-06-29T00:00:00Z",
    "chunk_count": 100
  },
  "legal": {
    "license": "MIT",
    "copyright_holder": "Vercel",
    "attribution": ""
  },
  "refresh_strategy": {
    "tier": "hot",
    "auto_days": 7
  }
}
EOF

# --- Failure manifests ---
cat > "$TMPDIR/bad_schema.json" <<'EOF'
{
  "docset": "nextjs", "doc_version": "1.0.0", "nowdocs_schema_version": 2,
  "embedder": {"model_id":"jinaai/jina-embeddings-v2-small-en","model_version":"512","model_revision":"x","model_sha256":"y","vector_dim":512,"engine":"candle","dtype":"f16"},
  "retrieval": {"tokenizer":"default","chunk_size_tokens":384,"window_tokens":2048},
  "source": {"entry_url":"https://x.com","source_url":"https://github.com/nowdocs-registry/x/releases/download/x.tar","scraped_at":"2026-01-01","chunk_count":1},
  "legal": {"license":"MIT","copyright_holder":"X","attribution":""},
  "refresh_strategy": {"tier":"hot","auto_days":7}
}
EOF

cat > "$TMPDIR/bad_model.json" <<'EOF'
{
  "docset": "nextjs", "doc_version": "1.0.0", "nowdocs_schema_version": 1,
  "embedder": {"model_id":"wrong-model","model_version":"512","model_revision":"x","model_sha256":"y","vector_dim":512,"engine":"candle","dtype":"f16"},
  "retrieval": {"tokenizer":"default","chunk_size_tokens":384,"window_tokens":2048},
  "source": {"entry_url":"https://x.com","source_url":"https://github.com/nowdocs-registry/x/releases/download/x.tar","scraped_at":"2026-01-01","chunk_count":1},
  "legal": {"license":"MIT","copyright_holder":"X","attribution":""},
  "refresh_strategy": {"tier":"hot","auto_days":7}
}
EOF

cat > "$TMPDIR/bad_dim.json" <<'EOF'
{
  "docset": "nextjs", "doc_version": "1.0.0", "nowdocs_schema_version": 1,
  "embedder": {"model_id":"jinaai/jina-embeddings-v2-small-en","model_version":"512","model_revision":"x","model_sha256":"y","vector_dim":768,"engine":"candle","dtype":"f16"},
  "retrieval": {"tokenizer":"default","chunk_size_tokens":384,"window_tokens":2048},
  "source": {"entry_url":"https://x.com","source_url":"https://github.com/nowdocs-registry/x/releases/download/x.tar","scraped_at":"2026-01-01","chunk_count":1},
  "legal": {"license":"MIT","copyright_holder":"X","attribution":""},
  "refresh_strategy": {"tier":"hot","auto_days":7}
}
EOF

cat > "$TMPDIR/bad_engine.json" <<'EOF'
{
  "docset": "nextjs", "doc_version": "1.0.0", "nowdocs_schema_version": 1,
  "embedder": {"model_id":"jinaai/jina-embeddings-v2-small-en","model_version":"512","model_revision":"x","model_sha256":"y","vector_dim":512,"engine":"ort","dtype":"f16"},
  "retrieval": {"tokenizer":"default","chunk_size_tokens":384,"window_tokens":2048},
  "source": {"entry_url":"https://x.com","source_url":"https://github.com/nowdocs-registry/x/releases/download/x.tar","scraped_at":"2026-01-01","chunk_count":1},
  "legal": {"license":"MIT","copyright_holder":"X","attribution":""},
  "refresh_strategy": {"tier":"hot","auto_days":7}
}
EOF

cat > "$TMPDIR/bad_dtype.json" <<'EOF'
{
  "docset": "nextjs", "doc_version": "1.0.0", "nowdocs_schema_version": 1,
  "embedder": {"model_id":"jinaai/jina-embeddings-v2-small-en","model_version":"512","model_revision":"x","model_sha256":"y","vector_dim":512,"engine":"candle","dtype":"f32"},
  "retrieval": {"tokenizer":"default","chunk_size_tokens":384,"window_tokens":2048},
  "source": {"entry_url":"https://x.com","source_url":"https://github.com/nowdocs-registry/x/releases/download/x.tar","scraped_at":"2026-01-01","chunk_count":1},
  "legal": {"license":"MIT","copyright_holder":"X","attribution":""},
  "refresh_strategy": {"tier":"hot","auto_days":7}
}
EOF

cat > "$TMPDIR/bad_revision.json" <<'EOF'
{
  "docset": "nextjs", "doc_version": "1.0.0", "nowdocs_schema_version": 1,
  "embedder": {"model_id":"jinaai/jina-embeddings-v2-small-en","model_version":"512","model_revision":"","model_sha256":"y","vector_dim":512,"engine":"candle","dtype":"f16"},
  "retrieval": {"tokenizer":"default","chunk_size_tokens":384,"window_tokens":2048},
  "source": {"entry_url":"https://x.com","source_url":"https://github.com/nowdocs-registry/x/releases/download/x.tar","scraped_at":"2026-01-01","chunk_count":1},
  "legal": {"license":"MIT","copyright_holder":"X","attribution":""},
  "refresh_strategy": {"tier":"hot","auto_days":7}
}
EOF

cat > "$TMPDIR/bad_sha256.json" <<'EOF'
{
  "docset": "nextjs", "doc_version": "1.0.0", "nowdocs_schema_version": 1,
  "embedder": {"model_id":"jinaai/jina-embeddings-v2-small-en","model_version":"512","model_revision":"abc","model_sha256":"","vector_dim":512,"engine":"candle","dtype":"f16"},
  "retrieval": {"tokenizer":"default","chunk_size_tokens":384,"window_tokens":2048},
  "source": {"entry_url":"https://x.com","source_url":"https://github.com/nowdocs-registry/x/releases/download/x.tar","scraped_at":"2026-01-01","chunk_count":1},
  "legal": {"license":"MIT","copyright_holder":"X","attribution":""},
  "refresh_strategy": {"tier":"hot","auto_days":7}
}
EOF

cat > "$TMPDIR/bad_tokenizer.json" <<'EOF'
{
  "docset": "nextjs", "doc_version": "1.0.0", "nowdocs_schema_version": 1,
  "embedder": {"model_id":"jinaai/jina-embeddings-v2-small-en","model_version":"512","model_revision":"abc","model_sha256":"def","vector_dim":512,"engine":"candle","dtype":"f16"},
  "retrieval": {"tokenizer":"lindera","chunk_size_tokens":384,"window_tokens":2048},
  "source": {"entry_url":"https://x.com","source_url":"https://github.com/nowdocs-registry/x/releases/download/x.tar","scraped_at":"2026-01-01","chunk_count":1},
  "legal": {"license":"MIT","copyright_holder":"X","attribution":""},
  "refresh_strategy": {"tier":"hot","auto_days":7}
}
EOF

cat > "$TMPDIR/bad_license.json" <<'EOF'
{
  "docset": "nextjs", "doc_version": "1.0.0", "nowdocs_schema_version": 1,
  "embedder": {"model_id":"jinaai/jina-embeddings-v2-small-en","model_version":"512","model_revision":"abc","model_sha256":"def","vector_dim":512,"engine":"candle","dtype":"f16"},
  "retrieval": {"tokenizer":"default","chunk_size_tokens":384,"window_tokens":2048},
  "source": {"entry_url":"https://x.com","source_url":"https://github.com/nowdocs-registry/x/releases/download/x.tar","scraped_at":"2026-01-01","chunk_count":1},
  "legal": {"license":"Proprietary","copyright_holder":"X","attribution":""},
  "refresh_strategy": {"tier":"hot","auto_days":7}
}
EOF

cat > "$TMPDIR/bad_ccby.json" <<'EOF'
{
  "docset": "react", "doc_version": "19.0.0", "nowdocs_schema_version": 1,
  "embedder": {"model_id":"jinaai/jina-embeddings-v2-small-en","model_version":"512","model_revision":"abc","model_sha256":"def","vector_dim":512,"engine":"candle","dtype":"f16"},
  "retrieval": {"tokenizer":"default","chunk_size_tokens":384,"window_tokens":2048},
  "source": {"entry_url":"https://x.com","source_url":"https://github.com/nowdocs-registry/react/releases/download/react.tar","scraped_at":"2026-01-01","chunk_count":1},
  "legal": {"license":"CC-BY-4.0","copyright_holder":"Meta","attribution":""},
  "refresh_strategy": {"tier":"hot","auto_days":7}
}
EOF

cat > "$TMPDIR/bad_url.json" <<'EOF'
{
  "docset": "nextjs", "doc_version": "1.0.0", "nowdocs_schema_version": 1,
  "embedder": {"model_id":"jinaai/jina-embeddings-v2-small-en","model_version":"512","model_revision":"abc","model_sha256":"def","vector_dim":512,"engine":"candle","dtype":"f16"},
  "retrieval": {"tokenizer":"default","chunk_size_tokens":384,"window_tokens":2048},
  "source": {"entry_url":"https://x.com","source_url":"https://evil.com/malicious.tar","scraped_at":"2026-01-01","chunk_count":1},
  "legal": {"license":"MIT","copyright_holder":"X","attribution":""},
  "refresh_strategy": {"tier":"hot","auto_days":7}
}
EOF

cat > "$TMPDIR/bad_url_lookalike.json" <<'EOF'
{
  "docset": "nextjs", "doc_version": "1.0.0", "nowdocs_schema_version": 1,
  "embedder": {"model_id":"jinaai/jina-embeddings-v2-small-en","model_version":"512","model_revision":"abc","model_sha256":"def","vector_dim":512,"engine":"candle","dtype":"f16"},
  "retrieval": {"tokenizer":"default","chunk_size_tokens":384,"window_tokens":2048},
  "source": {"entry_url":"https://x.com","source_url":"https://github.com/nowdocs-registry.evil.com/path.tar","scraped_at":"2026-01-01","chunk_count":1},
  "legal": {"license":"MIT","copyright_holder":"X","attribution":""},
  "refresh_strategy": {"tier":"hot","auto_days":7}
}
EOF

echo "=== manifest-check tests ==="

echo "Valid manifest:"
assert_pass "valid manifest passes" run_check "$TMPDIR/valid.json"

echo "Schema version:"
assert_fail "schema v2 rejected" run_check "$TMPDIR/bad_schema.json"

echo "Model ID:"
assert_fail "wrong model_id rejected" run_check "$TMPDIR/bad_model.json"

echo "Vector dim:"
assert_fail "dim 768 rejected" run_check "$TMPDIR/bad_dim.json"

echo "Engine:"
assert_fail "engine=ort rejected" run_check "$TMPDIR/bad_engine.json"

echo "Dtype:"
assert_fail "dtype=f32 rejected" run_check "$TMPDIR/bad_dtype.json"

echo "Revision:"
assert_fail "empty revision rejected" run_check "$TMPDIR/bad_revision.json"

echo "SHA256:"
assert_fail "empty sha256 rejected" run_check "$TMPDIR/bad_sha256.json"

echo "Tokenizer:"
assert_fail "tokenizer=lindera rejected (v1)" run_check "$TMPDIR/bad_tokenizer.json"

echo "License:"
assert_fail "Proprietary license rejected" run_check "$TMPDIR/bad_license.json"

echo "CC-BY attribution:"
assert_fail "CC-BY-4.0 without attribution rejected" run_check "$TMPDIR/bad_ccby.json"

echo "URL domain:"
assert_fail "evil.com URL rejected" run_check "$TMPDIR/bad_url.json"

echo "URL lookalike:"
assert_fail "lookalike domain rejected" run_check "$TMPDIR/bad_url_lookalike.json"

echo ""
echo "Results: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
