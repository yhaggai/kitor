# Owlboy בעברית

A complete Hebrew text translation and custom pixel font for **Owlboy**, with a
right-to-left rendering patch. Requires your own compatible copy of the game.

**Beta 0.2.0:** all 4,537 catalog entries are covered, including dialogue, menus,
signs, descriptions, and optional conversations. Nonverbal punctuation and
controller identifiers remain unchanged. Five localized area-title images are
generated locally. Full-game playtesting and native Windows gameplay validation
are still pending; this is not a claim of a polished, fully tested release.

## What is included

- Editable Hebrew in normal logical reading order, split into three JSON files.
- A 27-glyph Hebrew pixel alphabet, including final forms, with no niqqud.
- Runtime glyph placement for right-to-left reveal, highlighted text, and numbers.
  The game's original pause, sound, and event logic remains in control.
- Unicode BiDi layout maps for the static translation, plus a limited fallback
  for dynamic text (mixed Latin/number runs). Dynamic mixed text needs playtesting.
- Font-width-based dialogue wrapping and Hebrew area titles.
- Selective install/uninstall with hashes, backups, crash recovery, and conflict checks.
- Windows and macOS source installers and locally generated Kitor package manifests.

This repository contains original patch source, Hebrew translations, and glyph
artwork. It does **not** contain Owlboy.exe, original game atlases, an English
script dump, saved games, or prebuilt modified game files. Build those locally.

## Compatibility

The current guarded target is the Steam **1.4 macOS FNA build** recorded in
[compatibility.json](compatibility.json). The builder refuses unknown executable
or source-catalog hashes. A Windows edition with different assemblies needs a
separate compatibility check; its source installer does not imply gameplay
compatibility. No Intel/Apple Silicon/Windows game-runtime conversion is performed.

The full translation and RTL runtime passed an opening-scene smoke test on
macOS, with readable multiline Hebrew dialogue. Automated checks cover layout
and installation; see [validation](docs/validation.md) for remaining playtesting.

## Install

Close Owlboy first. Remove an older Hebrew patch using its own uninstall journal.
Do not build from an already modified installation.

Prerequisites: Python 3.10+ and the **.NET 9 SDK**. The scripts install the pinned
Pillow and python-bidi build dependencies into a local virtual environment and
restore Mono.Cecil from NuGet. Internet is needed for this first setup.

**macOS** (Steam's default install location is detected):

```sh
./install-macos.command
```

For another installation:

```sh
./install-macos.command '/path/to/Owlboy.app/Contents/Resources'
```

**Windows** (edition must match the compatibility guard):

```bat
install-windows.cmd "C:\path\to\Steam\steamapps\common\Owlboy"
```

The game directory must contain `Owlboy.exe` and `Data`. The installer prints the
recovery journal location. Keep the entire `recovery` folder. Start Owlboy, open
Languages, and select the Israeli flag. Existing saves are not part of the patch.

### Manual build and installation

From this repository, after installing `requirements.txt` into your environment:

```sh
python tools/build.py --game '/path/to/resources' --out build/package
python tools/manage.py install --package build/package \
  --game '/path/to/resources' --recovery recovery/0.2.0-beta.2
```

`build/package/patch.json` is also a Kitor lifecycle manifest for both `osx` and
`windows`. Use the same resources directory as the installation root. Generated
packages contain game-derived data and are ignored by Git; do not upload them.

## Uninstall

Select English in Owlboy, exit the game, then run:

```sh
python tools/manage.py uninstall \
  --journal recovery/0.2.0-beta.2/journal.json --game '/path/to/resources'
```

Only files recorded by this installation are restored or removed. Later-edited
files cause a conflict instead of being silently overwritten. Missing or corrupt
backups are reported. Backups/journal and the stable `.owlboy-hebrew.lock` file
are retained; patch-created directories are removed only if empty. After a
successful uninstall, use a new recovery directory for a fresh installation.

If installation was interrupted, repeat the same install command to resume or
use its journal to uninstall. Normal write failures automatically roll back.

## Contribute or review

Read [the glossary](docs/translation-guide.md) and [translation QA](docs/translation-qa.md).
To see the English context locally from your own copy:

```sh
python tools/extract.py --game '/path/to/resources'
```

This writes `.local/source.json`, which is deliberately ignored. Entry IDs are
checked against `catalog-contract.json`. Edit `translations/part-*.json` and run:

```sh
python -m unittest discover -s tests -v
dotnet build patcher/Patcher.csproj -c Release
dotnet build tests/runtime/RuntimeTests.csproj -c Release
python tools/runtime_fixture.py
cd .local/runtime-fixture
dotnet ../../tests/runtime/bin/Release/net9.0/RuntimeTests.dll
```

Tests cover catalog coverage, control tokens, branch IDs, BiDi permutations,
RTL reveal and highlight positioning, and selective install/uninstall recovery.
CI runs these on Linux, macOS, and Windows without requiring any game files.

## Credits

Owlboy is by **D-Pad Studio**. This is an unofficial fan translation, created
with AI-assisted translation and review. Original Hebrew glyphs and patch tools
are maintained here. The underlying game and its original assets remain the
property of their respective owners.

Build dependencies: [python-bidi](https://pypi.org/project/python-bidi/),
[Pillow](https://pypi.org/project/Pillow/), and
[Mono.Cecil](https://www.nuget.org/packages/Mono.Cecil).
