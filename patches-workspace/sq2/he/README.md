# Space Quest II: Vohaul’s Revenge — Hebrew

Version: 1.0-kitor2. Corrects the VOL.1 and VOL.2 upstream delta selections to match the owned Steam edition. Input and output checksum validation remains enabled.

Validated on macOS using the actual Kitor lifecycle on disposable copies of the affected Steam files: all nine originals restored byte-for-byte, only affected originals backed up, all four created files removed, and unrelated files preserved. Windows and macOS share the same file plan; both archives and a universal archive were built. Native Windows and gameplay checks remain pending.

Install and uninstall through Kitor’s declarative lifecycle. Only files listed in patch.json are affected; originals are backed up individually outside the game. Never run the upstream installer against the game.

The registry points to the published `patch-sq2-he-v1.0-kitor2` release and its checksum-verified universal archive. Restart the app to reload the local registry before retrying installation.

See provenance.json for source URLs, edition restrictions, dependencies and validation gaps.
