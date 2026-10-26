"""Selective, journaled installation. Requires only Python 3's standard library."""
import argparse
import contextlib
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import stat
import tempfile


def digest(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':')).encode('utf-8')


def safe_root(value):
    path = Path(os.path.abspath(value))
    for component in [path, *path.parents]:
        if component.is_symlink():
            raise ValueError('Symlink refused: ' + str(component))
    return path


def relative(value):
    if not isinstance(value, str) or not value or '\\' in value or ':' in value:
        raise ValueError('Invalid relative path')
    p = PurePosixPath(value)
    if p.is_absolute() or any(x in ('', '.', '..') for x in value.split('/')):
        raise ValueError('Unsafe relative path: ' + value)
    if any(x.rstrip(' .') != x or x.split('.')[0].upper() in
           {'CON', 'PRN', 'AUX', 'NUL', *('COM%d' % i for i in range(1, 10)),
            *('LPT%d' % i for i in range(1, 10))} for x in p.parts):
        raise ValueError('Nonportable relative path: ' + value)
    return p


def child(root, name):
    p = root.joinpath(*relative(name).parts)
    safe_root(p)
    if p.exists() and not p.is_file():
        raise ValueError('Expected a regular file: ' + str(p))
    return p


def current(path):
    safe_root(path)
    return digest(path.read_bytes()) if path.exists() else None


