#!/usr/bin/env bash
# Install r_tvui from GitHub Releases into ~/.local/bin (or PREFIX).
set -euo pipefail

REPO="${RTVUI_REPO:-dfunani/r_tvui}"
VERSION="${RTVUI_VERSION:-}"
PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="${PREFIX}/bin"

# Must not be `local` — EXIT trap runs after main() returns and would see an unbound var with `set -u`.
INSTALL_TMP=""

cleanup() {
  if [[ -n "${INSTALL_TMP}" && -d "${INSTALL_TMP}" ]]; then
    rm -rf "${INSTALL_TMP}"
  fi
}

detect_target() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"
  case "$os" in
    Linux)
      case "$arch" in
        x86_64) echo "x86_64-unknown-linux-gnu" ;;
        aarch64|arm64) echo "aarch64-unknown-linux-gnu" ;;
        *) echo "unsupported Linux arch: $arch" >&2; exit 1 ;;
      esac
      ;;
    Darwin)
      case "$arch" in
        arm64) echo "aarch64-apple-darwin" ;;
        x86_64) echo "x86_64-apple-darwin" ;;
        *) echo "unsupported macOS arch: $arch" >&2; exit 1 ;;
      esac
      ;;
    *)
      echo "unsupported OS: $os (see docs/INSTALL.md for Windows or build from source)" >&2
      exit 1
      ;;
  esac
}

latest_version() {
  # Portable sed (BSD + GNU): avoid \? which GNU-only sed treats as optional "v".
  curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | sed -n 's/.*"tag_name": "\([^"]*\)".*/\1/p' \
    | head -1 \
    | sed 's/^v//'
}

main() {
  local target asset_alias asset_versioned base_url
  target="$(detect_target)"

  if [[ -z "${VERSION}" ]]; then
    VERSION="$(latest_version)"
    if [[ -z "${VERSION}" ]]; then
      echo "No GitHub release found. Set RTVUI_VERSION or build from source." >&2
      exit 1
    fi
  fi

  asset_alias="r_tvui-${target}.tar.gz"
  asset_versioned="r_tvui-${VERSION}-${target}.tar.gz"
  base_url="https://github.com/${REPO}/releases/download/v${VERSION}"

  echo "Installing r_tvui v${VERSION} for ${target} -> ${BIN_DIR}"
  mkdir -p "${BIN_DIR}"

  INSTALL_TMP="$(mktemp -d)"
  trap cleanup EXIT

  if curl -fsSL "${base_url}/${asset_alias}" -o "${INSTALL_TMP}/pkg.tar.gz" 2>/dev/null; then
    :
  else
    curl -fsSL "${base_url}/${asset_versioned}" -o "${INSTALL_TMP}/pkg.tar.gz"
  fi
  tar -xzf "${INSTALL_TMP}/pkg.tar.gz" -C "${INSTALL_TMP}"
  install -m 755 "${INSTALL_TMP}/r_tvui" "${BIN_DIR}/r_tvui"

  if [[ ":$PATH:" != *":${BIN_DIR}:"* ]]; then
    echo ""
    echo "Add to your shell profile:"
    echo "  export PATH=\"${BIN_DIR}:\$PATH\""
  fi

  echo ""
  "${BIN_DIR}/r_tvui" --version 2>/dev/null || true
  echo "Done."
  echo ""
  echo "Run:"
  echo "  r_tvui"
  echo "  r_tvui ~/Projects"
  echo ""
  if [[ "$(uname -s)" == "Darwin" ]]; then
    echo "If macOS blocks the binary:"
    echo "  xattr -d com.apple.quarantine \"${BIN_DIR}/r_tvui\""
    echo ""
  fi
  echo "Config: ~/.r_tvui/.config.toml"
  echo "More options: docs/INSTALL.md"
}

main "$@"