"""Exercise the actual Kitor installer on the dedicated test copies only."""
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[4]
HARNESS = ROOT / 'kitor/tools/lifecycle-harness/target/release/monkey_native_patch.exe'
TESTS = ROOT / 'dist/monkey-native-tests'


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(chunk)
    return h.hexdigest()


for game, folder in [('monkey-se-native', 'mi1-packaged'), ('monkey2-se-native', 'mi2-packaged')]:
    root = TESTS / folder
    assert root.resolve().is_relative_to(TESTS.resolve())
    package = ROOT / 'patches-workspace' / game / 'he'
    manifest = json.loads((package / 'patch.windows.candidate.json').read_text())
    sentinel = root / 'kitor-validation-save.bin'
    sentinel.write_bytes(b'Unrelated save data must survive installation and removal.')
    hashes = {f['path']: digest(root / f['path']) for f in manifest['files']}
    for f in manifest['files']:
        assert hashes[f['path']] == f['original_sha256'], f['path']
    def run(action, storage):
        return subprocess.check_output([str(HARNESS), action, game, str(root), str(storage)], text=True).strip()
    storage = TESTS / 'recovery'
    journal = run('install', storage)
    for f in manifest['files']:
        assert digest(root / f['path']) == f['installed_sha256']
    run('uninstall', journal)
    for name, expected in hashes.items():
        assert digest(root / name) == expected, name
    assert sentinel.read_bytes() == b'Unrelated save data must survive installation and removal.'
    journal = run('install', storage)
    report = dict(game=game, install_uninstall_roundtrip='passed', reinstall='passed',
                  affected_files=len(hashes), unrelated_save='preserved', journal=journal,
                  runtime='pending', scope='Owned game copies; not original installation')
    (package / 'validation.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    print(json.dumps(report), flush=True)
