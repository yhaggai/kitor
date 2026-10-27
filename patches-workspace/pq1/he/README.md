# Police Quest: In Pursuit of the Death Angel — Hebrew

Catalog 16; Kitor ID `pq1`; bundled patch version `1.0-kitor2`.

The Steam Collection Windows depot (app 494740, depot 494741, manifest
427993029009177791) includes the supported original EGA version 2.0G under
`PQ1/EGA`. The VGA remake and Police Quest 2–4 are not covered by this patch.

Kitor uses ScummVM detection where available and seven publisher MD5 fingerprints
to identify supported original or patched EGA data. It resolves collection roots,
direct EGA folders, case differences, and GOG macOS wrappers. Automatic scanning
includes Kitor’s managed games folder. A temporary database scan of the owned
collection twice produced exactly one library entry at the actual EGA data root.

The bundled installer selects `patch.json` or `patch.steam.json` from the source
fingerprints. The Steam plan uses upstream variant 1 for VOL.0 and LOGDIR;
the other five original fingerprints match variant 0. Both versions retain
original and installed MD5 guards. Payloads retain SHA-256 integrity checks.
No original game content is included in the patch packages.

Install/uninstall uses Kitor’s existing declarative lifecycle. Only seven changed
originals are backed up, and three new files are tracked for removal. The native
macOS disposable test restores originals byte-for-byte and preserves unrelated
files, including a later save. It also exercises lowercase file names.

The rebuilt Kitor app detected the managed Steam installation and records the
patch active. All ten installed hashes and the seven-file backup were checked.
The user confirmed that Hebrew launch works on macOS on 2026-10-28. ScummVM’s
visible game entry identifies the edition as DOS/Hebrew. Save/load has not been
independently tested. Existing hosted English/Hebrew cover mappings are reused.

Validation: 27 Python tests; 30 lifecycle tests passed (2 ignored); two PQ1
adapter tests with owned data; repeated temporary-database scan; TypeScript,
frontend, and macOS app build. See `steam-validation.json` for scoped evidence.
Windows and macOS patch archives build, but native Windows gameplay, GitHub
Actions, and publication remain pending. `release_ready` remains false.

Build either manifest using `kitor/tools/build_patch_packages.py --output
dist/patches`, or add `--universal` for its combined archive. CI now includes the
Steam manifest as well as the original plan. The app embeds both plans and their
payloads so installation does not depend on an unpublished download URL.
