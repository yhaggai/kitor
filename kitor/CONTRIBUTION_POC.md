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

## Local bot setup

Contributors do not sign into GitHub. The desktop sends reviewed edits to a
separate local backend at `127.0.0.1:47831`. See
[the bot setup guide](../tools/contribution-bot/README.md).

The maintainer installs Kitor Translation Contributions on only `yhaggai/kitor`.
Its private key stays in a restricted file outside the repository and desktop
bundle. The backend obtains short-lived installation tokens restricted to that
repository, with Contents and Pull requests write access. PRs are authored by
the bot; contributor identity is not verified in this local POC.

## Submission

The server validates the pinned base against upstream, downloads the canonical
CSV, applies the corrections and verifies the preview digest. Clients cannot
choose repositories, paths or arbitrary file contents. It creates a deterministic
branch in upstream and a draft PR. Repeated submission returns the existing PR;
existing branch contents are checked before reuse and never force-pushed.

Only edited CSV records and the manifest payload SHA-256 change. Untouched CSV
bytes are preserved. Unknown rows, empty translations and token changes are
rejected. A submission is limited to 200 corrections. The manifest version is
unchanged: PRs are labeled **POC — do not merge**. Submission does not build,
install or release a patch. Merge conflicts remain visible in GitHub.

This backend binds only to loopback. Public deployment needs authentication,
rate limits and abuse controls; the desktop header is not authentication.

## Validation

```sh
bun run build
cd src-tauri
cargo test --lib services::contributions
```

Tests cover all 9,799 editable source rows, no-op byte preservation, surgical CSV
replacement, multiline quoting, placeholder validation, draft persistence and
stale-save rejection. Live bot PR verification requires the repository installation and server key.
