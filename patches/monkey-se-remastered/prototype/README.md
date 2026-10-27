# Monkey Island Special Edition — Hebrew prototype

A separate experiment for the **remastered graphics**, not the existing classic
Hebrew ScummVM patch. Run `MISE.exe` on Windows. ScummVM only plays the embedded
classic game; it does not test these remastered subtitles/fonts.

This folder contains reproducible tooling, not a released Kitor patch. Windows
execution and in-game rendering have not been verified. `release_ready` is false.

## Windows preparation (PowerShell)

Install Python 3.10+ and Git. Install your owned GOG Windows edition of The Secret
of Monkey Island: Special Edition. The tested source was GOG 1.0 (18587), game ID
1207666253. Set `$game` to the directory containing `MISE.exe`, `Monkey1.pak`,
`audio/speech.info`, and `localization/uiText.info`.

Run these commands from this prototype folder. Downloads and generated resources
stay under the Git-ignored `generated/` directory.

```powershell
py -3 -m venv .venv
.\.venv\Scripts\python.exe -m pip install -r requirements.txt
New-Item -ItemType Directory -Force generated\sources

git clone https://github.com/adventurebrew/scummworks generated\sources\scummworks
git -C generated\sources\scummworks checkout 7c8217e3746f3bc92b42623d444001789df6519c
git clone https://github.com/jmnunezizu/scummkit generated\sources\scummkit
git -C generated\sources\scummkit checkout a6ec88b02c97e66ce1c57069814c971248e62566

$game = 'C:\GOG Games\The Secret of Monkey Island Special Edition'
.\.venv\Scripts\python.exe map_dialogue.py --game "$game" --hebrew generated\sources\scummworks\monkey\strings.txt --table generated\sources\scummkit\third_party\ultimate-talkie\mi1\monster.tbl --output generated\mapping
.\.venv\Scripts\python.exe build_candidate.py --game "$game" --mapping generated\mapping\mapping.json --font C:\Windows\Fonts\arialbd.ttf --output generated\candidate
.\.venv\Scripts\python.exe -m unittest discover -p 'test_*.py'
```

Stop if any command fails. A candidate output directory must not already exist;
choose a new name for another build. Font rasterization can differ across machines.
Windows uses python-bidi, avoiding the original Mac-only FriBidi library path.

## Test the remastered game

First launch the unmodified `MISE.exe` and verify that it works. Then **quit it**.
Prepare a fresh disposable game copy (roughly 2.5 GB), and apply the generated
files only to that copy:

```powershell
$test = Join-Path (Get-Location) 'generated\windows-test'
if (Test-Path "$test") { throw 'Test directory already exists; choose a new directory' }
Copy-Item -LiteralPath "$game" -Destination "$test" -Recurse
Copy-Item -Path generated\candidate\fonts -Destination "$test" -Recurse -Force
Copy-Item -LiteralPath generated\candidate\audio\speech.info -Destination (Join-Path "$test" 'audio\speech.info') -Force
Start-Process -FilePath (Join-Path "$test" 'MISE.exe') -WorkingDirectory "$test"
```

Keep the text language **English**, enable subtitles, and use remastered graphics.
Check the opening conversation for readable Hebrew, right-to-left order, wrapping,
voice synchronization and intact English. Loose `fonts/` overrides are still an
unverified loading assumption; square or missing characters are useful evidence.

The Windows game may share saves/settings with the original installation under
its normal user profile. This prototype does not isolate them: do not overwrite
existing saves during testing. No production installer or uninstaller is included.
The original game resources are unchanged; the disposable copy can be removed
when finished. Do not delete saves you want to keep.

## What has been validated

- 4,652 fixed-size speech records, each 0x530 bytes.
- 4,301 mapped Hebrew subtitle records; 351 unchanged: 196 unmapped, 57 ambiguous,
  97 with unresolved control codes, and one with an unsupported shekel symbol.
- 42 font/PNG pairs extended with all 27 Hebrew letter forms.
- All bytes outside selected English fields, original font metrics, existing glyph
  pixels, and unrelated character mappings checked for preservation during builds.
- Five regression tests for malformed input, overflow, mapping by voice ID,
  ambiguous/control-code mappings, and unrelated-data preservation.
- Menus and item labels are unchanged. The 600-pixel wrapping limit is provisional.
- No Windows gameplay, audio, save/load, Kitor integration, or CI certification.

The source scripts and generated local data were moved here from
`patches-workspace/monkey-se-remastered/he/prototype/` for this branch.

## Provenance and distribution

Format reference: [MISETranslator](https://github.com/ShadowNate/MISETranslator),
revision `e5d8518d2c341a73c14e77a325b11dbfa47627f4`, by ShadowNate / Classic
Adventures in Greek. Hebrew source: [AdventureBrew/scummworks](https://github.com/adventurebrew/scummworks),
revision pinned above. Preserve the original translation credits, including Niv
Bar, Tomer Feiner and Miki Hoz. Voice mapping: LogicDeLuxe's Ultimate Talkie table,
provided in [SCUMMKit](https://github.com/jmnunezizu/scummkit), at the pinned revision;
its original third-party terms apply, not SCUMMKit's MIT license.

Only tooling is tracked. Game resources, translation inputs, and generated
Arial-derived fonts are local-use inputs/outputs and must not be published.
The exact Russian technical post mentioned in the original request was not
recovered; implementation references the source tool above and owned GOG data.

Verified GOG input SHA-256 values:

| File | SHA-256 |
| --- | --- |
| MISE.exe | `88877ee4ac62083861252962753bfd4266bcba90aeabd4743d2a7d36a70cc096` |
| Monkey1.pak | `fb66e7b1646f8bcacc097d3ed3c30691495cdd60dcbab0317a9e27dfd7dffa51` |
| audio/speech.info | `14dfac82c4f58796a8fbf22501be5c92dcbc4806065bd6a6e78e9d04f72b7096` |
