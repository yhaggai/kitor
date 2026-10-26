# Translation patch updates

## Version ownership

The registry describes available translations. It never changes the version recorded as installed. `kitor.db` contains the installed version and transaction path. The transaction's `manifest.json` records the installed version, language, `release_revision`, affected paths, original/installed hashes, and original backups. This durable manifest is authoritative; library loading reconciles a stale SQLite version after a crash.

Transaction storage is outside the game, under the app data directory's `patch-transactions/<installation-hash>/<transaction>/`. On macOS the current app identifier resolves under `~/Library/Application Support/haggai/`. The cached remote registry is `patch-registry.json` in the same app data directory.

`version` is a human-facing immutable release label (for example `1.2-kitor3`). `release_revision` is a positive, increasing integer **per game and language**, used for ordering. Existing packages/journals without the field are revision 0. Different game editions have different game IDs. Never compare arbitrary version labels alphabetically. Every published change, including a packaging-only fix, needs both a new version label and a higher revision. For a translation change, update the translator version portion; for a packaging fix, increment the `-kitorN` suffix.

## Publishing on GitHub

1. Update `patches-workspace/<game>/he/patch.json`: version, release_revision, filename, payload hashes and file plans. Keep plans for both Windows and macOS. Update the corresponding bundled registry entry when changing a bundled package; its version and revision must match that package.
2. Write `release-notes.md` alongside the manifest. Describe translation changes separately from packaging/runtime fixes. Record compatibility and native validation evidence in the existing provenance file; set `release_ready: true` only after the package is reviewed and validated on both platforms.
3. Open a PR. The existing validation workflow builds and tests both platforms. After a trusted push to `main` or `v1` passes that workflow, Deploy Patches checks out that exact validated SHA and runs `publish_patch_releases.py`. GitHub `workflow_run` workflows must exist on the repository's default branch to trigger; land this workflow there before expecting automatic publication from another supported branch.
4. The publisher builds the deterministic package, creates a draft release tagged `patch-<game>-<language>-v<version>` at the validated commit, uploads the archive, verifies publication, and makes it public. An existing asset must match byte-for-byte; it is never overwritten. Failed draft uploads can be retried.
5. Only after all referenced packages are published does the publisher update `registry.json` on the `patch-registry` branch. This branch is created on first publication. Catalog updates use optimistic concurrency and an increasing catalog revision. Previously published entries are retained. A publication failure leaves the previous catalog intact.

The app reads `https://raw.githubusercontent.com/yhaggai/kitor/patch-registry/registry.json` at startup and from **Check Translation Updates**. The repository is currently private; public distribution starts after its owner makes it public. The publisher refuses private-only distribution and this PR does not change repository visibility. The endpoint and release assets must be publicly readable; no GitHub token is shipped to users. Before the first approved publication, a missing endpoint is treated as an unavailable check and bundled/cached translations remain usable. Deploying this code does not itself mark any existing unverified translations release-ready.

The feed contains only patch release data for game/language pairs already known to the app. It cannot replace launch commands, detection rules, or installer scripts. Each entry includes platform support, release revision, immutable versioned GitHub URL, package SHA-256, file size, author and release notes. Downloads are checksum-checked and manifest identity/revision must match the registry before installation. Registry schema 1 requires this updater-capable app; adding unknown games or new installer formats still needs an app release.

The current trust root is the fixed HTTPS GitHub repository, with SHA-256 binding the catalog to package bytes. This is not a cryptographically signed catalog. Cache validation rejects revision rollback or changing a published revision in place. Offline/network/validation failure preserves the last valid cache. Signing/key rotation and release withdrawal are future extensions; this version intentionally rejects removing published catalog entries.

## Updating safely

The user chooses **Update Translation** on a game card, which shows installed → available versions and release notes. There is no automatic installation. The update checks that current patch files and original backups are intact, rejects ownership conflicts, and builds the next translation against a sparse view of original files. Only affected files and declared decoder inputs are copied, never the whole game directory.

A nested `pending-update` journal records the union of paths affected by the old and new translations. Its backups contain the previous working contents; a separate originals directory contains the original-game baseline for future uninstall. Removed patch-created files disappear and originals no longer modified by the new translation are restored. Saves and unrelated files are left alone. File-to-directory migrations are rejected rather than guessed.

The active manifest is replaced atomically only after the new files verify. Before this commit, failure or interruption rolls back to the previous translation. After it, recovery retains the new translation and reconciles SQLite. Recovery occurs on library load, verification, retry and uninstall. Unknown later edits or missing backups stop recovery and preserve its files for repair; Kitor never silently overwrites them. Cleanup is detached atomically so an interruption while deleting temporary recovery data cannot corrupt recovery state.

Original backup generations are retained conservatively. The temporary previous-translation snapshot is removed after a successful commit; there is no user-facing downgrade/rollback history. Updates require additional disk space for affected originals, staged outputs and rollback snapshots. Close the game before updating; external launchers cannot be locked by Kitor.

## Verification

Run from the repository root:

```sh
python3 -m unittest discover -s kitor/tools -p 'test_*.py'
python3 kitor/tools/run_lifecycle_tests.py
```

Update fixtures cover both platform plans, multi-version updates, immutable originals, obsolete files, selective uninstall, payload corruption, missing backups, externally changed files, partial apply failure, interruption before/after commit, recovery conflicts and retry. Native Windows/macOS CI runs these same backend tests; fixtures do not establish gameplay compatibility for each translation release.
