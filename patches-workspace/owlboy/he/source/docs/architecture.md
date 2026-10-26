# Rendering and build design

Catalog strings stay in logical Hebrew order. The builder preserves dialogue
controls, normalizes punctuation to supported glyphs, and inserts line breaks
using actual font advances. Unicode BiDi processing produces a permutation of
the visible characters in each completed line, including mirrored punctuation.

The runtime changes glyph positions, not the speech printer's string or length.
Owlboy can keep revealing a logical prefix, triggering the same pauses and
impact events. Visible characters are placed at positions from the completed
line's layout, so the right edge stays fixed as letters appear to its left.

Owlboy's highlight renderer slices strings before drawing. The patch annotates
those temporary slices with unmapped private-use markers identifying absolute
logical indices. Measurements ignore those markers. The runtime uses them to
align colored segments with the full line and strips them before drawing.

Mono.Cecil adds a wrapper around Font.InternalDraw and speech-renderer context
hooks. The original draw body is kept and called for individual positioned
glyphs through a direct managed bridge bound during the build, retaining the game's texture/GPU behavior. Non-Hebrew text uses the
original method. No decompiled game source is shipped in this repository.

Static lines use generated Unicode maps. Dynamic strings use a limited RTL
fallback preserving contiguous Latin/number runs; this is deliberately not
claimed as a complete Unicode BiDi implementation for arbitrary dynamic text.

The builder hashes the original executable and catalog before producing local
assets. Installation operates only on the explicit file plan; recovery storage
is outside the game and the package. Backups are immutable originals, and every
retry/restore verifies ownership and content hashes.
