#!/usr/bin/env bash
set -euo pipefail

if [[ -z "${IN_NIX_SHELL:-}" ]]; then
  exec nix develop --command "$0" "$@"
fi

REPO_ROOT="$(cd "$(dirname "$0")" && pwd)"
APP_NAME="transcrust"
REPO="jaycee1285/transcrust"
VERSION="0.1.0"
TAG="v${VERSION}"
ARCH="$(uname -m)"
PLATFORM="$(uname -s | tr '[:upper:]' '[:lower:]')"
TARBALL="${APP_NAME}-${TAG}-${PLATFORM}-${ARCH}.tar.xz"
DIST_DIR="$REPO_ROOT/dist"
STAGING="$(mktemp -d)"
trap 'rm -rf "$STAGING"' EXIT

echo "==> Building ${APP_NAME} ${TAG} (${PLATFORM}/${ARCH})"
cd "$REPO_ROOT"
cargo build --release --offline

install -d "$STAGING/bin"
cp "$REPO_ROOT/target/release/${APP_NAME}" "$STAGING/bin/${APP_NAME}"

if command -v patchelf >/dev/null 2>&1; then
  echo "==> Stripping Nix-specific RPATH for release portability"
  patchelf --remove-rpath "$STAGING/bin/${APP_NAME}" || true
  patchelf --set-interpreter /lib64/ld-linux-x86-64.so.2 "$STAGING/bin/${APP_NAME}" || true
fi

install -d "$STAGING/share/${APP_NAME}"
cp "$REPO_ROOT/config.example.toml" "$STAGING/share/${APP_NAME}/config.example.toml"
cp "$REPO_ROOT/Smoke-Human-transcrust.md" "$STAGING/share/${APP_NAME}/Smoke-Human-transcrust.md"

mkdir -p "$DIST_DIR"
tar -cJf "$DIST_DIR/$TARBALL" -C "$STAGING" bin share

echo "==> Tarball: $DIST_DIR/$TARBALL"
echo "==> SHA256: $(sha256sum "$DIST_DIR/$TARBALL" | awk '{print $1}')"
echo "==> Publishing ${TAG} asset to GitHub"

if gh release view "$TAG" --repo "$REPO" >/dev/null 2>&1; then
  gh release upload "$TAG" "$DIST_DIR/$TARBALL" --clobber --repo "$REPO"
else
  gh release create "$TAG" "$DIST_DIR/$TARBALL" \
    --repo "$REPO" \
    --title "$TAG" \
    --notes "Personal binary release for NixOS install."
fi

echo "==> Release asset:"
echo "    https://github.com/${REPO}/releases/download/${TAG}/${TARBALL}"
