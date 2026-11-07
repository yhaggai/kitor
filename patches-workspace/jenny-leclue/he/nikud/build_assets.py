"""Build from a validated regular-Hebrew Steam resource file. Requires UnityPy, Pillow with RAQM, numpy and python-bidi."""
import argparse
import collections
import json
import re
import struct
import sys
from pathlib import Path

import numpy as np
import UnityPy
from PIL import Image, ImageDraw, ImageFilter, ImageFont, features

ROOT = Path(__file__).resolve().parent
BASE = ROOT.parent
sys.path.insert(0, str(BASE))
from layout import display, dialogue_display
from ui_assets import catalog as ui_catalog, replace_hebrew
from ui_fonts import extend as extend_ui_font
from names import normalize_mapping

U32 = struct.Struct('<I')
POINTED = re.compile(r'[א-ת][\u05b0-\u05bd\u05bf\u05c1\u05c2\u05c7]+')


def field(text):
    data = text.encode()
    return U32.pack(len(data)) + data + bytes(-len(data) % 4)


def font_table(obj, texture, glyphs, windows, manuscript):
    raw = obj.get_raw_data()
    stride = 52 if windows else 48
    first = raw.index(U32.pack(10000), 100)
    chars = first + 27 * stride
    count = U32.unpack_from(raw, chars)[0]
    start = chars - count * stride - 4
    assert U32.unpack_from(raw, start)[0] == count
    end = chars + 4 + 16 * count
    tex = texture.read()
    original = tex.image.convert('RGBA')
    assert original.size == (1024, 2048)
    width, height = 4096, 4096
    canvas = Image.new('RGBA', (width, height), 'white')
    canvas.putalpha(0)
    canvas.paste(original, (0, height - original.height))
    face = 'Courier New.ttf' if manuscript else 'Arial Bold.ttf'
    font = ImageFont.truetype('/System/Library/Fonts/Supplemental/' + face, 150)
    assert features.check('raqm')
    x = y = row_height = 0
    records, characters = [], []
    for index, (cluster, symbol) in enumerate(glyphs.items()):
        base = cluster[0]
        metrics = struct.unpack_from('<I5f', raw, first + (ord(base) - 1488) * stride)
        _, bw, bh, bx, by, advance = metrics
        left, top, right, bottom = font.getbbox(base, anchor='ls')
        sx, sy = bw / (right-left), bh / (bottom-top)
        l, t, r, b = font.getbbox(cluster, anchor='ls')
        mask = Image.new('L', (r-l, b-t))
        ImageDraw.Draw(mask).text((-l, -t), cluster, font=font, fill=255, anchor='ls')
        w, h = max(1, round(mask.width*sx)), max(1, round(mask.height*sy))
        tile = Image.new('L', (w+20, h+20))
        tile.paste(mask.resize((w,h), Image.Resampling.LANCZOS), (10,10))
        if x + tile.width > width:
            y += row_height
            x = row_height = 0
        assert y + tile.height <= height-original.height
        distance = np.zeros((tile.height, tile.width), dtype=np.int16)
        for radius in range(1,9):
            distance += np.asarray(tile.filter(ImageFilter.MinFilter(2*radius+1))) > 127
            distance -= np.asarray(tile.filter(ImageFilter.MaxFilter(2*radius+1))) <= 127
        alpha = Image.fromarray((127+15*distance).astype('uint8'))
        rgba = Image.new('RGBA', tile.size, 'white')
        rgba.putalpha(alpha)
        canvas.paste(rgba,(x,y))
        gid = 11000 + index
        records.append(struct.pack('<I5f4ifi',gid,w,h,bx+(l-left)*sx,by+(top-t)*sy,advance,x+10,height-y-10-h,w,h,1.,0)+bytes(stride-48))
        characters.append(struct.pack('<IIIf',1,ord(symbol),gid,1.))
        x += tile.width
        row_height = max(row_height,tile.height)
    # Reuse existing punctuation shapes for Hebrew punctuation code points.
    original_characters = {
        struct.unpack_from('<IIIf', raw, chars + 4 + index * 16)[1]:
        struct.unpack_from('<IIIf', raw, chars + 4 + index * 16)[2]
        for index in range(count)
    }
    original_glyphs = {
        U32.unpack_from(raw, start + 4 + index * stride)[0]:
        raw[start + 4 + index * stride:start + 4 + (index + 1) * stride]
        for index in range(count)
    }
    for index, (hebrew, latin) in enumerate([('׳', "'"), ('״', '"'), ('־', '-')]):
        gid = 12000 + index
        original_record = original_glyphs[original_characters[ord(latin)]]
        records.append(U32.pack(gid) + original_record[4:])
        characters.append(struct.pack('<IIIf', 1, ord(hebrew), gid, 1.))
    tail = raw[end:]
    dims = struct.pack('<II',*original.size)
    assert dims in tail
    new_count = U32.pack(count+len(records))
    obj.set_raw_data(raw[:start]+new_count+raw[start+4:chars]+b''.join(records)+new_count+raw[chars+4:end]+b''.join(characters)+tail.replace(dims,struct.pack('<II',width,height)))
    tex.set_image(canvas)
    tex.save()


def update_materials(objects):
    for obj in objects.values():
        if obj.type.name != 'Material':
            continue
        material = obj.read()
        props = material.m_SavedProperties
        atlas = next((tex.m_Texture.m_PathID for key, tex in props.m_TexEnvs
                      if key == '_MainTex' and tex.m_Texture.m_FileID == 0), None)
        if atlas not in (315, 318, 322, 325):
            continue
        props.m_Floats = [(key, 4096.0 if key in ('_TextureWidth', '_TextureHeight') else value)
                          for key, value in props.m_Floats]
        if atlas == 315:
            for key, color in props.m_Colors:
                if key == '_UnderlayColor':
                    color.a = 0.0
        material.save()


