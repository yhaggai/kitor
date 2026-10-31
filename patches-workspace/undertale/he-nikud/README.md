# UNDERTALE Hebrew patch

Unofficial Hebrew localization for the macOS and Windows Steam versions of UNDERTALE. **Alpha 3 nikud draft** includes the full text catalog, pointed Hebrew font glyphs, right-to-left rendering and the Flowey speech-bubble margin fix. Steam integration is retained. The Windows build has passed archive and installer checks on macOS; it has **not yet been run on Windows**.

The translation is a draft, not a fully proofread or fully playtested release. Text embedded in images remains English. Long labels, special effects, choices and language-dependent puzzles need further review.

## Separate Kitor option

Choose **Hebrew with nikud** in Kitor’s Translation selector. **Hebrew without
nikud** remains the existing alpha 2 patch. Restore an installed translation
before installing the other option. Each option has its own release track.

## Nikud development build

The current source adds nikud to the Hebrew translation. Vowelization is a
**machine-generated draft**, using [Nakdimon](https://github.com/elazarg/nakdimon)
0.2.1 locally. It preserves the original consonants, whitespace, placeholders
and game commands. Context-dependent vowels, names, invented words and character
voices still need human review. Entries carry `nikud_status: machine_draft`.

The builder composes each pointed letter into one font glyph with HarfBuzz and
FreeType. RTL, wrapping and typewriter reveal therefore count a pointed letter
as one cell. All 20 fonts receive the catalog's pointed glyphs. Very small fonts
have limited nikud legibility; archive verification does not replace playtesting.
Raw pointed text entered dynamically outside the catalog is not shaped.

To vowelize new unpointed entries locally, install `nakdimon==0.2.1` in a separate
Python 3.12+ environment and run `python vowelize_catalog.py`. Existing pointed
entries are preserved. Normal patch building and installation do not need the
model. To edit nikud, update the Unicode `hebrew` field and rebuild both platforms.

## Install on Windows

Requires your own Windows Steam installation of UNDERTALE and Python 3.10+ with the `py` launcher. Save and close the game first.

1. Download/extract the Windows patch package or clone this repository.
2. Double-click **Install Hebrew Windows.cmd**. It looks for UNDERTALE through Steam's registry/library locations.
3. Launch through Steam and keep **English** selected.

For a custom Steam library, run:

```bat
py -3 install.py --platform windows --game-dir "D:\SteamLibrary\steamapps\common\Undertale"
```

The installer changes only `data.win`, after verifying its version and creating a backup in `%LOCALAPPDATA%\Undertale Hebrew\backups`. To undo it, drag the printed backup folder onto **Restore Original Windows.cmd**, or run `py -3 install.py --rollback "BACKUP_FOLDER"`. Saves and the executable are not changed.

Supported original Windows `data.win` SHA-256 (Steam depot 391541, manifest 3413030246586915448):

```text
b718f8223a5bb31979ffeed10be6140c857b882fc0d0462b89d6287ae38c81c7
```

**Windows runtime testing is outstanding.** Please treat this as an alpha and report any rendering or gameplay problems.

## Install on macOS

Requires macOS, Python 3.10+ and your own matching UNDERTALE installation. Close the game after saving your progress.

1. Clone/download this repository. If downloading a ZIP, extract it first.
2. Run `python3 install.py`, or double-click **Install Hebrew.command**.
3. Start UNDERTALE through Steam and keep the in-game language set to **English**.

The installer replaces only `Contents/Resources/game.ios`. It verifies all checksums, creates an original-file backup outside the game folder, and preserves Steam settings and save files. A Steam reinstall or file verification may remove the patch.

For a custom installation:

```sh
python3 install.py --game-app '/path/to/UNDERTALE.app'
```

Supported original `game.ios` SHA-256:

```text
91a8acf1f40bd7c45d0bdb978de55187e7fb2cd76a1e2cda41c078dd44d3565e
```

Only the exact macOS and Windows archive hashes listed here are supported. The installer refuses mismatched or already-modified versions rather than overwriting them.

## Roll back

The installer prints its backup path under `~/Library/Application Support/Undertale Hebrew/backups/`. Close the game and run:

```sh
python3 install.py --rollback '/path/printed/by/the/installer'
```

The original checksum is verified before restoring. Rollback refuses to overwrite an unrelated game update or another mod. Saves are not restored or modified: they are independent of the patch.

## Translation and rendering

- 16,455 catalog entries: 15,227 Hebrew drafts and 1,228 intentionally unchanged entries.
- Hebrew glyphs added to all 20 original fonts; existing glyph metrics preserved.
- Logical-order Hebrew text with mixed Latin/numbers, punctuation, wrapping and original typewriter/control commands.
- 378 ordinary drawing calls redirected to Hebrew-aware wrappers, plus specialized icon/name handling.
- Flowey bubble bounds include 12 pixels of right padding and reserve the final glyph cell.
- The Mettaton letter-count quiz retains the English name so its original answer remains valid.

Edit `translation-catalog.json`: change `hebrew`, `status` and `translator_note`; preserve `key`, `english`, placeholders and control commands. Translations in the catalog do not take effect until rebuilt. The committed deltas are the alpha 3 nikud draft for both supported Steam archives.

## Build from source

Requires Python 3.10+, Pillow, uharfbuzz, freetype-py, macOS Arial/Arial Bold fonts and [UndertaleModTool CLI 0.9.2.0](https://github.com/UnderminersTeam/UndertaleModTool/releases/tag/0.9.2.0). Download the appropriate official macOS CLI and extract it locally. The CLI is not bundled here.

```sh
python3 -m venv .venv
source .venv/bin/activate
python3 -m pip install -r requirements.txt
python3 build.py --original '/path/to/unmodified/game.ios' --utmt '/path/to/UndertaleModCli'
```

For a Windows build, pass the original `data.win` to the same command. The archive hash selects the platform automatically.

Use your installer backup if the installed archive is already patched. The builder checks the original hash, extracts the required code/font assets locally, validates the catalog, generates glyphs and GML changes, compiles a new archive, and reloads it for verification. It does not modify the installed game. Output: `build/game.hebrew-steam.ios` or `build/data.hebrew-steam.win`. The font generator currently uses macOS Arial fonts, so building on this Mac is supported; the Windows installer itself requires only Python.

Generated original assets/code and complete game archives are ignored by Git. A rebuild can differ byte-for-byte across dependency/platform versions even when behavior is equivalent. It does not automatically replace the committed delta. To generate and verify a new delta, run:

```sh
python3 create_delta.py --original "/path/to/data.win" --patched build/data.hebrew-steam.win --platform Windows --output patches/windows
```

## Checks

```sh
python3 full-patch/hebrew-runtime/reference.py
# After extracting the original catalog during a build:
python3 validate_translation_catalog.py
```

The runtime reference suite has 21 focused tests. The original alpha 2 archive was checked for font coverage, renderer hooks and live Hebrew display. The installer/rollback must also be tested on a disposable copy before distributing changes. A complete game playthrough remains outstanding.

## Repository contents

- `translation-catalog.json`: bilingual translation source.
- `full-patch/hebrew-runtime/`: GML bidi/layout helpers and Python reference tests.
- `full-patch/render-gml/`: draw wrappers.
- `full-patch/*.py`, `*.csx`: font generation, surgical code transforms and import/verification scripts.
- `build.py`: portable build pipeline.
- `patches/`: Steam-enabled binary delta and checksums.
- `install.py`: cross-platform installer and rollback with verified backups.
- `delta.py`: standard-library BSDIFF40 application; no external patching executable is needed.
- `create_delta.py`: generates and roundtrip-tests binary deltas.

## Credits and rights

UNDERTALE and its original content belong to Toby Fox and their respective rights holders. This is an unofficial fan project, not endorsed by the creators. The repository requires an owned original game; it does not ship a complete game, original executable, audio, font atlases or decompiled game source. The bilingual catalog contains original strings as translation references. UndertaleModTool is a separate project with its own license. No blanket license to original game content or third-party fonts is granted here.

## עברית

תרגום עברי ניסיוני ל־UNDERTALE ב־macOS. יש לשמור את ההתקדמות ולסגור את המשחק לפני ההתקנה. לאחר ההתקנה מפעילים דרך Steam ובוחרים בשפה English. המתקין יוצר גיבוי ומדפיס פקודת שחזור. התרגום עדיין דורש הגהה ובדיקות במהלך המשחק.
