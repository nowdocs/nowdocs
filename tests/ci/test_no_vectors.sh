#!/usr/bin/env bash
# Tests for scripts/ci-no-vectors.sh — CI vector file rejection.
# Run: bash tests/ci/test_no_vectors.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
CHECK="$REPO_ROOT/scripts/ci-no-vectors.sh"
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

pass=0
fail=0

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

echo "=== no-vectors tests ==="

# Test: directory with no vector files should pass
mkdir -p "$TMPDIR/clean"
echo "hello" > "$TMPDIR/clean/readme.md"
assert_pass "clean dir passes" bash "$CHECK" "$TMPDIR/clean"

# Test: directory with .lance file should fail
mkdir -p "$TMPDIR/dirty"
echo "data" > "$TMPDIR/dirty/data.lance"
assert_fail "dir with .lance file rejected" bash "$CHECK" "$TMPDIR/dirty"

# Test: directory with .lance directory should fail
mkdir -p "$TMPDIR/dirty2/db.lance"
echo "data" > "$TMPDIR/dirty2/db.lance/data.bin"
assert_fail "dir with .lance/ dir rejected" bash "$CHECK" "$TMPDIR/dirty2"

# Test: directory with vectors.json should fail
mkdir -p "$TMPDIR/dirty3"
echo "[]" > "$TMPDIR/dirty3/vectors.json"
assert_fail "dir with vectors.json rejected" bash "$CHECK" "$TMPDIR/dirty3"

echo ""
echo "Results: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
