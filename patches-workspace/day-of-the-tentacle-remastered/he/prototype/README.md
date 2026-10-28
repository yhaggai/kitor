# Day of the Tentacle Remastered: Hebrew dialogue prototype

## Windows candidate (2026-10-28)

The Windows Steam archive now has a separate native recipe at
`../patch.windows.candidate.json`. It contains 5,142 translated subtitles and
348 item/person labels, with all 12,800 original glyphs preserved. The user
confirmed the disposable Windows test build works. Broader gameplay review,
remaining dialogue, menus, and dialogue choices are still outstanding.

Windows speech uses a 4-byte count and 32-byte records with relative 32-bit
pointers at offsets 24/28. UI uses a 4-byte count and pairs of 32-bit pointers.
Unused trailing resource bytes are retained. Mac layouts remain supported.

Rebuild using Python 3.12+, Pillow, python-bidi (Windows), and the upstream
revisions below. Run `map_dialogue.py` and `align_dialogue.py` against the Windows
`tenta.cle`; extract Classic strings with nutcracker and run `map_item_labels.py`
as described below, also against the Windows resource. Then run:

```sh
python -X utf8 build_full_hebrew.py --source-windows "PATH/TO/GAME" --output generated/windows-v1 --font "C:/Windows/Fonts/arialbd.ttf" --item-labels generated/en.uitext.info.items
python -X utf8 validate_full_hebrew.py --archive "PATH/TO/GAME/tenta.cle" --output generated/windows-v1
python -X utf8 ../build_local_package.py "PATH/TO/GAME/tenta.cle" generated/windows-v1/game/tenta.cle --platform windows
python -X utf8 -m unittest discover -s . -p "test_*.py"
```

Build both lifecycle harness binaries in release mode, then run
`../validate_windows.py GAME_FOLDER HARNESS_BINARY_FOLDER`. It uses a temporary
copy to verify native install/uninstall, Classic coexistence, and save preservation.
Refresh the Classic recipes' alternate input hash and manifest recipe hashes when
rebuilding the native recipe. The original installed Steam files are not modified
by these build and validation commands. Local font/payload redistribution limits
still apply.

This is an offline mapping and resource-building prototype for the full remaster,
not a ScummVM patch or an installable Kitor release. No Steam files are modified.

## Reproduce

Requires Python 3, no third-party packages. Clone these public source repositories
and check out the recorded revisions:

- https://github.com/adventurebrew/scummworks — `7c8217e3746f3bc92b42623d444001789df6519c`
- https://github.com/BLooperZ/remonstered — `7d38eedaa9823a4a1c91315a24dfcb32bae5c114`

```sh
python3 map_dialogue.py \
  --archive '/path/to/Dott.app/Contents/Resources/tenta.cle' \
  --scummworks /path/to/scummworks \
  --remonstered /path/to/remonstered
```

The generated directory is ignored by Git: it contains locally extracted game
dialogue, upstream Hebrew translation text, review queues, and an experimental
`en.speech.info.candidate`. The script opens the source archive read-only.
Keep original translators' attribution and review upstream redistribution terms
before publishing any derived patch. The voice mapping source is GPL-3.0.

## Findings (2026-10-26, installed Steam macOS edition)

- Archive: KAPL 1.0, 2,865 members. Speech is `localization/en.speech.info`.
- Speech resource: 5,721 records, including 396 developer-commentary records.
- Layout: 64-bit count, 40-byte records, two self-relative 64-bit string pointers
  at record offsets 24 and 32. Other metadata is retained verbatim.
- Hebrew source uses CP1255 and escaped SCUMM control sequences. Its voice
  offsets match the remonstered voice-ID table, giving 4,370 unambiguous entry
  mappings; 16 entries have conflicting translations.
- 605 mapped entries contain classic multi-segment speech. The remaster can split
  these across distinct IDs. These are withheld from the candidate until aligned;
  joining the whole classic utterance into its first remastered subtitle would
  produce duplicated or mistimed text. Four entries contain other control codes.
- The conservative candidate replaces 3,763 records. Unresolved records retain
  their original text. 1,335 records have no direct Hebrew mapping, including the
  396 commentary entries; this is not a count of dialogue requiring new translation.
- 933 Hebrew source lines have no voice marker and need a separate mapping route
  (for example object names and dialogue choices).
- 318 mapped entries use legacy caret punctuation; preserve and review it rather
  than silently guessing its meaning.

## Validation and limits

The original resource parses and rebuilds byte-for-byte. The generated candidate
parses back to the intended strings and IDs, and retains all non-pointer metadata.
Original non-UTF-8 bytes are preserved using surrogateescape; new Hebrew strings
are experimental UTF-8. This does NOT establish that the engine accepts UTF-8 or
renders Hebrew. Fonts, encoding, RTL, wrapping, timing, menus, archive installation,
and gameplay remain unverified. No native Windows or macOS runtime test ran.

