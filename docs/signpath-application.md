# SignPath application preparation

Prepared 7 November 2026. Not submitted; approval is not guaranteed.

## Application draft

- Project: Kitor
- Maintainer: Haggai Yaniv / GitHub `yhaggai`
- Repository: https://github.com/yhaggai/kitor
- Description: An open-source desktop launcher and Hebrew translation manager
  for users' own games. It detects installations, downloads compatible community
  patches, backs up affected files, restores managed patches, and launches games.
- Code license: MIT for maintainer-owned code; third-party exclusions are
  documented in `LICENSING.md`. A complete component audit is pending.
- Requested signing: Kitor Windows x64 executable and NSIS installer.
- Existing Windows release: https://github.com/yhaggai/kitor/releases/tag/app-v0.1.1
- Build: GitHub Actions, `.github/workflows/release-app.yml`, branch `v1`;
  Bun frontend build and Rust/Tauri build with locked dependencies.
- Policy: https://github.com/yhaggai/kitor/blob/v1/CODE_SIGNING_POLICY.md
- Privacy: https://github.com/yhaggai/kitor/blob/v1/PRIVACY.md
- Signing approver: proposed maintainer `yhaggai`; MFA confirmation pending.
- Contact email: maintainer must supply directly in the application.

The policy links become usable after these changes are merged to `v1`.
Disclose community patch downloads, game artwork, store authentication,
automatic network requests, and any separately downloaded runtime tools.
Do not represent these as independently approved or covered by the MIT license.

## Before submitting

- [ ] Merge/publish licensing and policy documents, and verify their public links.
- [ ] Confirm copyright ownership of code covered by MIT and retain upstream notices.
- [ ] Inventory the actual Windows installer: compiled dependencies, embedded
  frontend assets, fonts, images, scripts, and any bundled runtime components.
  Record source, version, license, and redistribution evidence for each.
  Resolve missing permissions or remove/replace affected content.
- [ ] Review patch provenance and restrictions separately. In particular the
  classic Tentacle registry entry says personal use only / do not distribute;
  resolve that restriction against actual published packages before claiming
  application readiness. Credits alone are not a redistribution license.
- [ ] Review automatic update/catalog, GOG refresh, and remote image requests
  against SignPath's privacy requirements. Implement installation disclosure and
  disable controls if required; a Markdown policy alone does not satisfy them.
- [ ] Confirm MFA for every repository/signing team member and role assignments.
- [ ] Provide truthful project history and usage/reputation evidence. Do not
  invent download counts or claim that public-source availability guarantees approval.
- [ ] Submit using https://signpath.org/apply after outstanding items are resolved.

## After acceptance

Configure the approved SignPath project, artifact configuration and signing
policy, GitHub build provenance integration, and manual approval. Restrict product
name/version metadata. Sign only the approved Kitor artifacts. Integrate signing
before Tauri generates the final installer's updater signature; then test
Authenticode verification, clean installation, uninstall, and an update from an
older release. Verify all platforms before publication.

Activate the provider attribution in the code signing policy only when true.
Include a Code signing policy link in release notes and the project homepage.
Do not rotate or replace the independent Tauri updater key for this integration.

## Sources

- https://signpath.org/terms.html
- https://signpath.org/apply

This is a readiness record, not a certification of compliance or legal ownership.
