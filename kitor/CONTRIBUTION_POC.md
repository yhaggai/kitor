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

The desktop uses the Kitor Translation Contributions **GitHub App**, with Device
Flow and expiring user tokens. Its only repository permissions are Contents
read/write, Pull requests read/write and mandatory Metadata read. No account,
organization, administration, workflow or repository-creation permission is requested.
No client secret or private key is distributed. The public client ID can be
overridden with `KITOR_GITHUB_APP_CLIENT_ID`.

Contributors sign in, create their Kitor fork using GitHub's web page if needed,
and install the app with **Only select repositories**, choosing just their fork.
The upstream owner selects `yhaggai/kitor` instead. The editor verifies that the
installation selects exactly one owned Kitor repository and rejects all-repository
installations. The maintainer must also install the app on the upstream repository
for upstream API operations. “Check access again” resumes after browser setup.

Automatic fork creation is intentionally removed: GitHub's fork API requires
all-repository installation access on the destination account, defeating this
restriction. A contributor does this once in GitHub's UI instead.

Tokens remain in a dedicated native credential-store entry, separate from the old
OAuth experiment; the webview never receives them. Cancel sign-in stops polling
and removes the pending local session. Start again requests a new device code.
Cancelling locally does not revoke an authorization already granted on GitHub;
users can revoke it in GitHub's Authorized GitHub Apps settings.

## Submission

The app uses HTTPS GitHub APIs, with no contributor-side Git installation or
clone. It writes a deterministic branch and commit only in the verified selected
repository, then opens a draft PR against the pinned upstream base branch.
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