## Split-dialogue alignment

After generating the initial candidate, run:

```sh
python3 align_dialogue.py --archive '/path/to/Dott.app/Contents/Resources/tenta.cle'
python3 -m unittest discover -s . -p 'test_*.py'
```

The alignment stage extracts original English speech from the embedded classic
resource and verifies consecutive remastered subtitle text in the same utterance.
It normalizes whitespace, quote glyphs, classic caret ellipses, the observed
trademark/macramé glyph differences, and the classic dialogue indentation marker.
It does not use fuzzy wording matches. Translator-inserted breaks can be joined
only when the complete original English matches one HD subtitle.

Results: 600 of 605 split groups resolve automatically. The remaining five have
explicit, English-text-guarded resolutions in `alignment-overrides.json`, including
two small new Hebrew adaptations (the opening three beats and the omitted OW
interjection). A conflicting gibberish direct mapping is replaced by the matching
Hebrew sentence from the complete classic utterance. These editorial decisions
still require in-game timing review.

The resulting `generated/en.speech.info.aligned-candidate` replaces 5,142 of 5,721
entries. All 605 identified split groups are handled. The 579 unchanged entries
include 396 commentary entries and 183 other records; this does not complete the
translation of all dialogue or menus. Detailed evidence and pre-resolution review
queues remain in the generated JSON files.

Validation: binary parse/build round-trip, ID and non-pointer metadata preservation,
unaffected-record preservation, and three regression tests covering repeated sound
cues, intervening unmatched dialogue, and translator-added breaks. No runtime test.

Next work: resolve the remaining dialogue mappings and prove encoding/font/RTL
behavior in one HD scene.
An eventual release needs Windows and macOS packaging and selective manifest-based
backup/install/uninstall; none is claimed by this offline prototype.

## Local runtime test (2026-10-26)

An isolated APFS copy is prepared at
`generated/runtime/Dott-Hebrew-Test.app`, bundle ID `local.kitor.dott-hebrew-test`.
Only the copy's archive is patched: the aligned speech resource is appended and
its existing archive entry and data-section size updated. Re-extraction confirms
exact candidate bytes. The original Steam archive has not been patched.

The test app has a local ad-hoc development signature. Its `DottTest` launcher
sets the working directory to `generated/runtime`, where `steam_appid.txt`
contains the owned game's ID, 388210. This prevents the normal Steam restart from
opening the unmodified installation. The launcher writes `test-runtime.log` there;
the game created a separate `savedata/global.gsv` in that directory. Initial test
settings request a 1280×720 window. Existing Steam saves were not copied or edited.

The test window was observed through the accessibility API and Steam initialization
logged successfully. Screenshot capture repeatedly timed out, so subtitle rendering,
font support, RTL, and timing are **not visually verified**. The font is unchanged.
To check manually, open the test app, enable subtitles if necessary, and start a
new game. Inspect Purple Tentacle's first line (ממממ… אני צמא!) in remastered mode.
Record missing glyphs, ordering, punctuation, and wrapping separately.

Quit the test app when done. Cleanup is limited to the generated runtime directory;
there is no installed patch to uninstall from Steam. Runtime manifest:
`generated/runtime-manifest.json`.

## Font probe

`build_font_probe.py` builds a separate fresh app under `generated/font-probe`.
The original font's Hebrew character-map entries are all zero. The probe replaces
Latin A–Z and `[` in all 25 `a-talk` font sizes with the 27 Hebrew letter forms,
repacking atlas cells and updating glyph rectangles and advances. This is a
diagnostic mapping, not a production encoding: later English capitals will render
as Hebrew. Only 13 opening-scene subtitles are translated in this probe.

The FTX2 payload decodes as two-channel luminance/alpha. Glyphs are rendered from
the local Arial Bold system font (local testing only; do not redistribute). FriBidi
preorders each explicit subtitle line; automatic wrapping remains unverified.
The first subtitle includes `test 123` plus Hebrew to distinguish archive loading
and glyph rendering. All 51 replaced archive members were re-extracted and checked
byte-for-byte. Font-atlas preview was inspected and shows the Hebrew letter forms.

```sh
python3 build_font_probe.py --source-app '/path/to/Dott.app' --output generated/font-probe
codesign --force --deep --sign - generated/font-probe/DottFontProbe.app
```

The user confirmed on 2026-10-26 that the opening probe renders Hebrew
correctly in remastered mode. This validates the ASCII-slot diagnostic only.

## Full mapped dialogue candidate

`build_full_hebrew.py` extends the font glyph table from 160 to 187 records in all
25 subtitle sizes. Original glyph bitmaps and metrics remain intact. Hebrew uses
single bytes C0–DA mapped to the new records: the builder checks that none of these
bytes occurs anywhere in the original English speech resource. English capitals,
existing accents, and punctuation used by that resource therefore keep their
original mappings. Other language selections are unsupported by this candidate.

