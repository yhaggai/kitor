# Desktop translation contributions

Select **Improve translation** on Undertale, Owlboy, Disco Elysium, or Moomin:
Midsummer Madness, or The Drifter. The editor pins the CSV from Kitor's default branch at the time the draft
opens. Saved cell edits live under
`contributors/<contributor-id>/translation-drafts/<catalog>-he.json`. Older display-order Drifter drafts retain their original filename. The new
Drifter editor starts a separate draft from the logical-order catalog. Reopening uses the same base and
edits, even if a newer CSV is available upstream.

The same grid provides search, filters, undo and redo, and a review of the exact
saved changes. Enter Hebrew in normal reading order.

- Undertale offers separate regular and niqqud catalogs. Rows marked intentionally
  unchanged are excluded.
- Owlboy and Moomin show regular and niqqud variants together. If you change the
  wording, update both rows. Niqqud alone can be corrected in the pointed row.
- Disco Elysium allows filling empty translations as well as correcting existing
  text where the game has a matching localization entry. For a blank\n  translation, keep the original English formatting tokens.

## Sign-in and local Worker setup

Use the top-right **Sign in with Google** control, or open **Improve translation**
to see the sign-in prompt. Cancel and retry are supported. Each account has its
own local draft; signing out does not delete it. Existing anonymous drafts remain
untouched in the old folder.

Debug builds use the staging Worker by default. To test the Worker and D1 on
loopback, follow [the Worker and Google OAuth setup guide](../tools/contribution-worker/README.md)
and set `KITOR_CONTRIBUTION_API_URL=http://127.0.0.1:8787` when starting Kitor.
Contributors do not sign into GitHub. The bot authors the PR and includes the
verified session's Google display name and public Kitor ID; email stays private.
The GitHub private key and Google client secret remain on the backend.

## Submission

The server checks that the pinned commit is part of upstream, applies corrections
to an allowlisted CSV, and verifies the preview digest. Your submission adds to
your open draft PR for that catalog or opens a new draft PR. The bot does not
force-push. Untouched CSV records are preserved byte for byte.

Current contribution PRs change the source CSV. Maintainers review the text and rebuild and
test the patch separately. The Drifter retains optional contribution emails. Its build
converts the source to the game’s display order, then updates the payload checksum. A submission is limited
to 200 cells and does not install or release a patch.

For rollout, land the CSVs on the default branch, deploy the contribution Worker
with the D1 catalog migration, and ship a desktop build configured to use that
Worker. Test one signed-in public draft PR per catalog in staging before release.

## Validation

```sh
bun run build
cd src-tauri
cargo test --lib services::contributions
```

Desktop and Worker tests cover the Drifter regression format, the beta CSVs,
multiline quoting, formatting rules, and byte-for-byte agreement between preview
and submission. Worker tests also check catalog-isolated PR routing. Live PR
verification requires the repository installation and server key.
