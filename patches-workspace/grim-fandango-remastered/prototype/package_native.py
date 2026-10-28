"""Package a prepared font/text payload for Kitor's reversible native lifecycle."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--game', type=Path, required=True)
    parser.add_argument('--payload', type=Path, required=True)
    parser.add_argument('--font-license', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        parser.error('Choose a new output directory.')
    files = sorted(p for p in args.payload.rglob('*') if p.is_file())
    allowed = lambda name: name == 'grim.en.tab' or (name.startswith('FontsHD/') and name.endswith(('.ttf', '.txt')))
    if not files or any(not allowed(p.relative_to(args.payload).as_posix()) for p in files):
        parser.error('Payload must contain only grim.en.tab and FontsHD font files/specifications.')
    if not (args.payload / 'grim.en.tab').is_file():
        parser.error('Payload is missing grim.en.tab.')
    license_bytes = args.font_license.read_bytes()
    args.output.mkdir(parents=True)
    changes = []
    for path in files:
        name = path.relative_to(args.payload).as_posix()
        destination = args.output / 'payload' / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, destination)
        original = args.game / name
        change = {'path': name, 'operation': 'modify' if original.is_file() else 'create',
                  'installed_sha256': digest(path),
                  'source': {'kind': 'copy', 'path': 'payload/' + name, 'sha256': digest(path)}}
        if original.is_file():
            change['original_sha256'] = digest(original)
        changes.append(change)
    licenses = args.output / 'licenses'
    licenses.mkdir()
    (licenses / 'FONT-OFL.txt').write_bytes(license_bytes)
    (licenses / 'CREDITS.txt').write_text(
        'Hebrew translation: Niv Bar and Shani Roz, original AdventureBrew patch.\n'
        'Source: https://drive.google.com/file/d/1DGyW6h9Szx-rVCRmXkR6lBRGAgpVdQIf/view\n'
        'Native Remastered adaptation: Kitor. No new dialogue translation.\n'
        'Kitor Hebrew is a renamed, static, byte-remapped derivative of Noto Sans Hebrew.\n'
        'Font source: https://github.com/google/fonts/tree/main/ofl/notosanshebrew\n'
        'Font derivative remains under the included SIL OFL 1.1.\n', encoding='utf-8')
    manifest = {'schema_version': 1, 'scummvm_id': 'grim-fandango-remastered',
                'language': 'he', 'version': '0.1.0-native-candidate',
                'filename': 'grim-fandango-remastered-he-0.1.0-native-candidate-windows.zip',
                'platforms': {'windows': {}}, 'files': changes}
    (args.output / 'patch.windows.candidate.json').write_text(json.dumps(manifest, indent=2)+'\n', encoding='utf-8')
    print(f'Prepared {len(changes)} guarded file operations in {args.output}')


if __name__ == '__main__':
    main()
