"""Build local native Hebrew subtitle/font overlays from inspected mappings.

Requires Pillow and python-bidi. Never modifies an installed game.
Generated files include original game resources: keep local, do not distribute.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct
from PIL import Image, ImageDraw, ImageFont
from bidi.algorithm import get_display
from inspect_resources import speech_rows

SLOTS = [0x118, 0x11a, 0x11c, 0x11e, 0x120, 0x122, 0x126, 0x128,
         0x12a, 0x12c, 0x12e, 0x130, 0x132, 0x134, 0x136, 0x138,
         0x13a, 0x13c, 0x13e, 0x140, 0x142, 0x144, 0x146, 0x14a,
         0x14e, 0x150, 0x152]
CHARS = ''.join(chr(c) for c in range(0x5d0, 0x5eb))
ENCODING = {c: (slot - 0x1a) // 2 for c, slot in zip(CHARS, SLOTS)}


def encode(text):
    if '\\x' in text or '\ufffd' in text:
        raise ValueError('Unresolved source encoding/control sequence')
    # SCUMM ^ splits speech segments; a native subtitle needs a visible break.
    text = text.replace('^', '\n').replace('@', '')
    result = bytearray()
    for i, line in enumerate(text.split('\n')):
        if i:
            result.append(10)
        line = re.sub(r'%[0-9.]*[disf]', lambda m: '\u200e' + m[0] + '\u200e', line)
        for char in get_display(line).replace('\u200e', ''):
            if char in ENCODING:
                result.append(ENCODING[char])
            elif ord(char) < 128:
                result.append(ord(char))
            else:
                raise ValueError(f'Unsupported character {char!r}')
    return bytes(result)


def extend_font(path, target, ttf):
    data = bytearray(path.read_bytes())
    count = struct.unpack_from('<I', data, 4)[0]
    assert count in (154, 155)
    assert len(data) == 0x4260 + count * 16
    original = Image.open(path.with_suffix('.png')).convert('RGBA')
    height = struct.unpack_from('<h', data, 0x4260 + 6)[0]
    font = ImageFont.truetype(str(ttf), max(6, int(height * .77)))
    stroke = 1 if '_bo_' in path.name and height >= 16 else 0
    cells = []
    for char in CHARS:
        box = font.getbbox(char, stroke_width=stroke)
        width = box[2] - box[0] + 3
        cell = Image.new('RGBA', (width, height))
        y = max(0, (height - (box[3] - box[1])) // 2)
        ImageDraw.Draw(cell).text((1 - box[0], y - box[1]), char, font=font,
                                 fill='white', stroke_width=stroke, stroke_fill='black')
        cells.append(cell)
    x, y = 2, original.height
    positions = []
    for cell in cells:
        if x + cell.width + 2 > original.width:
            x, y = 2, y + height + 2
        positions.append((x, y))
        x += cell.width + 2
    output_height = 1 << (y + height + 1).bit_length()
    image = Image.new('RGBA', (original.width, output_height))
    image.paste(original, (0, 0))
    for i, (cell, (x, y), slot) in enumerate(zip(cells, positions, SLOTS)):
        image.paste(cell, (x, y))
        struct.pack_into('<H', data, slot, count + i)
        data.extend(struct.pack('<8h', x, y + 1, x + cell.width, y + height,
                                0, cell.width, cell.width, 0))
    struct.pack_into('<I', data, 4, count + len(CHARS))
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(data)
    image.save(target.with_suffix('.png'))


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--input', type=Path, required=True)
    p.add_argument('--font', type=Path, required=True)
    a = p.parse_args()
    mapping = json.loads((a.input / 'mapping.json').read_text(encoding='utf-8'))
    game = mapping['game']
    name = 'audio/speech.info' if game == 1 else 'localization/en.speech.info'
    original = (a.input / 'original' / name).read_bytes()
    data = bytearray(original)
    translated, skipped = [], []
    for row in mapping['rows']:
        if 'hebrew' not in row:
            continue
        try:
            value = encode(row['hebrew'])
            if game == 1 and len(value) >= 256:
                raise ValueError('Text exceeds fixed 255-byte field')
        except ValueError as error:
            skipped.append(dict(index=row['index'], reason=str(error)))
            continue
        if game == 1:
            start = row['index'] * 1328 + 16
            data[start:start + 256] = value + bytes(256 - len(value))
        else:
            # Keep every record, original string and unknown field in place.
            # Field 24 is the original English lookup key; preserve it.
            for field in (28,):
                ptr = 4 + 32 * row['index'] + field
                struct.pack_into('<I', data, ptr, len(data) - ptr)
            data.extend(value + b'\0')
        translated.append(row['index'])
    # Verify unchanged IDs and every fallback string using the independent reader.
    reparsed = speech_rows(data, game)
    changed = set(translated)
    for before, after in zip(mapping['rows'], reparsed):
        assert before['id'] == after['id']
        if before['index'] not in changed:
            assert before['text'] == after['text']
    if game == 1:
        # English is supplied by the embedded classic engine in this edition.
        # Use the French localization slot, as the upstream translator does.
        for row in mapping['rows']:
            start = row['index'] * 1328 + 16
            data[start + 256:start + 512] = data[start:start + 256]
            data[start:start + 256] = original[start:start + 256]
    else:
        name = 'localization/fr.speech.info'
    target = a.input / 'payload' / name
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(data)
    fonts = list((a.input / 'original/fonts').glob('*.font'))
    for path in fonts:
        extend_font(path, a.input / 'payload/fonts' / path.name, a.font)
    report = dict(game=game, translated=len(translated), total=len(mapping['rows']),
                  skipped=skipped, fonts=len(fonts), runtime='not_tested',
                  original_sha256=hashlib.sha256(original).hexdigest(),
                  installed_sha256=hashlib.sha256(data).hexdigest(),
                  limitations=['UI and unvoiced choices not translated',
                               'RTL paragraph wrapping requires gameplay validation',
                               'Conflicting and unmatched speech retains English'])
    (a.input / 'build-report.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    print(json.dumps({k:v for k,v in report.items() if k != 'skipped'}))


if __name__ == '__main__':
    main()
