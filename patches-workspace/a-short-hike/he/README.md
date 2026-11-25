# A Short Hike Hebrew rendering prototype

Target: Steam macOS build 20289045. The probe uses two entries from the GoatStart scene. Drafted by Codex (gpt-6.2-sol) for rendering verification.

Run `python3 build_font.py` with fontTools installed to generate the regular TrueType font. It covers all 27 Hebrew letters, including final forms. Latin, punctuation, and digits need the game's fallback fonts. The prototype includes niqqud with OpenType mark positioning. Styled faces are pending. The niqqud preview uses Pillow with RAQM, outside Unity. The offline preview compares both regular faces at equal nominal sizes and shared baselines. No in-game size adjustment has been measured.

To stage the probe, copy the app bundle and copy `Contents/_LANG_Custom.yarn_lines.csv` to `Contents/LANG_HebrewProbe.yarn_lines.csv` within it. Replace only the two matching LineText cells from rendering-probe.csv, keeping the original IDs and all other columns. Keep Hebrew text in logical order.

The game's developer documents a FONT selector for Chinese, Korean, Japanese, and Cyrillic. The local assembly exposes separate Unity and TMP custom font members. No native Hebrew or dialogue bidi path has been verified. The standalone TTF has yet to be wired into either renderer. The temporary app aborted at startup, and Launch Services rejected the staged bundle. Once startup works, check font loading, punctuation, wrapped lines, progressive reveal, and switching back to English before deciding where to apply bidi processing.
