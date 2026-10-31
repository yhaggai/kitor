"""Build loose resources for the Space Quest IV Steam CD edition.

python build.py ORIGINAL_GAME EMPTY_OUTPUT
Requires dclimplode, Pillow, python-bidi and the Windows Arial Bold face.
"""
import hashlib
import json
import re
import struct
import sys
import textwrap
from pathlib import Path

from graphics import patch_view, patch_pic, localize_heap, HEAP_ROOMS, SIGNS
import dclimplode
from bidi.algorithm import get_display
from PIL import Image, ImageDraw, ImageFont

HERE = Path(__file__).resolve().parent
TOKEN = re.compile(r'%[ds]|\\0[a-z]|[\x01-\x08][a-f]?')


def encode_line(line):
    tokens = []
    def reserve(match):
        tokens.append(match.group())
        return f'QxToken{len(tokens)}xQ'
    protected = TOKEN.sub(reserve, line)
    visual = get_display(protected, base_dir='R')
    for i, token in enumerate(tokens, 1):
        visual = visual.replace(f'QxToken{i}xQ', token)
    encoded = visual.encode('cp1255')
    return re.sub(rb'!(?=[\xe0-\xfa])', bytes([251]), encoded)


def encode(text, width=32):
    lines = []
    for line in re.split(r'\r\n|\n|\r', text):
        wrapped = textwrap.wrap(line, width, break_long_words=False, break_on_hyphens=False)
        lines.extend(encode_line(part) for part in wrapped or [''])
    return b'\r\n'.join(lines)


