# Moomin Midsummer Madness Hebrew translation notes

## Source and approach

Character and scene identity come from the game's `GameDatabase` character cards, stable dialogue object IDs, and surrounding dialogue fields. Draft all player-facing English text in natural, contemporary Hebrew with a warm, playful storybook tone. Preserve puzzle clue meaning and interface clarity. Hebrew in the catalog stays in logical reading order; build.py converts it for the game's left-to-right text renderer.

## Character and naming glossary

| English label | Hebrew | Gender / address | Voice and evidence |
|---|---|---|---|
| Moomintroll | מומינטרול; מומין in familiar direct address | masculine | Curious, brave, kind; first-person protagonist. Grounded in `character_moomintroll` and dialogue keys containing `mt` / `moomintroll`. |
| Little My | מאי הקטנה; מאי | feminine | Bold, blunt, mischievous, decisive; keep short punchy lines. Grounded in `character_little_my` and `my` dialogue keys. |
| Moominmamma | מומינאמא | feminine | Warm, practical, reassuring. Grounded in `character_mamma` and family dialogue. |
| Moominpappa | מומינאבא | masculine | Affectionate, imaginative, sometimes grandiose. Grounded in `character_pappa` and family dialogue. |
| Snufkin | סנופקין | masculine | Calm, independent, reflective; understated phrasing. Grounded in `character_snufkin`. |
| Snorkmaiden | סנורקה | feminine | Expressive, affectionate, spirited. Grounded in `character_snorkmaiden`. |
| Mymble | מייבל | feminine | Older, responsible sister; confident and caring. Grounded in `character_mymble`. |
| Edward the Booble | אדוארד הבובל | masculine | Enormous, gentle creature; formal label. Grounded in `character_edward`. |
| Fillyjonk | פיליונק | feminine | Anxious, proper, easily alarmed; retain her fussiness with rhythm. Grounded in `character_fillyjonk`. |
| Misabel | מיסאבל | feminine | Nervous, earnest; grounded in `character_misabel`. |
| Emma | אמה | feminine | Theatrical and commanding, as a theatre rat; grounded in `character_emma`. |
| Park Keeper | שומר הפארק | masculine | Strict, officious; grounded in `character_park_keeper`. |
| Policeman | השוטר | masculine | Official, self-important; grounded in `character_policeman`. |
| Hattifatteners | השמנטפים | plural | Keep this canonical Moomin creature name consistent. |
| The Groke | הגרוק | feminine | Use the established Hebrew Moomin name הגרוק. |
| Woodie / Bushkid / Trainer | וודי / ילד-שיח / מאמן | preserve distinct labels | Forest children; playful, simple speech. Cards `character_artist`, `character_bushkid`, and `character_trainer` share Woodie label but identify distinct portraits, so translate displayed label consistently as וודי. |
| Bird | הציפור | grammatical feminine | Keep as a role label, not a personal name. |
| Spider | העכביש | masculine | Edward is a separate named character. |
| Someone | מישהו / מישהי by context; otherwise ״מישהו״ | unresolved | Preserve mystery and use unmarked Hebrew wherever possible. |

## Translation and markup rules

- Keep proper names above consistent. Preserve technical keys, asset names and credits people/company names in Latin script unless the visible text is a localizable role or heading.
- Use natural Israeli Hebrew and address characters by scene relationship. Match singular/plural and gender in every dialogue exchange.
- Preserve every `{placeholder}`, exact rich-text control token and number. Keep meaningful breaks. Do not visually reverse Hebrew in this catalog.
- The model draft uses OpenAI Codex CLI `gpt-6-luna`, low reasoning effort, in isolated read-only translation batches. Each batch is validated for field count, placeholders, tags and digits; all entries remain editable in `catalog.json`.


## Hebrew naming references

Israeli character-name usage was checked against [Cartoons.co.il's Moomins page](https://www.cartoons.co.il/series/moomins), which uses מומינטרול, מומינאמא, מומינאבא, סנופקין, מאי הקטנה and סנורקה, and the [Ben-Yehuda Project's discussion of the Hebrew Moomin books](https://benyehuda.org/read/21434), which uses השמנטפים. These sources support established Hebrew spellings. The spelling הגרוק is also attested in [Hebrew literary references](https://einat.sf-f.org.il/story/74-%D7%A2%D7%9E%D7%A7-%D7%94%D7%A9%D7%9C%D7%95%D7%95%D7%94/) and Israeli media about the Moomins. Less prominent characters use straightforward transliteration or descriptive role labels.

## Pointed-name spellings

`niqqud-glossary.json` pins the reviewed pronunciations used in the pointed edition while preserving every base letter: Moomintroll מוּמִינְטְרוֹל, Moomin מוּמִין, Little My מַאי הַקְּטַנָּה, Moominmamma מוּמִינְאִמָּא, Moominpappa מוּמִינְאַבָּא, Snufkin סְנוּפְקִין, Snorkmaiden סְנוֹרְקָה, Hattifatteners הַשְּׁמַנְטַפִּים, and the Groke הַגְּרוֹק. The Hebrew name spellings are supported by the references above; the vowel marks are editorial pronunciations for this edition. Other niqqud is model-generated and was checked for exact base-text and markup preservation, not manually proofread line by line.
