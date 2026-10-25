# Portal 2 — Hebrew

Experimental Windows GLADOS candidate, **not release-ready**. The real NSIS installer and Linux archive are preserved for audit; do not execute them through Kitor.

The upstream Windows installer applies a VPatch and then overwrites its output with a bundled VPK. The candidate instead guards the original game VPK using the upstream MD5, then copies the final files. All installed assets have SHA-256 hashes.

Upstream also changes machine-wide Tahoma font registrations. This candidate makes no registry changes. As an experiment, it replaces the four remaining Tahoma font references in the game-local VPK with Courier New. Re-reading the VPK confirms all other entries remain byte-identical. This does **not** establish that every hardcoded engine font request is covered; Windows gameplay must verify Hebrew rendering before activation.

The candidate uses Kitor selective backup/uninstall, restoring only changed originals and removing only patch-created files. Native macOS support remains blocked; the Linux zip is not evidence of macOS compatibility.

See provenance.json for hashes, conversion evidence and unresolved validation.
