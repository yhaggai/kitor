"""Compile pointed Hebrew into atomic font glyphs for GameMaker 1.

The editable catalog always stays Unicode. Only generated game strings use PUA.
F000-F7FF is reserved here; E000 is already used by the icon renderer.
"""
import re
import unicodedata

START, END = 0xF000, 0xF7FF
MARKS = frozenset(chr(c) for c in (*range(0x5B0, 0x5BE), 0x5BF, 0x5C1, 0x5C2, 0x5C7))
CLUSTER = re.compile(r'[א-ת][\u05b0-\u05bd\u05bf\u05c1\u05c2\u05c7]+')


def strip_points(text):
    return ''.join(c for c in text if c not in MARKS)


def clusters(text):
    remainder = CLUSTER.sub('', text)
    if any(c in MARKS for c in remainder):
        raise ValueError('Nikud must immediately follow a Hebrew letter')
    if any(START <= ord(c) <= END for c in text):
        raise ValueError('Catalog contains a reserved nikud glyph codepoint')
    return [unicodedata.normalize('NFD', c) for c in CLUSTER.findall(text)]


def glyph_map(catalog):
    found = sorted({g for e in catalog['entries'] for g in clusters(e['hebrew'])})
    if len(found) > END - START + 1:
        raise ValueError('Too many pointed Hebrew glyphs')
    return {g: chr(START + i) for i, g in enumerate(found)}


def encode(text, mapping):
    clusters(text)  # Reject orphan marks and collisions before compiling.
    return CLUSTER.sub(lambda m: mapping[unicodedata.normalize('NFD', m[0])], text)
