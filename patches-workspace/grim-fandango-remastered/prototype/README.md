# Current native Windows candidate: 0.3

The current package adds **Hebrew** as a seventh option under Options ג†’ Language. It uses `grim.he.tab`, preserves the original six tables, and uses English speech. Native executable and menu archive changes are distributed as hash-guarded reconstruction recipes. The running test copy is `runtime/game-native-v2`.

Build the payload with `build.py --separate-language --resources payload` plus the existing game, patch, font, movie, and output arguments. Generate recipes with `native_language.py --game ORIGINAL_GAME --output NEW_RECIPE_DIRECTORY`, then pass `--native-recipes NEW_RECIPE_DIRECTORY --version 0.3.0-native-candidate` to `package_native.py`. Install through the lifecycle so both original binaries can be restored. `build.py` alone does not add the native menu choice.

Four regression tests pass with `KITOR_GRIM_APP` set to the original game directory. The 0.3 lifecycle roundtrip verified all 21 file hashes and exact restoration; English remained untouched. Native startup succeeded. The user confirmed the new Hebrew language option works.

The following notes describe earlier prototypes, including the English-slot and ScummVM experiments; use the native candidate above for current work.

---

Current movie update (0.2.0-native-candidate): all 401 native cutscene subtitle references are covered, with 345 new AI-assisted Hebrew lines and 56 preserved original translations. Movie timings are unchanged. The historical 0.1 test notes below predate movie coverage.

# Grim Fandango Remastered Hebrew prototype

Development handoff, 2026-10-26. This is an experimental Hebrew resource builder, not a released patch or Kitor registry entry. The user confirmed Hebrew letters and visible menu text in the native Windows game after the CRLF fix. Full dialogue layout and playthrough validation remain outstanding.

## Native Windows experiment

Build with `--resources copy` using the commands below, then run the copied native executable through the new launcher:

```powershell
.\.venv\Scripts\python.exe launch_native.py --game runtime\game
```

This runs `GrimFandango.exe` directly with the copied game as its working directory. It redirects the child process's `APPDATA` to `runtime/native-profile` to separate test saves and settings, and enables `runtime/game/Grim.log`. Keep the game's language set to English, whose text table is replaced, and enable subtitles in the native options. Do not run the copied executable directly if you want profile isolation.

The builder now also supplies the remapped font under the original TTF filenames: the native executable references `FontsHD/FuturaStd-Heavy.ttf` directly as well as loading the LAF specifications. This is an experimental byte-font mapping; line wrapping and original-renderer fonts still need visual validation.

The first native test showed a blank main menu according to user feedback, including untranslated English labels. The builder had normalized the Windows table's CRLF line endings to LF. It now preserves the source line-ending style; the user confirmed visible menu text after the native retest. A regression test preserves CRLF, font specifications, original source files, and font coverage.

Windows GOG test on 2026-10-26: 7,704 of 9,639 rows reused. Source table SHA-256: `e2bc894fb4ef6cd2901befc052fa57f2741bdc25ccf724df51908ea8f1c98d4b`. Native logs reached `MoviesHD/intro.ogv` and confirmed the isolated profile path. The source text and executable were verified unchanged, and every generated text byte has a font mapping (excluding whitespace controls). Window capture timed out. Hebrew letters and menu visibility were confirmed by the user; speech and dialogue wrapping remain unverified. The initial direct-launch test used the normal profile; subsequent launches through `launch_native.py` use the isolated profile.

The current Windows build is `runtime/game-native-v2`; launch it with `launch_native.py --game runtime/game-native-v2`. It reuses 7,727 rows (7,704 direct IDs plus 23 menu aliases); 1,912 rows retain English.

The builder also reuses old menu translations for new UI IDs when the English labels match after ignoring accelerator markers, case, and whitespace. All matching old labels must have the same translation; ambiguous labels and dialogue are not remapped. The build report records each alias and the reason for every English fallback. No new translations are authored.

Run regression checks with `.\.venv\Scripts\python.exe -m unittest discover -p "test_*.py"` from this directory.

## What is included

`build.py` adapts the original AdventureBrew Hebrew translation to Remastered text IDs, applies bidirectional display ordering, and generates a matching single-byte TTF character map. It reused 7,704 translation rows in the tested Mac edition. Unmatched or undecodable rows retain English. `launch.py` runs an isolated ScummVM test on Windows or macOS with separate configuration, saves, and logs.

