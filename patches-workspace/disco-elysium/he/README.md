# Disco Elysium Hebrew patch

Kitor downloads the ZIP listed in its registry and applies five binary deltas. The installer verifies the original Steam files and backs up only the files it changes. Uninstall restores those files from the journal. Python 3 is required. Select HEBREW in the game's language settings to activate Hebrew.

Edit the `hebrew` column in `translations.csv` for dialogue and interface text. Keep the keys, English source and formatting tokens intact. The catalog combines the dialogue snapshot at `6b71fce` with the interface translations used by the current patch. The package uses the original game's RTL renderer. The Hebrew and Greek atlas additions use Assistant and Noto Sans under the SIL Open Font License. The notices are in `licenses/`.

To rebuild, run `python text_pipeline.py extract --data PATH_TO_MAC_DATA` with the matching owned Steam game to generate the source index. Use Python with UnityPy 1.25.0, bsdiff4 1.4.2, Pillow with RAQM, NumPy, and SciPy. Run `python build_package.py --mac PATH_TO_DISCO_APP --windows PATH_TO_WINDOWS_DEPOT --font Assistant.ttf` from this directory. The build reads the original game files and writes the manifest and deltas here. Keep owned game data and the generated source index out of Git.
