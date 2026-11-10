---
name: hebrew-game-localization
description: Translate or revise a video game's text into Hebrew, build a speaker and mood glossary, and verify the localization in the game's renderer. Use for Hebrew game patches, dialogue editing, and RTL or missing-glyph issues.
---

# Hebrew game localization

Match the requested scope. A glossary review can end with editorial changes. A playable patch needs a verified build and a reversible install.

The 2026-10-31 skill edit used OpenAI Codex with model gpt-6-astra and low reasoning effort.

## Start from the game's evidence

Identify the build, text storage and active language slot. Keep an untouched source and inventory tables by stable entry ID. Preserve duplicate English strings under separate IDs. Look for shared key tables, scene names and speaker IDs before translating dialogue. A displayed label can hide an identity, and a phone or TV can carry several voices.

## Check built-in RTL support first

Before adding manual text reversal, visual-order conversion, or custom reveal logic, inspect the game's existing RTL support. Look for shipped Arabic or Hebrew language settings, font-set direction flags, RTL libraries, localized layouts, and the dialogue renderer's reveal path. Prefer reusing the native path with Hebrew fonts and logical-order text when it works.

Test the complete path in an isolated build, not just a renderer toggle. Verify final appearance and progressive character/word reveal separately, including wrapped lines, punctuation, numbers, Latin runs, rich text, timing tokens, choices, and switching back to an LTR language. Check nikud and custom atlas glyphs: private-use symbols may need decoding before native bidi processing and re-encoding afterwards. Avoid combining native RTL with already reversed text. Scope changes to the verified components; static screens may use a different layout path.

Use a manual workaround only for a demonstrated gap in the native support, and record the evidence and remaining limitations. Keep editable translations in logical order and put any necessary conversion in the build or rendering layer.

For Jenny LeClue, read [the native RTL implementation notes](../../../patches-workspace/jenny-leclue/he/NATIVE-RTL.md) before changing dialogue direction. Its dialogue, choices, menus, and classroom typewriter reuse native Arabic RTL processing with a nikud adapter. Other static screens and manuscript layouts retain separate handling; do not globally enable RTL on still-reversed text.

## Glossary and translation model roles

For this workspace, use **GPT-6 Luna (`gpt-6-luna`) for glossary drafts** and **GPT-6 Astra (`gpt-6-astra`) for game translation**, unless the user specifies otherwise. Delegate these roles with the explicit model overrides when available; if a requested model is unavailable, report that instead of silently substituting another model. Luna is an OpenAI model, not an open-source model. The Argos configuration below is historical Silver Pines context, not the glossary model.

Give Luna source dialogue, speaker IDs, conversation context and the existing glossary. Preserve established Hebrew names; attach source references to new terms and mark uncertain gender, identity and pronunciation as provisional. Astra should use the saved draft while translating and resolve consequential ambiguities against the source. Record each model's actual contribution separately; do not relabel earlier entries as Luna-authored. Keep glossary and translation writers on separate files to avoid concurrent overwrites.

For Disco Elysium, write translations directly to `patches-workspace/disco-elysium/he/translations.csv` in logical Hebrew order. Preserve source English, stable keys and protected tokens; validate saved batches with `text_pipeline.py validate`. Continue authorized batches without asking for approval between them, and report coverage without treating a partial draft as a complete translation.

## Establish the voices

Maintain a glossary with the English label, canonical Hebrew name, aliases, grammatical gender, pronouns, forms of address and the source that establishes them. Distinguish speaker, addressee and third-person subject. If a name or role leaves gender unresolved, use natural unmarked wording such as ״אפשר לעזור?״. Preserve what the player knows at that scene.

Give each recurring speaker a short voice note grounded in their lines. Record how the mood changes between scenes and how to express it in Hebrew through vocabulary, rhythm, pauses or interruptions. Read both sides of an exchange together. In a car argument, Alice telling Eddie ״תעצור״ and Eddie telling Alice ״תירגעי״ require different agreement even when their lines occur in the same table.

