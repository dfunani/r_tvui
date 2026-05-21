#!/usr/bin/env bash
# Package r_tvui release archives for GitHub Releases.
# Usage: ./scripts/package-release.sh 0.1.0 aarch64-apple-darwin
set -euo pipefail

VERSION="${1:?Usage: $0 <version> <target>  e.g. 0.1.0 aarch64-apple-darwin}"
TARGET="${2:?Missing target triple}"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

BIN="target/${TARGET}/release/r_tvui"
if [[ "${TARGET}" == *"windows"* ]]; then
  BIN="target/${TARGET}/release/r_tvui.exe"
fi

if [[ ! -f "$BIN" ]]; then
  echo "Binary not found: $BIN"
  echo "Build first: cargo build --release --target ${TARGET}"
  exit 1
fi

mkdir -p dist
cp "$BIN" dist/
cp docs/DISTRIBUTION.md dist/README.txt 2>/dev/null || cp README.md dist/README.txt

OUT="r_tvui-${VERSION}-${TARGET}"
if [[ "${TARGET}" == *"windows"* ]]; then
  (cd dist && zip -r "../${OUT}.zip" .)
  rm -rf dist
  echo "Created ${OUT}.zip"
else
  tar -czvf "${OUT}.tar.gz" -C dist .
  rm -rf dist
  echo "Created ${OUT}.tar.gz"
fi