Only tooling is included. Obtain your own installed game, the source patch, and a Hebrew TTF locally. Game resources, downloaded patch files, and generated fonts must stay under the ignored `runtime/` directory. The Arial-derived font used for the Mac experiment must not be distributed.

Translation credits: Niv Bar and Shani Roz. [Original AdventureBrew patch download](https://drive.google.com/file/d/1DGyW6h9Szx-rVCRmXkR6lBRGAgpVdQIf/view), `GrimFandango-Heb-AdventureBrew.zip`. Extract `datausr.lab`; this is the builder's patch input. Catalog reference: `patches-metadata/011_grim_fandango_not_the_remastered_version.json`.

## Windows setup (PowerShell)

Use Python 3 and a ScummVM build that detects Remastered. The Mac test used ScummVM 2026.4.0. Start in this prototype directory. Replace the example game and executable paths with your installed locations. The game resource directory must contain `grim.en.tab`, `FontsHD`, and the game LAB archives.

```powershell
py -3 -m venv .venv
.\.venv\Scripts\python.exe -m pip install -r requirements.txt
New-Item -ItemType Directory -Force runtime\source
# Download and extract the original patch into runtime\source first.
$game = 'C:\Games\Grim Fandango Remastered'
$scummvm = 'C:\Program Files\ScummVM\scummvm.exe'
.\.venv\Scripts\python.exe build.py --game "$game" --patch runtime\source\datausr.lab --font C:\Windows\Fonts\arial.ttf --output runtime\game --resources copy
.\.venv\Scripts\python.exe launch.py --scummvm "$scummvm" --game runtime\game
```

Copy mode avoids Windows symlink privilege requirements, but needs space for another copy of the game resources. The output directory must not already exist. Use a new output directory when rebuilding. For an unpatched comparison, quit the test instance and run:

```powershell
.\.venv\Scripts\python.exe launch.py --scummvm "$scummvm" --game "$game" --baseline
```

Each launch writes its test configuration afresh. Regular ScummVM preferences and saves are not used. Quit one instance before starting the other. The baseline uses the original resources directly.

## macOS setup

Use the same Python dependencies and commands with `python3` (or a virtual environment's Python). The original resource path used was `/Applications/Grim Fandango Remastered.app/Contents/Resources`. Supply the ScummVM executable inside its `.app/Contents/MacOS/scummvm`. The builder defaults to symlinks on macOS; do not edit game files through those symlinks. Use `--resources copy` for an independent resource copy.

## Findings and next investigation

- On macOS, the prototype booted into Manny's office and Hebrew menu labels rendered in the correct letter order, with overlapping menu text. The user reported dialogue triggering and confirmed Hebrew subtitles/music or effects in a combined question; subtitle layout still needs a dedicated visual check.
- Manny's speech was inaudible in the patched test **and in an unpatched baseline using the original application resources in the same ScummVM build**. Speech was enabled at volume 192. This reproduces the audio issue without the Hebrew patch; it does not establish whether the cause is ScummVM compatibility, configuration, or the installed game data.
- Voice archives are present, including `en_moma069.WAV` and its lip-sync entry in `VOX0000.LAB`. Presence alone does not prove playback works. Logs contain unimplemented Remastered UI functions such as `OverlayCreate` and `GetTextObjectDimensions`.
- First test on Windows: compare the same dialogue in unpatched ScummVM and the Hebrew prototype. If both are silent, compare with the native Remastered executable before changing translation code. Record ScummVM version, game edition, audio results, and logs.
- Optional reproducible line: in Manny's office, press F9, enter `lua_do manny:say_line("/moma069/")`, then enter `exit`. This line describes his scythe. Natural gameplay dialogue should also be tested.
- Remaining patch work: proper RTL wrapping (preordered paragraphs may wrap incorrectly), menu sizing, dialogue/cutscene validation, missing Remastered translations, a redistributable Hebrew font, and reversible production installation on both platforms.

Mac source table SHA-256: `341fc3fb5be8409dc45b395e00a59b66f6292cb4e3d4a10be09872968cb36077`. Input `datausr.lab` SHA-256: `4199b6dc216c8c25b8ee247fc05ec2542a839a6845b572946c9790bce6cb49e0`. The builder writes a report beside its output with input hashes, row counts, and limitations. Other editions may differ.

## Cleanup

The builder creates a separate resource directory and never patches the original installation. Quit the test and remove only its generated output and test configuration when finished. Preserve test saves if needed. Removing the prototype's `runtime/` directory also removes its test saves and locally downloaded source patch. Production installers and uninstallers have not been implemented.
