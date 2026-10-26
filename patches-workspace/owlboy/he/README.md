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
Windows gameplay and its executable edition are not verified, so this candidate
has no Windows install plan. `release_ready` remains false.

## Package

From the repository root:

```sh
python3 kitor/tools/build_patch_packages.py --check --platform osx patches-workspace/owlboy/he/patch.osx.candidate.json
python3 kitor/tools/build_patch_packages.py --platform osx --output dist/patches patches-workspace/owlboy/he/patch.osx.candidate.json
```

Kitor's lifecycle applies the explicit 30-file plan, backs up only affected
originals, and removes only patch-created files on uninstall. No save/config
files are changed. Game-derived assets are delta encoded against the user's
owned files; the package contains no complete game executable or English script.

To reproduce, build `source/tools/build.py` using pristine compatible game files,
then run `import_package.py --package BUILD_DIRECTORY --original GAME_DIRECTORY`
with `bsdiff4==1.3.2` installed. Build source needs its pinned requirements and
.NET 9 SDK. All emitted assets are hash-checked by Kitor's packaging tool.

`build_prototype.py`, `translations.json`, `font-preview.png`, and the prior
`lifecycle-validation.json` describe the historical font experiment only.
Existing `recovery/` journals are preserved for their original installations.
This is a local package import, not a published Kitor registry release.
