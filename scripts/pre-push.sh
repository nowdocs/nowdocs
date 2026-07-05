#!/usr/bin/env bash
# L2: Pre-push hook (本地，与 CI Quality Gates 对齐)
# 安装方式:
#   1. git config core.hooksPath scripts
#   2. 或 ln -s ../../scripts/pre-push.sh .git/hooks/pre-push
#
# 实际检查逻辑在 check.sh (fmt + clippy + test，与 gates.yml 同款)。
# 紧急避险: --no-verify 仅经用户批准可临时绕过 L1/L2

set -euo pipefail

# 可移植的符号链接解析: readlink -f 是 GNU 扩展, 在 macOS/BSD 上会以
# "illegal option -- f" 退出非零, 导致 set -e 在调用 check.sh 前就挂掉。
# 改用纯 POSIX readlink (无 flag, 返回链接目标) + 循环跟随, 解析物理路径。
# 参考: https://pubs.opengroup.org/onlinepubs/9799919799/utilities/readlink.html
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

# --- 拦截低于 15 行修改的代码直接 push ---
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

