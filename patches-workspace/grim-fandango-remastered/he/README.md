# Native Windows Hebrew patch candidate

Version `0.1.0-native-candidate`, prepared 2026-10-26 for the tested GOG Windows edition of Grim Fandango Remastered. Runs with the native game executable. No ScummVM dependency.

Reuses Niv Bar and Shani Roz's original AdventureBrew translation: 7,704 direct-ID rows and 23 unambiguous UI aliases. The remaining 1,912 rows retain English. Native Hebrew letters and menu visibility were confirmed by the user with the prototype. The packaged Noto-derived font has not yet received a gameplay visual check. Paragraph wrapping and full-game validation remain outstanding.

## Installation and removal

This is a Kitor lifecycle data package, not a standalone installer. Use the package through Kitor's `patch_lifecycle::install` with game ID `grim-fandango-remastered`, language `he`, version `0.1.0-native-candidate`, platform `windows`, targeting the directory containing `GrimFandango.exe`. Registry publication and automatic discovery integration are not included in this candidate. Do not manually overwrite the payload if reversible installation is required.

Close the game before installation or removal. All 18 existing target files have required original SHA-256 hashes. The lifecycle refuses mismatched editions, backs up changed files, and journals the single new font. Uninstall through the same lifecycle journal restores originals and removes the added font; saves and other game files are outside the file plan. Keep English selected and enable subtitles in the native game options.

The actual Windows lifecycle passed a disposable install/uninstall roundtrip using the installed game's original affected files. See `validation.json`. Its incomplete-fingerprints label counts the newly created font, which has no original; all 18 modified files are hash-guarded.

## Package contents

Build sources are under [prototype/windows](prototype/windows/README.md); the
earlier macOS experiment is preserved under [prototype/macos](prototype/macos/README.md).

Only the English text table, generated font aliases, font specifications, manifest, and notices are shipped. No game executable, archive, movie, save, or proprietary Arial font is included. Kitor Hebrew is a renamed static derivative of Noto Sans Hebrew under the included SIL OFL 1.1. Translation credits and source are included in `licenses/CREDITS.txt`.

Build the ZIP from the repository root:

```powershell
python3 kitor/tools/build_patch_packages.py patches-workspace/grim-fandango-remastered/he/patch.windows.candidate.json --platform windows --output dist/patches
```

This is a local experimental candidate, not a published release. macOS native packaging and upstream translation redistribution review are still outstanding. See `provenance.json`.
