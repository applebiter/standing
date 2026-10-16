#!/usr/bin/env bash
# Builds dist/Standing-<version>-x86_64.AppImage.
#
# A portable build: the profile is kept in a "Standing Data" folder
# beside the .AppImage file, so the pair can be moved or copied together.
#
# The binary links against the build machine's glibc, so build on the oldest
# distribution you intend to support.
set -euo pipefail

APPIMAGETOOL_VERSION=1.9.1

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"

command -v cargo >/dev/null || { echo "cargo not found. Open a new terminal, or run: . ~/.cargo/env" >&2; exit 1; }

version="$(cargo metadata --no-deps --format-version 1 \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["packages"][0]["version"])')"
tools="$root/target/tools"
appimagetool="$tools/appimagetool-$APPIMAGETOOL_VERSION-x86_64.AppImage"
appdir="$root/target/appimage/Standing.AppDir"
out="$root/dist/Standing-$version-x86_64.AppImage"

if [[ ! -x "$appimagetool" ]]; then
  mkdir -p "$tools"
  echo "Fetching appimagetool $APPIMAGETOOL_VERSION"
  curl -fsSL -o "$appimagetool.part" \
    "https://github.com/AppImage/appimagetool/releases/download/$APPIMAGETOOL_VERSION/appimagetool-x86_64.AppImage"
  chmod +x "$appimagetool.part"
  mv "$appimagetool.part" "$appimagetool"
fi

cargo build --release --locked

rm -rf "$appdir"
mkdir -p "$appdir/usr/bin" "$appdir/usr/share/icons/hicolor/512x512/apps"
install -m 755 target/release/standing "$appdir/usr/bin/standing"
install -m 644 assets/icons/standing.png "$appdir/usr/share/icons/hicolor/512x512/apps/standing.png"
install -m 644 assets/icons/standing.png "$appdir/standing.png"
install -m 644 packaging/linux/standing.desktop "$appdir/standing.desktop"
ln -s usr/bin/standing "$appdir/AppRun"

mkdir -p dist
# --appimage-extract-and-run lets this work on machines without FUSE (e.g. CI).
ARCH=x86_64 "$appimagetool" --appimage-extract-and-run --no-appstream "$appdir" "$out"
echo "Built $out"
