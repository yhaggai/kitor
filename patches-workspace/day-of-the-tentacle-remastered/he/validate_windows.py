"""Exercise Kitor's native installers on a disposable copy of an owned archive."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('game', type=Path)
    parser.add_argument('binaries', type=Path)
    args = parser.parse_args()
    manifest = json.loads((ROOT / 'patch.windows.candidate.json').read_text())
    entry = manifest['platforms']['windows']['files'][0]
    assert digest(args.game / 'tenta.cle') == entry['original_sha256']
    with tempfile.TemporaryDirectory(prefix='kitor-dott-windows-') as temporary:
        root = Path(temporary)
        game = root / 'game'
        game.mkdir()
        shutil.copy2(args.game / 'tenta.cle', game / 'tenta.cle')
        (game / 'Dott.exe').write_bytes(b'detection fixture; never executed')
        (game / 'savedata').mkdir()
        (game / 'savedata/sentinel').write_bytes(b'preserve saves')

        def run(binary, action, storage):
            result = subprocess.run([str(args.binaries / (binary + '.exe')),
                                     action, str(game), str(storage)],
                                    capture_output=True, text=True, check=True)
            print(result.stdout.strip(), flush=True)
            if action == 'install':
                return Path(result.stdout.split('Recovery journal: ')[1].strip())

        native = run('tentacle_remastered_patch', 'install', root / 'native-backup')
        assert digest(game / 'tenta.cle') == entry['installed_sha256']
        classic = run('tentacle_patch', 'install', root / 'classic-backup')
        classic_manifest = json.loads((ROOT.parents[1] / 'tentacle/he/patch.remastered-source.json').read_text())
        for file in classic_manifest['platforms']['windows']['files']:
            assert digest(game / file['path']) == file['installed_sha256']
        run('tentacle_remastered_patch', 'uninstall', native)
        assert digest(game / 'tenta.cle') == entry['original_sha256']
        assert (game / 'hebrew/tentacle.000').is_file()
        run('tentacle_patch', 'uninstall', classic)
        assert not (game / 'hebrew/tentacle.000').exists()
        assert (game / 'savedata/sentinel').read_bytes() == b'preserve saves'
    report = {'platform': 'windows', 'input_sha256': entry['original_sha256'],
              'installed_sha256': entry['installed_sha256'],
              'checks': ['native install matches expected hash',
                         'classic installs from native-patched archive',
                         'all classic output hashes match',
                         'native uninstall restores original archive exactly',
                         'native uninstall preserves classic patch',
                         'classic uninstall preserves saves'],
              'runtime': 'User confirmed the Windows test build works on 2026-10-28.'}
    (ROOT / 'validation.windows.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
