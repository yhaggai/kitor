# Hebrew game banners

34 Hebrew-only banners cover every game with a patch in `kitor/patches/registry.json` as of 2026-11-07. Classic and remastered editions use separate artwork, including both Monkey Island games and Day of the Tentacle.

- Public gallery: https://assets.kitor.click/banners/index.html
- Public manifest: https://assets.kitor.click/banners/manifest.json
- Bucket: `kitor-assets`
- Keys: `banners/<registry-id>/he/<sha256-prefix>.png`
- Dimensions: 2172 × 724 (3:1), PNG
- App mapping: `kitor/src/data/game-banners.json`

`manifest.json` records the selected local generation, source artwork URLs, Hebrew titles, complete SHA-256 hashes, dimensions and public URLs. These are AI-localized adaptations, not official Hebrew publisher artwork. Steam/GOG wide artwork was preferred; classic box illustrations were adapted where a suitable edition-specific wide source was unavailable. Local `originals/` and `hebrew/` include sources and drafts; only selected Hebrew outputs are published. Artwork binaries follow the repository's existing assets ignore rule.

The app supports Hebrew-only banner records and hides its language selector when only Hebrew is present. All published records use this mode. Existing cover selection remains independent.

Uploads use the authenticated Wrangler CLI with `--remote`, Hebrew content language, PNG MIME type and immutable caching for content-addressed images. The manifest/gallery use a five-minute cache lifetime. `upload-result.json` lists uploaded keys; `verification.json` records public byte/hash verification.
