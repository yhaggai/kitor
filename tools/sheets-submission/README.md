# Drifter Sheets submission POC

Bound Apps Script for the private Drifter spreadsheet. No website, local server,
S3, CLI token, or draft database is required at runtime. Google stores the draft;
GitHub stores submitted changes.

## Setup

1. In the spreadsheet, open Extensions → Apps Script.
2. Install `Code.gs`, `Sidebar.html`, and `appsscript.json`.
3. Register a dedicated GitHub OAuth app with Device Flow enabled. The callback
   URL is unused by device flow (a loopback URL may be supplied for registration).
4. Set the non-secret `GITHUB_CLIENT_ID` in Apps Script project Script Properties.
5. Reload the sheet. Choose Kitor → Submit corrections and authorize Google.
6. Click Connect GitHub and complete device authorization. The OAuth scope is
   `public_repo`; GitHub OAuth cannot restrict it to just one public repository.
   The implementation fixes all repository writes to `yhaggai/kitor`.

User tokens are stored in Apps Script User Properties, never cells, source code,
browser HTML, or logs. Keep this personal POC sheet private: another script editor
could change the code that later runs as an authorized user. Disconnect removes
the local token; revoke the application in GitHub to invalidate its authorization.
Expired tokens currently require reconnecting; refresh-token support is not built.

## Usage and limitations

- Edit only column F in the original translation tab, then Review changes →
  Create draft pull request. Up to 50 corrections per submission.
- Hebrew is still in the source CSV's display order. Logical-order editing is
  not implemented by this submission experiment.
- The sheet is pinned to commit `33928fb8a744ec0d5ebc29f7841e3f346098b194`, branch
  `v1`. New upstream work is handled by the resulting GitHub PR, not overwritten.
- The original first five columns form a unique identity. Row sorting is allowed;
  missing/duplicate rows, changed source cells, settings, formulas, empty corrections,
  and changed recognized control tokens are rejected.
- Only changed CSV records are reserialized. Other bytes remain intact.
- A single commit updates the CSV and manifest checksum. Release version is not
  changed: these are explicitly non-release draft PRs, not ready to merge/publish.
- This personal POC requires repository write permission. Community fork support
  is not implemented.
- The preview is checked again before mutation. A document lock serializes
  submissions. A deterministic branch name and PR lookup avoid duplicate PRs.
- No native gameplay testing or package build runs inside Apps Script.

## Tests

Run `node tools/sheets-submission/test.cjs` from the repository root. This checks
the complete 9,802-record source, exact unchanged roundtrips, minimal edit diffs,
reordered rows, multiline quoted edits, and rejection paths.

## Live setup

Spreadsheet: https://docs.google.com/spreadsheets/d/1XT5NkMfIbb7vIK1PJ8Qp-PgskRuhMiN7tG7_mnz6vLk/edit

Bound script: https://script.google.com/home/projects/1avoiRyWHeI13-lZNi_4k59vtKoFQYbEL1ly006p3ut-0dEhCr9K1E7ZI/edit

The browser-installed version currently inlines the sidebar in `Code.gs` because
Safari exposes limited accessibility controls in the Apps Script file manager.
The separate files here are the maintainable source; the inline version has the
same public entry points and data handling.
