#!/usr/bin/env bash
set -euo pipefail

LOOM_DIR_ROOT="${LOOM_DIR_ROOT:-$PWD}"

echo "==> Building loom (release)..."
cargo build --release

echo "==> Installing to ${LOOM_DIR_ROOT}/.loom/bin/loom..."
mkdir -p "${LOOM_DIR_ROOT}/.loom/bin"
cp target/release/loom "${LOOM_DIR_ROOT}/.loom/bin/loom"
chmod +x "${LOOM_DIR_ROOT}/.loom/bin/loom"

echo "==> Running loom init..."
"${LOOM_DIR_ROOT}/.loom/bin/loom" init "${LOOM_DIR_ROOT}"

echo "==> Cleaning build artifacts..."
cargo clean

echo ""
echo "Done. Restart Claude Code and run /mcp to connect."
