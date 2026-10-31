#!/usr/bin/env python3
"""Publish immutable reviewed packages, then advertise them in patch-registry."""
import base64
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
from build_patch_packages import build, require


def gh(*args, input=None):
    result = subprocess.run(['gh', *args], input=input, capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(result.stderr.strip())
    return result.stdout


def optional_api(endpoint):
    result = subprocess.run(['gh', 'api', endpoint], capture_output=True, text=True)
    if result.returncode:
        if '(HTTP 404)' in result.stderr:
            return None
        raise RuntimeError(result.stderr.strip())
    return json.loads(result.stdout)


def validate_revision(manifest, previous):
    revision = manifest.get('release_revision')
    require(type(revision) is int and 0 < revision <= 9007199254740991,
            'Published patches need a positive release_revision')
    for key in ('scummvm_id', 'language', 'version', 'filename'):
        require(re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_.-]{0,159}', manifest[key]), f'Invalid release {key}')
    require(manifest['filename'].endswith('.zip'), 'Release filename must end in .zip')
    if previous:
        require(revision >= previous['release_revision'], 'Release revision moved backwards')
        if revision > previous['release_revision']:
            require(manifest['version'] != previous['version'], 'A new revision needs a new immutable version')
        else:
            require(manifest['version'] == previous['version'], 'Version changed without a revision bump')


def merge_release(catalog, release):
    key = (release['scummvm_id'], release['language'])
    old = next((p for p in catalog['patches'] if (p['scummvm_id'], p['language']) == key), None)
    validate_revision(release, old)
    if old and release['release_revision'] == old['release_revision']:
        require(release == old, 'Published metadata or bytes changed; bump version and revision')
        return False
    catalog['patches'] = [p for p in catalog['patches'] if (p['scummvm_id'], p['language']) != key] + [release]
    catalog['patches'].sort(key=lambda p: (p['scummvm_id'], p['language']))
    return True


def digest(path):
    with path.open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()


def publish_asset(repo, commit, tag, artifact, notes):
    release = optional_api(f'repos/{repo}/releases/tags/{tag}')
    if release is None:
        gh('release', 'create', tag, '--repo', repo, '--target', commit, '--draft', '--title', tag, '--notes', notes)
        release = json.loads(gh('api', f'repos/{repo}/releases/tags/{tag}'))
    require(not release['prerelease'], 'Stable registry cannot reference a prerelease')
    if artifact.name in {a['name'] for a in release['assets']}:
        with tempfile.TemporaryDirectory() as temp:
            gh('release', 'download', tag, '--repo', repo, '--pattern', artifact.name, '--dir', temp)
            require(digest(Path(temp) / artifact.name) == digest(artifact),
                    f'Immutable release {tag} differs; bump version and release_revision')
    else:
        require(release['draft'], f'Published release {tag} is incomplete; do not mutate it')
        gh('release', 'upload', tag, str(artifact), '--repo', repo)
    if release['draft']:
        gh('release', 'edit', tag, '--repo', repo, '--draft=false')
    published = json.loads(gh('api', f'repos/{repo}/releases/tags/{tag}'))
    require(not published['draft'] and any(a['name'] == artifact.name and a['size'] == artifact.stat().st_size
                                         for a in published['assets']), 'Release publication failed')


def main():
    repo = os.environ['GITHUB_REPOSITORY']
    require(repo == 'yhaggai/kitor', 'Publisher is configured for yhaggai/kitor')
    commit = os.environ['PATCH_SOURCE_SHA']
    require(re.fullmatch('[0-9a-f]{40}', commit), 'Exact validated source commit required')
    destination = json.loads(gh('api', f'repos/{repo}'))
    require(not destination['private'], 'Public update distribution starts when yhaggai/kitor is made public')
    branch = 'patch-registry'
    existing = optional_api(f'repos/{repo}/contents/registry.json?ref={branch}')
    catalog = json.loads(base64.b64decode(existing['content'])) if existing else {
        'schema_version': 1, 'catalog_revision': 0, 'patches': []}
    require(catalog['schema_version'] == 1, 'Unsupported registry schema')
    registry = json.loads(Path('kitor/patches/registry.json').read_text())
    changed = False
    for path in sorted(Path('patches-workspace').glob('*/*/patch.json')):
        provenance = path.parent / 'provenance.json'
        if not provenance.exists() or json.loads(provenance.read_text()).get('release_ready') is not True:
            continue
        manifest = json.loads(path.read_text())
        old = next((p for p in catalog['patches'] if p['scummvm_id'] == manifest['scummvm_id'] and p['language'] == manifest['language']), None)
        validate_revision(manifest, old)
        game = next(g for g in registry['games'] if g['scummvm_id'] == manifest['scummvm_id'])
        metadata = next(p for p in game['patches'] if p['language'] == manifest['language'])
        if metadata['version'] == manifest['version']:
            require(metadata.get('release_revision', 0) == manifest['release_revision'],
                    'Bundled registry entry must use the same release_revision as its package')
        notes_path = path.parent / 'release-notes.md'
        require(notes_path.is_file(), f'Missing release notes: {notes_path}')
        notes = notes_path.read_text(encoding='utf-8').strip()
        require(bool(notes), 'Release notes are empty')
        with tempfile.TemporaryDirectory() as temp:
            artifact, = build(path, Path(temp), universal=True)
            tag = f"patch-{manifest['scummvm_id']}-{manifest['language']}-v{manifest['version']}"
            release = {
                'scummvm_id': manifest['scummvm_id'], 'language': manifest['language'],
                'version': manifest['version'], 'release_revision': manifest['release_revision'],
                'download_url': f'https://github.com/{repo}/releases/download/{tag}/{artifact.name}',
                'filename': artifact.name, 'checksum': digest(artifact), 'file_size': artifact.stat().st_size,
                'author': metadata['author'], 'description': notes, 'platforms': ['windows', 'osx'],
            }
            did_change = merge_release(catalog, release)
            publish_asset(repo, commit, tag, artifact, notes + f'\n\nSource: {repo}@{commit}')
            changed = changed or did_change
    if not changed:
        print('No new reviewed releases. Registry unchanged.')
        return
    catalog['catalog_revision'] += 1
    if optional_api(f'repos/{repo}/git/ref/heads/{branch}') is None:
        gh('api', f'repos/{repo}/git/refs', '--method', 'POST', '--input', '-', input=json.dumps({
            'ref': f'refs/heads/{branch}', 'sha': commit}))
    body = {'message': f"Publish translation catalog revision {catalog['catalog_revision']}",
            'branch': branch, 'content': base64.b64encode((json.dumps(catalog, indent=2) + '\n').encode()).decode()}
    if existing:
        body['sha'] = existing['sha']
    gh('api', f'repos/{repo}/contents/registry.json', '--method', 'PUT', '--input', '-', input=json.dumps(body))
    print(f"Published translation catalog revision {catalog['catalog_revision']}")


if __name__ == '__main__':
    main()
