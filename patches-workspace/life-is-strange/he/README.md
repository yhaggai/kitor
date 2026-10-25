# Life is Strange Episode 1-3 — Hebrew

Staged upstream payload; **not release-ready**. Windows manifest is provided; macOS is blocked pending the work below. No upstream executable was run.

- Targets original Windows Life Is Strange episodes 1-3, not Remastered. Verify ownership/installation of episodes 2 and 3.
- Upstream archive has three redundant Life Is Strange wrapper directories, stripped here. It also contains episode 4/5 menu files; narrative translation remains only episodes 1-3.
- Native macOS port resource layout and cooked font package compatibility are unverified.
- Windows native install/uninstall and gameplay validation have not run. macOS package is not built or verified. Original supported game version hashes must be established before release.

Install and uninstall must use Kitor’s selective journaled lifecycle. Do not run the original installer or restore the entire game folder. Each manifest entry is hashed; only affected existing files are backed up, and only patch-created files may be removed during uninstall.

Source provenance, archive hash, platform evidence and validation state are in `provenance.json`.
