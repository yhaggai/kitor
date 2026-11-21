# Game build checks

To check immediately, open GitHub Actions → **Check Steam patch files** → **Run workflow**, select `v1`, and run it. The manual run refreshes store builds, checks GOG and Steam patch inputs, then sends new patch alerts. Alerts are deduplicated; rerunning a check does not resend an already delivered comparison. A new build alone does not generate email unless a comparison finds changed patch inputs.

CLI equivalent:

```sh
gh workflow run check-steam-patch-files.yml --repo yhaggai/kitor --ref v1
```

The existing daily schedules remain: store scan at 05:23 UTC, GOG comparisons at 05:43, Steam comparisons at 06:03, and email alerts at 06:33. GitHub scheduling can be delayed.

Manual Worker operations use `POST /admin/check`, a Bearer token, and a JSON action of `scan`, `gog`, or `alerts`. They share the scheduled handlers. No token means access is denied. The dashboard has no public button that can start scans or send email.

Credentials are stored in Infisical, project `603f030a-87bd-4525-9656-3be1f1e7d84d`, `prod`, root path:

- `KITOR_MONITOR_TRIGGER_TOKEN`: mirrored to the GitHub secret of the same name and Worker secret `MANUAL_CHECK_TOKEN`.
- `KITOR_MONITOR_CLOUDFLARE_TOKEN`: mirrored to the matching GitHub secret; requires Workers KV Storage Edit on the monitor account.
- `KITOR_MONITOR_CLOUDFLARE_ACCOUNT_ID`: mirrored to the matching GitHub variable.
- `KITOR_STEAM_CLOUD_SESSION`: mirrored to the matching GitHub secret. Reconnect and refresh it when Steam expires or revokes the session.

Run authentication and trigger tests with `node --test tools/game-build-monitor/manual-check.test.mjs`; the comparison workflow runs this command before accessing credentials.

## Automatic patch update issues

Both daily and manual Steam comparison runs report confirmed changed patch inputs from Steam and GOG to repository issues. A new build alone, an unchanged comparison, or an unavailable manifest does not create an issue. Each issue lists the patch, supported and current builds, and changed paths. Comparison identities deduplicate repeated checks; open issues are updated if the report changes. Closed issues are acknowledged and are not reopened or recreated for the same comparison. New comparisons can create new issues. Reporting uses the workflow’s built-in GitHub token with job-scoped Issues write access; no new external credential is needed.
