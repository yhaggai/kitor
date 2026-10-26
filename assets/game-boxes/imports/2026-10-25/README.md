# Catalog cover import

Input: `/Users/haggaiyaniv/Downloads/json/all_games.json` (174 records): 22 already represented, 152 new entries. Added 149 English/Hebrew pairs to the existing 25-pair catalog, yielding 174 pairs. The new Hebrew variants comprise 135 AI title composites and 14 existing Hebrew retail scans.

## Preservation and review

English downloads remain unchanged. With explicit user authorization, translated title areas were composited onto the original images. `pixel-verification.json` records zero changed pixels outside the masks for all 135 composites. Background beneath replaced lettering is reconstructed, so preservation within those regions is not pixel-exact. Typography follows the original title treatment. Existing Hebrew scans retain their historical packaging and may differ from the English edition.

All composites were visually reviewed; corrected titles include Portal, Ankh, Duke Grabowski, and Reversion. The input omitted Hebrew names for Contradiction and Reversion; the proposed titles are סתירה and רוורסיה and are labeled in metadata.

## Files

- `queue.json`: all input records, provenance, source hashes, publication state, and exceptions.
- `sources/`: unchanged source downloads and retained candidates.
- `generated/`, `batch*.json`, `fix-batch.json`, `prompts.json`: generation drafts and provenance.
- `composited/`, `composite_titles.py`, `pixel-verification.json`: final Hebrew composites, masks, and preservation checks.
- `review.html`: searchable import gallery, including unresolved records.
- `prepare_publish.py`: constructs content-addressed canonical assets and upload staging. Running it resets publication states to ready-to-publish.
- `publish-manifest.json`: 149 new publication records; `manifest-before-import.json` retains the prior catalog.
- `upload-verification.json`: hosted image checksum and catalog verification.
- `cloudflare-upload.png`: final batch success confirmation. Three batches contained 100, 100, and 99 files, including the catalog.

All 348 hosted images passed SHA-256 verification, and the hosted manifest matches the local catalog. Temporary upload staging copies were removed after verification.

Canonical assets are in `../../covers/<game>/<en|he>/` and served from `https://assets.kitor.click/covers/`. App cover switching is a separate implementation step.

## Unresolved entries

- Boundless Osher: native Hebrew game; no English original box located.
- Loretta: image generation declined the edit; no Hebrew version published.
- Apprentice 1 Deluxe: developer source provides a gameplay screenshot, but no usable original box was located.
