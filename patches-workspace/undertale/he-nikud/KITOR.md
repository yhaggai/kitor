# Kitor integration

Vendored from https://github.com/yhaggai/undertale-hebrew-patch at commit
`f15f94d4d2c29647b85b5e8121dc8d2007e25cb9` (Alpha 2 Steam).
Kitor now adds an alpha 3 nikud draft: local machine vowelization, atomic pointed
glyph generation and matching RTL classification. `patch.json` provides Kitor
packaging. See README.md for credits, rights, and translation limitations.

Requires Python 3.10+ (`python3` on macOS, `py -3` on Windows).
Only the exact Steam archive hashes in the manifests are supported. Windows
runtime testing remains outstanding. Linux is unsupported. Keep English selected.

Kitor reads the per-platform file plan in `patch.json`, validates the original
archive and delta hashes, and stages the output with its own BSDIFF decoder.
It backs up only `game.ios` (macOS) or `data.win` (Windows), once, outside the game
folder. Kitor owns rollback/uninstall and verifies the installed file before
restoring. Package tools and build sources are never installed into the game.

Kitor does not run upstream `install.py`. That script retains independent
backup/rollback for standalone users following the upstream README.

The release workflow includes patch.json and publishes
`patch-undertale-he-x-nikud-valpha3-nikud-kitor1/Undertale-Hebrew-alpha3-nikud.zip`.
Publish this release before distributing the catalog. Close the game before
installing or restoring the patch.

The unpointed Hebrew track remains `he` (alpha 2). This independent track is
`he-x-nikud`; it never upgrades or replaces the unpointed option. Restore the
installed translation before selecting and installing the other option.
The new package must be published before its download is available to users. Nikud is a
machine draft and needs linguistic review and in-game checks on both platforms.
