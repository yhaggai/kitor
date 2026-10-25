# Catalog cover import — in progress

Source: `/Users/haggaiyaniv/Downloads/json/all_games.json` (174 records).
22 entries match the existing bilingual catalog; 152 entries are new.

Current checkpoint: 116 downloaded English sources (including one title-free Simon image rejected for final use), 14 existing Hebrew retail scans, 15 AI draft edits. All 29 available pairs are shown in `review.html`. The original catalog and Cloudflare objects have not been changed by this import.

`queue.json` records the complete import, exact catalog titles, existing-game mapping, source URLs, dimensions, SHA-256 checksums, and review states. `prompts.json` contains the prompts used with built-in image_gen. `generated/` holds the full AI draft outputs; `sources/` holds unchanged downloads. `batch2.json` and `batch3.json` preserve generation provenance. Run `python3 build_review.py` to refresh the review.

## Outstanding work

- Resolve artwork preservation: full-cover AI edits change some non-text details. A question is pending about locally compositing only translated title regions onto the original artwork. Do not treat drafts as final text-only edits.
- Fix Portal with RTX: the decorative ring replaced the wrong Hebrew letter. Required title: פורטל.
- Source 36 missing English covers and replace the title-free Simon source.
- Inspect the downloaded English covers and their editions; currently source acquisition does not imply visual approval.
- Generate/localize the remaining Hebrew covers and verify spelling and typography.
- Resolve blank Hebrew catalog names for Contradiction and Reversion; check the grouped Dracula 4+5 and Reversion entries before assigning final assets.
- Organize final approved pairs into the canonical content-addressed `covers/` structure and upload to Cloudflare, then verify public hashes and image transforms.

Existing Hebrew retail scans deliberately preserve their historic titles and packaging; these may differ from the translation project's current Hebrew name or English regional packaging. Source candidates with English titles are not classified as completed Hebrew covers.
