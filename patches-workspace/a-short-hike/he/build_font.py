"""Build the original Hebrew glyph prototype (requires fonttools)."""

from pathlib import Path

from fontTools.feaLib.builder import addOpenTypeFeaturesFromString
from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen

# Five columns, top to bottom. Final forms with descenders use extra rows.
BITMAPS = """
10001 11010 00100 01010 10001
11110 00010 00010 00010 11111
01110 00010 00010 00110 11010
11111 00010 00010 00010 00010
11111 00001 10001 10001 10001
01100 00100 00100 00100 00100
01110 00100 00100 00100 00100
11111 10001 10001 10001 10001
10001 10011 10001 10001 01110
01100 00100 00000 00000 00000
11111 00001 00001 00001 00001 00001 00001
11110 00001 00001 00001 11110
11000 01000 01111 00001 00010 00100
11111 10001 10001 10001 11111
10110 11001 10001 10001 10111
01100 00100 00100 00100 00100 00100 00100
01100 00100 00100 00100 11100
11110 10001 10001 10001 01110
10001 10001 01001 01010 11100
11110 10001 11001 00001 00001 00001 00001
11110 10001 11001 00001 11110
10001 10001 01010 00100 00100 00100 00100
10001 10010 01100 00010 11100
11111 00001 10001 10010 10100 10000 10000
11110 00001 00001 00001 00001
10101 10101 11101 10010 01100
11111 01001 01001 01001 11001
""".strip().splitlines()


# Mark shapes at their attachment point, in 70-unit pixels.
MARKS = {
    0x05B0: ("below", "010 000 010"),
    0x05B1: ("below", "10101 00000 01001"),
    0x05B2: ("below", "11101 00000 00001"),
    0x05B3: ("below", "11101 01000 01001"),
    0x05B4: ("below", "010"),
    0x05B5: ("below", "101"),
    0x05B6: ("below", "101 000 010"),
    0x05B7: ("below", "111"),
    0x05B8: ("below", "111 010 010"),
    0x05B9: ("topLeft", "010"),
    0x05BA: ("topRight", "010"),
    0x05BB: ("below", "100 010 001"),
    0x05BC: ("inside", "010"),
    0x05BD: ("below", "010 010 010"),
    0x05BF: ("topCenter", "111"),
    0x05C1: ("topRight", "010"),
    0x05C2: ("topLeft", "010"),
    0x05C7: ("below", "111 010 010"),
}


def rectangle(pen, x, y, size):
    pen.moveTo((x, y))
    pen.lineTo((x, y + size))
    pen.lineTo((x + size, y + size))
    pen.lineTo((x + size, y))
    pen.closePath()


def build():
    glyphs = {}
    metrics = {}
    cmap = {}
    for name, rows in [
        (".notdef", "11111 10001 10101 10001 11111"),
        ("space", ""),
        *((f"uni{code:04X}", rows) for code, rows in enumerate(BITMAPS, 0x05D0)),
    ]:
        pen = TTGlyphPen(None)
        for row, pixels in enumerate(rows.split()):
            for column, pixel in enumerate(pixels):
                if pixel == "1":
                    x = 50 + column * 100
                    y = (4 - row) * 100
                    rectangle(pen, x, y, 100)
        glyphs[name] = pen.glyph()
        metrics[name] = (350 if name == "space" else 650, min(
            (50 + col * 100 for row in rows.split()
             for col, pixel in enumerate(row) if pixel == "1"), default=0
        ))
        if name.startswith("uni"):
            cmap[int(name[3:], 16)] = name
    classes = []
    for code, (position, rows) in MARKS.items():
        name = f"uni{code:04X}"
        pen = TTGlyphPen(None)
        for row, pixels in enumerate(rows.split()):
            for col, pixel in enumerate(pixels):
                if pixel == "1":
                    x = col * 70 - len(pixels) * 35
                    rectangle(pen, x, -70 * row, 70)
        glyph = pen.glyph()
        glyph.recalcBounds(None)
        glyphs[name] = glyph
        metrics[name] = (0, glyph.xMin)
        cmap[code] = name
        classes.append(f"markClass {name} <anchor 0 0> @{position};")

    bases = []
    for code in range(0x05D0, 0x05EB):
        name = cmap[code]
        glyphs[name].recalcBounds(None)
        center = (glyphs[name].xMin + glyphs[name].xMax) // 2
        below = min(-130, glyphs[name].yMin - 130)
        bases.append(
            f"pos base {name} <anchor {center} {below}> mark @below "
            f"<anchor 85 550> mark @topLeft "
            f"<anchor 515 550> mark @topRight "
            f"<anchor {center} 550> mark @topCenter "
            f"<anchor {center} 210> mark @inside;"
        )
    cmap[32] = "space"
    font = FontBuilder(1000, isTTF=True)
    font.setupGlyphOrder(list(glyphs))
    font.setupCharacterMap(cmap)
    font.setupGlyf(glyphs)
    font.setupHorizontalMetrics(metrics)
    font.setupHorizontalHeader(ascent=800, descent=-560)
    font.setupNameTable({
        "familyName": "Hike Hebrew Prototype", "styleName": "Regular",
        "uniqueFontIdentifier": "Kitor Hike Hebrew Prototype Regular",
        "fullName": "Hike Hebrew Prototype Regular",
        "psName": "HikeHebrewPrototype-Regular",
    })
    font.setupOS2(sTypoAscender=800, sTypoDescender=-560,
                  usWinAscent=800, usWinDescent=560)
    font.setupPost()
    addOpenTypeFeaturesFromString(font.font, "\n".join([
        "languagesystem DFLT dflt;", "languagesystem hebr dflt;",
        *classes, "feature mark {", *bases, "} mark;",
    ]))
    font.save(Path(__file__).with_name("HikeHebrewPrototype.ttf"))


if __name__ == "__main__":
    build()
