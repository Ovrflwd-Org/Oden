#!/usr/bin/env bash
# Builds oden-<arch>.AppImage from an already-built `oden` binary.
#
# Usage: build-appimage.sh <binary-path> <arch> <output-dir>
#   <arch> is the AppImage/appimagetool arch string: x86_64 or aarch64.
#
# Named oden-x86_64.AppImage / oden-aarch64.AppImage in the caller, but
# the release workflow renames the final file to
# oden-<rust-target-triple>.AppImage (e.g. oden-x86_64-unknown-linux-gnu.AppImage)
# so the in-app self-updater's target-triple matching finds it.
set -euo pipefail

BIN_PATH="$1"
ARCH="$2"
OUT_DIR="$3"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

case "$ARCH" in
  x86_64) ;;
  aarch64) ;;
  *) echo "unsupported arch: $ARCH" >&2; exit 1 ;;
esac

WORK_DIR="$(mktemp -d)"
trap 'rm -rf "$WORK_DIR"' EXIT

APPDIR="$WORK_DIR/Oden.AppDir"
mkdir -p "$APPDIR/usr/bin"
cp "$BIN_PATH" "$APPDIR/usr/bin/oden"
chmod +x "$APPDIR/usr/bin/oden"

cp "$REPO_ROOT/packaging/linux/com.outoforder.oden.desktop" "$APPDIR/com.outoforder.oden.desktop"
mkdir -p "$APPDIR/usr/share/metainfo"
cp "$REPO_ROOT/packaging/linux/com.outoforder.oden.metainfo.xml" "$APPDIR/usr/share/metainfo/com.outoforder.oden.metainfo.xml"
cp "$REPO_ROOT/assets/icon-512.png" "$APPDIR/com.outoforder.oden.png"

cat > "$APPDIR/AppRun" <<'APPRUN'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
exec "${HERE}/usr/bin/oden" "$@"
APPRUN
chmod +x "$APPDIR/AppRun"

APPIMAGETOOL="$WORK_DIR/appimagetool-x86_64.AppImage"
curl -fsSL -o "$APPIMAGETOOL" \
  "https://github.com/AppImage/appimagetool/releases/latest/download/appimagetool-x86_64.AppImage"
chmod +x "$APPIMAGETOOL"

mkdir -p "$OUT_DIR"
export APPIMAGE_EXTRACT_AND_RUN=1
ARCH="$ARCH" "$APPIMAGETOOL" "$APPDIR" "$OUT_DIR/oden-${ARCH}.AppImage"
