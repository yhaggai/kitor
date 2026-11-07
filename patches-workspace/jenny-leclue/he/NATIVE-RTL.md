# Native RTL dialogue

`nikud/build_assets.py` now emits logical-order dialogue and choices directly,
without reversal, forced right-alignment tags, or fixed 44-character wrapping.
`native_dialogue.py` remains an idempotent migration/validation step for older
merged assets. It replaces the legacy visual-order `he` and
`he-x-nikud` dialogue and choice fields with logical-order text, keeping timing
tokens and rich text in their original positions. It validates every expected
legacy field before writing. Manuscripts and other static UI retain their separate handling; menus and typewriter are migrated separately below.

Run with the same Python dependencies as the nikud builder:

```
python native_dialogue.py merged/data.unity3d native/data.unity3d --glyph-source NativeGlyphs.cs
python native_dialogue.py merged/resources.assets native/resources.assets
```

Compile `HebrewShortcut.cs`, `NativeDialogue.cs`, and generated `NativeGlyphs.cs`
into `HebrewShortcut.dll`, referencing each target game's managed assemblies,
including `RTLTMPro.dll`. The generated glyph table must come from the same
pointed/manuscript catalogs used to build the font atlases. Do not regenerate
that table independently of the matching assets.

Compile `PatchNativeDialogue.cs` with Mono.Cecil and run it for each platform:

```
PatchNativeDialogue baseline-Assembly-CSharp.dll HebrewShortcut.dll output-Assembly-CSharp.dll Managed-directory
```

The baseline must include existing Kitor and letter-layout hooks, but not this
hook. The patcher rejects an already-patched input. It reuses the game's Arabic
RTL branch in `JLCSubtitleControls.SetSubtitle` and `JLCResponseButton.SetUp`,
and its alignment branch in `JLCSubtitleTypeOn.ToggleMiniSubtitles`. Renderer
RTL is set at each dialogue/choice assignment and resets for LTR languages.

Nikud atlas symbols are decoded to Hebrew before native RTL processing and
encoded after the game's existing reversal. This prevents private-use glyphs
being treated as Latin and keeps a pointed letter one animation unit. The
existing Hebrew font set is not made globally RTL, because static screens still
contain visual-order text.

After replacing managed assemblies and data, sign only the Mac app root (without
`--deep`), preserving the stock nested engine-library signatures. Verify with
`codesign --verify --deep --strict`; package both
platforms against the pristine hashes in `patch.json`. Update the installed copy
through Kitor's lifecycle update function and existing journal; never replace
its original backups with patched files.

Validation on macOS, 2026-11-07: actual patched subtitle method, RTL/LTR language
switches, pause index preservation, all 303 nikud glyph round trips, and the
original type-on coroutine. Native RTL still reveals Latin letters and digits
in reverse order within their correctly displayed runs. Native Windows gameplay
and a full scene-by-scene playthrough remain unverified.

## Menus and classroom typewriter (0.4.5, revision 10)

`native_static.py` migrates the 89 menu keys and eight typewriter keys to logical
order in both Hebrew modes (194 records), preserving the pointed atlas mapping.
Run it after the merged assets and dialogue migration:

```
python native_static.py merged/data.unity3d native/data.unity3d --stock pristine/data.unity3d --keys-source NativeStaticKeys.cs
python native_static.py merged/resources.assets native/resources.assets
```

Compile `NativeStatic.cs` and generated `NativeStaticKeys.cs` into the same
helper assembly as the dialogue sources. Apply `PatchNativeStatic` to each
platform's dialogue-patched Assembly-CSharp using the same four arguments as
`PatchNativeDialogue`. It rejects an already-applied hook.

For those keys only, localized lookup uses the game's RTLTMPro processing with
the existing nikud adapter. LocalisedText sets renderer direction and restores
original alignment for LTR languages. ClassroomTypewriter selects the native
Arabic speed entry for Hebrew. No global Hebrew font-set direction change is
made, since unrelated static text still uses visual ordering.

The Mac migration restores the pristine classroom text objects and the displaced
Line3_Eng transform. The localized typewriter fields no longer contain per-letter
position tags or reversed text. English-only line objects return to stock.

macOS isolated runtime checks: main menu screenshot inspected; RTL/LTR switches;
real ClassroomTypewriter completion with 100 plain and 99 pointed character
units; no body overflow; actual SlateTextColor fade changes opacity from 0 to 1
on both TMP_Typewriter_Dead clips. Main-menu overflow flags also occur in the
original English and do not indicate visible clipping in the inspected image.
The tests load the classroom database and invoke its components directly; a
normal intro playthrough was separately checked by the user, who confirmed
typing and fade on installed 0.4.5 on 2026-11-07. Native Windows gameplay
remains unverified.

## Dynamic settings values (0.4.6, revision 11)

Settings value setters bypass LocalisedText. The patcher routes MenuButton
SetText calls through NativeStatic.SetMenuText, enabling RTL for Hebrew and
pointed glyphs while retaining LTR for resolutions and Latin language names.
Both Hebrew modes and switching back to English passed a native runtime test
of the actual setters, glyph positions and options-screen captures. The user
confirmed the corrected options screen on 2026-11-07.
