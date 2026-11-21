"""Apply or undo a hash-verified staged repair, without touching game saves.

Staging requires repair-plan.json with path, staged, original_sha256 and
installed_sha256 for every affected file. Existing patch backups stay intact.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import tempfile


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def rooted(root, relative):
    path = (root / relative).resolve()
    if not path.is_relative_to(root) or path == root:
        raise ValueError(f"Escaping path: {relative}")
    return path


def atomic_copy(source, target):
    fd, name = tempfile.mkstemp(dir=target.parent)
    os.close(fd)
    temp = Path(name)
    try:
        shutil.copy2(source, temp)
        temp.replace(target)
    finally:
        temp.unlink(missing_ok=True)


def run(action, root, stage):
    root, stage = root.resolve(strict=True), stage.resolve(strict=True)
    state = root / '.kitor-moomin-repair-26803369'
    record = state / 'manifest.json'
    if action == 'install':
        if state.exists():
            raise ValueError('Repair state already exists; inspect it or uninstall first.')
        plan = json.loads((stage / 'repair-plan.json').read_text())
        targets = [rooted(root, e['path']) for e in plan['files']]
        if len(set(targets)) != len(targets):
            raise ValueError('Duplicate targets')
        for e, target in zip(plan['files'], targets):
            if digest(target) != e['original_sha256']:
                raise ValueError(f"Unexpected game version: {e['path']}")
            if digest(rooted(stage, e['staged'])) != e['installed_sha256']:
                raise ValueError(f"Damaged staged repair: {e['path']}")
        state.mkdir()
        plan.update(game_root=str(root), status='backing_up')
        for index, e in enumerate(plan['files']):
            e.update(backup=str(index), originally_existed=True)
        record.write_text(json.dumps(plan, indent=2))
        for e, target in zip(plan['files'], targets):
            backup = rooted(state, e['backup'])
            shutil.copy2(target, backup)
            if digest(backup) != e['original_sha256']:
                raise ValueError('Backup verification failed; no game files changed')
        plan['status'] = 'applying'
        record.write_text(json.dumps(plan, indent=2))
        changed = []
        try:
            for e, target in zip(plan['files'], targets):
                if digest(target) != e['original_sha256']:
                    raise ValueError('Game changed during repair')
                changed.append((e, target))
                atomic_copy(rooted(stage, e['staged']), target)
                if digest(target) != e['installed_sha256']:
                    raise ValueError('Installed hash mismatch')
        except BaseException:
            for e, target in reversed(changed):
                atomic_copy(rooted(state, e['backup']), target)
            plan['status'] = 'rolled_back'
            record.write_text(json.dumps(plan, indent=2))
            raise
        plan['status'] = 'installed'
    else:
        plan = json.loads(record.read_text())
        if plan['game_root'] != str(root):
            raise ValueError('Wrong installation')
        for e in plan['files']:
            if digest(rooted(state, e['backup'])) != e['original_sha256']:
                raise ValueError('Missing or damaged backup')
            if digest(rooted(root, e['path'])) not in (e['original_sha256'], e['installed_sha256']):
                raise ValueError('File changed after repair; refusing to overwrite')
        for e in plan['files']:
            target = rooted(root, e['path'])
            atomic_copy(rooted(state, e['backup']), target)
            if digest(target) != e['original_sha256']:
                raise ValueError('Restore hash mismatch')
        plan['status'] = 'restored'
    record.write_text(json.dumps(plan, indent=2))
    print(plan['status'], record)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['install', 'uninstall'])
    parser.add_argument('game_root', type=Path)
    parser.add_argument('stage', type=Path)
    args = parser.parse_args()
    run(args.action, args.game_root, args.stage)
