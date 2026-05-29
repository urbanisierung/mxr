#!/bin/sh
set -eu
REPO="urbanisierung/mxr"
BIN_DIR="${HOME}/.local/bin"
mkdir -p "$BIN_DIR"
ARCH=$(uname -m)
case "$ARCH" in
  x86_64)  ASSET="mxr-linux-x86_64" ;;
  aarch64) ASSET="mxr-linux-aarch64" ;;
  *) echo "Unsupported arch: $ARCH" >&2; exit 1 ;;
esac
# releases/latest only resolves full releases; mxr's 0.x releases are
# prereleases, so resolve the newest release (prereleases included) via the API.
TAG=$(curl -sSfL "https://api.github.com/repos/${REPO}/releases?per_page=1" \
  | grep -m1 '"tag_name":' | cut -d'"' -f4)
if [ -z "$TAG" ]; then
  echo "Could not resolve a release for ${REPO}." >&2
  echo "See https://github.com/${REPO}/releases" >&2
  exit 1
fi
URL="https://github.com/${REPO}/releases/download/${TAG}/${ASSET}"
curl -sSfL "$URL" -o "${BIN_DIR}/mxr"
chmod +x "${BIN_DIR}/mxr"
echo "Installed mxr ${TAG} to ${BIN_DIR}/mxr"
