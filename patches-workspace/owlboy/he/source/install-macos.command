#!/bin/sh
set -eu
cd "$(dirname "$0")"
GAME=${1:-"$HOME/Library/Application Support/Steam/steamapps/common/Owlboy/Owlboy.app/Contents/Resources"}
command -v dotnet >/dev/null || { echo 'Install the .NET 9 SDK before continuing (see README).'; exit 1; }
python3 -m venv .venv
.venv/bin/python -m pip install -r requirements.txt
.venv/bin/python tools/build.py --game "$GAME" --out build/package
.venv/bin/python tools/manage.py install --package build/package --game "$GAME" --recovery recovery/0.2.0-beta.2
