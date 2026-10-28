# Grim Fandango Remastered Hebrew experiment

Local proof of concept, tested on macOS with ScummVM 2026.4.0 on 2026-10-26. Not a release patch or Kitor registry entry.

Run `launch-macos.command` to open the prepared experiment. It uses its own configuration and saves in the ignored `runtime/` directory. The original app, ScummVM preferences, and existing saves are unchanged. Game resources are symlinked from the installed app; only the English text table and HD font files are generated locally. Do not edit files through those resource symlinks.

The builder decodes the AdventureBrew original-edition patch, matches Remastered text IDs, applies bidirectional display ordering, and creates a matching single-byte font character map. It reuses 7,704 translation rows; unmatched or undecodable rows retain English. The original patch credits Niv Bar and Shani Roz in Kitor metadata. Source: https://drive.google.com/file/d/1DGyW6h9Szx-rVCRmXkR6lBRGAgpVdQIf/view

Observed: game boots to Manny's office; Hebrew menu labels render in correct letter order. Screenshot: `runtime/hebrew-menu.png`. Menu labels overlap. Runtime logs report unimplemented Remastered UI functions, including GetTextObjectDimensions and OverlayCreate. Full dialogue subtitles have not yet been visually verified. Do not infer full Remastered compatibility from launching successfully.

Remaining: proper RTL wrapping (preordered paragraphs can wrap in the wrong order), dialogue/cutscene validation, menu sizing, new Remastered translations, redistributable Hebrew font, Windows testing, and reversible production packaging/integration. The local Arial-derived font must not be redistributed with a patch. No Windows artifact or installer is claimed.

Dependencies: Python 3, fonttools 4.67.0, python-bidi 0.6.12. Rebuild with `build.py --game <Resources> --patch <datausr.lab> --font <Hebrew TTF> --output <new directory>`; output must not already exist. The original patch is preserved locally in `runtime/source-patch.zip`.

To discard the experiment, quit its ScummVM instance and remove only this prototype's `runtime/` folder. No restoration of original game files is needed. Preserve `runtime/saves/` if you want any test progress retained.