## Translate and edit

Keep editable Hebrew in logical reading order, keyed to the original ID and source. Use recurring names, clue wording and UI labels consistently. For each model-assisted pass, record the model ID, reasoning effort if available, and which entries it drafted or reviewed. Review puzzle instructions, dialogue with gendered verbs, idioms, threats and recurring motifs in context. Check each entry's placeholders, rich text, sprite references, numbers and intentional line breaks against the source. Apply exceptions deliberately when natural Hebrew changes a written-out number or punctuation.

## Automatic nikud and proper names

Treat automatic nikud as a draft, especially for character names, surnames and places. Maintain a persistent pronunciation dictionary alongside the translation, keyed to the English source and canonical Hebrew spelling. Prefer the game's spoken English or explicit pronunciation evidence; label readings inferred from spelling and leave ambiguous or invented names pending audio review rather than claiming they are verified.

Apply reviewed overrides after automatic vowelization and in the asset build path so regeneration cannot erase them. Update the pointed translation files themselves. Cover dialogue, menus, UI and manuscript text, including attached Hebrew prefixes and names split across rich-text positioning tags. Require source context when a name's Hebrew spelling is also an ordinary word. For a nikud-only edit, preserve consonants, stable IDs, punctuation, controls and markup; verify that applying overrides again makes no further changes. Track source edits separately from rebuilt packages and renderer verification.

For Jenny LeClue, use `patches-workspace/jenny-leclue/he/nikud/names.json` and `names.py`; the generated text lives in `pointed.json` and `manuscript.json`. Both `vowelize.py` and `build_assets.py` apply the overrides. Keep unresolved pronunciations recorded with the dictionary and localization provenance.

## Match the original typography

When choosing or changing a Hebrew font, compare representative text from the game with its Hebrew translation at the same rendering scale and baseline. Judge the shape of the strokes, contrast, weight and spacing. Compare visible letter height against the original body letters and capitals. Tune the Hebrew size by its appearance at menu or subtitle size, since equal nominal point sizes can look different.

Check regular, bold, italic and bold italic wherever the game uses them. Verify that each style maps to its intended Hebrew face and stays distinguishable after the game builds or loads its fonts. Include a word with a descender to expose baseline or clipping problems.

Show the user a labeled side-by-side image with one row per style and columns for the original language and the proposed Hebrew. Include the previous Hebrew when replacing a font. Prefer an in-game capture or a preview drawn from the rebuilt font assets. State which was used, report the size adjustment, and identify any style that still needs an in-game check.

## Test in the renderer

Prototype a visible menu label and a punctuated subtitle before scaling up. If text is blank, inspect the game's font warnings, active atlas and fallback chain. Test final Hebrew letter forms and styled text. Check reading order on wrapped lines containing names, numbers, controller icons and Latin text. Apply visual-order conversion only at build time when the renderer needs it. Reversing an entire paragraph breaks line flow and mixed-direction text.

Reopen the built artifact and verify stable IDs, entry coverage, untouched data and protected tokens. Install from a staged file with a known rollback copy when installation is in scope. Distinguish bundle validation, startup checks and scenes actually observed in-game. Track image-only signs or voiced audio separately if the task includes them.

## Continuing Silver Pines

The first English-to-Hebrew draft used Argos Translate package `translate-en_he` version 1.5, run locally with CTranslate2 on CPU using INT8 computation and beam size 2. Subsequent edits to names, forms of address and dialogue are part of the reviewed catalog.

In the kitor workspace, read the current character glossary and catalog in `patches-workspace/silver-pines/he/`. Use the existing `build.py` with the pristine English bundle, the logical-order catalog and a staging destination. The builder owns visual ordering. The English slot and patched fonts are part of the tested setup. Use the Unity shared dialogue keys to identify speakers, including keys with numeric suffixes. Preserve the displayed mystery labels. Treat the catalog as the edited source of truth so regenerating a machine draft cannot discard reviewed lines.
