#!/usr/bin/env bash
# L2: Pre-push hook (本地，与 CI Quality Gates 对齐)
# 安装方式:
#   1. git config core.hooksPath scripts
#   2. 或 ln -s ../../scripts/pre-push.sh .git/hooks/pre-push
#
# 实际检查逻辑在 check.sh (fmt + clippy + test，与 gates.yml 同款)。
# 紧急避险: --no-verify 仅经用户批准可临时绕过 L1/L2

set -euo pipefail

# 解析 symlink 真实路径，找到 scripts/ 目录（hook 可能通过
# .git/hooks/pre-push -> ../../scripts/pre-push.sh 软链调用）
SCRIPT_DIR="$(cd "$(dirname "$(readlink -f "$0")")" && pwd)"
bash "$SCRIPT_DIR/check.sh"
