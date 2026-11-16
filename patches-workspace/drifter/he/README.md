# The Drifter — Hebrew

Kitor bundles version `2026-07-09-kitor2` for Windows and macOS. Both platform ZIP artifacts are also built from `patch.json`. Payload data is architecture-independent; use the native game from Steam.

## Rebuild the translation

Edit the Hebrew column of [translation-catalog.csv](translation-catalog.csv) in
normal reading order. Keep existing placeholders and game IDs. The build
converts Hebrew to the display order used by the game's translation loader and
updates the generated payload and its manifest checksum.

```sh
python3 -m pip install -r patches-workspace/drifter/he/requirements.txt
python3 patches-workspace/drifter/he/build.py
python3 patches-workspace/drifter/he/build.py --check
python3 kitor/tools/build_patch_packages.py patches-workspace/drifter/he/patch.json --output dist/patches
```

For a new release, give the rebuilt patch a new version and matching ZIP name
in the manifest. The saved build receipt ties the editable catalog to the payload
used to verify contribution release notifications.

On Windows, the translation installs into the detected game directory. On macOS, Steam build 22758595 is checked against the assembly SHA-256 in `macos-loader-evidence.json` before installation. Its loader reads `translation.csv` from `~/Library/Application Support/com.Powerhoof.TheDrifter`. Start the game once to create this directory before installing. Symlinked user-data paths are rejected.

Kitor journals this user-data target separately from the game bundle. Only an existing `translation.csv` is backed up. Uninstall restores it (or removes the patch-created translation), preserving saves, settings, unrelated files and subsequent additions. Retry preserves the original backup; conflicting later edits are rejected. Uninstall does not require the original game assembly version.

Validation: Windows and macOS packages built; both file plans roundtrip in fixtures on macOS. User-data tests cover existing and absent translation files, reinstall, unrelated saves, and symlink rejection. The full lifecycle suite also checks interrupted operations and missing backups. Installed macOS assembly hash matches the supported build. Native Windows execution and gameplay on either platform remain unverified; `release_ready` remains false.

Kitor’s Play button opens the native `TheDrifter.app` directly on macOS, from either the detected bundle or its Steam parent folder. It does not require ScummVM.
