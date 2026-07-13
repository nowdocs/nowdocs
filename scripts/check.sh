#!/usr/bin/env bash
# L2: Local quality checks aligned with CI Quality Gates (gates.yml).
#
# Run fmt --check, Clippy, and tests using the same checks as pull-request CI.
# pre-push.sh calls this script; run `bash scripts/check.sh` for a manual preflight.
#
# protoc is required by the LanceDB lance-encoding build script. Prefer $PROTOC,
# then protoc on PATH; exit with an error if neither exists (CI installs it via apt).
#
# Emergency exception: --no-verify may bypass L1/L2 only with user approval.

set -euo pipefail

# --- Detect protoc ---
if [ -z "${PROTOC:-}" ]; then
  if command -v protoc >/dev/null 2>&1; then
    PROTOC="$(command -v protoc)"
  elif [ -x "$HOME/.local/protoc/bin/protoc" ]; then
    PROTOC="$HOME/.local/protoc/bin/protoc"
  else
    echo "✗ protoc was not found. Install it with: sudo apt install protobuf-compiler, or set PROTOC." >&2
    exit 1
  fi
fi
export PROTOC

# --- fmt (seconds) ---
echo "=== [1/3] cargo fmt --check ==="
cargo fmt --check
echo "✓ fmt passed"

# --- Clippy (seconds incrementally; minutes on the first build) ---
echo "=== [2/3] cargo clippy --locked --all-targets -- -D warnings ==="
cargo clippy --locked --all-targets -- -D warnings
echo "✓ Clippy reported no warnings"

# --- Tests (serial: registry tests have a known flake and embedder cache files are shared) ---
echo "=== [3/3] cargo test --locked -- --test-threads=1 ==="
cargo test --locked -- --test-threads=1
echo "✓ tests passed"

echo "=== L2 local checks passed (aligned with CI Quality Gates) ==="
