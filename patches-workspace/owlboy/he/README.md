# Owlboy Hebrew font prototype

Experimental font proof of concept, not a full Hebrew translation. Version 0.1.0.

The prototype adds 27 original hand-drawn Hebrew letter forms (including final
forms) and translates 14 menu/dialogue entries. Everything else falls back to
English. `font-preview.png` is an offline atlas preview, not a game screenshot.

## Current installation

Installed in the owned Steam macOS build on 25 October 2026, at:

`~/Library/Application Support/Steam/steamapps/common/Owlboy/Owlboy.app/Contents/Resources`

22 files affected: 16 existing files backed up, six files created. Saves and
options were not modified by the installer. English is still the configured
language. Restart Owlboy, open Languages, and select the Israeli flag to try it.
Some opening dialogue and the main menu are translated; do not overwrite an
existing save just to test the opening scene.

Recovery journal (keep until uninstall succeeds):

`recovery/83ba8ef99d205add1eacc047c8c7e6e606ecb82fefd11b13a597c6e979d100ca/install-WiaRZq`

## Known limitations

- Hebrew strings are stored in visual order for the existing LTR renderer. This
  is deliberately restricted to simple all-Hebrew samples. Typewriter animation
  remains LTR; mixed English/numbers, highlights and timing tags need a proper
  rendering solution before a full translation.
- Glyph style and spacing are a first draft. No niqqud.
- Native gameplay is **not verified**. The computer-use service returned stale
  window captures, `noWindowsAvailable`, and a ScreenCaptureKit capture failure
  during the relaunch/selection attempt. No Hebrew in-game screenshot was obtained.
- The Windows ZIP is a data candidate using the macOS edition's asset hashes.
  It must not be described as native-Windows tested. Kitor refuses mismatched
  originals; an owned Windows installation is needed for compatibility checks.
- The packages use Kitor's existing install/uninstall lifecycle format but are
  intentionally not published or added to the production registry.

## Build and verification

Run from this directory (Pillow is required only to build):

```sh
python3 build_prototype.py '/path/to/Owlboy/resources'
cargo build --offline --manifest-path native/Cargo.toml --release
python3 verify_lifecycle.py
python3 ../../../kitor/tools/build_patch_packages.py patch.json --output dist
```

`verify_lifecycle.py` currently points to this Mac's owned installation as its
read-only input. It creates disposable subsets and tests both osx/windows plans
with the actual Kitor lifecycle implementation. Both plans passed: installed
hashes match, 16 affected originals are restored byte-for-byte, six new files
are removed, and untouched/later-added sentinel files survive. These are
fixture checks on macOS, not native Windows gameplay tests.

The builder derives every original hash from the supplied installation. Do not
rebuild against an already patched installation: uninstall first. Build inputs
and generated ZIPs contain original game data and remain local/ignored.
Original Hebrew glyph definitions and translations are in `build_prototype.py`.
The generated `translations.json` is a readable export, not the build input.

## Install and uninstall

The small native CLI delegates to Kitor's journaled lifecycle; it does not
implement a separate backup mechanism. Build it on the target platform with
Rust if using the CLI. macOS ARM64 CLI was built locally; no Windows CLI was built.
Alternatively, the generated ZIPs are data packages for the existing native
Kitor lifecycle (`osx` / `windows`). Select the directory containing Data and
Owlboy.exe, not the outer macOS application directory.

Close Owlboy before installation. From this directory:

```sh
native/target/release/owlboy-font-test install "$PWD" '/path/to/resources' "$PWD/recovery" osx
```

Select English in-game before uninstalling, then exit. To undo this installation:

```sh
native/target/release/owlboy-font-test uninstall \
  "$PWD/recovery/83ba8ef99d205add1eacc047c8c7e6e606ecb82fefd11b13a597c6e979d100ca/install-WiaRZq" \
  "$HOME/Library/Application Support/Steam/steamapps/common/Owlboy/Owlboy.app/Contents/Resources"
```

Restore only the journaled files. Do not remove recovery data or clear the game
folder. The lifecycle checks for later modifications and refuses conflicts.
