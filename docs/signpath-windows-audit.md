# Kitor Windows release audit for SignPath

Evidence from the published Windows x64 installer and source tag `app-v0.1.3`, reviewed 12 November 2026 (Israel time). A new release needs an inventory from its own commit.

## Licensing and installer contents

| Component | Evidence and remaining work |
| --- | --- |
| NSIS wrapper | Listing the [published installer](https://github.com/yhaggai/kitor/releases/tag/app-v0.1.3) showed `kitor.exe`, two bitmaps and six DLLs: `System.dll`, `nsDialogs.dll`, `nsis_tauri_utils.dll`, `StartMenu.dll`, `LangDLL.dll` and `NSISdl.dll`. The locked [Tauri CLI 2.9.6 source](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.9.6/crates/tauri-bundler/src/bundle/windows/nsis/mod.rs) pins NSIS 3.11 and Tauri NSIS utilities 0.5.2. The extracted Tauri DLL matches the pinned hash. [NSIS](https://nsis.sourceforge.io/License) documents zlib/libpng for its standard code and plug-ins and CPL 1.0 for LZMA, which the installer uses. The [Tauri plug-in](https://github.com/tauri-apps/nsis-tauri-utils/blob/nsis_tauri_utils-v0.5.2/Cargo.toml) declares MIT or Apache-2.0. Collect the applicable notices for the Windows release. |
| Rust code | Locked Cargo metadata filtered for `x86_64-pc-windows-msvc` resolved 490 packages, including build dependencies. Every resolved package declared a license. This is a dependency inventory, and the Windows build still needs to establish what it links or redistributes. The two `webpki-roots` versions (0.26.11 and 1.0.5) declare [CDLA-Permissive-2.0](https://spdx.org/licenses/CDLA-Permissive-2.0.html) for trust-root data. Ask SignPath how it treats that data under its OSI requirement and preserve the applicable license text. |
| Frontend code | Rebuilt the tagged frontend using the frozen Bun lockfile. Rollup reported emitted code from 55 third-party packages. Declared licenses were MIT (49), MIT or Apache-2.0 (3), Apache-2.0 (1), ISC (1) and 0BSD (1). Collect notices for the shipped build. |
| Frontend assets | The Vite production output had 40 files, including the Monkey Island cover scans, Jenny LeClue artwork, and Steam, GOG and ScummVM marks. The Monkey Island provenance records a source URL and retrieval details. It does not record a redistribution grant. Locate permission and confirm eligibility for bundled assets with SignPath, or replace them with assets with documented terms. Inspect the production output rather than inferring from UI usage, because Vite copies files from `public/` into the bundle. |
| WebView2 | The release config uses Tauri's default [WebView2 bootstrapper download](https://v2.tauri.app/distribute/windows-installer/#webview2-installation-options) if the runtime is missing. The published file listing showed no embedded WebView2 installer. Disclose the conditional Microsoft download and verify the install and uninstall experience in a Windows VM. |

## Privacy and network behavior

| Trigger | Destination and data observed in source |
| --- | --- |
| App launch | Tauri checks the GitHub updater manifest. Kitor also fetches the GitHub patch catalog. These are manifest/catalog requests that expose connection and HTTP metadata. The reviewed calls do not send the local game inventory. |
| App launch with a connected GOG account | Kitor requests the owned-game list from GOG with the access token and can send a refresh token to GOG's authentication service when needed. The app first checks whether an account is connected. Its disconnect control stops future authenticated refreshes. |
| Rendering catalog and library views | Remote artwork URLs can identify the game or image requested to the host and expose connection metadata. |
| Optional store and contribution actions | The user initiates store authentication, downloads, and contributions. The current `v1` includes additional native Steam and contribution flows beyond 0.1.3. Review and describe their data and recipients before signing a release from that branch. |

The tagged NSIS configuration installs per user and does not define a privacy page or installation-time controls for these requests. The [SignPath privacy condition](https://signpath.org/terms.html) applies when software collects user data and transfers it to a destination the user did not specify. Ask SignPath how it applies to request metadata for automatic GitHub and artwork GETs, and to background authentication and library refresh after the user connects GOG. If SignPath says a flow falls under the condition, disclose it in the policy and installer and make its disable option effective before the first automatic request. Test fresh installation and uninstall on Windows with and without WebView2.

Application wording should disclose these flows and the unresolved asset and data-license questions. The above checks establish source and package evidence. Windows runtime behavior and eligibility still need verification.

## Changes in the source for the next release

The branch that includes the newer Steam and GOG flows resolves 502 Windows-target Cargo packages under the same metadata command. Relative to 0.1.3, this adds 12 dependency identities plus the updated Kitor version and drops the old Kitor version. The additions declare MIT, Apache-2.0, ISC or Unlicense alternatives. The newer frontend build emits code from 58 third-party packages, adding AG Grid Community, AG Grid React and AG Stack 36.3.0, all declared MIT. The 0.1.3 installer remains the physical artifact reviewed here.

The newer contribution editor stores drafts locally. On explicit submission, it sends the edits, source commit, content digest and email-notification preference to the configured contribution service using a session token. The flow uses Google sign-in. Update the desktop privacy policy to name those recipients, describe the account and submission data, and verify the service’s retention and publication behavior before releasing the new installer.

## Questions for SignPath

> Kitor checks GitHub for update and catalog manifests on launch and loads remote artwork in relevant views. Those requests expose ordinary connection metadata, and artwork requests identify the requested image. If a user connects GOG, Kitor refreshes that account's library in the background using GOG tokens. Does your installation disclosure and disable-option rule apply to any of these flows?
>
> The Windows application resolves the upstream webpki-roots package, which declares CDLA-Permissive-2.0 for its trust-root data. How do you treat this data under the open-source license requirement? Does the same requirement cover bundled artwork and store marks where the project has redistribution permission under separate terms?


## Privacy review after PR #38

The Windows 0.1.4 preview's NSIS archive was unpacked and its executable checked
against the 15 removed covers and banners. The desktop sources match
[PR #38](https://github.com/yhaggai/kitor/pull/38), merged to v1 as
`9b501554234a2b5dfe66efba5cb13ce0ac71abbc`.
[Windows preview build](https://github.com/yhaggai/kitor/actions/runs/43309936646)

| Trigger | Data and recipient | Existing control |
| --- | --- | --- |
| Fresh installation when WebView2 is missing | The NSIS template downloads and executes the Microsoft bootstrapper. | The template shows download and installation status. It does not expose a dedicated privacy choice for this dependency. |
| App startup | The updater fetches the GitHub release manifest. The catalog fetches from GitHub's API and can attach a previously saved GitHub personal access token. | No installation-time switch. A fresh installation without saved credentials uses an anonymous catalog request. |
| Startup with a connected Steam or GOG account | Native Steam signs in with stored credentials and requests owned games using the Steam account ID. GOG sends the access token and may refresh it using the refresh token. | The user first connects the account and can disconnect it later. No separate installation-time switch for background account refresh. |
| Rendering library or catalog views | Image hosts see connection metadata and the image URL. | No installation-time switch. The title placeholder allows the UI to render without artwork. |
| Optional translation contributions | Google authenticates the user. Kitor's service stores the verified email, Google account identifier and display name. An explicit submission sends edits and notification preferences to the service, which uses GitHub for the pull request. | Sign-in and submission require user action. An existing contribution session can be checked when the user opens the editor. |

The privacy policy draft now describes these flows. The review used source, the
matching NSIS template and an unpacked installer. It did not capture live traffic
or execute installation and uninstall on Windows.

Proposed application disclosure:

> Kitor's per-user Windows installer downloads Microsoft's WebView2 bootstrapper
> when the runtime is missing. At app startup, Kitor checks GitHub for updates and
> fetches the public translation catalog. If the user connected Steam or GOG,
> the app refreshes that account's game library in the background. Catalog views
> can load remote artwork. The current installer has no opt-out controls for
> those app requests. Store and contribution account connections are optional.
> Does SignPath require installation-time disclosure and controls for the
> automatic requests described here, including those to providers the user
> previously connected?

If SignPath requires controls, present the privacy disclosure in NSIS and persist
the user's choices before the first launch. Test the choices on fresh installs
and upgrades. In a clean Windows VM, capture installer and first-launch traffic
with WebView2 present and absent, repeat with connected store accounts, and check
uninstall. Keep account tokens out of the test logs.

### Public-only GitHub patch access

The branch `codex/remove-github-patch-token` removes the desktop GitHub token
commands and uses anonymous requests for the public catalog and release ZIPs.
The credential entry above describes the 0.1.4 preview before this change.
Use the proposed disclosure for an installer built with the removal. The public
catalog and a representative release ZIP were fetched without credentials, and
the ZIP matched the bundled catalog checksum.
