# דיסקו אליסיום — Hebrew translation

## Current state

The canonical translation is **translations.csv**, UTF-8, with the shared Kitor columns `key,english,hebrew`. Edit this CSV only. Blank Hebrew means untranslated, never reviewed or complete. JSON files contain generated context, validation and provenance; `opening.json` is a historical seed, not an editable translation source. The current author translates directly in Codex without a separate draft model or external translation service.

The extracted scope is 79,586 source entries / 1,129,354 English words. Run `text_pipeline.py status` for current CSV coverage; `translation-progress.json` records conversation completion and the next entry to translate. Completion here means all text-bearing branches have Hebrew text, not that editorial or in-game review is finished. The Arabic slot retains untranslated Arabic until the corresponding CSV row is translated. It is not an English fallback build.

Arial is the selected font. The split Assistant/Frank Ruhl experiment failed in dialogue and is abandoned. Existing user screenshots and confirmation validate the earlier Arial prototype only; the expanded opening batch needs in-game review.

## Reproducible translation loop

Run from the repository root with Python 3 and UnityPy 1.25.0; font construction also requires Pillow, NumPy and SciPy. This session has SciPy under `/private/tmp/disco-he-python`.

1. `python3 patches-workspace/disco-elysium/he/text_pipeline.py extract` reads the owned installed macOS game and creates the local source index. On an existing CSV it verifies the complete source inventory and retains every translation. A different source build fails rather than silently migrating translations.
   `extract --expand-source` explicitly permits adding newly mapped keys while rejecting changes or removals to existing English. The 2026-11-10 expansion added 3,819 source-backed alternate dialogue and tooltip rows; existing Hebrew was preserved.
2. `python3 patches-workspace/disco-elysium/he/text_pipeline.py next --conversation 8 --limit 30` returns untranslated rows with speaker, addressee, branch links, conditions and scripts. Omit the conversation filter to get the next batch globally.
3. Translate directly into the CSV, preserving stable keys, source English, markup, placeholders and logical Hebrew reading order. Update `translation-provenance.json` and `glossary.json` for each reviewed batch. Luna (`gpt-6-luna`) drafts source-backed glossary entries; Astra (`gpt-6-astra`) translates game text directly into the CSV. Preserve separate authorship and mark glossary uncertainties. No automatic translator is used.
4. `python3 patches-workspace/disco-elysium/he/text_pipeline.py validate` checks source hashes, exact key inventory, duplicate IDs, protected tokens, emphasis and unexpected manual bidi controls. `status` reports exact completion counts.
5. `python3 patches-workspace/disco-elysium/he/text_pipeline.py build` writes a staged localization bundle under `build/`. The output is reopened and every translated key is checked against the CSV. It refuses to build inside the original source installation.
6. `python3 patches-workspace/disco-elysium/he/prototype.py` builds from that CSV and updates the existing disposable `/private/tmp/disco-he-test.app` with the Arial atlas. It adds missing characters required by the actual translated text. It is a local development installer, not the distributable Kitor lifecycle installer.
7. Quit/reopen the test app, select Arabic and test the scene, all reachable choices, progressive reveal, emphasis, wrapping, punctuation, saves and return to English. Record observations separately from asset checks.
8. `python3 -m unittest discover -s patches-workspace/disco-elysium/he -p 'test_*.py'` runs CSV contract regression tests.
9. `python3 patches-workspace/disco-elysium/he/text_pipeline.py release-check` deliberately fails while translation, mapping and onboarding gates remain incomplete. No release-ready flag is inferred from successful local compilation.

## Source audit and remaining onboarding

The source index currently records 193 source keys absent from the shipped Arabic tables and 127 localization terms lacking a matched English source, including malformed Arabic keys. These must be reconciled against the English source tables before full coverage is certified. Repeated English strings remain separate stable-ID rows. Non-text branch nodes stay in the game database untouched.

Steam app 632470, Final Cut. The owned macOS build is 23980936. Windows depot 632471 / manifest 5267467363497400856 completed and its executable and actual bundles were verified. Both platform file plans passed selective install/uninstall on disposable subsets on macOS; this is not a Windows-native gameplay test.

Catalog record 177 (`patches-metadata/177_disco_elysium.json`) and registry ID `disco-elysium` are integrated. Detection verifies the title-specific `app.info` plus executable, game manager and both required Arabic bundles. It accepts a Steam container, macOS app, or inner data folder and stores the actual install root. Automatic and manual detection share this adapter. A temporary database scan on the owned macOS installation passed, including repeated scanning without duplicates.

English and Hebrew Final Cut covers and banners are hosted in the production `kitor-assets` R2 bucket. Original hashes, byte sizes, six cover transformations and the hosted cover manifest were verified. See `assets/game-boxes/disco-elysium-provenance.json` and `disco-upload-verification.json`. The registry intentionally has no public patch download while release gates remain open.

