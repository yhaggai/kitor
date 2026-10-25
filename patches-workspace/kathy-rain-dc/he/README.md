# Kathy Rain: Director's Cut — Hebrew

Staged upstream payload; **not release-ready**. Windows manifest is provided; macOS is blocked pending the work below. No upstream executable was run.

- Director's Cut only, not original Kathy Rain. Windows Unity resources.assets replaces one 28829728-byte asset file.
- Switch to English temporarily for the chapter 2 tape puzzle and final lock puzzle, then switch back to Hebrew (upstream guidance).
- Windows Unity resources.assets cannot be assumed compatible with macOS .app bundle. Exact source-game build hash is not supplied.
- Windows native install/uninstall and gameplay validation have not run. macOS package is not built or verified. Original supported game version hashes must be established before release.

Install and uninstall must use Kitor’s selective journaled lifecycle. Do not run the original installer or restore the entire game folder. Each manifest entry is hashed; only affected existing files are backed up, and only patch-created files may be removed during uninstall.

Source provenance, archive hash, platform evidence and validation state are in `provenance.json`.
