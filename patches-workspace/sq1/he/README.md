# Space Quest 1: The Sarien Encounter (EGA) — Hebrew

Status: packaged, awaiting native Windows/macOS and owned-game validation.

Install and uninstall through Kitor’s declarative lifecycle. Only the files listed in patch.json are affected; originals are backed up individually outside the game. Uninstall restores affected originals and removes patch-created files, preserving unrelated files and saves. Never run the upstream installer against the game.

See provenance.json for exact source URLs, edition restrictions, dependencies and validation gaps. The same payload supports Windows and macOS runtime plans; this is not a claim of tested native builds.

## CSV translation source and rebuild pipeline

`translations.csv` is the editable source for **logic messages, inventory names,
and view/item descriptions** in CSV-built SQ1 EGA patches. Columns are `key`,
`english`, `hebrew`; edit only Hebrew. The initial catalog was extracted from the
actual reconstructed Hebrew payload, not copied from an upstream working draft.
English comes from the matching original game's resource/message IDs.

There are 2,184 rows: 2,139 logic-message slots, 25 inventory names, and 20 view
descriptions. 1,819 rows contain text; unused and empty slots remain in the catalog
to preserve identity. Two nonempty patch messages have no original English text.
Hebrew is logical reading order, encoded to CP1255 only during compilation.
Stable keys such as `logic.000.message.002` survive spreadsheet sorting.

Default build requirements: Python 3.10+ (standard library only) and this repository.
Original game archives are **not required to build packages**. Installation still
requires SQ1 EGA 2.2 files matching all nine original SHA-256 values in `patch.json`.
The locally available Steam copy matched these hashes; this does not imply that
every Steam/GOG edition is compatible. No upstream executable is executed.

From the repository root, build directly from the checked-in CSV:

```sh
python3 kitor/tools/build_patch_packages.py \
  patches-workspace/sq1/he/patch.json --output dist/patches

# The same compiler runs for release ZIPs and validation-only builds:
python3 kitor/tools/build_patch_packages.py \
  patches-workspace/sq1/he/patch.json --universal --output dist/patches
python3 kitor/tools/build_patch_packages.py \
  patches-workspace/sq1/he/patch.json --check
```

The standard packager recognizes `build-source.json`, verifies its pinned
`text-template.json`, compiles the current CSV in temporary storage, validates
the resulting file plan, and packages it. It runs this on every call, including
calls from `publish_patch_releases.py`, download packaging, and `--check`.
There is no cached-output or missing-CSV fallback. The canonical SQ1 source also
rejects a missing `build-source.json`. Other games retain their existing builds.

Each ZIP contains the generated `patch.json`, payloads, and `build-report.json`
with the CSV/template hashes and changed keys. The source `patch.json` contains
release metadata plus the baseline asset plan; generated hashes and offsets are
written only into the build outputs. Edit `version`, `release_revision`, and
`filename` there for future releases; do not manually update its baseline hashes.
Choose a new immutable version and revision greater than the latest published
revision. This build-process change uses `1.0-kitor2`, revision 1. The application
registry continues pointing at its existing published download until a reviewed
new release is available; no release or registry update happens during building.
`provenance.json` remains `release_ready: false`.

`text-template.json` is immutable compilation context extracted once from the
verified patch: logic resources, the 20 views with descriptions, inventory and
directory tables, and original English text. It contains no original VOL archives.
It retains baseline strings to detect changes and preserve unchanged resource
bytes; those strings are never used as a fallback for missing CSV rows. All active
text comes from a complete, validated CSV. Existing non-text payloads remain
pinned assets. This lets Windows/macOS CI compile without installing the game or
fetching new private build inputs.

To re-extract the baseline for comparison (never overwrite an edited catalog):

```sh
python3 patches-workspace/sq1/he/text_pipeline.py extract \
  --original "/path/to/unmodified/SQ1" \
  --output /tmp/sq1-baseline.csv
```

The original-game rebuild is retained as an independent verification path:

```sh
python3 patches-workspace/sq1/he/text_pipeline.py build \
  --original "/path/to/unmodified/SQ1" \
  --output dist/sq1-owned-verification \
  --version 1.0-verification1 --revision 1
```

This command requires a new output directory outside the game and builds both
platform packages. To intentionally replace the baseline template, use
`prepare_template(original_path, output_path)` in `text_pipeline.py` against
matching originals, review the result, and update the checksum in
`build-source.json`. Ordinary translation edits never regenerate the template.

### How rebuilding works

