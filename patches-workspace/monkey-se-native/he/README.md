# Native Monkey Island Special Editions: Hebrew candidates

Two independent Kitor entries (`monkey-se-native`, `monkey2-se-native`) run the
Windows remasters. Existing `monkey-se` and `monkey2-se` entries continue to run
the classic Hebrew versions in ScummVM.

The native candidates replace the French localization slot. MI1's French display label is replaced with Hebrew. MI2 retains its original
executable because changing its display strings causes an early exit; select
French for Hebrew in MI2. Enable subtitles.
Steam must be running for the second game's Steam edition.

Dialogue reuses AdventureBrew's classic translation through stable speech IDs.
Conflicts, unresolved controls and unmatched lines remain English. Current
coverage: 4,301/4,652 speech records in MI1 and 6,657/8,725 in MI2. Main menus,
settings, save/load confirmations and PC help use new Hebrew translations.
There are 133 translated UI records in MI1 and 183 in MI2; object names,
bonus descriptions, some help pages and dialogue choices are not all covered.

Version 0.2 includes Hebrew in the embedded classic engine for the F1 graphics
toggle. It rebuilds the original embedded scripts with translated strings and
AdventureBrew bitmap fonts, then synchronizes the native speech lookup keys.
Both directions of the toggle and speech were tested in the opening scenes;
MI1's lookout was checked separately after resolving repeated-text variants.
Unmatched lines and some interface text remain English; this is not a full-game
translation certification. Existing saves retain script positions, so validate
with a new game before relying on a save made with the previous script data.

Each package fingerprints every modified file, reconstructs from the user's
original files, and installs through Kitor's existing recovery journal. Removal
restores those files and leaves unrelated saves alone. See each package's
`validation.json` for actual lifecycle results; runtime verification is distinct
from a successful install/uninstall roundtrip.

## Shared build tools

The `prototype/` directory here builds both native Monkey Island packages. MI2 uses the same tools with its own input directory; its installable files remain in `monkey2-se-native/he/`. Translation credits are in `licenses/CREDITS.txt`.

## Rebuilding

Use Python 3.11+, Pillow and python-bidi. Run `prototype/inspect_resources.py`
with the owned PAK, upstream `strings.txt` and Ultimate Talkie `monster.tbl`;
then `build_candidate.py`, `build_menus.py` and `package_candidate.py`.
For F1 support, use Python 3.12 with BLooperZ/nutcracker and run
`build_classic.py --input generated/miN --fonts <scummworks-game>/userdata`
before packaging with `package_candidate.py --classic`. `test_classic.py`
checks control boundaries, equivalent text keys and bitmap character mapping.
Inspection also reads the first game's external `localization/uiText.info`
and generates both games' menu source tables locally.
Run `python -m unittest discover -s prototype -p test_candidate.py`.

Generated/ and dist/ contain owned game resources and are ignored by Git.
These are local candidates; full-game playback and RTL paragraph wrapping are
not certified. Credits and upstream references are in licenses/CREDITS.txt.
