# The Drifter — Hebrew

Staged upstream payload; **not release-ready**. Windows manifest is provided; macOS is blocked pending the work below. No upstream executable was run.

- Steam The Drifter; standalone translation.csv payload.
- Native macOS SystemText.get_SaveDirectory returns Application.persistentDataPath when Application.platform==1 (OSXPlayer), versus ./ on Windows. InitialiseCustomLanguage appends translation.csv.
- macOS installation therefore needs a separately journaled user-data target; the current game-root-only manifest cannot safely represent this. Do not write into game .app and claim success.
- Windows native install/uninstall and gameplay validation have not run. macOS package is not built or verified. Original supported game version hashes must be established before release.

Install and uninstall must use Kitor’s selective journaled lifecycle. Do not run the original installer or restore the entire game folder. Each manifest entry is hashed; only affected existing files are backed up, and only patch-created files may be removed during uninstall.

Source provenance, archive hash, platform evidence and validation state are in `provenance.json`.
