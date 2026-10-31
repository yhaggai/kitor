# מסע בחלל IV

The patch targets the English voiced CD edition included in Steam's Space Quest Collection (10110). Extract the entire ZIP and open **Play Hebrew.cmd** on Windows with ScummVM installed. The launcher reads the installed Steam game and loads translated resources through ScummVM's extra search path. To go back to English, use the original launcher. Kitor can also install the bundled package and track the installed files for reversal.

## Translation notes

The catalog contains 1,793 entries across 107 original message tables, including repeated text under distinct IDs, internal names, and source-code remnants. It is a playable draft. English voices are original. Hebrew dialogue, menu artwork, the title slides, the wait cursor, and selected signs have been localized. Other scenery can still contain English. The full game has not been played through.

Editable text is in logical reading order. The build uses visual ordering for the English SCI slot. Do not select the Hebrew engine language for this package. The approved exclamation mark uses a separate glyph with a one-pixel spacing adjustment. The status font also contains a composite for the complete series title, so it fits the original fixed-size script strings.

## Names and voices

| English | Hebrew | Dialogue evidence and editorial direction |
|---|---|---|
| Space Quest / SQ | מסע בחלל | Use the full name with every episode number, including the time-travel status line. |
| Roger Wilco | רוג'ר וילקו | Male. Talker 7, 16/2 and 557/1–7. Hesitant in the bar, quick with a joke, then struggles for words when he meets his son. |
| Narrator | המספר | Talker 99, 60/0–16. Address Roger in masculine singular. Let a straight description set up the joke. |
| Sludge Vohaul | סלאדג' ווהול | Male. Talker 6, 17/0–3. Formal threats that give way to wounded pride. |
| Zondra | זונדרה | Female. Talker 19, 321/0 and 322/5–21. Gives crisp orders, addresses Roger in masculine singular, then speaks to her companions in feminine plural. |
| Maebot | מייבוט | Female voice. Talker 9, 371/14–41 and voice credit 21/8. Moves from a sales pitch to blunt demands for payment. |
| Roger Wilco Jr. | רוג'ר וילקו הבן | Male. Talker 31, 557/8–28. Use the family relationship only after the reveal. The farewell slows down before “אבא”. |
| Xenon / Estros / Magmetheus | קסנון / אסטרוס / מגמתאוס | Locations at 9/0, 531/1–3, 557/16. |
| PocketPal / Time Pod | פוקטפאל / תא הזמן | Inventory 0/63–68 and keypad 531/12–23. Keep the interface label ENT where it is a puzzle clue. |

The broad draft used local Argos Translate en→he 1.5 with CTranslate2 on CPU, INT8, beam size 3. Codex (GPT-6; exact serving variant and reasoning effort unavailable) drafted or revised the entries listed in editorial-pass.json. The remaining model draft has not had a line-by-line human edit. The extra TEXT entries in text.json were drafted by Codex. Internal identifiers and diagnostic strings are preserved where needed.

The English box uses the [original SpaceQuest.Net retail scan](https://spacequest.net/misc/boxscans/sq4front.jpg). The Hebrew box is an image-generated localization of that front. It is a fan edition; the hardware sticker on the source scan identifies the Macintosh package. The supported game files are from the Steam CD edition.
