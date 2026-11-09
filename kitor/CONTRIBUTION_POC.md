# Drifter desktop contribution POC

The Drifter card/details has an **Improve translation · POC** button. The editor
loads the CSV from the repository's default branch (currently `v1`), pins the
commit, and keeps committed cell edits in the application data directory under
`contributors/<contributor-id>/translation-drafts/drifter-he.json`. Reopening restores the same base and edits;
it does not silently replace an unfinished draft with the latest remote version.

Search, column filters, changed-only filtering and session undo/redo work over a
virtualized grid. Review shows the exact persisted corrections before submission.
The POC edits the CSV's existing **display-order Hebrew**, not logical Hebrew.
Normal Hebrew input conversion and in-game rendering verification remain future work.

## Sign-in and local Worker setup

Use the top-right **Sign in with Google** control, or open **Improve translation**
to see the sign-in prompt. Cancel and retry are supported. Each account has its
own local draft; signing out does not delete it. Existing anonymous drafts remain
untouched in the old folder.

Start the local Cloudflare Worker and D1 at `127.0.0.1:8787` using
[the Worker and Google OAuth setup guide](../tools/contribution-worker/README.md).
Contributors do not sign into GitHub. The bot authors the PR and includes the
verified session's Google display name and public Kitor ID; email stays private.
The GitHub private key and Google client secret remain on the backend.

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

This POC runs only on loopback. The Worker enforces authentication and rate limits.
A hosted build must set `KITOR_CONTRIBUTION_API_URL` to its HTTPS endpoint at compile time.

## Validation

```sh
bun run build
cd src-tauri
cargo test --lib services::contributions
```

Tests cover all 9,799 editable source rows, no-op byte preservation, surgical CSV
replacement, multiline quoting, placeholder validation, draft persistence and
stale-save rejection. Live bot PR verification requires the repository installation and server key.
