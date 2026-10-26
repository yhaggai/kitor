"""Build a local-only ScummVM Hebrew experiment; never modifies source game files."""
import argparse
import hashlib
import json
import re
import os
import shutil
import struct
from pathlib import Path
from bidi.algorithm import get_display
from fontTools.ttLib import TTFont


def read_lab(path):
    data = path.read_bytes()
    assert data[:4] == b'LABN'
    count, names_size = struct.unpack_from('<II', data, 8)
    names = data[16 * (count + 1):16 * (count + 1) + names_size]
    result = {}
    for i in range(count):
        name_offset, offset, size, _ = struct.unpack_from('<IIII', data, 16 + i * 16)
        name = names[name_offset:].split(b'\0')[0].decode('ascii').lower()
        assert offset + size <= len(data)
        result[name] = data[offset:offset + size]
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--game', type=Path, required=True)
    parser.add_argument('--patch', type=Path, required=True)
    parser.add_argument('--font', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--resources', choices=('copy', 'symlink'),
                        default='copy' if os.name == 'nt' else 'symlink',
                        help='Windows defaults to copying resources; symlinks require suitable permissions.')
    args = parser.parse_args()
    game, output = args.game.resolve(), args.output.resolve()
    if output.exists():
        raise SystemExit('Output already exists; choose a fresh directory.')
    data = read_lab(args.patch)['grim.tab']
    assert data[:4] == b'RCNE'
    decoded = bytes(c ^ 0xdd for c in data[4:]).decode('cp1255', errors='replace')
    translations = dict(line.split('\t', 1) for line in decoded.splitlines() if '\t' in line)
    original = (game / 'grim.en.tab').read_bytes()
    source = original.decode('cp1252')
    lines, reused = [], 0
    for line in source.splitlines():
        if '\t' not in line:
            lines.append(line)
            continue
        key, value = line.split('\t', 1)
        translated = translations.get(key)
        if translated and '\ufffd' not in translated:
            value = get_display(translated, base_dir='R')
            reused += 1
        lines.append(key + '\t' + value)
    text = '\n'.join(lines) + '\n'
    chars = sorted({c for c in text if ord(c) >= 128})
    if len(chars) > 128:
        raise SystemExit('Too many characters for the experimental byte font.')
    mapping = {chr(i): i for i in range(128)}
    mapping.update({c: i + 128 for i, c in enumerate(chars)})
    font = TTFont(args.font)
    cmap = font.getBestCmap()
    missing = [c for c in chars if ord(c) not in cmap]
    if missing:
        raise SystemExit(f'Font missing characters: {missing}')
    for table in font['cmap'].tables:
        if table.isUnicode():
            table.cmap = {byte: cmap[ord(char)] for char, byte in mapping.items() if ord(char) in cmap}
    output.mkdir(parents=True)
    for path in game.iterdir():
        if path.name.lower() not in ('grim.en.tab', 'fontshd'):
            destination = output / path.name
            if args.resources == 'symlink':
                destination.symlink_to(path, target_is_directory=path.is_dir())
            elif path.is_dir():
                shutil.copytree(path, destination)
            else:
                shutil.copy2(path, destination)
    (output / 'grim.en.tab').write_bytes(bytes(mapping[c] for c in text))
    fonts = output / 'FontsHD'
    fonts.mkdir()
    font.save(fonts / 'HebrewPrototype.ttf')
    for path in (game / 'FontsHD').glob('*.txt'):
        spec = path.read_text()
        (fonts / path.name).write_text(re.sub(r'^\S+', 'HebrewPrototype.ttf', spec))
    report = {'status': 'experimental', 'source_game': str(game), 'translated_rows': reused,
              'total_rows': sum('\t' in line for line in lines),
              'source_table_sha256': hashlib.sha256(original).hexdigest(),
              'patch_sha256': hashlib.sha256(args.patch.read_bytes()).hexdigest(),
              'font_source': str(args.font), 'resource_mode': args.resources, 'source_mutations': [],
              'limitations': ['Preordered bidi text may wrap in the wrong order.',
                             'Local system font is for this local test only; do not redistribute.',
                             'Windows and full-game playthrough not tested.']}
    (output.parent / (output.name + '-report.json')).write_text(json.dumps(report, indent=2))
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
