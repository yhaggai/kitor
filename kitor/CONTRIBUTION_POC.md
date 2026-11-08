# Drifter desktop contribution POC

The Drifter card/details has an **Improve translation · POC** button. The editor
loads the CSV from the repository's default branch (currently `v1`), pins the
commit, and keeps committed cell edits in the application data directory under
`translation-drafts/drifter-he.json`. Reopening restores the same base and edits;
it does not silently replace an unfinished draft with the latest remote version.

Search, column filters, changed-only filtering and session undo/redo work over a
virtualized grid. Review shows the exact persisted corrections before submission.
The POC edits the CSV's existing **display-order Hebrew**, not logical Hebrew.
Normal Hebrew input conversion and in-game rendering verification remain future work.

## GitHub setup

The registered Kitor Desktop Contributions OAuth app is configured by default.
Override its public client ID with `KITOR_GITHUB_CLIENT_ID` when building or starting Tauri. No client
secret is used or distributed. Example:

```sh
KITOR_GITHUB_CLIENT_ID=YOUR_PUBLIC_CLIENT_ID bun run tauri dev
```

Sign-in opens GitHub's device authorization page. The contributor approves
`public_repo` access (OAuth's scope covers their public repositories, not just
Kitor). Tokens stay in the native credential store and never reach the webview.
Disconnect deletes the local credential; users can revoke the app in GitHub's
Authorized OAuth Apps settings. Expired credentials require sign-in again.

## Submission

The app uses HTTPS GitHub APIs, with no contributor-side Git installation or
clone. It creates a fork when the contributor lacks upstream push permission,
then a deterministic branch, commit, and draft PR against the pinned base branch.
Owners with upstream push permission create a separate upstream branch instead.
Repeated submission returns the existing PR. Existing branch contents are checked
before reuse; branches are never force-pushed.

Only edited CSV records and the manifest payload SHA-256 change. Untouched CSV
bytes are preserved. Unknown rows, empty translations and token changes are
rejected. A submission is limited to 200 corrections. The manifest version is
unchanged: resulting PRs are explicitly labeled **POC — do not merge**. Submission
does not build, install or release a patch. Merge conflicts remain visible in GitHub.
A new fork may take a few seconds to become available; retrying is safe.

## Validation

```sh
bun run build
cd src-tauri
cargo test --lib services::contributions
```

Tests cover all 9,799 editable source rows, no-op byte preservation, surgical CSV
replacement, multiline quoting, placeholder validation, draft persistence and
stale-save rejection. Live OAuth/fork/PR verification requires configured sign-in.
