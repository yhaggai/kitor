# Hebrew translation QA

Review date: 2026-10-25.

This is a source-based consistency and mechanical QA pass on the three translation JSON files. It does not claim a complete in-game playthrough or a professional editorial review.

## Coverage and mechanical validation

- Source entries: **4,537**.
- Translation entries: **4,537**, with all source IDs present and no extra IDs or empty translations.
- Angle-bracket markup: **2,875 tokens across 1,436 entries**; zero token-count/value mismatches. This includes emphasis, explicit line breaks and dialogue timing.
- Square-bracket impact markers: **78 tokens across 71 entries**; zero mismatches.
- Menu branch identifiers: **15 identifiers across 15 entries**; zero mismatches. The identifiers after `|` remain unchanged.
- Curly-brace and printf-style percent placeholders: none occur in this source; neither was introduced by the translations.
- Remaining Latin text outside markup and branch identifiers occurs in **13 entries**, all in `default.ini`. These are the game name, controller buttons, the F11 key, and PS4 platform name. No untranslated English dialogue was found.
- **187 entries** are identical to source. These are nonlinguistic punctuation/timing strings and eight controller-button labels; no English prose was left unchanged.

Markup was compared as a multiset per entry. Hebrew word order can move emphasis and line breaks relative to other tokens; it is not required to mirror English token order.

## Corrections made in this pass

**35 entries changed.**

- Kernelle's first-person forms changed to feminine: `0682`, `0687`, `0688`.
- Gawk spelling normalized to **גוק**: `1778`, `1985`.
- Repeated scene dialogue synchronized: `3684`, `3685`, `3686`, `3690`, `3995`, `3996`, `4014`, `4018`.
- Recurring token/key message synchronized: `3356`.
- Repeated book quotations synchronized: `1875`, `1876`, `1877`, `2585`, `2586`, `2587`, `2588`, `3710`.
- Eleven entries had adjacent numeric timing tags at the same visible-character position, which the game parser rejects. Wording or expressive elongation now separates every pause while preserving exact timing values and counts: `1615`, `1673`, `1687`, `1691`, `1715`, `1730`, `1736`, `1776`, `1895`, `2341`, `3870`.
- The pirate dreadnought is consistently **ספינת המערכה**, including checkpoint and objective text: `4222`, `4427`.

Shared character and place names were checked against their source mentions after stripping timing markup. The inventory description at `4311` continues onto `4312`, where Vellie's name appears; this is an intentional sentence split, not an omission.

## Contextual variations retained

There are **183 distinct repeated source strings**. Before review, 26 had varying Hebrew translations; **nine** remain, all short conversational expressions. Their variations preserve speaker voice, singular/plural address, grammatical gender, or scene meaning.

For example, `0156` and `2202` legitimately use different grammatical gender. The expressions at `0406` and `3351` refer respectively to handing something over and helping someone recover. These should not be normalized blindly.

## Remaining review needs

- A full playthrough is still needed to assess speech bubbles, UI widths, text reveal direction, timing, and transitions between separately stored description lines.
- Letter-by-letter emphasis uses every original timing token. Hebrew words often contain fewer letters, so some timing pauses occur around punctuation or within expressive elongation. No adjacent numeric timing collisions remain. Mechanical parity does not establish that every dramatic pause looks ideal in game.
- Dialogue source paths can contain several speakers. Shop scripts include male assistants as well as Buccanary; masculine dialogue within those files was not automatically changed to feminine.
- Ancient-history names are transliterated consistently as **נוקטה**, **אגוליוס**, and **סורניה**. The source excerpt does not explicitly establish Surnia's gender; `3615` uses feminine agreement and should be checked against the character's presentation.
- The term for the recurring cosmological cycle is **הלולאה**; the countermeasure is **אנטי־הקס**. The temple reference to higher mysteries is **הסודות הנשגבים**.
- The translated game is a complete first translation with a consistency pass, not a claim that every line has had independent human linguistic review.
