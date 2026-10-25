# Tesla Effect: A Tex Murphy Adventure — Hebrew

Staged upstream payload; **not release-ready**. Windows manifest is provided; macOS is blocked pending the work below. No upstream executable was run.

- Steam Tesla Effect; upstream allows other stores but only Steam ownership was matched.
- macOS Unity .app ExternalData location and compatibility must be established before adding osx.
- Windows native install/uninstall and gameplay validation have not run. macOS package is not built or verified. Original supported game version hashes must be established before release.

Install and uninstall must use Kitor’s selective journaled lifecycle. Do not run the original installer or restore the entire game folder. Each manifest entry is hashed; only affected existing files are backed up, and only patch-created files may be removed during uninstall.

Source provenance, archive hash, platform evidence and validation state are in `provenance.json`.
