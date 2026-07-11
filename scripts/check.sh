#!/usr/bin/env bash
# L2: 本地质量检查 — 与 CI Quality Gates (gates.yml) 对齐。
#
# 跑 fmt --check + clippy + test，与云端 PR 门禁同款，确保"本地过则 CI 过"。
# pre-push.sh 调用本脚本；也可手动 `bash scripts/check.sh` 预检。
#
# protoc: lancedb 的 lance-encoding build script 需要。优先用 $PROTOC，其次
# PATH 中的 protoc；都没有则报错退出（CI 端由 gates.yml/ci.yml apt 安装）。
#
# 紧急避险: --no-verify 仅经用户批准可临时绕过 L1/L2

set -euo pipefail

# --- protoc 探测 ---
if [ -z "${PROTOC:-}" ]; then
  if command -v protoc >/dev/null 2>&1; then
    PROTOC="$(command -v protoc)"
  elif [ -x "$HOME/.local/protoc/bin/protoc" ]; then
    PROTOC="$HOME/.local/protoc/bin/protoc"
  else
    echo "✗ 未找到 protoc。安装: sudo apt install protobuf-compiler，或设 PROTOC 环境变量" >&2
    exit 1
  fi
fi
export PROTOC

# --- fmt (秒级) ---
echo "=== [1/3] cargo fmt --check ==="
cargo fmt --check
echo "✓ fmt 干净"

# --- clippy (增量秒级，首次编译分钟级) ---
echo "=== [2/3] cargo clippy --locked --all-targets -- -D warnings ==="
cargo clippy --locked --all-targets -- -D warnings
echo "✓ clippy 无 warning"

# --- test (串行: registry 测试已知 flake，embedder 共享缓存文件需互斥) ---
echo "=== [3/3] cargo test --locked -- --test-threads=1 ==="
cargo test --locked -- --test-threads=1
echo "✓ test 全过"

echo "=== L2 本地检查全过 (与 CI Quality Gates 对齐) ==="
