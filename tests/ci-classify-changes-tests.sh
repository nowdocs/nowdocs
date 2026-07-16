#!/usr/bin/env bash
# Regression tests for the path classifier used by Quality Gates.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLASSIFY="$ROOT/scripts/ci-classify-changes.sh"

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

assert_paths() {
    local expected="$1"
    shift
    local actual
    actual="$(printf '%s\n' "$@" | bash "$CLASSIFY")" || fail "classifier failed for: $*"
    [ "$actual" = "$expected" ] || fail "expected [$expected], got [$actual] for: $*"
}

assert_stdin() {
    local expected="$1"
    local input="$2"
    local actual
    actual="$(printf '%s' "$input" | bash "$CLASSIFY")" || fail "classifier failed for stdin"
    [ "$actual" = "$expected" ] || fail "expected [$expected], got [$actual] for stdin"
}

docs_only=$'docs_only=true\nrun_rust=false\nrun_release=false\nrun_cross=false'
ordinary_rust=$'docs_only=false\nrun_rust=true\nrun_release=false\nrun_cross=false'
sensitive=$'docs_only=false\nrun_rust=true\nrun_release=true\nrun_cross=true'

assert_paths "$docs_only" README.md docs/ARCHITECTURE.md
assert_paths "$ordinary_rust" src/retrieve.rs tests/retrieve_tests.rs
assert_paths "$sensitive" Cargo.lock
assert_paths "$sensitive" src/automation/setup.rs
assert_paths "$sensitive" tests/client_execution_contract_tests.rs
assert_paths "$sensitive" deny.toml src/sanitize.rs tests/sanitize_tests.rs
assert_paths "$sensitive" scripts/ci-classify-changes.sh tests/ci-classify-changes-tests.sh
assert_paths "$sensitive" .github/workflows/gates.yml build.rs
assert_stdin "$docs_only" ''

WORKFLOW="$ROOT/.github/workflows/gates.yml"
grep -Fq 'name: Change classification' "$WORKFLOW" \
    || fail "workflow must classify changed paths"
grep -Fq 'name: Documentation diff' "$WORKFLOW" \
    || fail "documentation-only changes need a lightweight diff check"
grep -Fq "if: needs.changes.outputs.docs_only == 'true'" "$WORKFLOW" \
    || fail "documentation diff must run only for documentation-only changes"
grep -Fq -- '--no-renames --diff-filter=ACMRD' "$WORKFLOW" \
    || fail "classification must include deleted paths without rename ambiguity"
grep -Fq 'name: Lint (fmt + clippy)' "$WORKFLOW" \
    || fail "workflow must retain the protected Lint check name"
grep -Fq 'name: Test' "$WORKFLOW" \
    || fail "workflow must retain the protected Test check name"
grep -Fq "if: needs.changes.outputs.run_rust == 'true'" "$WORKFLOW" \
    || fail "Rust gates must depend on the Rust-risk output"
grep -Fq "if: needs.changes.outputs.run_cross == 'true'" "$WORKFLOW" \
    || fail "cross-platform gate must depend on the cross-platform-risk output"
grep -Fq "if: needs.changes.outputs.run_release == 'true'" "$WORKFLOW" \
    || fail "release build must depend on the release-risk output"
grep -Fq 'shared-key: linux-debug-protoc-v2' "$WORKFLOW" \
    || fail "Linux debug cache must have its own domain"
grep -Fq 'shared-key: linux-release-protoc-v2' "$WORKFLOW" \
    || fail "Linux release cache must have its own domain"
grep -Fq 'needs: [changes, test]' "$WORKFLOW" \
    || fail "Lint must follow Test before restoring the shared debug cache"
grep -Fq 'cancel-in-progress: ${{ github.event_name == '\''pull_request'\'' }}' "$WORKFLOW" \
    || fail "only PR runs may cancel obsolete runs"

echo "CI change-classification regression tests passed"
