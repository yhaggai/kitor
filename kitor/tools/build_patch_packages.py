#!/usr/bin/env python3
"""Validate Kitor lifecycle manifests and package referenced assets only.

This builds data packages, not native executables or compatibility certifications.
Install/uninstall entry points are Kitor's journaled patch lifecycle. Never execute
upstream installers. BSDIFF needs Python 3; macOS Xdelta needs xdelta3 on PATH.
"""
import argparse
import hashlib
import json
import re
import stat
import sys
import zipfile
from pathlib import Path


class InvalidPackage(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise InvalidPackage(message)


def fields(value, allowed, required=()):
    require(isinstance(value, dict), 'Expected JSON object')
    require(not (set(value) - set(allowed)), f'Unsupported fields/status: {set(value) - set(allowed)}')
    require(set(required) <= set(value), f'Missing fields: {set(required) - set(value)}')


def safe_name(value):
    require(isinstance(value, str) and value, 'Empty or invalid path')
    require(not any(c in value for c in '\\:\x00'), f'Unsafe path: {value!r}')
    require(not any(ord(c) < 32 or c in '<>\"|?*' for c in value), f'Unsafe path: {value!r}')
    parts = value.split('/')
    require(all(p not in ('', '.', '..') and not p.endswith((' ', '.')) for p in parts), f'Unsafe path: {value}')
    require(not any(re.fullmatch(r'(CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])(\..*)?', p, re.I) for p in parts), f'Reserved Windows path: {value}')
    return value


def checksum(value):
    require(isinstance(value, str) and re.fullmatch('[0-9a-fA-F]{64}', value), 'Invalid SHA256')


def validate(manifest_path, platform):
    root = manifest_path.parent.resolve()
    require(not manifest_path.is_symlink(), 'Manifest cannot be a symlink')
    provenance_path = root / 'provenance.json'
    if provenance_path.exists():
        provenance = json.loads(provenance_path.read_text(encoding='utf-8'))
        require(provenance.get('status') not in ('blocked', 'no_patch_required', 'built_in'), 'Provenance marks patch noninstallable')
    manifest = json.loads(manifest_path.read_text(encoding='utf-8'))
    fields(manifest, ('schema_version', 'scummvm_id', 'language', 'version', 'filename', 'platforms', 'files'),
           ('schema_version', 'scummvm_id', 'language', 'version', 'filename', 'platforms'))
    require(manifest['schema_version'] == 1, 'Unsupported schema version')
    for key in ('scummvm_id', 'language', 'version', 'filename'):
        safe_name(manifest[key])
        require('/' not in manifest[key], f'{key} must be a single component')
    require(isinstance(manifest['platforms'], dict) and platform in manifest['platforms'], f'Missing {platform} platform')
    for plan in manifest['platforms'].values():
        fields(plan, ('files',))
        require(isinstance(plan.get('files', []), list), 'files must be a list')
    changes = manifest['platforms'][platform].get('files') or manifest.get('files', [])
    require(isinstance(changes, list) and changes, f'Empty {platform} file plan')
    assets = {}
    targets = set()

    def asset(name, digest):
        safe_name(name)
        checksum(digest)
        require(name.lower() != 'patch.json', 'Asset collides with manifest')
        path = root
        for part in name.split('/'):
            path = path / part
            require(not path.is_symlink(), f'Symlink asset: {name}')
        require(path.is_file() and path.resolve().is_relative_to(root), f'Missing asset: {name}')
        data = path.read_bytes()
        require(hashlib.sha256(data).hexdigest() == digest.lower(), f'SHA256 mismatch: {name}')
        for other in assets:
            require(other == name or other.lower() != name.lower(), f'Case-colliding assets: {name}')
        assets[name] = data

    for change in changes:
        fields(change, ('path', 'operation', 'source', 'original_sha256', 'installed_sha256', 'original_md5', 'installed_md5'), ('path', 'operation'))
        name = safe_name(change['path']).lower()
        require(name not in targets, f'Duplicate target: {name}')
        require(not any(name.startswith(p + '/') or p.startswith(name + '/') for p in targets), f'Overlapping target: {name}')
        targets.add(name)
        operation = change['operation']
        require(operation in ('install', 'modify', 'create', 'delete'), f'Unsupported operation: {operation}')
        for key in ('original_sha256', 'installed_sha256'):
            if change.get(key) is not None:
                checksum(change[key])
        for key in ('original_md5', 'installed_md5'):
            if change.get(key) is not None:
                require(isinstance(change[key], str) and re.fullmatch('[0-9a-fA-F]{32}', change[key]), 'Invalid MD5')
        source = change.get('source')
        if operation == 'delete':
            require(source is None, 'Delete cannot have a source')
            continue
        require(isinstance(source, dict), f'Missing source: {name}')
        kind = source.get('kind')
        required = {'kind', 'path', 'sha256'}
        require(kind in ('copy', 'bsdiff', 'xdelta', 'reconstruct'), f'Unsupported source kind: {kind}')
        if kind != 'copy':
            required.add('input')
        if kind == 'xdelta':
            required.update(('windows_tool', 'windows_tool_sha256'))
        fields(source, required, required)
        asset(source['path'], source['sha256'])
        if kind != 'copy':
            safe_name(source['input'])
        if kind == 'xdelta':
            # Validate even on macOS: every declaration must identify a real tool.
            asset(source['windows_tool'], source['windows_tool_sha256'])
    licenses = root / 'licenses'
    require(not licenses.is_symlink(), 'Symlink licenses directory')
    if licenses.exists():
        require(licenses.is_dir(), 'licenses must be a directory')
        for path in sorted(licenses.rglob('*')):
            require(not path.is_symlink(), 'Symlink license file/directory')
            if path.is_dir():
                continue
            require(path.is_file(), 'License must be a regular file')
            require(path.suffix.lower() in ('', '.txt', '.md'), 'License must be a text file')
            data = path.read_bytes()
            try:
                data.decode('utf-8')
            except UnicodeDecodeError as error:
                raise InvalidPackage('License must be UTF-8 text') from error
            require(b'\x00' not in data, 'License contains binary NUL')
            name = path.relative_to(root).as_posix()
            safe_name(name)
            require(not any(other.lower() == name.lower() and other != name for other in assets), 'Case-colliding license')
            assets[name] = data
    # Narrow platform declarations so a ZIP never promises absent platform assets.
    result = dict(manifest, platforms={platform: {'files': changes}}, files=[])
    assets['patch.json'] = (json.dumps(result, indent=2, ensure_ascii=False) + '\n').encode()
    return manifest, assets


def build(manifest_path, output, universal=False, platform=None):
    # Validate both before writing either artifact.
    require(not (universal and platform), 'Universal packages require both platforms; omit --platform')
    selected = (platform,) if platform else ('windows', 'osx')
    plans = [(p, *validate(manifest_path, p)) for p in selected]
    if universal:
        for platform in plans[0][1]['platforms']:
            if platform not in ('windows', 'osx'):
                plans.append((platform, *validate(manifest_path, platform)))
        merged = {}
        for _, _, assets in plans:
            merged.update(assets)
        merged['patch.json'] = (json.dumps(plans[0][1], indent=2, ensure_ascii=False) + '\n').encode()
        plans = [('universal', plans[0][1], merged)]
    output.mkdir(parents=True, exist_ok=True)
    results = []
    for platform, manifest, assets in plans:
        filename = f"{manifest['scummvm_id']}-{manifest['language']}-{manifest['version']}-{platform}.zip"
        if platform == 'universal':
            filename = manifest['filename']
        destination = output / filename
        temporary = destination.with_suffix('.zip.tmp')
        with zipfile.ZipFile(temporary, 'w', compression=zipfile.ZIP_STORED) as archive:
            for name, data in sorted(assets.items()):
                info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
                info.create_system = 3
                info.external_attr = (stat.S_IFREG | 0o644) << 16
                archive.writestr(info, data)
        temporary.replace(destination)
        results.append(destination)
    return results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('manifests', type=Path, nargs='+', help='Explicit patch.json paths; unsupported entries fail')
    parser.add_argument('--output', type=Path, default=Path('dist/patches'))
    parser.add_argument('--platform', choices=('windows', 'osx'), help='Explicitly build/check one supported platform candidate')
    parser.add_argument('--universal', action='store_true', help='Build one validated multi-platform ZIP using manifest filename')
    parser.add_argument('--check', action='store_true', help='Validate both platforms without writing ZIPs')
    args = parser.parse_args()
    try:
        require(not (args.universal and args.platform), 'Universal packages require both platforms; omit --platform')
        for manifest in args.manifests:
            if args.check:
                for platform in ((args.platform,) if args.platform else ('windows', 'osx')):
                    validate(manifest, platform)
                print(f'Validated assets and plans: {manifest}')
            else:
                for artifact in build(manifest, args.output, universal=args.universal, platform=args.platform):
                    print(artifact)
    except (InvalidPackage, OSError, ValueError, TypeError) as error:
        print(f'Package rejected: {error}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
