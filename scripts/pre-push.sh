#!/usr/bin/env bash
# L2: Pre-push hook (十秒级，本地)
# 安装方式:
#   1. git config core.hooksPath scripts
#   2. 或 ln -s ../../scripts/pre-push.sh .git/hooks/pre-push
#
# 紧急避险: --no-verify 仅经用户批准可临时绕过 L1/L2

set -euo pipefail

echo "=== L2 Pre-push: Running cargo test ==="
# --test-threads=1: registry 测试已知 flake (见 INDEX.md)，串行避免竞态
cargo test --test-threads=1 2>&1

echo "=== L2 Pre-push: All checks passed ==="
# 注: release build 已下沉至 CI gates.yml build job，保持 L2 十秒级定位
