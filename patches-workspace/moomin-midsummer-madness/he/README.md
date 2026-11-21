# Moomin: Midsummer Madness — Hebrew localization

## Font spacing update 0.1.3, revision 4

Kitor downloads this patch as `0.1.3-download1` (catalog revision 5); no game
patch payload is bundled in the app. The ZIP retains the same five-file plan
and uses the standard checksum-verified download and selective journaled
installation path.

All 3,866 dialogue CSV speaker labels now resolve from the pristine
GameDatabase's per-phrase `character_card`, correcting 3,492 scene-inferred
labels without editing translations. `speaker-context.json` retains character
IDs, names, emotions and events for subsequent agreement review. Recreate it
with `merge_speakers.py <pristine-database-bundle>`. The references establish
who speaks; they do not automatically establish each addressee or grammatical
number.

Hebrew base letters and composite pointed glyphs now reserve at least two atlas
pixels of ink clearance on each side. Composite advances expand when needed,
rather than retaining an original-font advance that is narrower than the Arial
glyph. Lower vowel marks move down two atlas pixels in regular faces and three
in bold; dagesh and shin/sin dots retain HarfBuzz anchor positions. Letter size
and vertical baseline are unchanged. `dialogue-font-advances.json` is regenerated
from the widest active face, including the composite PUA glyphs, and dialogue
wrapping uses the composite advance for each pointed cluster.

`niqqud-font-preview.png` compares English, previous Hebrew and updated Hebrew
for all four active faces at the same rendering scale. It is an offline atlas
preview, without TMP material effects. `verify_niqqud_fonts.py` reopens both font
copies and checks unique indices, coverage, atlas bounds and ink margins;
`font-spacing-report.json` records the results. No running game is controlled.

