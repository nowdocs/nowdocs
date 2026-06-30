#!/usr/bin/env bash
# CI DCO (Developer Certificate of Origin) check — verifies that every
# commit in a PR branch has a Signed-off-by: line.
# D8: DCO not CLA — Rust ecosystem convention.
#
# Usage: bash scripts/ci-check-dco.sh <base_ref> <head_ref>
# Example: bash scripts/ci-check-dco.sh origin/main HEAD
# Exit 0 = all commits signed, Exit 1 = unsigned commits found.
set -euo pipefail

BASE="${1:?Usage: ci-check-dco.sh <base_ref> <head_ref>}"
HEAD="${2:?Usage: ci-check-dco.sh <base_ref> <head_ref>}"

echo "DCO check: $BASE..$HEAD"

unsigned=0
while IFS= read -r sha; do
    # Check if the commit message contains Signed-off-by:
    if ! git log -1 --format='%B' "$sha" | grep -q '^Signed-off-by:'; then
        subject=$(git log -1 --format='%s' "$sha")
        echo "  MISSING Signed-off-by: $sha $subject" >&2
        unsigned=$((unsigned + 1))
    fi
done < <(git rev-list "$BASE".."$HEAD")

if [ "$unsigned" -eq 0 ]; then
    echo "DCO check passed: all commits have Signed-off-by"
    exit 0
else
    echo "DCO check FAILED: $unsigned commit(s) missing Signed-off-by" >&2
    echo "  Fix with: git commit --signoff (or -s flag)" >&2
    exit 1
fi
