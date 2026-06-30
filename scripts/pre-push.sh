#!/usr/bin/env bash
# L2: Pre-push hook (十秒级，本地)
# 安装方式:
#   1. git config core.hooksPath scripts
#   2. 或 ln -s ../../scripts/pre-push.sh .git/hooks/pre-push
#
# 紧急避险: --no-verify 仅经用户批准可临时绕过 L1/L2

set -euo pipefail

echo "=== L2 Pre-push: Running cargo test ==="
cargo test 2>&1

echo "=== L2 Pre-push: Running cargo build --release ==="
cargo build --release 2>&1

echo "=== L2 Pre-push: All checks passed ==="