Shmantaf readings are pinned as שְׁמַנְטַף / שְׁמַנְטַפִים. The plural is attested
in [Uriel Ofek's literary discussion](https://benyehuda.org/read/21434); the
singular follows the user's pronunciation. The latest user-requested spelling
for Misabel is אומללית / אוּמְלָלִית, superseding the earlier אומללת revision.
`reviewed_terms.py` applies those readings after draft generation and when
reading the canonical CSV for font and database builds. The CSV and pointed
reference catalog contain the corrections themselves.

The update targets the same Windows Steam build, with the existing selective
five-file installation and restoration plan. macOS is not supported by this
game patch. Visual validation is offline; live gameplay has not been checked.

This folder contains the editable logical-order Hebrew catalog, its source inventory, character glossary, extraction and build scripts, and model attribution.

## Scope

The catalog covers all 4,692 nonempty player-facing `text` fields from the 1,931-card `GameDatabase`: dialogue, chapter recaps, journals, UI, inventory, credits and demo text. Each entry has a stable recursive JSON path and exact English source. The builder writes Hebrew into the rendered `text` field and its `data.en.translation` slot where present. Names of real people and companies are retained in credits; role labels are translated.

## Rebuild

`catalog.csv` is the editable source of truth for the patch. It contains one UTF-8-BOM row per stable text path, category, scene, speaker when the scene key identifies one, English source, unpointed Hebrew and pointed Hebrew. `catalog.json` is the preserved unpointed catalog; `catalog-niqqud.json` is an intermediate model draft. The text and font builders read the CSV. Edit both Hebrew columns together when changing wording; the JSON drafts are retained for reference.

Use Python 3.11 with UnityPy 1.25.4 and python-bidi to build the database from the pristine bundle. The builder checks unique IDs, full path coverage, placeholders and tags, and exact base wording after removing niqqud:

```powershell
$env:PYTHONPATH = '.tmp/silver-py'
py -3.11 patches-workspace/moomin-midsummer-madness/he/extract.py `
  '.tmp/moomin-release-26803369/original/Summer/Adventure_UE5_Data/StreamingAssets/aa/StandaloneWindows64/database_assets_all_451c9c2dc5087ceed02f737272b90df7.bundle' `
  'patches-workspace/moomin-midsummer-madness/he/source.json'
py -3.11 patches-workspace/moomin-midsummer-madness/he/build.py `
  '.tmp/moomin-release-26803369/original/Summer/Adventure_UE5_Data/StreamingAssets/aa/StandaloneWindows64/database_assets_all_451c9c2dc5087ceed02f737272b90df7.bundle' `
  'patches-workspace/moomin-midsummer-madness/he/catalog.csv' `
  '.tmp/moomin-render-test/moomin-he-database.bundle' `
  --report 'patches-workspace/moomin-midsummer-madness/he/build-report.json'
```

Pointed text stays in logical reading order in the CSV. `build.py` wraps paragraphs and treats each Hebrew letter-plus-niqqud cluster as one unit during bidi conversion, then substitutes its composite PUA glyph. That keeps marks attached to the right letter in the game's left-to-right renderer. `build_niqqud_fonts.py` extends the four active TMP SDF font atlases using HarfBuzz mark anchors and FreeType glyph rendering. Both the fonts bundle and `sharedassets0.assets` must be patched. A raster preview made from the rebuilt atlases is `niqqud-font-preview.png`.

Dialogue wrapping uses advances extracted from `Pangolin-Regular SDF MAIN` in `dialogue-font-advances.json`, rather than character counts. The 550-unit line width is calibrated to the observed bubble break for “אחותי מחפשת אותי בכל מקום.” (420.9 and 439.9 units fit; extending the first line by the next word is 565.8). Dialogue breaks are emitted as TMP `<br>` tags because the game normalizes literal newlines before text layout. The builder asserts source indices stay aligned while mapping each wrapped visual line, preventing Hebrew base letters and niqqud clusters from shifting across whitespace.

## Kitor package

`patch.windows.candidate.json` and `deltas/` provide the installable Windows patch for Steam app 4463520, depot 4463521, build 26803369 (2026-11-20), patch 0.1.2 revision 3. It contains five verified bsdiff files (database, TMP font bundle, `sharedassets0.assets`, `catalog.bin`, and the managed save-filename validator); it does not include raw game bundles. The save validator recognizes the private-use Hebrew glyphs in translated chapter names, preventing valid Hebrew-named save files from being discarded during save discovery. `build_moomin_package.py` recreates the delta manifest from a pristine and a patched game tree; run it with `--original-root`, `--patched-root`, and (after rebuilding text) `--database-output`. It needs Python's `bsdiff4` and `UnityPy` packages. It recalculates both database and font CRCs in the pristine Addressables catalog before making the catalog delta. Each nonzero original CRC must occur exactly once; unknown or ambiguous catalogs fail closed. The official Steam header art at `kitor/public/covers/moomin-midsummer-madness/header.jpg` is composed into portrait English and Hebrew GameBox covers.

Dialogue line wrapping is validated against the reported screenshot phrase and each source character is checked against its original grapheme/style index. The computer screenshot channel timed out on the running game window after relaunch, so the corrected line order has not been visually captured in-game. The 550-unit calibration comes from one observed bubble width; other resolutions and unusual long unbreakable tokens still need visual review.

## Translation record

The unpointed translation was prepared with OpenAI Codex CLI, `gpt-6-luna`, low reasoning effort, in isolated read-only batches. Niqqud was added locally with Nakdimon 0.2.1's cached ONNX model. The automatic vowel pass is not a manual line-by-line Hebrew proofread. `niqqud-glossary.json` and `apply_niqqud_glossary.py` pin reviewed pronunciations for recurring Moomin names and terms; the base-letter spelling stays identical. Name spellings and their sources are recorded in `character-glossary.md`.

To rebuild font atlases, install `UnityPy`, `numpy`, `scipy`, `Pillow`, `uharfbuzz` and `freetype-py`, then run `build_niqqud_fonts.py` against the existing Hebrew-capable font bundle and sharedassets backups. The game loads font copies from both files. Installation is reversible; retain the current installed bundle and catalog backups. The complete pointed database is reopened and validated by UnityPy; startup-log checks are recorded with the build artifacts. A full playthrough of all 4,692 strings has not been observed.

## Local repair for Steam build 26803369 (2026-11-21)

Steam updated the database bundle, catalog and managed assembly while leaving the
Hebrew fonts and sharedassets installed. The new catalog expected the original
font CRC (`18adf005`) and rejected the translated font bundle (`eab3a5eb`),
preventing bootstrap completion.

The local repair preserves the new game code and data. The new 4,692-field
database was rebuilt using the existing CSV, with one staging-only source update:
the credit `Pavel Arkhipov` became `Pavel Vophira`, retained verbatim in both
Hebrew columns. The new catalog's database CRC changed from `fd17f90c` to
`9f7587a8`, and the font CRC changed to `eab3a5eb`. CRCs are calculated over the
concatenated decompressed bundle members, using each SerializedFile's original
reader bytes (not a reserialization). Each old CRC occurred exactly once.
`catalog.hash` was not changed. The two save filename regexes were reapplied to
the **new** Assembly-CSharp.dll using `repair-save-validator.cs`; the old DLL was
not restored. This preserves recognition of existing PUA Hebrew chapter names.

`repair-26803369.json` records the exact before/after hashes. `repair-install.py`
installs a staged `repair-plan.json` or restores just its three affected files,
with preflight hashes, selective verified backups, conflict checks and rollback.
The live recovery manifest and original bytes are in
`C:/SteamLibrary/steamapps/common/Summer/.kitor-moomin-repair-26803369/`.
The staging directory is `.tmp/moomin-repair-26803369/`; it contains the adapted
CSV, rebuilt data, build report, startup log and a pre-launch save snapshot.

Validation: reopened all 4,692 translated fields; selective install/uninstall on
a temporary game layout restored all three original hashes and preserved both
an untouched file and a file added after install. The live game passed bootstrap,
loaded 1,931 cards, initialized settings and remained responsive. No CRC mismatch
or bootstrap exception appeared. All 65 pre-existing save files remained present
with identical SHA-256 hashes after launch. Window capture timed out twice, so
menu rendering and loading a save were not visually tested. The direct EXE test
used the dummy achievements manager because SteamAPI did not initialize.

The local recovery above was subsequently incorporated into candidate 0.1.2 revision 3.
The previous 0.1.1 package targets build 26355427 and must not be applied to the
new build. macOS was not built or tested; the registry continues to advertise
Windows only. No public release or installed desktop binary update is implied.


## Candidate 0.1.2, revision 3

Compatibility and original/installed fingerprints are recorded in `compatibility.json`.
The registry, Steam download required files, original/patched launch requirements
and embedded asset map now reference the current database bundle. The Steam cache
key includes 26803369 so an older cached download cannot satisfy the new request.
The editable contributor CSV and preserved source catalogs retain the new credit
name Pavel Vophira verbatim. All 4,692 text fields were rebuilt and reopened; the
result matches the locally boot-tested database byte-for-byte.

Rebuild using Python 3.11 with bsdiff4, UnityPy and python-bidi, and .NET 9 for the
save validator. Run the save validator against the current pristine managed DLL,
never against the old version or a previously patched DLL:

```powershell
dotnet run --project patches-workspace/moomin-midsummer-madness/he/repair-save-validator.csproj -- '<pristine>/Summer/Adventure_UE5_Data/Managed/Assembly-CSharp.dll' '<patched>/Summer/Adventure_UE5_Data/Managed/Assembly-CSharp.dll'
py -3.11 patches-workspace/moomin-midsummer-madness/he/build_moomin_package.py --original-root '<pristine>' --patched-root '<patched>' --database-output '<rebuilt-database.bundle>' --version 0.1.2 --release-revision 3
py -3.11 kitor/tools/build_patch_packages.py patches-workspace/moomin-midsummer-madness/he/patch.windows.candidate.json --platform windows --output dist/candidates
```

Installation and uninstall use Kitor's selective journaled lifecycle, backing up
only the five changed files. This package never writes to the LocalLow Saves
folder. The current user's game installation and save files were not modified
while producing this candidate. Native macOS packaging, visual menu verification
and loading a saved game remain unverified.

Validation of candidate 0.1.2: all three dedicated Python regressions and all
three Rust Moomin integration tests passed. The rebuilt native Windows lifecycle
validator installed/uninstalled the five affected pristine files, backed up only
those five originals, restored every original hash, and preserved the untouched
and added-after-install sentinels. The general package suite had 12 passes, one
skip, and one unrelated failure from the pre-existing untracked
`space-quest-iv/he/patch.json` declaring only Windows as a release manifest.
