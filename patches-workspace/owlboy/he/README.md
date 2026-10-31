# Owlboy — full Hebrew translation for Kitor

The current patch is `patch.osx.candidate.json`: all 4,537 catalog entries,
Hebrew pixel fonts, RTL rendering, and the corrected full-size 142×89 flag.
It supersedes the earlier 14-line font prototype in this directory.

Source: https://github.com/yhaggai/owlboy-hebrew (commit 2cfe6b3).
The original patch source is retained in `source/`; `licenses/` contains its license.

## Supported target

Steam Owlboy 1.4 macOS FNA edition, at `Owlboy.app/Contents/Resources`.
Original executable SHA-256:
`e399de418effb376c7c48916f6da19e2aa7fa91a2fea0cce7479e52cb17e653c`.
Python 3 is required by Kitor's bundled BSDIFF decoder.
Windows Steam FNA build 105-e322 is supported by `patch.windows.candidate.json`.
Original Windows executable SHA-256:
`036eff99b4a693dc090f23fbba781b71a6f747191d8c6ad54e1a7b5e3d2ad073`.
It has a separately compiled runtime, executable delta, and nikud assets under
`windows/`. The legacy XNA edition is unsupported.
Full-game playtesting is pending. `release_ready` remains false.

## Package

From the repository root:

```sh
python3 kitor/tools/build_patch_packages.py --check --platform osx patches-workspace/owlboy/he/patch.osx.candidate.json
python3 kitor/tools/build_patch_packages.py --platform osx --output dist/patches patches-workspace/owlboy/he/patch.osx.candidate.json
python3 kitor/tools/build_patch_packages.py --check --platform windows patches-workspace/owlboy/he/patch.windows.candidate.json
```

Kitor's lifecycle applies the explicit 30-file macOS or 42-file Windows plan, backs up only affected
originals, and removes only patch-created files on uninstall. No save/config
files are changed. Game-derived assets are delta encoded against the user's
owned files; the package contains no complete game executable or English script.

To reproduce, build `source/tools/build.py` using pristine compatible game files,
then run `import_package.py --package BUILD_DIRECTORY --original GAME_DIRECTORY`
with `bsdiff4==1.3.2` installed. Build source needs its pinned requirements and
.NET 9 SDK. All emitted assets are hash-checked by Kitor's packaging tool.
For Windows import, add `--platform windows`; the importer preserves macOS assets.
The current Windows version is `0.2.0-beta.2-kitor2`; see the nikud build below.

`build_prototype.py`, `translations.json`, `font-preview.png`, and the prior
`lifecycle-validation.json` describe the historical font experiment only.
Existing `recovery/` journals are preserved for their original installations.
This is a local package import, not a published Kitor registry release.

## Windows nikud beta (kitor2)

The Windows package combines regular Hebrew (`he`) and Hebrew with nikud (`hn`)
from `yhaggai/owlboy-hebrew` branch `niqqud-beta`, commit
`a9c152404ba43c145ce1fbd49a87a09121193f35`. Choose the Israeli flag marked **N**
in the language menu. All 4,537 entries are included; the upstream nikud is a
reviewed machine-assisted beta, not an exhaustively proofread translation.

Kitor installs both locales in one transaction, so uninstall restores the
original game files without leaving a separate add-on. Before uninstalling,
select English in Owlboy; a saved `he` or `hn` preference cannot load after its
language files are removed. Save-game files are never included in the package.
The existing macOS payload remains the regular Hebrew edition.

Rebuild with Python 3.10+, the pinned `source/requirements.txt`, bsdiff4 1.3.2,
and .NET 9 SDK against **pristine Windows FNA files**:

```sh
python source/tools/build_niqqud.py --game ORIGINAL_GAME --out BUILD_DIRECTORY
python import_package.py --platform windows --package BUILD_DIRECTORY --original ORIGINAL_GAME
```

The importer keeps Windows assets in `windows/` and regenerates its Rust
embedding table. Nikud catalogs are checked in; no model download or online
text service is needed to build.
