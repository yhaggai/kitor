"""Upgrade the two registered local test copies through Kitor's recovery system."""
import datetime
import hashlib
import json
from pathlib import Path
import sqlite3
import subprocess

ROOT = Path(__file__).resolve().parents[4]
HARNESS = ROOT / 'kitor/tools/lifecycle-harness/target/release/monkey_native_patch.exe'


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def main():
    db = Path.home() / 'AppData/Roaming/haggai/kitor.db'
    connection = sqlite3.connect(db)
    backup = ROOT / 'dist/monkey-native-tests' / ('kitor-before-f1-' + datetime.datetime.now().strftime('%Y%m%d-%H%M%S') + '.db')
    with sqlite3.connect(backup) as dest:
        connection.backup(dest)
    for game in ('monkey-se-native', 'monkey2-se-native'):
        row = connection.execute('SELECT g.id,g.install_path,p.backup_path FROM games g JOIN installed_patches p ON p.game_id=g.id WHERE g.scummvm_id=? AND p.status=?', (game, 'active')).fetchone()
        assert row
        game_id, folder, journal_path = row
        target, journal = Path(folder), Path(journal_path)
        normalized_target = Path(str(target).removeprefix('\\\\?\\')).resolve()
        assert normalized_target.is_relative_to((ROOT / 'dist/monkey-native-tests').resolve())
        sentinel = target / 'kitor-validation-save.bin'
        before = sentinel.read_bytes()
        for action in ('update', 'verify'):
            result = subprocess.run([str(HARNESS), action, game, folder, journal_path], capture_output=True, text=True)
            if result.returncode:
                raise RuntimeError(f'{game} {action}: {result.stderr}')
            print(game, action, result.stdout.strip(), flush=True)
        state = json.loads((journal / 'manifest.json').read_text())
        assert state['status'] == 'active' and state['version'] == '0.2.0-native-candidate'
        for index, record in enumerate(state['files']):
            assert digest(journal / state['originals_dir'] / record.get('original_backup', str(index))) == record['original_hash']
        assert sentinel.read_bytes() == before
        with connection:
            connection.execute('UPDATE installed_patches SET version=? WHERE game_id=? AND backup_path=?', (state['version'], game_id, journal_path))
        package = ROOT / 'patches-workspace' / game / 'he'
        report_path = package / 'validation.json'
        report = json.loads(report_path.read_text())
        report['classic_update'] = dict(version=state['version'], update='passed', installed_hashes='passed', original_backups='verified', unrelated_save='preserved', runtime='User confirmed Hebrew in both graphics modes and working speech in opening scenes; MI1 lookout rechecked')
        report_path.write_text(json.dumps(report, indent=2))
        print(game, 'upgrade verified and recorded in Kitor', flush=True)


if __name__ == '__main__':
    main()
