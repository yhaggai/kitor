# Native Windows Hebrew patch candidate

Version `0.3.0-native-candidate`, prepared 2026-10-27 for the tested GOG Windows edition of Grim Fandango Remastered. Runs with the native game executable. No ScummVM dependency.

Reuses Niv Bar and Shani Roz's original AdventureBrew translation: 7,704 direct-ID rows and 23 unambiguous UI aliases. Version 0.2 adds 345 AI-assisted movie translations while retaining 56 existing translated movie entries: all 401 references from the native cutscene script are covered. One previously script-only line is added to the text table. There are 8,072 translated rows in total; 1,568 rows retain English, including director commentary and other non-movie additions. The user confirmed movie subtitles work; full playback and editorial review remain outstanding. Native Hebrew letters and menu visibility were confirmed by the user with the prototype. The previous packaged font was exercised during that test; the user also confirmed the seventh-language option works. Paragraph wrapping and full-game validation remain outstanding.

## Installation and removal

This is a Kitor lifecycle data package, not a standalone installer. Use the package through Kitor's `patch_lifecycle::install` with game ID `grim-fandango-remastered`, language `he`, version `0.3.0-native-candidate`, platform `windows`, targeting the directory containing `GrimFandango.exe`. The development registry now includes this candidate as a bundled Windows patch, detects GrimFandango.exe, and launches the native executable directly. Rebuild/restart Kitor to use the new bundled installer and launch code. Public registry publication is not included. Do not manually overwrite the payload if reversible installation is required.

Close the game before installation or removal. All 19 existing target files have required original SHA-256 hashes. The lifecycle refuses mismatched editions, backs up changed files, and journals the new font and Hebrew text table. Uninstall through the same lifecycle journal restores originals and removes the added font and Hebrew table; saves and other game files are outside the file plan. Select Hebrew (after Portuguese) under Options ג†’ Language and enable subtitles. English and the other five original language tables remain available. Speech uses English for the Hebrew slot.

The actual Windows lifecycle passed a disposable install/uninstall roundtrip using the installed game's original affected files. See `validation.json`. Its incomplete-fingerprints label counts the newly created font and Hebrew table, which have no originals; all 19 modified files are hash-guarded.

## Package contents

The Hebrew text table, generated font aliases, font specifications, manifest, binary reconstruction recipes, and notices are shipped. No game executable, archive, movie, save, or proprietary Arial font is included. Kitor Hebrew is a renamed static derivative of Noto Sans Hebrew under the included SIL OFL 1.1. Translation credits and source are included in `licenses/CREDITS.txt`.

Build the ZIP from the repository root:

```powershell
python3 kitor/tools/build_patch_packages.py patches-workspace/grim-fandango-remastered/he/patch.windows.candidate.json --platform windows --output dist/patches
```

This is a local experimental candidate, not a published release. macOS native packaging and upstream translation redistribution review are still outstanding. See `provenance.json`.

Development integration check (2026-10-27): the bundled stage function passed a real install/uninstall byte-for-byte roundtrip on the separate game copy, was reinstalled, and launched through the same native command builder used by Kitor. The original installation remained unchanged. The Kitor Rust application passed cargo check; UI-driven installation was not exercised.

Movie translations live in `../prototype/movie-he.json`, with per-line source hashes. The builder preserves original patch wording, validates English source lines before applying new Hebrew, and retains native movie timings. Video and audio are unchanged. Version 0.3 reconstructs the native executable with a seventh text-table slot and DATA000.LAB with the seventh menu choice, using hash-guarded recipes against local originals. No complete executable or archive is distributed.

Kitor native launch selects Hebrew in the native version-2 settings file before starting a patched game. The game has no language-index command-line argument. This preserves other settings and keeps a one-time `registry.sav.kitor-before-he` backup beside the settings file. Direct launches still use the last language selected in the game menu. Unknown settings formats are rejected without modification.
