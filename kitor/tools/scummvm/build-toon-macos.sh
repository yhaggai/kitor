#!/bin/bash
# Build a separate runtime; do not overwrite Kitor's shared ScummVM app.
set -euo pipefail
script_dir="$(cd "$(dirname "$0")" && pwd)"
build_dir="${1:?Usage: build-toon-macos.sh BUILD_DIRECTORY}"
mkdir -p "$build_dir"
build_dir="$(cd "$build_dir" && pwd)"
archive="$build_dir/scummvm-v2026.4.0.tar.gz"
if [[ ! -f "$archive" ]]; then
  curl -fL https://github.com/scummvm/scummvm/archive/refs/tags/v2026.4.0.tar.gz -o "$archive"
fi
echo "e9615bad3794121be740aef7270fe3bec75e9b372fbaadaf8a07bb8fe2aa016a  $archive" | shasum -a 256 -c -
source_dir="$build_dir/scummvm-2026.4.0"
if [[ -e "$source_dir" ]]; then
  echo "Use a fresh build directory; source already exists: $source_dir" >&2
  exit 1
fi
tar -xzf "$archive" -C "$build_dir"
cd "$source_dir"
patch -p1 < "$script_dir/toon-hebrew-string-lifetime.patch"
./configure --disable-all-engines --enable-engine=toon --enable-fribidi --disable-sparkle
make -j"${KITOR_BUILD_JOBS:-8}"
echo "Built $source_dir/scummvm"
