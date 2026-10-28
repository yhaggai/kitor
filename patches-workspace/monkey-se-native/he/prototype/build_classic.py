"""Adapt the embedded classic resources, keeping native dialogue lookup keys aligned.

Uses nutcracker's SCUMM parser/relocator and AdventureBrew's existing bitmap fonts.
Writes a separate local candidate; never changes installed games.
"""
import argparse
import collections
import contextlib
import json
from pathlib import Path
import re
import struct

from build_candidate import encode, ENCODING
from inspect_resources import cstr
from nutcracker.sputm.tree import open_game_resource, narrow_schema
from nutcracker.sputm.schema import SCHEMA
from nutcracker.sputm.strings import get_all_scripts, get_optable, get_script_map, update_element_strings
from nutcracker.sputm.build import rebuild_resources


def normalize_key(key):
    return re.sub(rb'\s+', b' ', key.replace(b'^', b'...')).strip()


def translation_map(rows):
    candidates = collections.defaultdict(collections.Counter)
    for row in rows:
        texts = [row['hebrew']] if 'hebrew' in row else [c['text'] for c in row.get('candidates', [])]
        for text in texts:
            try:
                value = encode(text).replace(b'\n', b'^')
                key = normalize_key(row['text'].encode('cp1252', errors='surrogateescape'))
            except ValueError:
                continue
            candidates[key][value] += 1
    # Repeated English phrases can have several valid Hebrew phrasings in the
    # talkie translation. Use a consistent existing phrasing in both engines.
    return {key: values.most_common(1)[0][0] for key, values in candidates.items()}


def translate(msg, mapping):
    if normalize_key(msg) in mapping:
        return mapping[normalize_key(msg)]
    # Preserve SCUMM speech-page boundaries and commands byte for byte.
    parts = re.split(b'(\xff[\x01\x02\x03\x08])', msg)
    return b''.join(mapping.get(normalize_key(part), part) if i % 2 == 0 else part for i, part in enumerate(parts))


def classic_font(source, fallback=None):
    data = bytearray(source[8:])
    count = struct.unpack_from('<H', data, 23)[0]
    for char, slot in ENCODING.items():
        code = char.encode('cp1255')[0]
        assert slot < count and code < count
        offset = struct.unpack_from('<I', data, 25 + code * 4)[0]
        if not offset:
            assert fallback is not None and fallback[29] == data[21]
            other = fallback[8:]
            other_count = struct.unpack_from('<H', other, 23)[0]
            offsets = [struct.unpack_from('<I', other, 25 + c * 4)[0] for c in range(other_count)]
            start = offsets[code]
            assert start
            end = min([x for x in offsets if x > start] + [len(other) - 21])
            offset = len(data) - 21
            data.extend(other[21 + start:21 + end])
        struct.pack_into('<I', data, 25 + slot * 4, offset)
    struct.pack_into('<I', data, 0, len(data) - 15)
    return bytes(data)


def replace_fonts(elements, fonts):
    for elem in elements:
        if elem.tag == 'CHAR' and elem.attribs.get('gid') in fonts:
            elem.update_raw(fonts[elem.attribs['gid']])
        elif elem.tag in {'LECF', 'LFLF'}:
            elem.update_children(list(replace_fonts(elem.children(), fonts)))
        yield elem


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input', type=Path, required=True)
    parser.add_argument('--fonts', type=Path, required=True)
    a = parser.parse_args()
    a.input = a.input.resolve()
    report = json.loads((a.input / 'mapping.json').read_text(encoding='utf8'))
    game = report['game']
    mapping = translation_map(report['rows'])
    output = a.input / 'classic-candidate'
    output.mkdir(exist_ok=True)
    with (output / 'build.log').open('w', encoding='utf8') as log, contextlib.redirect_stdout(log):
        resources = open_game_resource(a.input / f'original/classic/en/monkey{game}.000')
        ops, scripts = get_optable(resources.game), get_script_map(resources.game)
        root = list(resources.read_resources(schema=narrow_schema(SCHEMA, {'LECF', 'LFLF', 'RMDA', 'ROOM', 'OBCD', 'TLKE', 'CHAR', *scripts})))
        original = list(get_all_scripts(root, ops, scripts))
        updated = [translate(msg, mapping) for msg in original]
        sources = {int(p.name.split('_')[1]): p.read_bytes() for p in a.fonts.rglob('CHAR_*')}
        fonts = {i: classic_font(data, sources[2]) for i, data in sources.items()}
        root = list(replace_fonts(root, fonts))
        root = list(update_element_strings(root, iter(updated), ops, scripts))
        classic = output / 'classic/en'
        classic.mkdir(parents=True, exist_ok=True)
        rebuild_resources(resources, str(classic / f'monkey{game}'), root)
        rebuilt = open_game_resource(classic / f'monkey{game}.000')
        check = list(get_all_scripts(rebuilt.read_resources(schema=narrow_schema(SCHEMA, {'LECF', 'LFLF', 'RMDA', 'ROOM', 'OBCD', 'TLKE', *scripts})), ops, scripts))
        assert check == updated, 'SCUMM relocation roundtrip failed'
    # Classic changed keys must resolve to the same native speech IDs.
    name = 'audio/speech.info' if game == 1 else 'localization/fr.speech.info'
    data = bytearray((a.input / 'payload' / name).read_bytes())
    keys_changed = 0
    for row in report['rows']:
        i = row['index']
        ptr = i * 1328 + 16 if game == 1 else 4 + i * 32 + 24
        start = ptr if game == 1 else ptr + struct.unpack_from('<I', data, ptr)[0]
        key = data[start:data.index(0, start)]
        new = translate(bytes(key), mapping)
        if new == key:
            continue
        if game == 1:
            assert len(new) < 256
            data[ptr:ptr + 256] = new + bytes(256 - len(new))
        else:
            struct.pack_into('<I', data, ptr, len(data) - ptr)
            data.extend(new + b'\0')
        keys_changed += 1
    target = output / name
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(data)
    summary = dict(game=game, classic_strings=len(original), translated=sum(a != b for a, b in zip(original, updated)),
                   native_keys_updated=keys_changed, font_ids=sorted(fonts), relocation_roundtrip='passed', runtime='pending')
    (output / 'report.json').write_text(json.dumps(summary, indent=2))
    print(json.dumps(summary))


if __name__ == '__main__':
    main()
