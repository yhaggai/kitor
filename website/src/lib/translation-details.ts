type Localized = readonly [he: string, en: string];

export const translationDetails: Partial<Record<string, {
  scope?: Localized;
  status?: Localized;
  edition?: string;
}>> = {
  undertale: {
    scope: ["טקסט המשחק בעברית ללא ניקוד. הטקסט שבתמונות נשאר באנגלית.", "Game text in Hebrew without vowel marks. Text embedded in images remains in English."],
    status: ["אלפא. בדיקות משחק ב־Windows עדיין ממתינות.", "Alpha. Windows gameplay testing is pending."],
  },
  "monkey-se": {
    scope: ["דיאלוג, שמות דמויות וקטעי גרפיקה ומעבר בגרסה הקלאסית.", "Dialogue, character names, and selected graphics and transitions in the classic game."],
    edition: "Steam / GOG Special Edition → Classic (ScummVM)",
  },
  "monkey2-se": {
    scope: ["טקסט המשחק ורקעים נבחרים בגרסה הקלאסית.", "Game text and selected backgrounds in the classic game."],
    edition: "Steam / GOG Special Edition → Classic (ScummVM)",
  },
  sq1sci: { edition: "VGA Remake (SCI)" },
  "kyra1-cd": { edition: "CD", status: ["בטא.", "Beta."] },
  pq1: { edition: "AGI / EGA · GOG / Steam Collection (ScummVM)" },
  sq1: { edition: "DOS EGA 2.2 · GOG Space Quest 1–3 / Steam Collection (ScummVM)" },
  sq2: { edition: "AGI · GOG / Steam Collection (ScummVM)" },
  sq3: { edition: "SCI · GOG / Space Quest Collection (ScummVM 2.2.0+)" },
  sq4: {
    scope: ["דיאלוג, תפריטים וגרפיקה.", "Dialogue, menus, and graphics."],
    status: ["טיוטה שעברה עריכה חלקית.", "Partially edited draft."],
    edition: "Steam CD",
  },
  sq5: {
    edition: "Steam Space Quest Collection (ScummVM)",
    status: ["נותרו בעיות גרפיות קלות. בדיקת משחק עדיין ממתינה.", "Minor graphical issues remain. Gameplay testing is pending."],
  },
  torin: {
    scope: ["כתוביות בעברית. הפעילו אותן בתפריט המשחק לאחר תחילת המשחק.", "Hebrew subtitles. Enable them from the in-game menu after play begins."],
    edition: "GOG (ScummVM)",
  },
  spyfox: { edition: "Steam (ScummVM)", status: ["תרגום בתהליך.", "Translation in progress."] },
  pajama: { edition: "GOG (ScummVM)", status: ["תרגום בתהליך.", "Translation in progress."] },
  toonstruck: { edition: "GOG (ScummVM)", status: ["תרגום חלקי.", "Partial translation."] },
  gob2: { edition: "GOG DOS CD v1.02 (ScummVM)" },
  tlj: { edition: "GOG (ScummVM Stark)" },
  sky: { edition: "Freeware (ScummVM)" },
  "stanley-parable": {
    scope: ["החבילה כוללת כתוביות בעברית בלשון זכר.", "The patch includes Hebrew subtitles in masculine grammatical form."],
    edition: "Steam (2013)",
  },
  "tesla-effect": { edition: "Steam" },
  "ethan-carter": { edition: "Original (UE3)" },
  "life-is-strange": {
    scope: ["החבילה מיועדת לפרקים 1–3.", "The patch covers episodes 1–3."],
    edition: "Original · Episodes 1–3",
  },
  portal2: {
    status: ["חבילה ניסיונית. הצגת העברית במשחק עדיין לא אומתה.", "Experimental patch. Hebrew rendering has yet to be verified in-game."],
    edition: "Steam",
  },
  "journey-down1": { edition: "Chapter One" },
  "kathy-rain-dc": { edition: "Director’s Cut" },
  drifter: {
    scope: ["טקסט המשחק שנכלל בקטלוג התרגום המובנה.", "Game text in the built-in translation catalog."],
    status: ["בדיקת משחק עדיין ממתינה.", "Gameplay testing is pending."],
    edition: "Steam",
  },
  owlboy: {
    scope: ["תרגום מלא של הטקסט עם גופן עברי וכתיבה מימין לשמאל. אפשר לבחור גרסה עם ניקוד.", "Full text translation with a Hebrew font and right-to-left layout. Vowel marks are optional."],
    status: ["בטא. בדיקת משחק מלאה עדיין ממתינה.", "Beta. A full playthrough is pending."],
    edition: "Steam FNA",
  },
  tentacle: { edition: "Classic / Remastered game files (ScummVM)" },
  "day-of-the-tentacle-remastered": {
    scope: ["5,142 כתוביות ו־348 שמות פריטים ודמויות. תפריטים, אפשרויות שיחה ו־579 רשומות דיבור עדיין באנגלית.", "5,142 subtitle entries and 348 item and character names. Menus, dialogue choices, and 579 speech entries remain in English."],
    status: ["אבטיפוס חלקי.", "Partial prototype."],
    edition: "Remastered",
  },
  "grim-fandango-remastered": {
    scope: ["החבילה כוללת 401 רשומות כתוביות לסרטוני המשחק. הדיבור נשאר באנגלית.", "Includes 401 movie subtitle entries. Speech remains in English."],
    edition: "GOG Remastered",
  },
  "monkey-se-native": {
    scope: ["כתוביות ותפריטים, כולל כתוביות במצב הקלאסי. שורות דיאלוג שלא נמצאה להן התאמה נשארו באנגלית.", "Subtitles and menus, including classic-mode subtitles. Unmatched dialogue remains in English."],
    edition: "Special Edition",
  },
  "monkey2-se-native": {
    scope: ["כתוביות ותפריטים, כולל כתוביות במצב הקלאסי. שורות דיאלוג שלא נמצאה להן התאמה נשארו באנגלית.", "Subtitles and menus, including classic-mode subtitles. Unmatched dialogue remains in English."],
    edition: "Special Edition",
  },
  "silver-pines": {
    scope: ["דיאלוג, תפריטים ורמזים.", "Dialogue, menus, and clues."],
    status: ["טיוטה שנוצרה בסיוע תרגום מכונה ועברה עריכה. בדיקת משחק מלאה עדיין ממתינה.", "Machine-assisted, edited draft. A full playthrough is pending."],
    edition: "Steam",
  },
  "jenny-leclue": {
    scope: ["5,511 רשומות דיאלוג ו־1,931 פריטי ממשק, ובהם תפריטים, יומנים, רמזים ומכתבים.", "5,511 dialogue entries and 1,931 UI entries, including menus, journals, clues, and letters."],
    status: ["תרגום מלא. בדיקת משחק מלאה ובדיקת הרצה ב־Windows עדיין ממתינות.", "Full translation. A full playthrough and native Windows gameplay verification are pending."],
  },
  "disco-elysium": {
    scope: ["75,021 רשומות, כולל כיתובי פתיחה, שמירה וממשק. בחלק מהתוכן עדיין מופיעה ערבית.", "75,021 entries, including startup, saving, and interface labels. Some content can still appear in Arabic."],
    status: ["תרגום חלקי.", "Partial translation."],
    edition: "Steam · The Final Cut",
  },
  "moomin-midsummer-madness": {
    scope: ["דיאלוג, תפריטים, יומן וקרדיטים, עם ניקוד וגופן עברי.", "Dialogue, menus, journal, and credits, with vowel marks and a Hebrew font."],
    status: ["התרגום זמין. הגהה ספרותית מלאה עדיין ממתינה.", "Available. Full literary proofreading is pending."],
    edition: "Steam",
  },
};

export const platformNames: Record<string, string> = {
  windows: "Windows",
  osx: "macOS",
  linux: "Linux",
};

export const storeNames: Record<string, string> = { steam: "Steam", gog: "GOG" };
