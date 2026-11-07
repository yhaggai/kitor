# Code signing policy

## Status

Kitor is preparing an application to SignPath Foundation. Acceptance and
SignPath integration are pending. Current Tauri updater signatures are not
Windows Authenticode signatures and do not imply SignPath endorsement.

If accepted and enabled, the attribution will be:
“Free code signing provided by [SignPath.io](https://about.signpath.io),
certificate by [SignPath Foundation](https://signpath.org).”

## Responsibilities

- Author/committer and reviewer: [Haggai Yaniv (@yhaggai)](https://github.com/yhaggai).
- Proposed signing approver: [Haggai Yaniv (@yhaggai)](https://github.com/yhaggai).

Outside contributions must be reviewed by the maintainer. Before signing is
enabled, every person with repository or signing access must use MFA.
Every signing request must receive manual approval from the signing approver.
These are policy requirements; account settings have not yet been verified.

## Scope and release controls

The proposed scope is Kitor's own Windows executable and NSIS installer,
built from a reviewed commit by `.github/workflows/release-app.yml`.
Downloaded games, community patch archives, and upstream executables are not
proposed for signing under Kitor's identity. Their provenance and licenses
must still be reviewed and disclosed; downloading separately does not establish
SignPath eligibility.

Before approving a request, verify the source commit, successful build checks,
artifact origin, product name (`kitor`), consistent release version, and the
component/license inventory. Do not sign manually substituted artifacts.
Keep the source commit, workflow run, and signing request associated with the
release. A failed or unapproved signing request must prevent publication of a
release advertised as Authenticode-signed.

Authenticode signing must finish before generating the Tauri updater signature
for the final Windows installer. Changing installer bytes afterward invalidates
that updater signature. Existing updater keys remain separate.

See the [privacy policy](PRIVACY.md), [licensing scope](LICENSING.md), and
[application preparation record](docs/signpath-application.md).
