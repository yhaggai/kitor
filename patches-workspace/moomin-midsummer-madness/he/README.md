# Moomin: Midsummer Madness — Hebrew localization

This folder contains the editable logical-order Hebrew catalog, its source inventory, character glossary, extraction and build scripts, and model attribution.

## Scope

The catalog covers all 4,692 nonempty player-facing `text` fields from the 1,931-card `GameDatabase`: dialogue, chapter recaps, journals, UI, inventory, credits and demo text. Each entry has a stable recursive JSON path and exact English source. The builder writes Hebrew into the rendered `text` field and its `data.en.translation` slot where present. Names of real people and companies are retained in credits; role labels are translated.

## Rebuild

`catalog.csv` is the editable source of truth for both editions. It contains one UTF-8-BOM row per stable text path, category, scene, speaker when the scene key identifies one, English source, unpointed Hebrew and pointed Hebrew. `catalog.json` is the preserved unpointed catalog; `catalog-niqqud.json` is an intermediate model draft. The CSV exporter preserves quoted commas, quotes and line breaks.

Use Python 3.11 with UnityPy 1.25.4 and python-bidi to build the database from the pristine bundle. The builder checks unique IDs, full path coverage, placeholders and tags, and exact base wording after removing niqqud:

```powershell
$env:PYTHONPATH = '.tmp/silver-py'
py -3.11 patches-workspace/moomin-midsummer-madness/he/extract.py `
  '.tmp/moomin-render-test/database_assets_all_c77c9f4b2caf03eae7708be1f1ce7d43.bundle.original' `
  'patches-workspace/moomin-midsummer-madness/he/source.json'
py -3.11 patches-workspace/moomin-midsummer-madness/he/build.py `
  '.tmp/moomin-render-test/database_assets_all_c77c9f4b2caf03eae7708be1f1ce7d43.bundle.original' `
  'patches-workspace/moomin-midsummer-madness/he/catalog.csv' `
  '.tmp/moomin-render-test/moomin-he-database.bundle' `
  --report 'patches-workspace/moomin-midsummer-madness/he/build-report.json'
```

Pointed text stays in logical reading order in the CSV. `build.py` wraps paragraphs and treats each Hebrew letter-plus-niqqud cluster as one unit during bidi conversion, then substitutes its composite PUA glyph. That keeps marks attached to the right letter in the game's left-to-right renderer. `build_niqqud_fonts.py` extends the four active TMP SDF font atlases using HarfBuzz mark anchors and FreeType glyph rendering. Both the fonts bundle and `sharedassets0.assets` must be patched. A raster preview made from the rebuilt atlases is `niqqud-font-preview.png`.

Dialogue wrapping uses advances extracted from `Pangolin-Regular SDF MAIN` in `dialogue-font-advances.json`, rather than character counts. The 550-unit line width is calibrated to the observed bubble break for “אחותי מחפשת אותי בכל מקום.” (420.9 and 439.9 units fit; extending the first line by the next word is 565.8). Dialogue breaks are emitted as TMP `<br>` tags because the game normalizes literal newlines before text layout. The builder asserts source indices stay aligned while mapping each wrapped visual line, preventing Hebrew base letters and niqqud clusters from shifting across whitespace.

## Kitor package

`patch.windows.candidate.json` and `deltas/` provide the installable Windows patch for Steam app 4463520, depot 4463521, build 26355427. It contains five verified bsdiff files (database, TMP font bundle, `sharedassets0.assets`, `catalog.bin`, and the managed save-filename validator); it does not include raw game bundles. The save validator recognizes the private-use Hebrew glyphs in translated chapter names, preventing valid Hebrew-named save files from being discarded during save discovery. `build_moomin_package.py` recreates the delta manifest from a pristine and a patched game tree; run it with `--original-root`, `--patched-root`, and (after rebuilding text) `--database-output`. It needs Python's `bsdiff4` package. The official Steam header art at `kitor/public/covers/moomin-midsummer-madness/header.jpg` is composed into portrait English and Hebrew GameBox covers.

Dialogue line wrapping is validated against the reported screenshot phrase and each source character is checked against its original grapheme/style index. The computer screenshot channel timed out on the running game window after relaunch, so the corrected line order has not been visually captured in-game. The 550-unit calibration comes from one observed bubble width; other resolutions and unusual long unbreakable tokens still need visual review.

## Translation record

The unpointed translation was prepared with OpenAI Codex CLI, `gpt-6-luna`, low reasoning effort, in isolated read-only batches. Niqqud was added locally with Nakdimon 0.2.1's cached ONNX model. The automatic vowel pass is not a manual line-by-line Hebrew proofread. `niqqud-glossary.json` and `apply_niqqud_glossary.py` pin reviewed pronunciations for recurring Moomin names and terms; the base-letter spelling stays identical. Name spellings and their sources are recorded in `character-glossary.md`.

To rebuild font atlases, install `UnityPy`, `numpy`, `scipy`, `Pillow`, `uharfbuzz` and `freetype-py`, then run `build_niqqud_fonts.py` against the existing Hebrew-capable font bundle and sharedassets backups. The game loads font copies from both files. Installation is reversible; retain the current installed bundle and catalog backups. The complete pointed database is reopened and validated by UnityPy; startup-log checks are recorded with the build artifacts. A full playthrough of all 4,692 strings has not been observed.
