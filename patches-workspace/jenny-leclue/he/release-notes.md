# Jenny LeClue – Detectivu: Hebrew patch 0.4.6

Hebrew localization for the supported Steam editions on Windows and macOS.
Includes 5,511 dialogue catalog entries and 1,931 UI entries, with optional nikud
available in the game’s language settings. The game starts in plain Hebrew.

Dialogue, choices, menus and the classroom typewriter use the game’s native RTL
processing. This release fixes reversed settings values while preserving numeric
resolutions and Latin language names. It also includes Hebrew journal/letter
fonts and reviewed proper-name nikud overrides.

## Install or remove

Install through Kitor for your owned Steam copy. The ZIP contains Kitor’s
Windows and macOS installation plans; it is not a standalone installer.
Python 3 is required. Kitor verifies supported original-file hashes, backs up
only affected files, and restores those originals when the patch is removed.
An unsupported game build is rejected rather than overwritten.

## Verification and limitations

Windows and macOS package/lifecycle checks and Kitor installer builds passed.
Native macOS checks covered menus, language switching, dialogue, intro typing
and fade; the user confirmed the intro and corrected settings screen.
Native Windows gameplay and a full story playthrough remain unverified.
Translation and generated nikud may still need editorial corrections.
Latin/digit runs display correctly but can reveal in reverse order within the run.
