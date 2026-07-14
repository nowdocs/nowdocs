#!/usr/bin/env bash
# CI DCO (Developer Certificate of Origin) check — verifies that every
# non-merge commit introduced by a PR has a Signed-off-by: line. GitHub's
# Update branch action creates an unsigned synchronization merge commit; its
# parents are checked independently, so the merge node itself is excluded.
# D8: DCO not CLA — Rust ecosystem convention.
#
# Usage: bash scripts/ci-check-dco.sh <base_ref> <head_ref>
# Example: bash scripts/ci-check-dco.sh origin/main HEAD
# Exit 0 = all commits signed, Exit 1 = unsigned commits found.
set -euo pipefail

BASE="${1:?Usage: ci-check-dco.sh <base_ref> <head_ref>}"
HEAD="${2:?Usage: ci-check-dco.sh <base_ref> <head_ref>}"

echo "DCO check: non-merge commits in $BASE..$HEAD"

unsigned=0
while IFS= read -r sha; do
    # Check if the commit message contains Signed-off-by:
    if ! git log -1 --format='%B' "$sha" | grep -q '^Signed-off-by:'; then
        subject=$(git log -1 --format='%s' "$sha")
        echo "  MISSING Signed-off-by: $sha $subject" >&2
        unsigned=$((unsigned + 1))
    fi
done < <(git rev-list --no-merges "$BASE".."$HEAD")

if [ "$unsigned" -eq 0 ]; then
    echo "DCO check passed: all non-merge PR commits have Signed-off-by"
    exit 0
else
    echo "DCO check FAILED: $unsigned commit(s) missing Signed-off-by" >&2
    echo "  Fix ordinary commits with: git commit --signoff (or -s flag)" >&2
    exit 1
fi