def hebrew_font(data):
    _,count,height=struct.unpack_from('<3H',data)
    chars=[]
    for c in range(count):
        off=struct.unpack_from('<H',data,6+2*c)[0]; w,h=data[off:off+2]
        chars.append(data[off:off+2+h*((w+7)//8)])
    while len(chars)<251: chars.append(bytes((1,1,0)))
    font=ImageFont.truetype('C:/Windows/Fonts/arialbd.ttf',max(8,height))
    for c in range(224,251):
        ch=bytes([c]).decode('cp1255'); w=round(font.getlength(ch))+1
        im=Image.new('1',(w,height)); ImageDraw.Draw(im).text((0,height-2),ch,font=font,fill=1,anchor='ls')
        chars[c]=bytes((w,height))+im.tobytes()
    # The visual-order exclamation sits left of the Hebrew word.
    # Reserve one blank column on its right without changing English.
    bang = chars[ord('!')]
    width, glyph_height = bang[:2]
    stride = (width + 7) // 8
    bitmap = bytearray()
    for y in range(glyph_height):
        bitmap.extend(bang[2+y*stride:2+(y+1)*stride])
        if width % 8 == 0:
            bitmap.append(0)
    while len(chars) <= 251:
        chars.append(bytes((1, 1, 0)))
    chars[251] = bytes((width + 1, glyph_height)) + bitmap
    # The status bar has fixed-size string slots. Give its repeated title a
    # single glyph built from this font's existing letters and space.
    title_letters = [chars[c] for c in get_display('מסע בחלל',base_dir='R').encode('cp1255')]
    full_width = sum(glyph[0] for glyph in title_letters)
    joined = Image.new('1',(full_width,height))
    x = 0
    for glyph in title_letters:
        w,h = glyph[:2]
        if w and h:
            joined.paste(Image.frombytes('1',(w,h),glyph[2:]),(x,0))
        x += w
    while len(chars) <= 252:
        chars.append(bytes((1,1,0)))
    chars[252] = bytes((full_width,height)) + joined.tobytes()
    header=bytearray(struct.pack('<3H',0,len(chars),height)); idx=6+2*len(chars)
    for ch in chars: header+=struct.pack('<H',idx); idx+=len(ch)

    return bytes(header) + b''.join(chars)


def main(game, output):
    resource_map = (game / 'RESOURCE.MAP').read_bytes()
    if hashlib.sha256(resource_map).hexdigest() != '62384fb1813c1a140072ff84455736c96f28454b3f96a82211ce3dc5844aa4b6':
        raise ValueError('Expected the Space Quest IV Steam CD resource map')
    volume = (game / 'RESOURCE.000').read_bytes()
    rows = json.loads((HERE / 'messages.json').read_text('utf-8-sig'))
    catalog = {(r['resource'], r['index']): r for r in rows}
    texts = json.loads((HERE / 'text.json').read_text('utf-8-sig'))
    visited = set()
    files = {}
    for directory in range(0, len(resource_map), 3):
        kind, start = struct.unpack_from('<BH', resource_map, directory)
        if kind == 255:
            break
        if kind not in (128,129,131,135,143,145):
            continue
        end = struct.unpack_from('<H', resource_map, directory+4)[0]
        for pos in range(start, end, 5):
            number = int.from_bytes(resource_map[pos:pos+2], 'little')
            if kind == 128 and number not in (150,393,394,610,710,947,972):
                continue
            if kind == 129 and number not in {1,392,600,703,*SIGNS}:
                continue
            if kind == 145 and number not in HEAP_ROOMS:
                continue
            offset = int.from_bytes(resource_map[pos+2:pos+5], 'little') * 2
            typ, num, packed, size, method = struct.unpack_from('<B4H', volume, offset)
            assert (typ, num) == (kind, number) and method in (0, 18, 19, 20)
            data = volume[offset+9:offset+9+packed]
            if method:
                data = dclimplode.decompressobj().decompress(data)
            assert len(data) == size
            if kind == 128:
                name, result = f'{num}.v56', patch_view(num,data)
            elif kind == 129:
                name, result = f'{num}.p56', patch_pic(num,data)
            elif kind == 145:
                name, result = f'{num}.hep', localize_heap(num,data,rows,encode)
                if result == data:
                    continue
            elif kind == 135:
                name, result = f'{num}.fon', hebrew_font(data)
            elif kind == 131:
                if str(num) not in texts:
                    continue
                strings = data.split(b'\0')
                for index, hebrew in texts[str(num)].items():
                    source = strings[int(index)]
                    assert re.findall(rb'%[ds]', source) == re.findall(rb'%[ds]', encode(hebrew))
                    strings[int(index)] = encode(hebrew)
                name, result = f'{num}.tex', b'\0'.join(strings)
            else:
                result = bytearray(data)
                for i in range(struct.unpack_from('<H', data, 8)[0]):
                    row = catalog[num, i]
                    visited.add((num, i))
                    record = 10 + i*11
                    assert list(data[record:record+5]) == [row[k] for k in ('noun','verb','condition','sequence','talker')]
                    field = record + 5
                    base = struct.unpack_from('<H', data, field)[0]
                    source = data[base:data.index(0,base)]
                    assert source.decode('cp437') == row['source']
                    if row['he'] == row['source']:
                        continue
                    translated = encode(row['he'], 40 if row['talker']==99 else 32)
                    assert re.findall(rb'%[ds]', translated) == re.findall(rb'%[ds]', source), (num,i)
                    assert b'\0' not in translated
                    struct.pack_into('<H', result, field, len(result))
                    result.extend(translated + b'\0')
                    pointer = struct.unpack_from('<H', result, field)[0]
                    assert result[pointer:result.index(0,pointer)] == translated
                if result == data:
                    continue
                assert len(result) < 65536
                name = f'{num}.msg'
            files[name] = bytes((kind, 0)) + (bytes(24) if kind in (128,129) else b"") + result
    assert visited == set(catalog) and len(catalog) == len(rows) == 1793
    output.mkdir(exist_ok=True, parents=True)
    if any(output.iterdir()):
        raise ValueError('Choose an empty output directory')
    for name, data in files.items():
        path = output / name
        path.write_bytes(data)
        assert path.read_bytes() == data


if __name__ == '__main__':
    main(Path(sys.argv[1]), Path(sys.argv[2]))
