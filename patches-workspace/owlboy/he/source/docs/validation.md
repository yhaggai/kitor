# Validation status — 25 October 2026

## Translation

All 4,537 entries have explicit catalog values. Every source timing token,
highlight marker, impact marker, and menu-branch identifier is accounted for.
The final consistency pass fixed gender/name differences and eleven pause-tag
collisions which would otherwise throw in the game's dialogue parser.

Natural-language review was AI-assisted, not a full human editorial review.
Technical button names, punctuation-only utterances, and a few proper identifiers
remain unchanged. See translation-qa.md for remaining contextual uncertainties.

## Automated checks

- 19 Python tests pass locally on macOS, including every catalog entry's BiDi map.
- Renderer fixture passes: unchanged English fallback, RTL character reveal,
  fixed full-line anchoring, highlighted slices, multiline alignment, preserved
  number order, and centered reveal anchoring.
- Patcher compiles and applies to the guarded original assembly. Modified short
  branches are widened to prevent insertion-related IL jump overflow.
- Installer tests cover selective byte-for-byte restoration, executable modes,
  idempotence, interrupted install/uninstall, normal-error rollback, corrupt or
  absent backups, unsafe paths/symlinks, ownership conflicts, and later-added files.

These renderer tests use controlled stand-ins for GPU/game types; they do not
replace real gameplay tests. Windows native CI fixtures exercise Python locking
and runtime source tests when the repository workflow runs. They do not launch
Owlboy or verify a Windows game edition.

## Gameplay

The user confirmed the earlier font-only prototype looked correct. The full
4,537-entry build now passes a native macOS opening-scene smoke test: the game
stays running and displays readable, right-aligned, multiline Hebrew dialogue.
An initial reflection-based rendering call crashed the bundled Mono runtime;
the released patch instead binds a direct managed call at build time.

A complete playthrough remains pending, including large text, dynamic mixed
text, credits, optional dialogue, highlighted phrases, and every title graphic.
Native Windows gameplay is untested. The project remains a beta.

## Publication boundary

Public source/release ZIPs include only the explicit source allowlist in
`tools/package_source.py`. No original executable, font atlas, English script,
saves, paths specific to a user's machine, or recovery backups are published.
Local `build/package` outputs must remain private.
