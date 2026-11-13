# SignPath application draft for Kitor

Updated 12 November 2026. The draft is ready for maintainer review. Resolve the submission items below before making eligibility or compliance claims.

## Application answers

- **Project and maintainer** — Kitor, maintained by Haggai Yaniv ([yhaggai](https://github.com/yhaggai)).
- **Source and project homepage** — [Public GitHub repository](https://github.com/yhaggai/kitor), default branch `v1`.
- **Description** — Kitor is a desktop launcher and Hebrew translation manager for adventure games users already own. It detects supported installations, downloads compatible community translations, backs up files before patching, and restores those files when a patch is removed. Optional integrations support Steam, GOG, and translation contributions.
- **License** — The maintainer's original code is MIT licensed. Third-party components and assets retain their own terms. See the [licensing scope](https://github.com/yhaggai/kitor/blob/v1/LICENSING.md). The Windows release inventory is still under review.
- **Release and requested signing scope** — [Kitor 0.1.3](https://github.com/yhaggai/kitor/releases/tag/app-v0.1.3) includes a Windows x64 NSIS installer. Request Authenticode signing for Kitor's own `kitor.exe` and its x64 NSIS setup executable. Community patches and independently downloaded runtimes are part of Kitor's functionality and should be disclosed.
- **Build provenance** — The [release workflow](https://github.com/yhaggai/kitor/blob/v1/.github/workflows/release-app.yml) checks out the release source, installs the locked Bun dependencies, and builds the Rust/Tauri app and NSIS installer in GitHub Actions. Proposed signing approval is manual for every release.
- **Desktop GitHub access** — Kitor fetches the public catalog and downloads public patches without a GitHub token.
- **Privacy wording for the next verified Windows release** — Steam and GOG account linking and Google sign-in for contributions are optional and user initiated. Before Steam, GOG or Google sign-in, Kitor explains what the connection is used for and who receives the data. Kitor refreshes Steam and GOG libraries on startup after the user connects the corresponding account. The connection screens disclose this behavior, and users can disconnect either account in Settings. Kitor reads a saved contributor profile locally at startup. Interactive Windows setup describes network activity and links to the privacy policy.
- **Public network requests** — Kitor requests the public app update manifest from GitHub Releases (`github.com`), the public translation catalog from GitHub (`raw.githubusercontent.com`), and game covers and banners served by Cloudflare at `assets.kitor.click`. These requests use no account credentials. GitHub and Cloudflare receive standard connection metadata, including the connecting IP address, and the requested URL.
- **Policies and roles** — [Code signing policy](https://github.com/yhaggai/kitor/blob/v1/CODE_SIGNING_POLICY.md) and [desktop privacy policy](https://github.com/yhaggai/kitor/blob/v1/PRIVACY.md). Haggai Yaniv is the listed maintainer and reviewer and the proposed signing approver.
- **Release history and adoption** — GitHub listed three published Windows installers, for 0.1.1, 0.1.2, and 0.1.3. Before the audit download on 12 November 2026, their asset counters were 2, 1, and 1 respectively. These are download events. Add independently verifiable evidence of adoption or community use if available.
- **Contact email** — The maintainer supplies the preferred address in the form.

## Before submitting

See the [Windows release audit](signpath-windows-audit.md) for the installer inventory and the source revisions reviewed.

1. **Document the classic Tentacle permission.** The maintainer confirmed permission to distribute the Kitor recipe package. The [updated recipe archive](https://github.com/yhaggai/kitor/releases/tag/patch-tentacle-he-v2.0-beta2-kitor1-download2) carries the corrected notice. Retain any record of the permission and check the other published patch packages for documented permissions.
2. **Complete the Windows release inventory.** A listing of the published 0.1.3 installer showed `kitor.exe`, six NSIS/Tauri plug-in DLLs, and two wizard bitmaps. Finish the inventory of compiled Rust dependencies, embedded frontend dependencies and assets, fonts or artwork if present, installer resources and plug-ins, and the WebView2 install path. Record each item's origin, version, license and applicable notices. Check both open-source eligibility and redistribution rights. Obtain SignPath's guidance on any ambiguous component and replace or remove items that cannot meet the requirements.
   - [PR #42](https://github.com/yhaggai/kitor/pull/42) removes bundled Steam and GOG logos, replaces the desktop provider marks with generic icons, and moves Disco Elysium artwork outside the desktop public directory. The frontend build was checked for the removed assets. Build and inspect a fresh Windows installer from v1 before making claims about the distributed artifact.
   - The Windows Rust dependency tree includes `webpki-roots` through `reqwest` and `sqlx`. Record the bundled certificate data and its terms in the inventory. Ask SignPath whether it affects eligibility before changing the Windows trust configuration.
3. **Verify the Windows privacy changes.** Test English and Hebrew interactive setup, a passive upgrade and uninstall. Confirm that Steam and GOG refresh only for connected accounts. Verify that the saved contributor profile loads locally at startup, and test manual refresh and disconnect. Startup app and catalog checks request public resources. Remote artwork requests may disclose ordinary connection metadata and the requested item. Confirm how SignPath applies its installation-time controls rule to these requests before making a compliance claim.
4. **Confirm the account and project details.** Confirm the contact email, signing approver, and MFA for all team members on GitHub and SignPath. Check that the public homepage and release/download pages expose the code signing policy, and add genuine public usage or community references if available. SignPath reviews project reputation for executable apps.
5. **Submit the reviewed answers** through the [SignPath application](https://signpath.org/apply). Update the provider attribution when SignPath authorizes the project to use it.

## After acceptance

Configure the SignPath project and GitHub build provenance, require manual approval for each signing request, and enforce Kitor product name and version metadata. Sign the app before bundling it. Sign and timestamp the final Windows installer before Tauri generates its updater signature. Verify both Authenticode signatures, fresh installation and launch, uninstall, and updating from an older release.

[SignPath Foundation requirements](https://signpath.org/terms.html)
