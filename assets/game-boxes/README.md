# Game box assets

Artwork binaries under `assets/` are an ignored local workspace. Git tracks the manifests, provenance, review pages, and preparation scripts; it does not track originals or generation drafts. Keep local copies or retrieve published covers from the URLs in the manifest. Optimized runtime assets under `kitor/public/` and `website/public/` remain tracked so fresh checkouts work.

`covers/manifest.json` is the bilingual catalog for all 174 games. Each game has `covers.en` and `covers.he`, including source provenance, dimensions, SHA-256, Cloudflare object key, original URL, and 240/480/960px transformation URLs.

Published paths follow `covers/<game-id>/<en|he>/<sha256-prefix>.<extension>` in the `kitor-assets` R2 bucket, served from https://assets.kitor.click. Filenames are content-addressed; new revisions should use new hashes.

Selection for the forthcoming UI feature: English when the Hebrew patch is absent, Hebrew when installed. This asset preparation does not change application selection logic.

## Local layout

- `covers/`: organized publication tree, including both languages and its manifest. Upload this folder at the bucket root to preserve object keys.
- `originals/` and `manifest.json`: original 25 collected covers and their provenance, retained unchanged except for a pointer to the bilingual catalog.
- `hebrew/`: 19 AI-localized cover drafts, generation specifications, and their review page. These are the versions approved for upload; AI editing can alter non-text artwork.
- `english/`: six additional English covers and source records. Earlier low-resolution candidates are retained but are not published.
- `review-pairs.html`: side-by-side review of all 174 English/Hebrew pairs, with links to hosted originals.

The six existing Hebrew/Israeli scans remain the Hebrew variants for Kyrandia, Space Quest III/V, Torin’s Passage, Gobliins 2 and The Longest Journey. The English Space Quest III scan is the Amiga packaging variant; its platform sticker differs from PC packaging. Kyrandia’s English scan is floppy-era packaging. The Longest Journey English variant uses the user-supplied retail box scan.

Existing legacy Cloudflare object keys are retained so current app cover URLs continue working. `upload-verification.json` records public URL and checksum verification after publication.

## JSON import (2026-10-25)

The 174-entry input contained 22 existing games and 152 new entries. Added 149 pairs: 135 AI-localized titles composited onto unchanged English originals and 14 existing Hebrew retail scans. Original pixels outside the recorded title masks are identical; reconstructed title regions can include background pixels. Font appearance follows the original lettering.

`imports/2026-10-25/` retains source files, generation drafts, masks, pixel verification, provenance, and a review page with unresolved entries. Its `upload-verification.json` records verification of the expanded catalog.

Three entries remain unresolved: Boundless Osher (native Hebrew game; English original unavailable), Loretta (image generation declined), and Apprentice 1 Deluxe (no original box found).
