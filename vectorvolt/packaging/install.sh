#!/usr/bin/env sh
# VectorVolt universal installer — workerruntime.com
# Supports: Linux (x86_64, aarch64), macOS (x86_64, arm64)
# Usage: curl -fsSL https://install.workerruntime.com | sh
#        or: sh install.sh [--version v0.1.0] [--prefix /usr/local]

set -e

REPO="workerruntime/vectorvolt"
DEFAULT_PREFIX="/usr/local"
VERSION="${VV_VERSION:-latest}"
PREFIX="${VV_PREFIX:-$DEFAULT_PREFIX}"

# ── helpers ───────────────────────────────────────────────────────────────────

info()  { printf '\033[0;34m[vv-install]\033[0m %s\n' "$*"; }
ok()    { printf '\033[0;32m[vv-install]\033[0m %s\n' "$*"; }
err()   { printf '\033[0;31m[vv-install]\033[0m %s\n' "$*" >&2; exit 1; }

need() { command -v "$1" >/dev/null 2>&1 || err "Required tool not found: $1"; }

need curl
need tar

# ── parse args ────────────────────────────────────────────────────────────────

while [ $# -gt 0 ]; do
  case "$1" in
    --version) VERSION="$2"; shift 2 ;;
    --prefix)  PREFIX="$2";  shift 2 ;;
    *) err "Unknown argument: $1" ;;
  esac
done

# ── detect platform ───────────────────────────────────────────────────────────

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
  x86_64)  ARCH="x86_64"  ;;
  aarch64|arm64) ARCH="aarch64" ;;
  *) err "Unsupported architecture: $ARCH" ;;
esac

case "$OS" in
  linux)  PLATFORM="${ARCH}-unknown-linux-musl" ;;
  darwin) PLATFORM="${ARCH}-apple-darwin"       ;;
  *) err "Unsupported OS: $OS" ;;
esac

# ── resolve version ───────────────────────────────────────────────────────────

if [ "$VERSION" = "latest" ]; then
  info "Fetching latest release version..."
  VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' | cut -d'"' -f4)
  [ -n "$VERSION" ] || err "Could not determine latest version"
fi

info "Installing vvctl ${VERSION} for ${PLATFORM}"

# ── download and install ──────────────────────────────────────────────────────

TARBALL="vvctl-${VERSION}-${PLATFORM}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${TARBALL}"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

info "Downloading ${URL}"
curl -fsSL --progress-bar "$URL" -o "${TMP}/${TARBALL}"

info "Extracting..."
tar -xzf "${TMP}/${TARBALL}" -C "$TMP"

# Verify checksum if available
CHECKSUM_URL="${URL}.sha256"
if curl -fsSL "$CHECKSUM_URL" -o "${TMP}/checksum.txt" 2>/dev/null; then
  info "Verifying checksum..."
  (cd "$TMP" && sha256sum -c checksum.txt --ignore-missing) || err "Checksum verification failed"
fi

INSTALL_DIR="${PREFIX}/bin"
mkdir -p "$INSTALL_DIR"
mv "${TMP}/vvctl" "${INSTALL_DIR}/vvctl"
chmod +x "${INSTALL_DIR}/vvctl"

# ── verify ────────────────────────────────────────────────────────────────────

"${INSTALL_DIR}/vvctl" version

ok "vvctl installed to ${INSTALL_DIR}/vvctl"
ok "Run 'vvctl --help' to get started"
ok "Documentation: https://docs.workerruntime.com"
