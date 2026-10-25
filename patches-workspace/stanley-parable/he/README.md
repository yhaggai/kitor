# The Stanley Parable (2013) — Hebrew

Staged upstream payload; **not release-ready**. Windows manifest is provided; macOS is blocked pending the work below. No upstream executable was run.

- Steam 2013 edition only; excludes Ultra Deluxe and Epic. Male subtitle variant selected; upstream also offers female variant.
- Upstream documents Windows and PlayOnMac, not native macOS. No native macOS rendering verification.
- Windows native install/uninstall and gameplay validation have not run. macOS package is not built or verified. Original supported game version hashes must be established before release.

Install and uninstall must use Kitor’s selective journaled lifecycle. Do not run the original installer or restore the entire game folder. Each manifest entry is hashed; only affected existing files are backed up, and only patch-created files may be removed during uninstall.

Source provenance, archive hash, platform evidence and validation state are in `provenance.json`.
