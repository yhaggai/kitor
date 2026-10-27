# Day of the Tentacle Remastered — local Hebrew prototype

This is a separate native patch for Remastered. It does not use ScummVM.
The current candidate translates 5,142 subtitles and 348 item/person label
records; menus, dialogue choices, and 579 speech records remain English.
Keep the in-game text language set to English.

The patch backs up only the changed archive and restores it on uninstall.
It uses local Arial-derived glyphs and is for personal testing only.
Windows and macOS have separate patch recipes for their native resource layouts.
The Windows Steam candidate includes the same subtitles and item labels; its
native rendering was confirmed by the user on 2026-10-28. Keep text language set
to English. This remains a partial translation, with broader gameplay review pending.

Both Classic and Remastered patches are available for a Remastered installation.
The Classic patch creates separate ScummVM data; the Remastered patch modifies
only `tenta.cle` on Windows. Either can be uninstalled independently.
