# Day of the Tentacle Remastered — local Hebrew prototype

This is a separate native patch for Remastered. It does not use ScummVM.
The current candidate translates 5,142 subtitles and 348 item/person label
records; menus, dialogue choices, and 579 speech records remain English.
Keep the in-game text language set to English.

The patch uses Noto Sans Hebrew Bold under SIL OFL 1.1. It backs up
`tenta.cle` and, on macOS, the files changed by ad-hoc signing. Uninstall
restores those originals.
The recipes target the hash-matched Steam builds from depot 388211
(Windows x86) and depot 388214 (macOS x86_64).
The Windows Steam candidate includes the same subtitles and item labels; its
previous font build was tested on Windows on 2026-10-28. Keep text language set
to English. This remains a partial translation, with broader gameplay review pending.

Both Classic and Remastered patches are available for a Remastered installation.
The Classic patch creates separate ScummVM data; the Remastered patch modifies
only `tenta.cle` on Windows. Either can be uninstalled independently.
