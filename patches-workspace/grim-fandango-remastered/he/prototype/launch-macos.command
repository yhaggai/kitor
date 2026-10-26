#!/bin/zsh
set -eu
prototype_dir="$(cd -- "$(dirname -- "$0")" && pwd)"
runtime_bin="$HOME/Library/Application Support/haggai/scummvm/ScummVM.app/Contents/MacOS/scummvm"
exec "$runtime_bin" --config="$prototype_dir/runtime/scummvm.ini" --logfile="$prototype_dir/runtime/scummvm.log" grim-he-prototype