def atomic(path, data, mode=None):
    safe_root(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp = tempfile.mkstemp(prefix='.owlboy-', dir=path.parent)
    try:
        with os.fdopen(fd, 'wb') as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        if mode is not None:
            os.chmod(tmp, mode)
        os.replace(tmp, path)
        if os.name != 'nt':
            directory = os.open(path.parent, os.O_RDONLY)
            try:
                os.fsync(directory)
            finally:
                os.close(directory)
    finally:
        if os.path.exists(tmp):
            os.unlink(tmp)


@contextlib.contextmanager
def lock(game):
    path = child(game, '.owlboy-hebrew.lock')
    with path.open('a+b') as stream:
        if os.name == 'nt':
            import msvcrt
            stream.seek(0)
            if not stream.read(1):
                stream.write(b'0'); stream.flush()
            stream.seek(0)
            msvcrt.locking(stream.fileno(), msvcrt.LK_NBLCK, 1)
        else:
            import fcntl
            fcntl.flock(stream.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        try:
            yield
        finally:
            if os.name == 'nt':
                stream.seek(0)
                msvcrt.locking(stream.fileno(), msvcrt.LK_UNLCK, 1)
            else:
                fcntl.flock(stream.fileno(), fcntl.LOCK_UN)


def validate_plan(plan):
    if plan.get('schema') != 1 or plan.get('id') != 'owlboy-hebrew' or not plan.get('files'):
        raise ValueError('Unsupported or empty installation plan')
    source_hash = plan.get("source_exe_sha256")
    if not isinstance(source_hash, str) or len(source_hash) != 64 or any(c not in "0123456789abcdef" for c in source_hash):
        raise ValueError("Invalid source executable hash")
    seen = set()
    for record in plan['files']:
        name = str(relative(record['path']))
        if name.casefold() in seen or name.casefold() == '.owlboy-hebrew.lock':
            raise ValueError('Duplicate or reserved path: ' + name)
        seen.add(name.casefold())
        for field in ('original_sha256', 'installed_sha256'):
            value = record[field]
            if field == 'original_sha256' and value is None:
                continue
            if not isinstance(value, str) or len(value) != 64 or any(c not in '0123456789abcdef' for c in value):
                raise ValueError('Invalid file hash')
    paths = sorted(seen)
    if any(b.startswith(a + '/') for a, b in zip(paths, paths[1:])):
        raise ValueError('Overlapping file paths')


def save(journal, data):
    atomic(journal, canonical({'data': data, 'sha256': digest(canonical(data))}))


def load(journal, game):
    envelope = json.loads(journal.read_text(encoding='utf-8'))
    data = envelope['data']
    if digest(canonical(data)) != envelope['sha256']:
        raise ValueError('Journal integrity check failed')
    if data['game'] != str(game):
        raise ValueError('Journal belongs to another game directory')
    validate_plan(data['plan'])
    if data.get('status') not in ('installing', 'installed', 'uninstalling', 'uninstalled'):
        raise ValueError('Invalid journal status')
    if len(data['records']) != len(data['plan']['files']):
        raise ValueError('Invalid journal records')
    return data


def backups_ok(recovery, data):
    for index, (entry, record) in enumerate(zip(data['plan']['files'], data['records'])):
        if entry['original_sha256'] is not None:
            backup = child(recovery, 'backups/%06d' % index)
            if current(backup) != entry['original_sha256']:
                raise ValueError('Missing or corrupt backup: ' + entry['path'])
        if record['state'] not in ('pending', 'installed', 'restored'):
            raise ValueError('Invalid journal state')


def install(package, game, recovery):
    package, game, recovery = map(safe_root, (package, game, recovery))
    if not game.is_dir():
        raise ValueError('Game directory does not exist')
    if recovery == game or game in recovery.parents or recovery == package or package in recovery.parents:
        raise ValueError('Recovery must be outside game and package directories')
    plan = json.loads(child(package, 'plan.json').read_text(encoding='utf-8'))
    validate_plan(plan)
    payload = safe_root(package / 'payload')
    for entry in plan['files']:
        if current(child(payload, entry['path'])) != entry['installed_sha256']:
            raise ValueError('Payload integrity check failed: ' + entry['path'])
    recovery.mkdir(parents=True, exist_ok=True)
    journal = child(recovery, 'journal.json')
    with lock(game):
        if journal.exists():
            data = load(journal, game)
            if data['plan'] != plan:
                raise ValueError('Recovery directory belongs to a different package')
            if data['status'] in ('uninstalling', 'uninstalled'):
                raise ValueError('Finish uninstall and use a new recovery directory')
            backups_ok(recovery, data)
        else:
            if current(child(game, 'Owlboy.exe')) != plan['source_exe_sha256']:
                raise ValueError('Executable edition differs or another patch is installed')
            # All originals are checked before any backup or game mutation.
            for entry in plan['files']:
                if current(child(game, entry['path'])) != entry['original_sha256']:
                    raise ValueError('Original file conflict: ' + entry['path'])
            records = []
            for index, entry in enumerate(plan['files']):
                target = child(game, entry['path'])
                mode = stat.S_IMODE(target.stat().st_mode) if target.exists() else 0o644
                if entry['original_sha256'] is not None:
                    backup = child(recovery, 'backups/%06d' % index)
                    if backup.exists() and current(backup) != entry['original_sha256']:
                        raise ValueError('Existing backup conflict')
                    contents = target.read_bytes()
                    if digest(contents) != entry['original_sha256']:
                        raise ValueError('Original changed during backup')
                    atomic(backup, contents, mode)
                records.append({'mode': mode, 'state': 'pending'})
            directories = set()
            for entry in plan['files']:
                parent = child(game, entry['path']).parent
                while parent != game:
                    if not parent.exists():
                        directories.add(parent.relative_to(game).as_posix())
                    parent = parent.parent
            data = {'game': str(game), 'plan': plan, 'records': records, 'status': 'installing',
                    'created_directories': sorted(directories)}
            save(journal, data)
        # A crash between replacement and journal update is recognized by its hash.
        for entry, record in zip(plan['files'], data['records']):
            actual = current(child(game, entry['path']))
            allowed = (entry['installed_sha256'],) if record['state'] == 'installed' else (entry['original_sha256'], entry['installed_sha256'])
            if actual not in allowed:
                raise ValueError('Installation conflict: ' + entry['path'])
        try:
            for entry, record in zip(plan['files'], data['records']):
                target = child(game, entry['path'])
                if current(target) != entry['installed_sha256']:
                    contents = child(payload, entry['path']).read_bytes()
                    if digest(contents) != entry['installed_sha256']:
                        raise ValueError('Payload changed during install')
                    atomic(target, contents, record['mode'])
                record['state'] = 'installed'
                save(journal, data)
            data['status'] = 'installed'
            save(journal, data)
        except Exception as failure:
            try:
                restore_locked(journal, game, data)
            except Exception as rollback_failure:
                raise RuntimeError('Install failed and rollback could not finish; retain recovery journal: %s' % rollback_failure) from failure
            raise
    return journal


def restore_locked(journal, game, data):
    recovery = journal.parent
    if data['status'] == 'uninstalled':
        return
    backups_ok(recovery, data)
    # Preflight every target and backup before touching any target.
    for entry, record in zip(data['plan']['files'], data['records']):
        actual = current(child(game, entry['path']))
        allowed = (entry['installed_sha256'],)
        if record['state'] in ('pending', 'restored') or data['status'] == 'uninstalling':
            allowed += (entry['original_sha256'],)
        if actual not in allowed:
            raise ValueError('File changed after install; preserved: ' + entry['path'])
    data['status'] = 'uninstalling'
    save(journal, data)
    for index, (entry, record) in enumerate(zip(data['plan']['files'], data['records'])):
        target = child(game, entry['path'])
        if current(target) != entry['original_sha256'] and current(target) == entry['installed_sha256']:
            if entry['original_sha256'] is None:
                target.unlink()
            else:
                contents = child(recovery, 'backups/%06d' % index).read_bytes()
                if digest(contents) != entry['original_sha256']:
                    raise ValueError('Backup changed during uninstall')
                atomic(target, contents, record['mode'])
        record['state'] = 'restored'
        save(journal, data)
    for name in sorted(data.get('created_directories', []), key=lambda x: len(relative(x).parts), reverse=True):
        directory = safe_root(game.joinpath(*relative(name).parts))
        if directory.is_dir() and not any(directory.iterdir()):
            directory.rmdir()
    data['status'] = 'uninstalled'
    save(journal, data)


def uninstall(journal, game):
    journal, game = map(safe_root, (journal, game))
    with lock(game):
        restore_locked(journal, game, load(journal, game))
    # Backups, journal and stable lock remain; only patch-created empty dirs removed.


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    add = sub.add_parser('install')
    for key in ('package', 'game', 'recovery'):
        add.add_argument('--' + key, required=True, type=Path)
    remove = sub.add_parser('uninstall')
    for key in ('journal', 'game'):
        remove.add_argument('--' + key, required=True, type=Path)
    args = parser.parse_args()
    try:
        if args.command == 'install':
            print('Installed; recovery journal:', install(args.package, args.game, args.recovery))
        else:
            uninstall(args.journal, args.game)
            print('Uninstalled; original files restored. Recovery retained.')
    except (OSError, ValueError, KeyError, TypeError, RuntimeError) as error:
        parser.exit(1, 'Stopped: %s\nKeep the recovery directory; retry or uninstall using its journal.\n' % error)


if __name__ == '__main__':
    main()
