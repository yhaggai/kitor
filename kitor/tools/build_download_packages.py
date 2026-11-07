#!/usr/bin/env python3
"""Package formerly embedded translations without changing their install plans.

Platform/edition manifests stay separate so candidates retain their exact scope.
These archives use the ordinary download, checksum and journaled install path.
"""
import argparse
import hashlib
import json
import re
from pathlib import Path
import stat
import subprocess
import tempfile
import zipfile

from build_patch_packages import require, validate

ROOT = Path(__file__).resolve().parents[2]
INDEX = ROOT / 'kitor/patches/download-packages.json'
REGISTRY = ROOT / 'kitor/patches/registry.json'


def registry_text(registry, previous):
    result = json.dumps(registry, indent=2, ensure_ascii=False) + '\n'
    # Keep the catalog's existing compact arrays readable and avoid unrelated
    # formatting changes when updating download metadata.
    for match in re.finditer(r'(?m)^(\s*)"([^"\n]+)": (\[[^\n]*\])([,]?)$', previous):
        indent, key, value, comma = match.groups()
        expanded = json.dumps(json.loads(value), indent=2, ensure_ascii=False)
        expanded = expanded.replace('\n', '\n' + indent)
        result = result.replace(f'{indent}"{key}": {expanded}{comma}', match.group(0))
    return result


def build(game, patch, plans, output):
    assets = {}
    filename = f"{game['scummvm_id']}-{patch['language']}-{patch['version']}.zip"
    supported = {p for p in ('windows', 'osx') if game['platforms'][p]}
    require(set(plans) == supported, 'Download platforms differ from catalog support')
    for platform, editions in plans.items():
        require(set(editions) == ({'classic', 'remastered'} if game['scummvm_id'] == 'tentacle' else {'default'}),
                'Unexpected edition selection')
        for edition, source in editions.items():
            manifest, files = validate(ROOT / source, platform)
            require(manifest['language'] == patch['language'], f'Mismatched language: {source}')
            require(manifest['version'] + '-download1' == patch['version'], f'Mismatched version: {source}')
            require(manifest['scummvm_id'] == game['scummvm_id'], 'Mismatched game identity')
            require(manifest.get('release_revision', 0) + 1 == patch.get('release_revision', 0),
                    'Mismatched release revision')
            packaged = json.loads(files['patch.json'])
            packaged.update(version=patch['version'], release_revision=patch['release_revision'], filename=filename)
            files['patch.json'] = (json.dumps(packaged, indent=2, ensure_ascii=False) + '\n').encode()
            for name, data in files.items():
                assets[f'_platforms/{platform}/{edition}/{name}'] = data
    output.mkdir(parents=True, exist_ok=True)
    destination = output / filename
    with zipfile.ZipFile(destination, 'w', compression=zipfile.ZIP_STORED) as archive:
        for name, data in sorted(assets.items()):
            info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            info.create_system = 3
            info.external_attr = (stat.S_IFREG | 0o644) << 16
            archive.writestr(info, data)
    tag = f"patch-{game['scummvm_id']}-{patch['language']}-v{patch['version']}"
    with destination.open('rb') as stream:
        checksum = hashlib.file_digest(stream, 'sha256').hexdigest()
    metadata = dict(source_type='github', filename=filename,
                    download_url=f'https://github.com/yhaggai/kitor/releases/download/{tag}/{filename}',
                    checksum=checksum, file_size=destination.stat().st_size, install_manifest='patch.json')
    return destination, metadata


def build_all(output, update_registry=False):
    previous = REGISTRY.read_text()
    registry = json.loads(previous)
    results = []
    for identity, plans in json.loads(INDEX.read_text()).items():
        game = next(g for g in registry['games'] if g['scummvm_id'] == identity)
        patch = next(p for p in game['patches'] if p['language'] == 'he')
        artifact, metadata = build(game, patch, plans, output)
        if update_registry:
            patch.update(metadata)
            game['behavior'].pop('bundled_resource', None)
            game['behavior'].pop('bundled', None)
        else:
            require(all(patch.get(k) == v for k, v in metadata.items()),
                    f'{identity}: download metadata is stale; run with --update-registry')
        results.append((artifact, game, patch))
    if update_registry:
        REGISTRY.write_text(registry_text(registry, previous))
    return results


def verify_published():
    """App releases must not point users at missing or mismatched ZIPs."""
    identities = json.loads(INDEX.read_text())
    for game in json.loads(REGISTRY.read_text())['games']:
        if game['scummvm_id'] not in identities:
            continue
        for patch in game['patches']:
            tag = patch['download_url'].split('/download/', 1)[1].split('/', 1)[0]
            release = json.loads(subprocess.check_output(
                ['gh', 'api', f'repos/yhaggai/kitor/releases/tags/{tag}'], text=True))
            require(not release['draft'] and not release['prerelease'],
                    f'Patch release is not published: {tag}')
            with tempfile.TemporaryDirectory() as temp:
                subprocess.run(['gh', 'release', 'download', tag, '--repo', 'yhaggai/kitor',
                                '--pattern', patch['filename'], '--dir', temp], check=True)
                artifact = Path(temp) / patch['filename']
                with artifact.open('rb') as stream:
                    checksum = hashlib.file_digest(stream, 'sha256').hexdigest()
                require(checksum == patch['checksum'] and artifact.stat().st_size == patch['file_size'],
                        f"Published package differs from catalog: {game['scummvm_id']}")


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT / 'dist/download-patches')
    parser.add_argument('--update-registry', action='store_true')
    parser.add_argument('--verify-published', action='store_true')
    args = parser.parse_args()
    if args.verify_published:
        verify_published()
    else:
        for artifact, _, _ in build_all(args.output, args.update_registry):
            print(artifact)