A native Unicode experiment in `generated/full-hebrew` displayed symbols in-game:
the speech path interprets UTF-8 bytes as legacy characters. Do not use that app.
The corrected candidate is **`generated/full-hebrew-v2/DottHebrewFull.app`**.
It has a separate working/save directory and modifies no Steam installation files.

All 5,142 mapped records are included; 579 remain original (396 commentary and
183 others). Menus, object labels, and dialogue choices are not translated.
Logical text is wrapped at word boundaries using font advances, then FriBidi
reorders each line separately. Carets become ellipses; the one Hebrew geresh
becomes an apostrophe. 2,028 subtitles contain explicit line breaks; 43 exceed
three lines and are listed in `long-subtitle-review.json` for scene/timing review.
The width budget is provisional; this is not a completed gameplay QA pass.

```sh
python3 build_full_hebrew.py --source-app '/path/to/Dott.app' --output generated/full-hebrew-v2
python3 validate_full_hebrew.py --archive '/path/to/Dott.app/Contents/Resources/tenta.cle' --output generated/full-hebrew-v2
codesign --force --deep --sign - generated/full-hebrew-v2/DottHebrewFull.app
python3 -m unittest discover -s . -p 'test_*.py'
```

Validation passed: 51 replacement member hashes, all 4,000 original glyph bitmaps
and metrics, every unaffected cmap entry, all IDs/non-pointer speech metadata,
579 unchanged records, and round-trip decoding of all 5,142 translated strings.
Seven tests cover alignment, punctuation, mixed English/numbers, and wrapping
before bidi reordering. Native Windows testing and install/uninstall packaging
remain future release work. Local Arial glyphs are for this local test only.

Runtime check on 2026-10-26: the corrected v2 app displayed readable Hebrew
in remastered mode. Observed the two-line Green Tentacle sentence “אני לא חושב
שכדאי לך / לשתות את זה...” with correct line order, and Purple Tentacle’s “זה גורם
לי להרגיש נ פ ל א!” with the exclamation on the left. Evidence is saved as
`generated/full-hebrew-v2/runtime-subtitles.png`. This verifies opening-scene
rendering and a two-line example, not all scenes, resolutions, or long subtitles.
Subtitles are enabled in this test copy; leave text language set to English.

## Item hover labels (2026-10-26)

Current test app: `generated/full-hebrew-items-v2/DottHebrewFull.app`.
Includes the same 5,142 subtitles plus 348 item/person label records (298 unique
English lookup labels), reused from the Classic Hebrew translation. Menus,
action verbs, dialogue choices, and other UI entries are unchanged.

`map_item_labels.py` pairs the 6,645 Classic English and Hebrew source lines;
all 5,269 voice anchors match, apart from six explicitly rejected `#REF!` source
cells. Label matching uses exact English case-insensitive text. Ambiguous names
are resolved by `CHOICES`, which is checked against existing Classic Hebrew
options; no new item translations were written. Every selected label records
its original source line(s) in `generated/item-label-map.json`.

English extraction used https://github.com/BLooperZ/nutcracker at revision
`1f9e34d1c9e3b9592e39942dc56dda3c1c18f8ae`:

```sh
nutcracker sputm strings_extract -t original-strings.txt /path/to/classic/en/tentacle.000
python3 map_item_labels.py --ui /path/to/extracted/en.uitext.info --english original-strings.txt --hebrew /path/to/scummworks/tentacle/strings.txt --output generated
python3 build_full_hebrew.py --source-app '/path/to/Dott.app' --output generated/full-hebrew-items-v2 --item-labels generated/en.uitext.info.items
```

UI resource layout: uint64 record count, then two self-relative uint64 string
pointers per record (key/value), followed by NUL-terminated byte strings. The
unmodified resource round-trips exactly. Keys are preserved, including the two
classic trademark control bytes. Only values in item-name sections 0–322 and
709–733 are replaced. 459 other records are byte-preserved after parsing.

Added Hebrew glyphs to all `a-verb` and `a-lilfnt` sizes as well as `a-talk`.
These two UI font families use FTX1 single-channel alpha, unlike the subtitle
font's FTX2 luminance/alpha. Both formats retain their original texture type.
Validation passed for 162 replaced archive members and 12,800 preserved original
glyph images/metrics; original English UI characters do not collide with the
chosen Hebrew byte slots. Seven existing regression tests also pass.

Native macOS runtime verified: loaded the copied Lobby save and observed the
clock hover label **שעון אורלוגין** in correct right-to-left order, fully inside
its blue label banner. Screenshot: `generated/full-hebrew-items-v2/runtime-item-label.png`.
Other rooms and unusually long item names still need gameplay review. The earlier
`generated/full-hebrew-items` directory is an incomplete build, not runnable.
