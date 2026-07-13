#!/usr/bin/env bash
# L2: Local pre-push hook aligned with CI Quality Gates.
# Installation:
#   1. git config core.hooksPath scripts
#   2. Or: ln -s ../../scripts/pre-push.sh .git/hooks/pre-push
#
# check.sh contains the actual checks (fmt, Clippy, and tests matching gates.yml).
# Emergency exception: --no-verify may bypass L1/L2 only with user approval.

set -euo pipefail

# Portable symlink resolution: readlink -f is a GNU extension and exits nonzero
# with "illegal option -- f" on macOS/BSD, causing set -e to abort before check.sh.
# Use POSIX readlink without flags and follow links in a loop to resolve the path.
# Reference: https://pubs.opengroup.org/onlinepubs/9799919799/utilities/readlink.html
resolve_script_dir() {
  src="$1"
  while [ -L "$src" ]; do
    dir=$(cd -P "$(dirname "$src")" >/dev/null 2>&1 && pwd -P)
    target=$(readlink "$src")
    case "$target" in
      /*) src="$target" ;;
      *) src="$dir/$target" ;;
    esac
  done
  cd -P "$(dirname "$src")" >/dev/null 2>&1 && pwd -P
}
SCRIPT_DIR="$(resolve_script_dir "$0")"

# --- Block feature-branch code pushes with fewer than 15 changed lines ---
CURRENT_BRANCH=$(git symbolic-ref --short HEAD)
if [ "$CURRENT_BRANCH" != "main" ] && [ "$CURRENT_BRANCH" != "master" ]; then
    if git rev-parse --verify origin/main >/dev/null 2>&1; then
        BASE_BRANCH="origin/main"
    elif git rev-parse --verify origin/master >/dev/null 2>&1; then
        BASE_BRANCH="origin/master"
    else
        BASE_BRANCH=""
    fi
    if [ -n "$BASE_BRANCH" ]; then
        git fetch origin $(echo $BASE_BRANCH | cut -d'/' -f2) --quiet || true
        # Separate code (non-.md) from documentation (.md) changes. The 15-line
        # floor targets trivial CODE pushes; pure documentation commits
        # (legal/README/spec) are legitimate and must NOT be blocked. Guard
        # every grep with `|| true` so set -e + pipefail don't abort on the
        # no-match case (a pure-.md push has zero non-.md lines, which made
        # grep return 1 and killed the script before the floor ran).
        NUMSTAT=$(git diff --numstat "$BASE_BRANCH...HEAD")
        CODE_CHANGES=$(echo "$NUMSTAT" | grep -v '\.md$' | awk '{add+=$1; del+=$2} END {print add+del+0}' || true)
        DOC_FILES=$(echo "$NUMSTAT" | grep -c '\.md$' || true)
        if [ "$CODE_CHANGES" -eq 0 ]; then
            echo "ℹ️  Documentation-only push ($DOC_FILES .md file(s), 0 code lines) — skipping 15-line code floor."
        elif [ "$CODE_CHANGES" -lt 15 ]; then
            echo "❌ ERROR: Total code lines changed compared to $BASE_BRANCH is $CODE_CHANGES (less than 15 lines)."
            echo "Pushing code changes under 15 lines is prohibited on feature branches."
            echo "Documentation-only pushes (.md) are exempt; if this is docs-only, remove non-.md changes."
            exit 1
        else
            echo "✅ Feature branch code changes: $CODE_CHANGES lines (>= 15 lines)."
        fi
    fi
fi

bash "$SCRIPT_DIR/check.sh"
