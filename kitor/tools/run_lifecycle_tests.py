#!/usr/bin/env python3
"""Run actual lifecycle tests without Tauri, from their required fixture cwd."""
import json
import os
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parent
build = subprocess.run(['cargo', 'test', '--manifest-path', str(root / 'lifecycle-harness/Cargo.toml'), '--locked', '--release', '--no-run', '--message-format=json'], capture_output=True, text=True)
print(build.stderr, end='', file=sys.stderr)
if build.returncode:
    print(build.stdout, end='')
    sys.exit(build.returncode)
binaries = []
for line in build.stdout.splitlines():
    event = json.loads(line)
    if event.get('reason') == 'compiler-artifact' and event.get('profile', {}).get('test') and event.get('executable'):
        binaries.append(event['executable'])
if not binaries:
    raise SystemExit('No lifecycle test binaries built')
# Exercise the same reviewed macOS decoder shipped in the app bundle.
if sys.platform == 'darwin':
    os.environ['PATH'] = str(root.parent / 'src-tauri/vendor/xdelta') + os.pathsep + os.environ.get('PATH', '')
for binary in binaries:
    result = subprocess.run([binary, *sys.argv[1:]], cwd=root.parent / 'src-tauri')
    if result.returncode:
        sys.exit(result.returncode)
