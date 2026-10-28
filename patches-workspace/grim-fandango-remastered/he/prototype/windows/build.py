"""Build a local Hebrew experiment; never modifies source game files."""
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
from fontTools.varLib.instancer import instantiateVariableFont


def menu_aliases(source, translations):
    """Reuse existing UI translations only when all matching labels agree."""
    rows = [line.split('\t', 1) for line in source.splitlines() if '\t' in line]
    normalize = lambda value: ' '.join(value.replace('&', '').casefold().split())
    index = {}
    for key, value in rows:
        translated = translations.get(key)
        if key.startswith('sytx') and translated and '\ufffd' not in translated:
            index.setdefault(normalize(value), []).append(key)
    aliases = {}
    for key, value in rows:
        if key in translations or not key.startswith(('sytx', 'utxt', 'keys', 'howp')):
            continue
        candidates = index.get(normalize(value), [])
        if candidates and len({translations[k] for k in candidates}) == 1:
            aliases[key] = candidates
    return aliases


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
    parser.add_argument('--resources', choices=('copy', 'symlink', 'payload'),
                        default='copy' if os.name == 'nt' else 'symlink',
                        help='Windows defaults to copying resources; symlinks require suitable permissions.')
    args = parser.parse_args()
    game, output = args.game.resolve(), args.output.resolve()
    if output.exists():
        raise SystemExit('Output already exists; choose a fresh directory.')
    if game in output.parents:
        raise SystemExit('Output must be outside the source game directory.')
    if not (game / 'FontsHD').is_dir():
        raise SystemExit('Game resource directory must contain FontsHD.')
    font_specs = [p for p in (game / 'FontsHD').iterdir() if p.suffix.lower() == '.txt']
    if not font_specs:
        raise SystemExit('FontsHD contains no font specifications.')
    data = read_lab(args.patch)['grim.tab']
    assert data[:4] == b'RCNE'
    decoded = bytes(c ^ 0xdd for c in data[4:]).decode('cp1255', errors='replace')
    translations = dict(line.split('\t', 1) for line in decoded.splitlines() if '\t' in line)
    original = (game / 'grim.en.tab').read_bytes()
    source = original.decode('cp1252')
    aliases = menu_aliases(source, translations)
    fallback_rows = []
    lines, reused = [], 0
    for line in source.splitlines():
        if '\t' not in line:
            lines.append(line)
            continue
        key, value = line.split('\t', 1)
        translated = translations.get(key)
        if key in aliases:
            translated = translations[aliases[key][0]]
        if translated and '\ufffd' not in translated:
            value = get_display(translated, base_dir='R')
            reused += 1
        else:
            reason = ('missing_id' if key not in translations else
                      'empty_translation' if not translated else 'undecodable_translation')
            fallback_rows.append({'id': key, 'reason': reason})
        lines.append(key + '\t' + value)
    # The native Windows table uses CRLF; do not normalize it to Unix lines.
    newline = '\r\n' if b'\r\n' in original else '\n'
    text = newline.join(lines) + newline
    chars = sorted({c for c in text if ord(c) >= 128})
    if len(chars) > 128:
        raise SystemExit('Too many characters for the experimental byte font.')
    mapping = {chr(i): i for i in range(128)}
    mapping.update({c: i + 128 for i, c in enumerate(chars)})
    font = TTFont(args.font)
    if 'fvar' in font:
        font = instantiateVariableFont(font, {axis.axisTag: axis.defaultValue for axis in font['fvar'].axes}, inplace=True)
    for record in font['name'].names:
        names = {1: 'Kitor Hebrew', 2: 'Regular', 3: 'KitorHebrew-Regular-1',
                 4: 'Kitor Hebrew Regular', 6: 'KitorHebrew-Regular',
                 16: 'Kitor Hebrew', 17: 'Regular'}
        if record.nameID in names:
            record.string = names[record.nameID].encode(record.getEncoding())
    cmap = font.getBestCmap()
    missing = [c for c in chars if ord(c) not in cmap]
    if missing:
        raise SystemExit(f'Font missing characters: {missing}')
    for table in font['cmap'].tables:
        if table.isUnicode():
            table.cmap = {byte: cmap[ord(char)] for char, byte in mapping.items() if ord(char) in cmap}
    output.mkdir(parents=True)
    for path in game.iterdir():
        if args.resources == 'payload':
            continue
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
    # The native executable also loads fonts directly (not through LAF specs).
    for path in (game / 'FontsHD').iterdir():
        if path.suffix.lower() == '.ttf':
            shutil.copy2(fonts / 'HebrewPrototype.ttf', fonts / path.name)
    for path in font_specs:
        # Keep the source encoding and line endings, including on Windows.
        spec = path.read_bytes()
        (fonts / path.name).write_bytes(re.sub(rb'^\S+', b'HebrewPrototype.ttf', spec))
    report = {'status': 'experimental', 'source_game': str(game), 'translated_rows': reused,
              'direct_id_rows': reused - len(aliases), 'menu_alias_rows': len(aliases),
              'menu_aliases': aliases, 'fallback_rows': fallback_rows,
              'total_rows': sum('\t' in line for line in lines),
              'source_table_sha256': hashlib.sha256(original).hexdigest(),
              'patch_sha256': hashlib.sha256(args.patch.read_bytes()).hexdigest(),
              'font_source': str(args.font), 'resource_mode': args.resources, 'source_mutations': [],
              'limitations': ['Preordered bidi text may wrap in the wrong order.',
                             'Local system font is for this local test only; do not redistribute.',
                             'Native Hebrew letters/menu visibility confirmed by user; dialogue layout and full-game playthrough not verified.']}
    (output.parent / (output.name + '-report.json')).write_text(json.dumps(report, indent=2))
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