NIKUD = 'he-x-nikud'
NAMES = ('he', 'Menu Text he')
LOCALIZED = U32.pack(4) + field('CustomFieldType_Localization')


def hebrew_fields(raw):
    pattern = re.compile(b'|'.join(re.escape(field(name)) for name in NAMES))
    matches = []
    end = 0
    for match in pattern.finditer(raw):
        if match.start() < end:
            continue
        length = U32.unpack_from(raw, match.end())[0]
        value_end = match.end() + 4 + (length + 3) // 4 * 4
        if not raw.startswith(LOCALIZED, value_end):
            continue
        end = value_end + len(LOCALIZED)
        name = next(name for name in NAMES if field(name) == match[0])
        matches.append((match.start(), end, name, raw[match.end():end]))
    return matches


def merge_dialogue(plain, pointed):
    source = hebrew_fields(plain)
    target = hebrew_fields(pointed)
    output = []
    cursor = 0
    index = 0
    while index < len(source):
        start = source[index][0]
        additions = []
        while True:
            _, end, name, _ = source[index]
            _, _, pointed_name, value = target[index]
            assert name == pointed_name
            additions.append(field(name.replace('he', NIKUD)) + value)
            index += 1
            if index == len(source) or source[index][0] != end:
                break
        assert plain.startswith(field('Title'), end)
        count = U32.unpack_from(plain, start-4)[0]
        assert 1 <= count <= 256
        output.extend((plain[cursor:start-4], U32.pack(count+len(additions)),
                       plain[start:end], b''.join(additions)))
        cursor = end
    assert index == len(target) and index > 0
    return b''.join(output) + plain[cursor:]


def add_language(plain, pointed, settings):
    start = plain.index(field('en'))
    he = pointed.index(field('he'), start)
    end = pointed.index(field('de'), he)
    entry = pointed[he+len(field('he')):end]
    if settings:
        names = field('Hebrew') * 2
        assert entry.startswith(names)
        entry = field('Hebrew with nikud') * 2 + entry[len(names):]
    end = plain.index(field('de'), plain.index(field('he'), start))
    assert U32.unpack_from(plain,start-4)[0] == 16
    return plain[:start-4] + U32.pack(17) + plain[start:end] + field(NIKUD) + entry + plain[end:]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('source',type=Path)
    parser.add_argument('output',type=Path)
    args = parser.parse_args()
    windows = args.source.name.endswith('.assets')
    shift = 90 if windows else 0
    mapping = normalize_mapping(json.loads((ROOT/'pointed.json').read_text()))
    manuscript = normalize_mapping(json.loads((ROOT/'manuscript.json').read_text()))
    ui = ui_catalog()
    clusters = sorted({c for text in [*mapping.values(), *manuscript.values()] for c in POINTED.findall(text)})
    glyphs = {c:chr(0xe000+i) for i,c in enumerate(clusters)}
    env = UnityPy.load(str(args.source))
    objects = {o.path_id:o for o in env.objects if windows or o.assets_file.name=='resources.assets'}
    groups = collections.defaultdict(dict)
    plain_groups = collections.defaultdict(dict)
    for row in json.loads((BASE/'catalog.json').read_text()):
        text = row['he']
        if text not in mapping:
            continue
        dialogue = row['field']=='Dialogue Text'
        prefix = field('he' if dialogue else 'Menu Text he')
        oid = int(row['id'].split(':')[0][6:])-shift
        legacy = prefix+field(display(text,dialogue))
        plain_groups[oid][legacy] = prefix+field(dialogue_display(text))
        groups[oid][legacy] = prefix+field(dialogue_display(mapping[text],glyphs))
    dialogue_ids = {int(row['id'].split(':')[0][6:])-shift
                    for row in json.loads((BASE/'catalog.json').read_text())}
    for oid in sorted(dialogue_ids | {14084-shift} | set(range(14089-shift,14124-shift))):
        raw = objects[oid].get_raw_data()
        pairs = groups.get(oid, {})
        assert all(key in raw for key in pairs), oid
        pointed = raw
        if 14089 <= oid+shift <= 14123:
            pointed = replace_hebrew(raw, oid+shift, ui, mapping, glyphs, manuscript)
        elif pairs:
            pattern = re.compile(b'|'.join(map(re.escape,pairs)))
            pointed = pattern.sub(lambda match: pairs[match[0]],raw)
            raw = pattern.sub(lambda match: plain_groups[oid][match[0]],raw)
        merged = (merge_dialogue(raw, pointed) if oid in dialogue_ids
                  else add_language(raw, pointed, oid == 14084-shift))
        objects[oid].set_raw_data(merged)
    for oid,tid,is_manuscript in [(14125,315,False),(14127,318,True)]:
        font_table(objects[oid-shift],objects[tid],glyphs,windows,is_manuscript)
    for oid,tid in [(14124,325),(14126,322)]:
        extend_ui_font(objects[oid-shift], objects[tid], glyphs, windows,
                       BASE/'fonts'/('AmaticSC-Bold.ttf' if tid == 325 else 'AmaticSC-Regular.ttf'))
    update_materials(objects)
    args.output.write_bytes(env.file.save() if windows else env.file.save(packer='lz4'))
    print(len(glyphs),'pointed glyphs in each atlas',flush=True)


if __name__ == '__main__':
    main()
