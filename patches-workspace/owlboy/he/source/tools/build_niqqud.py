"""Build regular Hebrew and the upstream hn add-on as one reversible package."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import tempfile

import build as regular
import niqqud_trial

VERSION = '0.2.0-beta.2-kitor2'


def build(game, out):
    game, out = game.resolve(), out.resolve()
    if out == game or game in out.parents:
        raise ValueError('Build output must be outside the game')
    with tempfile.TemporaryDirectory(prefix='owlboy-niqqud-') as temporary:
        work = Path(temporary)
        regular.build(game, work / 'regular')
        installed = work / 'installed'
        shutil.copytree(game / 'Data/Localizations', installed / 'Data/Localizations')
        shutil.copytree(work / 'regular/payload', installed, dirs_exist_ok=True)
        niqqud_trial.build(installed, game, work / 'niqqud', full=True)
        payload = out / 'payload'
        shutil.copytree(work / 'regular/payload', payload, dirs_exist_ok=True)
        shutil.copytree(work / 'niqqud/payload', payload, dirs_exist_ok=True)
        # Use the executable linked with this exact runtime helper.
        shutil.copy2(work / 'niqqud/Owlboy.exe', payload / 'Owlboy.exe')
        paths = set()
        for edition in ('regular', 'niqqud'):
            paths.update(f['path'] for f in json.loads((work / edition / 'plan.json').read_text())['files'])
        digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
        files = [{'path': name, 'original_sha256': digest(game / name) if (game / name).exists() else None,
                  'installed_sha256': digest(payload / name)} for name in sorted(paths)]
        plan = {'schema': 1, 'id': 'owlboy-hebrew', 'version': VERSION,
                'source_exe_sha256': digest(game / 'Owlboy.exe'), 'files': files}
        (out / 'plan.json').write_text(json.dumps(plan, indent=2) + '\n', encoding='utf-8')
        for name in ('samples.json', 'clusters.json'):
            shutil.copy2(work / 'niqqud' / name, out / name)
        print(f'Built combined Hebrew + nikud package: {len(files)} files')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--game', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    build(args.game, args.out)
