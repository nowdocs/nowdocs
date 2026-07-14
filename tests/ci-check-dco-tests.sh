#!/usr/bin/env bash
# Regression tests for scripts/ci-check-dco.sh. They construct an entirely
# local Git history: an unsigned synchronization merge must pass, while an
# unsigned ordinary commit remains a DCO failure.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$ROOT/scripts/ci-check-dco.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

git -C "$TMP" init -q
git -C "$TMP" config user.name "DCO Test"
git -C "$TMP" config user.email "dco-test@example.invalid"

printf 'base\n' >"$TMP/history.txt"
git -C "$TMP" add history.txt
git -C "$TMP" commit -s -qm "base"
BASE="$(git -C "$TMP" rev-parse HEAD)"

git -C "$TMP" checkout -qb feature "$BASE"
printf 'feature\n' >>"$TMP/history.txt"
git -C "$TMP" add history.txt
git -C "$TMP" commit -s -qm "signed feature commit"

git -C "$TMP" checkout -qb updated-main "$BASE"
printf 'main\n' >"$TMP/main.txt"
git -C "$TMP" add main.txt
git -C "$TMP" commit -s -qm "signed main update"

git -C "$TMP" checkout -q feature
git -C "$TMP" merge --no-ff updated-main -m "Update branch"

# GitHub's Update branch creates an unsigned two-parent merge. The DCO checker
# must ignore that merge while continuing to inspect ordinary PR commits.
if ! (cd "$TMP" && bash "$CHECK" "$BASE" HEAD); then
    fail "unsigned synchronization merge must not fail DCO"
fi

printf 'unsigned\n' >>"$TMP/history.txt"
git -C "$TMP" add history.txt
git -C "$TMP" commit -qm "unsigned ordinary commit"

if (cd "$TMP" && bash "$CHECK" "$BASE" HEAD) >"$TMP/dco.out" 2>&1; then
    fail "unsigned ordinary commit must fail DCO"
fi
grep -q "MISSING Signed-off-by" "$TMP/dco.out" \
    || fail "ordinary unsigned failure must identify the missing sign-off"

echo "DCO merge-commit regression tests passed"
