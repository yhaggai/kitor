"""Inspect owned Special Edition resources and classic translation speech IDs.

Format references: ShadowNate/MISETranslator and jmnunezizu/scummkit.
Outputs contain game resources and must stay local; never publish generated/.
"""
import argparse
import collections
import hashlib
import json
from pathlib import Path
import re
import struct


def archive_files(path):
    with path.open('rb') as f:
        magic, version, index, entries, names, data, ni, ne, nn, nd = struct.unpack('<4s9I', f.read(40))
        if magic != b'KAPL' or version != 0x3f800000 or ne % 20:
            raise ValueError('Unsupported archive')
        f.seek(entries)
        records = list(struct.iter_unpack('<5I', f.read(ne)))
        f.seek(names)
        filenames = f.read(nn).split(b'\0')
        assert len(filenames) == len(records) + 1
        for name, (offset, _, packed, size, compressed) in zip(filenames, records):
            name = name.decode('ascii')
            if not (name.lower().startswith(('fonts/', 'classic/en/')) or 'speech.info' in name.lower() or 'uitext.info' in name.lower()):
                continue
            if compressed or packed != size:
                raise ValueError('Compressed resource: ' + name)
            f.seek(data + offset)
            content = f.read(size)
            assert len(content) == size
            yield name, content


def cstr(data, start):
    return data[start:data.index(0, start)].decode('cp1252', errors='surrogateescape')


def speech_rows(data, game):
    if game == 1:
        assert len(data) % 1328 == 0
        return [dict(index=i // 1328, id=cstr(data, i + 1296), text=cstr(data, i + 16))
                for i in range(0, len(data), 1328)]
    count = (struct.unpack_from('<I', data, 24)[0] + 20) // 32
    assert 4 + count * 32 <= len(data)
    rows = []
    for i in range(count):
        base = 4 + 32 * i
        def ptr(offset):
            p = base + offset
            return cstr(data, p + struct.unpack_from('<I', data, p)[0])
        rows.append(dict(index=i, id=ptr(20), text=ptr(28)))
    return rows


def menu_rows(data, game):
    if game == 1:
        assert len(data) % 1536 == 0
        return [dict(id=cstr(data, i), en=cstr(data, i + 256))
                for i in range(0, len(data), 1536)]
    count = struct.unpack_from('<I', data)[0]
    assert 4 + count * 8 <= len(data)
    rows = []
    for i in range(count):
        base = 4 + i * 8
        values = [cstr(data, p + struct.unpack_from('<I', data, p)[0])
                  for p in (base, base + 4)]
        rows.append(dict(id=values[0], en=values[1]))
    return rows


def translations(strings, table):
    ids = {int(line[:8], 16): line[8:].strip() for line in table.read_text().splitlines()}
    prefix = re.compile(r'^(?:\\xFF\\x0A\\x[0-9A-Fa-f]{2}\\x[0-9A-Fa-f]{2}){4}')
    result = collections.defaultdict(list)
    missing = []
    for number, raw in enumerate(strings.read_bytes().splitlines(), 1):
        line = raw.decode('cp1255')
        match = prefix.match(line)
        if not match:
            continue
        ctrl = bytes(int(x, 16) for x in re.findall(r'\\x([0-9A-Fa-f]{2})', match[0]))
        offset = int.from_bytes(ctrl[2:4] + ctrl[6:8], 'little')
        if offset not in ids:
            missing.append(dict(line=number, offset=offset))
            continue
        result[ids[offset]].append(dict(line=number, text=line[match.end():]))
    return result, missing


def cue_wave_ids(path):
    """Read simple XACT cues; layout verified against FNA-XNA/FAudio FACT_internal.c."""
    data = path.read_bytes()
    assert data[:4] == b'SDBK'
    count = struct.unpack_from('<H', data, 19)[0]
    complex_count = struct.unpack_from('<H', data, 21)[0]
    assert complex_count == 0, 'Complex cues need explicit parsing'
    cues = struct.unpack_from('<I', data, 34)[0]
    names = struct.unpack_from('<I', data, 66)[0]
    result = {}
    for i in range(count):
        name = cstr(data, struct.unpack_from('<I', data, names + i * 6)[0])
        assert data[cues + i * 5] == 4
        sound = struct.unpack_from('<I', data, cues + i * 5 + 1)[0]
        assert not data[sound] & 1, 'Complex sound'
        wave, bank = struct.unpack_from('<HB', data, sound + 9)
        assert bank == 0
        result[name] = f'{wave:08x}'
    return result


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--game', type=int, choices=[1, 2], required=True)
    p.add_argument('--pak', type=Path, required=True)
    p.add_argument('--strings', type=Path, required=True)
    p.add_argument('--table', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    files = dict(archive_files(a.pak))
    if a.game == 1:
        files['audio/speech.info'] = (a.pak.parent / 'audio/speech.info').read_bytes()
        files['localization/uiText.info'] = (a.pak.parent / 'localization/uiText.info').read_bytes()
    for name, content in files.items():
        target = a.output / 'original' / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(content)
    menu_name = 'localization/uiText.info' if a.game == 1 else 'localization/en.uitext.info'
    (a.output / 'menu-source.json').write_text(
        json.dumps(menu_rows(files[menu_name], a.game), ensure_ascii=False, indent=2), encoding='utf-8')
    name = 'audio/speech.info' if a.game == 1 else 'localization/en.speech.info'
    rows = speech_rows(files[name], a.game)
    translations_by_id, missing = translations(a.strings, a.table)
    cue_ids = cue_wave_ids(a.pak.parent / 'audio/SpeechCues.xsb') if a.game == 2 else {}
    matched = []
    for row in rows:
        row['source_id'] = cue_ids.get(row['id'], row['id'])
        candidates = translations_by_id.get(row['source_id'], [])
        unique = {x['text'] for x in candidates}
        row['candidates'] = candidates
        if len(unique) == 1:
            row['hebrew'] = next(iter(unique))
            matched.append(row)
    report = dict(game=a.game, records=len(rows), mapped=len(matched),
                  conflicting=sum(len({x['text'] for x in r['candidates']}) > 1 for r in rows),
                  missing_classic_ids=missing, rows=rows,
                  source_sha256=hashlib.sha256(a.strings.read_bytes()).hexdigest())
    (a.output / 'mapping.json').write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding='utf-8')
    print(json.dumps({k:v for k,v in report.items() if k not in ('rows', 'missing_classic_ids')}))
    print('Unmapped classic offsets:', len(missing))


if __name__ == '__main__':
    main()
