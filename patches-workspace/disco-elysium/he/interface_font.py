"""Add Hebrew glyphs to the two embedded Noto Sans Arabic interface fonts."""
import hashlib
import io

import UnityPy
from fontTools.pens.recordingPen import DecomposingRecordingPen
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

FONT_NAMES = {"NotoSansArabic", "NotoSansArabic Included"}
HEBREW_LETTERS = range(0x05D0, 0x05EB)


def font_at_weight(source, weight):
    return instantiateVariableFont(
        source,
        {axis.axisTag: weight if axis.axisTag == "wght" else axis.defaultValue
         for axis in source["fvar"].axes},
    )


def add_hebrew(font_bytes, assistant_path):
    original = TTFont(io.BytesIO(font_bytes))
    base = font_at_weight(original, 400)
    assistant = font_at_weight(TTFont(assistant_path), 600)
    if base["head"].unitsPerEm != assistant["head"].unitsPerEm:
        raise ValueError("Interface fonts use different units per em")

    glyph_set = assistant.getGlyphSet()
    assistant_cmap = assistant.getBestCmap()
    mappings = {}
    order = list(base.getGlyphOrder())
    for code in HEBREW_LETTERS:
        source_name = assistant_cmap[code]
        target_name = f"hebrew{code:04X}"
        recording = DecomposingRecordingPen(glyph_set)
        glyph_set[source_name].draw(recording)
        pen = TTGlyphPen(None)
        recording.replay(pen)
        base["glyf"][target_name] = pen.glyph()
        base["hmtx"][target_name] = assistant["hmtx"][source_name]
        order.append(target_name)
        mappings[code] = target_name

    base.setGlyphOrder(order)
    for cmap in base["cmap"].tables:
        if cmap.isUnicode():
            cmap.cmap.update(mappings)
    output = io.BytesIO()
    base.save(output)
    modified = output.getvalue()
    check = TTFont(io.BytesIO(modified))
    cmap = check.getBestCmap()
    if not set(HEBREW_LETTERS) <= cmap.keys() or not original.getBestCmap().keys() <= cmap.keys():
        raise ValueError("Embedded font has incomplete glyph coverage")
    if len(modified) > len(font_bytes):
        raise ValueError("Embedded font no longer fits its Unity asset")
    return modified.ljust(len(font_bytes), b"\0")


def build_interface_font_recipe(source, assistant_path):
    env = UnityPy.load(source)
    edits = []
    names = set()
    generated = {}
    for obj in env.objects:
        if obj.type.name != "Font" or obj.peek_name() not in FONT_NAMES:
            continue
        font = obj.read_typetree()
        name = font["m_Name"]
        if name in names:
            raise ValueError(f"Duplicate embedded font: {name}")
        names.add(name)
        data = bytes(font["m_FontData"])
        raw = obj.get_raw_data()
        if raw.count(data) != 1:
            raise ValueError(f"Could not locate embedded font: {name}")
        start = obj.byte_start + raw.index(data)
        if source[start:start + len(data)] != data:
            raise ValueError(f"Invalid Unity font offset: {name}")
        if data not in generated:
            generated[data] = add_hebrew(data, assistant_path)
        edits.append((start, generated[data]))
    if names != FONT_NAMES:
        raise ValueError("Expected both Noto Sans Arabic interface fonts")

    operations = []
    cursor = 0
    output = hashlib.sha256()
    for start, replacement in sorted(edits):
        if start < cursor:
            raise ValueError("Overlapping embedded font data")
        if start > cursor:
            operations.append({"offset": cursor, "length": start - cursor})
            output.update(source[cursor:start])
        operations.append({"hex": replacement.hex()})
        output.update(replacement)
        cursor = start + len(replacement)
    if cursor < len(source):
        operations.append({"offset": cursor, "length": len(source) - cursor})
        output.update(source[cursor:])
    return {
        "schema_version": 1,
        "input_sha256": hashlib.sha256(source).hexdigest(),
        "output_size": len(source),
        "operations": operations,
    }, output.hexdigest()
