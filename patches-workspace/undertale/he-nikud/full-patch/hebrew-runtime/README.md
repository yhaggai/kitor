# Hebrew runtime helpers

Six GML1 script bodies, imported using their filenames without `.gml`: `he_bidi_type`, `he_mirror`, `he_bidi`, `he_layout_line`, `he_layout`, `he_visual`. Use GameMaker scripts with `argument0` style arguments; these files contain no function declarations, structs, or GML2-only array accessors.

## Public interfaces

`he_visual(text)` returns a plain drawing string. Hebrew-containing lines use RTL order, Latin and numeric runs retain their internal order, paired brackets mirror, and line order remains unchanged. `#`, LF and CRLF separate lines; the result uses `#`. Strings without Hebrew return unchanged. Writer commands are **not** interpreted: slash, percent, ampersand and backslash are literal here. Leading/trailing padding and gaps of two or more spaces retain original columns. This function allocates and destroys its own temporary grids.

`he_layout(text, maxColumns [, plainMode])` returns a **caller-owned ds_grid**, with width `string_length(text)+1` and height 6. Destroy this grid when replaced or when its owning writer is destroyed. Cache it for a whole source string; drawing only part of that source must use the same complete layout so a typewriter reveal progresses naturally from the Hebrew right edge.

Use `ds_grid_get(layout, sourceIndex, field)` for a 1-based logical source character index:

| Field | Value |
| --- | --- |
| 0 | Zero-based visual column, or -1 if no visible glyph |
| 1 | Zero-based visual row |
| 2 | Display glyph, including bracket mirroring |
| 3 | Kind: 0 control/suppressed, 1 glyph, 2 icon, 3 explicit line break |
| 4 | Anchor: -1 ordinary row, 0 for `\>0`, 1 for `\>1` |
| 5 | Cell width: 1 for visible glyph/icon, 0 otherwise |

At sourceIndex 0, fields are metadata: row count; maximum occupied columns; contains Hebrew (0/1); first `/` or `%` source index (0 if absent); source length; unused.

Writer mode consumes `\E`, `\M`, `\F`, `\T`, `\S`, `\z`, `\*`, `\>` plus one parameter. Other backslash commands consume two source characters. `^digit` pauses occupy no cell. `&`, LF and CRLF break lines. `/` and `%` stop this page. Every control stays in the original input: this script produces positions, not a rewritten command stream. The existing writer must continue executing commands and advancing logical indices normally. Icon mappings reside on the command **parameter** index, after the existing writer advances its index; the display glyph is U+FFFC and must be handled by the original sprite branch, never drawn as font text.

For ordinary glyphs, map the original logical `n` to `(originX + column*cellWidth, originY + row*lineHeight)`, and use field 2 instead of the original glyph. Choose the proper origin for anchor 0/1 using the game's existing choice positions. Skip mapped col -1. Hebrew normal rows right-align within `maxColumns`; anchored choices and plain mode do not. Soft wrapping prefers spaces, suppresses only the consumed wrap spaces, and hard-wraps long words. Repeated-space choice rows bypass wrapping so their field positions and selection geometry survive.

`plainMode=1` is used internally by `he_visual`: only plain line separators are controls, no right alignment is added, and the caller supplies a sufficiently large capacity.

## Internal helper

`he_bidi(segment)` returns a caller-owned grid: source indices 1..N with fields visual column, display glyph, resolved level. Metadata `[0,0]` is length; `[0,1]` is base direction. Destroy the returned grid after use. `he_layout_line` and `he_bidi_type` are implementation details.

## Algorithm scope

The implementation follows the relevant weak/neutral/bracket/implicit-level and reversal steps from [Unicode Bidirectional Algorithm, UAX #9](https://www.unicode.org/reports/tr9/), with a small character classifier suitable for the game's unpointed Hebrew, Latin and numbers. It is **not a complete Unicode-conforming bidi engine**. Hebrew anywhere in a segment selects an RTL base; numeric unary signs are kept with their following numeric run as a UI-specific tailoring. Leading/trailing spaces and repeated-space field boundaries are fixed-layout overrides. ASCII `()[]{}` receive bracket-pair resolution; `<>` receive mirroring only.

Pointed Hebrew in the catalog is compiled by `nikud.py` into atomic glyph IDs in
U+F000–U+F7FF. The font builder uses HarfBuzz and FreeType to shape each letter
with its vowel marks, dagesh and shin/sin dot. Both the text compiler and font
builder derive the same deterministic mapping from the catalog. These glyphs
have Hebrew bidi direction and occupy one writer cell, so marks cannot detach
during wrapping, reversal or typewriter reveal. U+E000 remains reserved for icons.
The catalog itself stays readable Unicode; generated glyph IDs are build-local.

Explicit embedding/isolate controls, a complete Unicode character database,
Arabic shaping, arbitrary runtime combining-mark input, cantillation, and
non-ASCII bracket pairs are not supported. Add pointed text to the catalog and
rebuild; do not inject raw combining marks directly into game runtime strings.

The writer layout is a fixed-cell map. It does not calculate font advances, half-size commands, wide icons, sprite offsets, or variable-height lines. The integrating renderer must account for these if used. Resolve dynamic name/value placeholders using the game's existing substitution step **before** obtaining the layout. A color command does not interrupt bidi ordering or affect logical reveal timing.

## Verification

Run the dependency-free Python equivalent and 21 focused tests:

```sh
python3 full-patch/hebrew-runtime/reference.py
```

Tests cover numbers and signed numbers, Latin runs, bracket pairs, explicit line breaks, choice columns and anchors, source-index preservation, controls and pauses, icons, word wrapping and RTL reveal positions. All 21 pass. A separate smoke check ran writer layout at 38 columns and plain conversion for all 16,455 completed catalog entries, with no exceptions or duplicate visible cells per anchor/row. The Python tests validate the algorithm; they do not execute compiled GML or replace in-game testing.
