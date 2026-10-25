#!/usr/bin/env python3
"""Build a portable validation handoff with patch payloads, never owned game data."""
import json
import zipfile
from pathlib import Path
from build_patch_packages import validate

ROOT = Path(__file__).resolve().parents[2]


def main():
    inventory = json.loads((ROOT / 'patches-workspace/owned-catalog.json').read_text())
    destination = ROOT / 'dist/kitor-windows-validation.zip'
    destination.parent.mkdir(exist_ok=True)
    files = {}
    for path in (ROOT / 'kitor/tools').rglob('*'):
        if path.is_file() and not any(part in ('target', '__pycache__') for part in path.relative_to(ROOT / 'kitor/tools').parts):
            if path.suffix in ('.py', '.ps1', '.json', '.md', '.toml', '.lock', '.rs'):
                files[str(path.relative_to(ROOT))] = path
    for relative in ['kitor/src-tauri/src/services/patch_lifecycle.rs',
                     'kitor/src-tauri/src/services/registry_service.rs',
                     'kitor/src-tauri/src/services/apply_bsdiff.py',
                     'kitor/patches/registry.json',
                     'patches-workspace/specification.json',
                     'patches-workspace/owned-catalog.json',
                     'patches-workspace/validation.json',
                     'patches-workspace/README.md']:
        files[relative] = ROOT / relative
    # Stream source files into the ZIP, keeping memory bounded for large archives.
    with zipfile.ZipFile(destination, 'w', zipfile.ZIP_DEFLATED, compresslevel=1) as archive:
        for relative, path in sorted(files.items()):
            archive.write(path, relative)
        for entry in inventory['entries']:
            directory = ROOT / entry['directory']
            archive.write(directory / 'provenance.json', entry['directory'] + '/provenance.json')
            if not entry['manifest']:
                continue
            path = ROOT / entry['manifest']
            manifest, assets = validate(path, 'windows')
            # Keep the source filename so the inventory and script agree.
            archive.write(path, entry['manifest'])
            for name in sorted(assets):
                if name == 'patch.json':
                    continue
                archive.write(directory / name, entry['directory'] + '/' + name)
            del assets
    with zipfile.ZipFile(destination) as archive:
        bad = archive.testzip()
        if bad:
            raise ValueError('Corrupt handoff member: ' + bad)
    print(destination)
    print(f'{destination.stat().st_size:,} bytes; includes patch payloads, not original games')


if __name__ == '__main__':
    main()
