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
        CHANGES=$(git diff --numstat "$BASE_BRANCH...HEAD" | grep -v '\.md$' | awk '{add+=$1; del+=$2} END {print add+del}')
        if [ -z "$CHANGES" ]; then
            CHANGES=0
        fi
        if [ "$CHANGES" -lt 15 ]; then
            echo "❌ ERROR: Total code lines changed compared to $BASE_BRANCH is $CHANGES (less than 15 lines)."
            echo "Pushing changes under 15 lines is prohibited on feature branches."
            exit 1
        fi
        echo "✅ Feature branch code changes: $CHANGES lines (>= 15 lines)."
    fi
fi

bash "$SCRIPT_DIR/check.sh"

