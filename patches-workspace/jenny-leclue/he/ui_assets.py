"""Read and rebuild the game's three-string UI records without changing other languages."""
import json
import struct
from pathlib import Path
from layout import display

BASE = Path(__file__).resolve().parent
U32 = struct.Struct('<I')


def field(text):
    data = text.encode('utf-8')
    return U32.pack(len(data)) + data + bytes(-len(data) % 4)


def read_language(raw, language):
    start = raw.index(field(language), raw.index(field('en')))
    pos = start + len(field(language))
    count = U32.unpack_from(raw, pos)[0]
    pos += 4
    records = []
    def string():
        nonlocal pos
        length = U32.unpack_from(raw, pos)[0]
        text = raw[pos + 4:pos + 4 + length].decode('utf-8')
        pos += 4 + (length + 3) // 4 * 4
        return text
    for _ in range(count):
        records.append((string(), string(), string()))
    return start, pos, records


def catalog():
    rows = json.loads((BASE / 'ui-catalog.json').read_text())
    result = {(row['object_id'], row['key']): row for row in rows}
    assert len(result) == len(rows) == 1931, 'Incomplete or duplicate UI source IDs'
    for row in rows:
        assert row.get('translation_model') == 'gpt-6-astra', row['id']
        for source, target in [('en', 'he'), ('alternate_en', 'alternate_he')]:
            assert target in row and (not row[source].strip() or row[target].strip()), row['id']
    return result


def render(text, mapping=None, glyphs=None):
    # Unity resolves escaped line breaks after lookup; bidi must see their lines first.
    text = (mapping or {}).get(text, text)
    return display(text.replace('\\n', '\n'), False, glyphs)


def replace_hebrew(raw, oid, rows, mapping=None, glyphs=None, manuscript=None):
    start, end, records = read_language(raw, 'he')
    output = []
    for key, previous, previous_alt in records:
        row = rows[(oid, key)]
        texts = []
        for old, name in [(previous, 'he'), (previous_alt, 'alternate_he')]:
            if '<pos=' in old:
                # Scripted typing requires the existing reviewed per-letter positions.
                new = (manuscript or {}).get(old, old)
                if glyphs:
                    import re
                    new = re.sub(r'[א-ת][\u05b0-\u05bd\u05bf\u05c1\u05c2\u05c7]+', lambda m: glyphs[m[0]], new)
            else:
                new = render(row[name], mapping, glyphs)
            texts.append(new)
        output.append(field(key) + field(texts[0]) + field(texts[1]))
    return raw[:start] + field('he') + U32.pack(len(records)) + b''.join(output) + raw[end:]


def main():
    import argparse
    import UnityPy
    parser = argparse.ArgumentParser(description='Update all UI tables in an unpointed Hebrew resource.')
    parser.add_argument('source', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    windows = args.source.name.endswith('.assets')
    env = UnityPy.load(str(args.source))
    objects = {o.path_id: o for o in env.objects
               if windows or o.assets_file.name == 'resources.assets'}
    rows = catalog()
    for oid in range(14089, 14124):
        obj = objects[oid - (90 if windows else 0)]
        raw = obj.get_raw_data()
        assert field('he-x-nikud') not in raw, 'Use the unpointed build as input'
        obj.set_raw_data(replace_hebrew(raw, oid, rows))
    args.output.write_bytes(env.file.save() if windows else env.file.save(packer='lz4'))


if __name__ == '__main__':
    main()