1. Verify the template checksum, baseline plan and delta/copy assets. The default
   build uses the extracted resource context. The independent owned-game path
   instead reconstructs the full Hebrew patch and verifies its final hashes.
2. Read AGI v2 directory entries (volume nibble + 20-bit offset), 5-byte resource
   headers, logic message pointers/XOR strings, encrypted inventory, and view
   description pointers. Recognize both original Sierra and upstream compiler
   message-size conventions. The XOR key is the format's `Avis Durgan` key.
3. Validate the entire CSV: exact keys, unchanged English, CP1255 encoding,
   preserved AGI substitution tokens, no duplicate/missing rows, no new text in
   unused slots, and no accidental blanking of existing text. Preserve spaces
   and embedded LF line breaks. CSV quoting handles commas and quotes.
4. Rebuild changed message tables without changing logic bytecode. Rebuild
   inventory names while retaining IDs, starting rooms, and animation limit.
   Append changed descriptions without moving sprite data. Place changed logic
   and view resources in VOL.3 and update directory offsets; reject size overflow.
   Every build starts from the pinned baseline, so edits do not accumulate stale
   appended resources across builds. Unreferenced baseline resource bytes may
   remain in the volume; active strings always match the CSV.
5. Re-extract active text entries and compare them with the CSV. Generate
   BSDIFF40 deltas for changed directories/inventory against their small pinned
   original tables and apply them with Kitor's decoder to verify the output.
   Updated resources are appended to the existing patch-owned VOL.3. Original
   VOL.0–2 deltas remain unchanged. Preserve original fingerprints, regenerate
   changed output hashes, and package both platform plans. Owned-game integration
   tests additionally apply all deltas to real originals and re-extract the output.

This is a **text rebuild using the existing patch as a structural template**, not
a complete source recompilation of its logic/graphics. Parser vocabulary
(`WORDS.TOK` and its extension), fonts, pictures, translated text painted into
graphics, and logic changes remain baseline assets. They are not represented by
the subtitle CSV. No translated gameplay code is reconstructed from guessed text.

### Verification

```sh
python3 -m unittest discover -s patches-workspace/sq1/he -p 'test_*.py' -v
python3 -m unittest discover -s kitor/tools -p 'test_sq1_csv_build.py' -v
SQ1_ORIGINAL="/path/to/unmodified/SQ1" \
  python3 -m unittest discover -s patches-workspace/sq1/he -p 'test_*.py' -v
```

The resource suite runs six synthetic format tests and, with `SQ1_ORIGINAL`,
four real-resource integration tests. The shared-tools suite adds four offline
default-build tests and runs in existing Windows/macOS CI. Verified locally:

- Unchanged extracted CSV reproduces all 12 Hebrew game files byte-for-byte.
- Editing dialogue, an inventory name, and a multiline item description changes
  exactly the requested active text; code, sprite data, other text, vocabulary,
  fonts, pictures, and sound resources are preserved.
- Both platform ZIPs decode to the expected edited files and repeat builds are
  byte-identical. The original game directory remains untouched.
- The default offline compiler and independent original-game compiler produce
  identical game bytes for edited dialogue, inventory, and item descriptions.
- Platform/release builds and validation compile the current CSV; invalid/missing
  CSV, changed template checksums, and unknown compilers fail before ZIP creation.
- Kitor's actual lifecycle harness passed a disposable install/uninstall of the
  edited package on macOS using its Windows plan: nine originals backed up and
  restored byte-for-byte, three new files removed, unrelated files preserved.
  Both plans share the same file operations. This is not native Windows testing.

See `csv-validation.json` for recorded evidence. Runtime line wrapping, Hebrew
rendering, and native Windows gameplay remain unverified; a successful resource
build is not sufficient to mark the patch release-ready.

### Format references and attribution

The existing Hebrew translation and patch come from
[sam-mfb/agi-ivrit-sq1](https://github.com/sam-mfb/agi-ivrit-sq1/releases/tag/v1).
Its working repository also contains `messages.json`, `objects.json`, and
`views.json`, but those are not necessarily identical to the released payload.
The CSV preserves the released payload, including untranslated strings.

Format behavior was checked against ScummVM's
[logic decoder](https://github.com/scummvm/scummvm/blob/master/engines/agi/logic.cpp),
[inventory decoder](https://github.com/scummvm/scummvm/blob/master/engines/agi/objects.cpp),
[view decoder](https://github.com/scummvm/scummvm/blob/master/engines/agi/view.cpp), and
[text formatter](https://github.com/scummvm/scummvm/blob/master/engines/agi/text.cpp).