## Private patch build and installation

`build_package.py` reads the canonical CSV and owned Windows/macOS sources, builds logical Hebrew text and the selected Arial glyph atlas, reopens the bundles, and emits BSDIFF deltas. Original game files are never changed by the builder. Build dependencies: UnityPy 1.25.0, bsdiff4 1.4.2, Pillow, NumPy and SciPy. Supply an installed Arial font with `--font`; no TTF is distributed.

```sh
python3 build_package.py --mac "/path/to/disco.app" --windows "/path/to/Windows installation" --font "/path/to/Arial.ttf"
```

Run packaging from the repository root:

```sh
python3 kitor/tools/build_patch_packages.py --check patches-workspace/disco-elysium/he/patch.preview.json
python3 kitor/tools/build_patch_packages.py --output dist/patches patches-workspace/disco-elysium/he/patch.preview.json
python3 kitor/tools/build_patch_packages.py --universal --output dist/patches patches-workspace/disco-elysium/he/patch.preview.json
cargo build --manifest-path kitor/tools/lifecycle-harness/Cargo.toml --locked --bin disco_patch
```

Packaging checks `package-build.json` against both the CSV and compiler hashes, refusing stale deltas after edits. It does not regenerate from owned data automatically: rerun the builder first. Preview payloads are local ignored files. Clean-checkout public packaging remains blocked until font redistribution is resolved and approved reproducible inputs exist. `patch.preview.json` is deliberately excluded from automatic release enumeration. No public patch URL is invented.

Extract the universal ZIP into `package/`, quit the game, then use the native CLI:

```sh
disco_patch install package "/path/to/game" "/path/to/recovery"
disco_patch uninstall "/path/to/game" "/path/to/printed/journal"
```

A ready local macOS handoff is in `dist/disco-elysium-preview/`, with a native installer, extracted package and instructions. Windows/macOS data ZIPs are in `dist/patches/`. Windows uses the same Rust installer source, but a Windows executable has not been compiled locally. Python 3 is needed by the existing BSDIFF lifecycle (`py -3` on Windows). Each platform modifies exactly two files, with original and installed SHA-256 guards. The common lifecycle performs staging, selective backup and restoration; it does not touch saves.

The Arial-derived atlas is a private prototype, not cleared for public redistribution. The reference guidance is https://learn.microsoft.com/en-us/typography/fonts/font-faq ; do not treat a converted bitmap atlas as freely redistributable. Public release, native Windows execution, expanded-scene renderer/save-load tests, full translation and exact-commit GitHub Actions remain pending. `release_ready` remains false. The workflow now compiles the preview CLI on both native runners when submitted; no run is claimed here.

Owned-game source indexes, reconstructed bundles, rollback copies and generated private deltas are excluded from Git. The CSV includes English for authoring context, but it is not a runtime payload.

## Restoration and history

Steam files remain untouched. `font-rollback/` contains the earlier working two-bundle Arial prototype. Restoring its `fonts/arabic` and `lockits/arabic` into the test copy rolls back the complete prototype, including its earlier eight-line translation. It is not a source-game backup. Production install/uninstall must use Kitor's journaled selective lifecycle with original/output hash guards; never package a whole-game backup or full proprietary game bundle.

## Testing through the updated desktop app

The debug-only `KITOR_DISCO_PREVIEW` environment variable points to the private
universal ZIP. It injects a local preview option for Disco after catalog loading,
copies that archive into normal staging, checks its checksum, and uses the same
journaled install/uninstall and database recording as downloaded patches. Other
local URLs remain unsupported; release builds do not compile this option.

`dist/disco-elysium-preview/Open-Kitor-Preview.command` opens the locally built
**Kitor Disco Preview.app** with this option enabled. The preview build uses
`click.kitor.disco-preview` as its application identifier, keeping its library
and recovery state separate from the installed Kitor app. Scan for games, choose
Disco Elysium and install the private Hebrew preview. Select Arabic in-game.
The public registry still has no published download.

## Compact translation glossary context

`glossary.json` is the canonical glossary shipped with this patch source, with attribution in `glossary-draft-provenance.json`. Keep the full glossary here for editorial review and reproducibility; it is not a runtime game payload. `glossary_context.py BATCH.source.json --output BATCH.glossary.json` generates a compact context using core terminology, exact English names/aliases, and speaker/addressee IDs. It includes relevant voice/grammar guidance and retains localized aliases/forms. Workers use this selected context, consulting full entries only for ambiguities. Generated batch files are temporary working inputs, not a second glossary source of truth. Reported savings are character reductions, not measured token billing or cache savings.
