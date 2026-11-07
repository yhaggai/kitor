# Kitor desktop privacy policy

Updated: 7 November 2026.

Kitor stores its game library, local installation paths, settings, patch history,
and related account state on the user's computer. Scanning and patching operate
on local game files. The application is not designed to upload game files or
local folder inventories to the project maintainer.

## Network activity

Kitor is not an offline-only application. On startup it checks GitHub for app
updates and refreshes the patch catalog. A connected GOG account may refresh
automatically. These requests are not all initiated by a separate click.
Remote cover images can also be requested when displaying the library/catalog.

User-requested features can contact GitHub and other catalog-listed patch hosts,
Steam, GOG and their download servers, and ScummVM download infrastructure.
Requests expose the user's IP address and normal HTTP metadata to the recipient;
URLs can identify the requested game, image, patch, or release.

Steam connection/library features use Steam authentication and, when configured,
a Steam Web API key. The API key is stored using the operating-system credential
store. GOG authentication and refresh tokens are stored in the local application
database and used to access the connected account. Do not share that database.
Steam-client downloads are also subject to the Steam client's behavior.

This policy does not promise that no information leaves the device. The reviewed application code contains no dedicated analytics or automatic
crash-reporting integration. Introducing one requires a fresh implementation
review and policy update.

## Controls and third parties

Store connections are optional. Use the application's disconnect controls to
remove the connection; do not initiate downloads or store connections you do not
want. The current installer does not provide a privacy screen or a switch to
disable all automatic network activity. This is an outstanding SignPath review
item, not a claim of compliance with its installation-consent requirements.

Third-party handling is governed by the respective providers, including
[GitHub](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement),
[Steam](https://store.steampowered.com/privacy_agreement/), and
[GOG](https://www.gog.com/en/policies/privacy).
Other patch/image hosts and externally opened sites have their own policies.

Removing the app may leave application data and patch backups. Restore installed
patches through Kitor before deleting their backups. Reports submitted to the
[public issue tracker](https://github.com/yhaggai/kitor/issues) are public;
remove credentials, private paths, and account details before posting.
Contact the maintainer through the [GitHub profile](https://github.com/yhaggai)
for privacy questions; do not post secrets in an issue.

This policy covers the desktop application, not third-party games or websites.
