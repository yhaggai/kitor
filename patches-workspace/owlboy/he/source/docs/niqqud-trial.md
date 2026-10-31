# Optional niqqud edition — experimental beta

The `hn` language reuses the existing Hebrew translation and adds vowel marks.
Regular `he.csv`, its dialogue font, layouts and titles remain intact. Select
the Israeli flag with **N** for nikud, or the unmarked Israeli flag for regular
Hebrew. No new translation from English was performed.

The original five-line trial was confirmed working in-game by the user. The
expanded catalog is an offline Nakdimon 0.2.1 draft with targeted English-context
review of names, gender, commands and ambiguous words. The first 100 lines were
manually pointed. Remaining lines have not received exhaustive manual linguistic
proofreading. Treat this as a beta and report suspect forms with dialogue context.
Model source: https://github.com/elazarg/nakdimon (MIT); runtime model from the
PyPI wheel. No dialogue is uploaded to a niqqud service.

Editable Unicode text lives in `translations/niqqud/part-*.json`. The builder
requires every original entry, verifies all original letters and punctuation,
and preserves timing, color, event and branch markers. It generates composite
letter/mark glyphs so they reveal and reorder together. Private-use codes occur
only in generated game files. Dialogue spacing is 20 pixels for marks beneath
descenders. Narrow letters have room for distinct dots.

## Build and install

Build against Hebrew with no earlier niqqud add-on installed. The `--original`
folder supplies the guarded pristine executable and framework/compiler inputs;
`--game` supplies the installed regular Hebrew files.

```sh
.venv/bin/python tools/niqqud_trial.py --full --game '/path/to/Resources' \
  --original .local/original --out build/niqqud-beta
.venv/bin/python tools/manage.py install --package build/niqqud-beta \
  --game '/path/to/Resources' --recovery recovery/niqqud-beta-1-r2
```

The local drafting tool is optional; ordinary builds use the checked-in reviewed
JSON and do not need Nakdimon. Do not overwrite reviewed files with raw model
output. `requirements-niqqud.txt` pins the optional draft dependency.

## Remove separately

The add-on adds the `hn` catalog/fonts/layouts/flag, extends shared fonts while
preserving every original glyph/rectangle, and updates the shared language
registry/runtime helper. Selective backups retain all replaced bytes.

**Remove this add-on before updating or uninstalling the underlying Hebrew patch
through Kitor.** Select regular Hebrew and close Owlboy first, then:

```sh
.venv/bin/python tools/manage.py uninstall \
  --journal recovery/niqqud-beta-1-r2/journal.json --game '/path/to/Resources'
```

This restores the previous shared fonts/helper/registry and removes only the
add-on files. Saves and the regular Hebrew translation are preserved. The older
five-line trial uses its own `recovery/niqqud-trial-1/journal.json` instead.

This beta is not a published Kitor update. macOS build/install verification and
stand-in rendering tests do not establish native Windows gameplay compatibility.
