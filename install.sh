#!/bin/sh
set -euo pipefail
REPO="urbanisierung/mxr"
BIN_DIR="${HOME}/.local/bin"
mkdir -p "$BIN_DIR"
ARCH=$(uname -m)
case "$ARCH" in
  x86_64)  ASSET="mxr-linux-x86_64" ;;
  aarch64) ASSET="mxr-linux-aarch64" ;;
  *) echo "Unsupported arch: $ARCH" >&2; exit 1 ;;
esac
URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"
curl -sSfL "$URL" -o "${BIN_DIR}/mxr"
chmod +x "${BIN_DIR}/mxr"
echo "Installed mxr to ${BIN_DIR}/mxr"
