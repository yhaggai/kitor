# Kitor desktop privacy policy

Updated: 12 November 2026.

Kitor stores its game library, local installation paths, settings, patch history,
and related account state on the user's computer. Scanning and patching operate
on local game files. The application is not designed to upload game files or
local folder inventories to the project maintainer.

## Network activity

On startup, Kitor checks GitHub for app updates and refreshes the patch catalog.
The public patch catalog and public patch downloads use unauthenticated requests.
Remote cover and banner images may load when you view the
catalog or library. Requests expose your IP address and ordinary connection
metadata. An image or download URL can identify the game or item requested.

After you connect Steam or GOG, Kitor can refresh the connected library when the
app starts. Steam receives the account ID and the credentials needed to retrieve
the library. GOG receives the access token and, when needed, the refresh token.
Kitor stores native Steam credentials in the operating-system credential store
and GOG tokens in its local application database. An optional Steam Web API key
also uses the credential store.

Requested downloads may contact GitHub, other catalog-listed patch hosts, Steam,
GOG, or ScummVM. On Windows, the installer downloads Microsoft's WebView2
bootstrapper if the required runtime is missing. Microsoft handles that download
and installation. Kitor does not include dedicated analytics or automatic crash
reporting.

## Translation contributions

The editor keeps drafts on your device. If you sign in to contribute, Google
authenticates you and Kitor's contribution service stores your Google account
identifier, display name and verified email. Kitor stores its contribution session
in the operating-system credential store. When you submit edits, the service
receives the edits and your email notification preference and creates or updates
a GitHub pull request. The submission displays your name and Kitor contributor
ID. The service keeps your Google account identifier and email private.

## Controls and third parties

You can disconnect Steam or GOG in Kitor to stop future library refreshes for
that account. The Windows installer currently has no switches for startup update
checks, catalog refreshes or remote artwork.

Third-party handling is governed by the respective providers, including
[GitHub](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement),
[Steam](https://store.steampowered.com/privacy_agreement/),
[GOG](https://www.gog.com/en/policies/privacy),
[Google](https://policies.google.com/privacy), and
[Microsoft](https://privacy.microsoft.com/privacystatement).
Other patch and image hosts have their own policies.

Removing the app may leave application data and patch backups. Restore installed
patches through Kitor before deleting their backups. Reports submitted to the
[public issue tracker](https://github.com/yhaggai/kitor/issues) are public;
remove credentials, private paths, and account details before posting.
Contact the maintainer through the [GitHub profile](https://github.com/yhaggai)
for privacy questions; do not post secrets in an issue.

This policy covers the desktop application, not third-party games or websites.
