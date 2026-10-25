# Windows validation using your owned games

Run on Windows with Rust/Cargo (MSVC toolchain), Visual Studio C++ Build Tools
with the Windows SDK, and Python 3 installed. Python's `py -3` launcher
is required for BSDIFF patches. The supplied Windows Xdelta executable is
SHA256-checked before use. No game is launched.

Extract the handoff outside all game folders. Build outputs go to a dedicated
Kitor directory under the Windows temporary directory. Before invoking Cargo,
the script resolves paths and rejects tool/build/Cargo-cache/report locations
inside any configured game folder.

Copy `windows-game-paths.example.json` to `windows-game-paths.json`. Fill only
paths you actually have, leaving other entries blank. JSON backslashes must be
escaped (`\\`). Use the game **data directory whose relative layout matches the
patch's manifest**, which may be below its launcher directory. Inspect each
patch's `provenance.json` for edition constraints; do not guess paths. The
example contains all 24 current candidate/dual-platform game IDs with blank
paths.

In PowerShell, from the handoff/repository root:

```powershell
.\kitor\tools\validate-patches-windows.ps1 -Config .\windows-game-paths.json
# Explicitly include staged Windows-only/experimental candidates:
.\kitor\tools\validate-patches-windows.ps1 -Config .\windows-game-paths.json -IncludeCandidates
```

If local PowerShell policy blocks scripts, use a process-scoped execution policy
only for this reviewed script; no permanent policy change is required. The
script builds a native Windows Rust validator and writes a timestamped JSON
report; `-Report` can choose a new filename outside every game directory.
Existing reports are never overwritten. Exit 0 means at least one roundtrip
passed and none failed. Failures or an empty/all-skipped configuration exit
nonzero. Blank paths and disabled candidates are recorded as skipped.

Each selected manifest first passes the Python package validator, including
payload/tool SHA256 hashes and path safety. The Rust validator **reads the owned
game directory only**. It copies only existing declared affected files and delta
inputs into a temporary subset, never the entire game directory. Existing
Kitor install/uninstall logic runs on that subset. Checks cover selective backup
count, declared installed hashes and copy payload bytes, exact original
restoration, removal of created files, and preservation of two unrelated
sentinels. Temporary subsets are removed when each check ends. Failed checks
do not alter the owned game; edition fingerprint mismatches are reported.

A report `pass` means this disposable install/uninstall test passed. It does
not prove ownership or gameplay compatibility. `guardedFileCount` and
`editionValidation` distinguish matched original fingerprints from incomplete
or absent guards; an unguarded copy patch can pass even on an inappropriate
folder. Manual testing in a separate disposable playable game installation is
still needed before enabling or publishing a patch. Keep the report for review.

For developers, the CLI accepts `--allow-non-windows` only for disposable
fixtures and reports `nativeWindows: false` there. macOS fixture passes are not
Windows execution evidence. Synthetic tests are in
`test_validate_owned_games.py` and require a built validator binary.
