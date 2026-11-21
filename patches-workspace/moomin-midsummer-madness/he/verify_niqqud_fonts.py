"""Reopen rebuilt font assets and verify Hebrew ink margins and PUA coverage."""
import argparse
import hashlib
import json
from pathlib import Path

from PIL import Image
import UnityPy

from build_niqqud_fonts import FONT_NAMES
from font_layout import SIDE_BEARING


def verify(path, codes, templates):
    env = UnityPy.load(path.read_bytes())
    results = []
    for name in FONT_NAMES:
        obj = next(o for o in env.objects if o.type.name == 'MonoBehaviour' and o.peek_name() == name)
        # The game strips sharedassets MonoBehaviour type trees. Reuse the
        # corresponding TMP type from the font bundle, as the builder does.
        obj.serialized_type.node = templates[name]
        font = obj.read()
        chars = {c.m_Unicode:c for c in font.m_CharacterTable}
        glyphs = {g.m_Index:g for g in font.m_GlyphTable}
        if len(chars) != len(font.m_CharacterTable) or len(glyphs) != len(font.m_GlyphTable):
            raise ValueError(f'Duplicate character or glyph index: {name}')
        texture = next(o for o in env.objects if o.path_id == font.m_AtlasTextures[0].path_id).read()
        atlas = Image.frombytes('L', (texture.m_Width, texture.m_Height), texture.image_data)
        margins = []
        for cp in list(range(0x05D0, 0x05EB)) + list(codes.values()):
            if cp not in chars:raise ValueError(f'Missing U+{cp:04X}: {name}')
            g = glyphs[chars[cp].m_GlyphIndex];r = g.m_GlyphRect;m = g.m_Metrics
            if not (0 <= r.m_X < r.m_X+r.m_Width <= texture.m_Width and 0 <= r.m_Y < r.m_Y+r.m_Height <= texture.m_Height):
                raise ValueError(f'Glyph outside atlas: {name} U+{cp:04X}')
            box = atlas.crop((r.m_X, r.m_Y, r.m_X+r.m_Width, r.m_Y+r.m_Height)).point(lambda a:255 if a>=128 else 0).getbbox()
            if box is None:raise ValueError(f'Empty Hebrew glyph: {name} U+{cp:04X}')
            left = m.m_HorizontalBearingX + box[0]
            right = m.m_HorizontalAdvance - m.m_HorizontalBearingX - box[2]
            if min(left, right) < SIDE_BEARING - 0.001:
                raise ValueError(f'Crowded Hebrew glyph: {name} U+{cp:04X}, {left}, {right}')
            margins.append(min(left, right))
        results.append({'font':name, 'hebrew_glyphs_checked':len(margins), 'minimum_side_margin':min(margins)})
    return {'file':path.name, 'sha256':hashlib.sha256(path.read_bytes()).hexdigest(), 'fonts':results}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory', type=Path)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    codes = json.loads(Path(__file__).with_name('niqqud-glyphs.json').read_text(encoding='utf-8'))
    bundle = UnityPy.load((args.directory/'fonts_assets_all.bundle').read_bytes())
    templates = {o.peek_name():o.serialized_type.node for o in bundle.objects
                 if o.type.name == 'MonoBehaviour' and o.peek_name() in FONT_NAMES}
    report = [verify(args.directory/name, codes, templates) for name in ('fonts_assets_all.bundle', 'sharedassets0.assets')]
    args.report.write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
    print(f'Verified both font copies, {len(codes)} pointed glyphs plus 27 base letters in each of four faces')


if __name__ == '__main__':
    main()
