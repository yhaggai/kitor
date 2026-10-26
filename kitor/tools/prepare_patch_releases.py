"""Build approved release artifacts in a job with no repository write token."""
import hashlib
import json
from pathlib import Path
import re

from build_patch_packages import build


def release_metadata(manifest):
    for key in ('scummvm_id', 'language', 'version'):
        if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9._-]{0,100}', manifest[key]):
            raise ValueError(f'Unsafe release {key}')
    filename = manifest['filename']
    if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9._-]{0,180}\.zip', filename):
        raise ValueError('Unsafe release filename')
    return {'tag': f"patch-{manifest['scummvm_id']}-{manifest['language']}-v{manifest['version']}",
            'filename': filename}


def prepare(workspace, output):
    output.mkdir(parents=True, exist_ok=True)
    releases = []
    tags, names = set(), set()
    for path in sorted(workspace.glob('*/he/patch.json')):
        provenance = path.with_name('provenance.json')
        if not provenance.is_file() or json.loads(provenance.read_text()).get('release_ready') is not True:
            continue
        metadata = release_metadata(json.loads(path.read_text()))
        if metadata['tag'] in tags or metadata['filename'] in names:
            raise ValueError('Duplicate release tag or asset filename')
        tags.add(metadata['tag'])
        names.add(metadata['filename'])
        archive, = build(path, output, universal=True)
        with archive.open('rb') as stream:
            metadata['sha256'] = hashlib.file_digest(stream, 'sha256').hexdigest()
        releases.append(metadata)
    (output / 'releases.json').write_text(json.dumps(releases, indent=2) + '\n')


if __name__ == '__main__':
    prepare(Path('patches-workspace'), Path('dist/release-packages'))
