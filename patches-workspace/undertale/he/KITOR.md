# Kitor integration

Vendored from https://github.com/yhaggai/undertale-hebrew-patch at commit
`f15f94d4d2c29647b85b5e8121dc8d2007e25cb9` (Alpha 2 Steam).
Upstream files are unchanged. `patch.json`, `.gitattributes`, and this file provide
Kitor packaging. See README.md for credits, rights, and translation limitations.

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
`patch-undertale-he-valpha2-steam-kitor2/Undertale-Hebrew-alpha2-steam.zip`.
Publish this release before distributing the catalog. Close the game before
installing or restoring the patch.
