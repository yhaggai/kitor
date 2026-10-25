#!/usr/bin/env python3
"""Fetch two pinned private build inputs; never execute an upstream installer."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

from build_patch_packages import safe_name


def verify(path, entry):
    if not path.is_file() or path.stat().st_size != entry['size']:
        raise ValueError(f"Missing or wrong-size input: {entry['asset']}")
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(block)
    if digest.hexdigest() != entry['sha256']:
        raise ValueError(f"SHA256 mismatch: {entry['asset']}")


def hydrate(root, specification, repository):
    root = root.resolve()
    if specification['schema_version'] != 1:
        raise ValueError('Unsupported build-input schema')
    for entry in specification['assets']:
        safe_name(entry['asset'])
        if '/' in entry['asset']:
            raise ValueError('Asset must be a filename')
        safe_name(entry['destination'])
        destination = root / entry['destination']
        cursor = root
        for part in Path(entry['destination']).parts:
            cursor = cursor / part
            if cursor.is_symlink():
                raise ValueError('Symlink build destination refused')
        if destination.exists():
            verify(destination, entry)
            print(f"Verified existing input: {entry['asset']}")
            continue
        destination.parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix='kitor-build-input-') as directory:
            subprocess.run(['gh', 'release', 'download', specification['release_tag'], '--repo', repository,
                            '--pattern', entry['asset'], '--dir', directory], check=True)
            downloaded = Path(directory) / entry['asset']
            verify(downloaded, entry)
            # Write only a verified source file, atomically within its destination FS.
            temporary = None
            try:
                with tempfile.NamedTemporaryFile(dir=destination.parent, prefix=destination.name + '.', suffix='.hydrate-tmp', delete=False) as output:
                    temporary = Path(output.name)
                    with downloaded.open('rb') as source:
                        shutil.copyfileobj(source, output, 1024 * 1024)
                temporary.replace(destination)
            finally:
                if temporary is not None:
                    temporary.unlink(missing_ok=True)
        print(f"Hydrated verified input: {entry['asset']}")


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', default=os.environ.get('GITHUB_REPOSITORY', 'yhaggai/kitor'))
    args = parser.parse_args()
    here = Path(__file__).resolve().parent
    hydrate(here.parent.parent, json.loads((here / 'patch-build-inputs.json').read_text()), args.repo)
