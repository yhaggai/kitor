// Edition and testing notes from the corresponding registry entries and patch READMEs.
type LocalizedNote = Record<"he" | "en", string>;

export const gameGuideNotes: Record<string, LocalizedNote> = {
  undertale: {
    he: "התרגום מיועד לגרסת Steam ל־Windows ול־macOS ודורש Python 3.10 ומעלה. זו גרסת אלפא ללא ניקוד. הטקסט שמופיע בתוך תמונות נשאר באנגלית, והמשחק לא נבדק ב־Windows. אחרי ההתקנה השאירו את שפת המשחק על English.",
    en: "The patch targets the Steam editions for Windows and macOS and requires Python 3.10 or later. This is a draft without vowel marks. Text embedded in images stays in English, and the patch has yet to be tested in-game on Windows. Keep English selected in the game settings.",
  },
  "monkey-se": {
    he: "המהדורה המיוחדת זמינה ב־Steam וב־GOG. כדי לשחק בגרסה הקלאסית בעברית, בחרו בקיטור את הערך Classic. המשחק המתורגם רץ דרך ScummVM.",
    en: "The Special Edition is available on Steam and GOG. Choose the Classic entry in Kitor to play the Hebrew translation of the original game through ScummVM.",
  },
  sq1: {
    he: "צריך את גרסת DOS EGA 2.2 מאוסף Space Quest 1–3 ב־GOG, וכן Python 3 וגרסת ScummVM יומית שתומכת בעברית במשחקי AGI. החבילה הוכנה ל־Windows ול־macOS ובדיקות משחק בשתיהן עדיין ממתינות.",
    en: "Use DOS EGA 2.2 from the GOG Space Quest 1–3 collection, Python 3, and a ScummVM daily build with Hebrew AGI support. The package was prepared for Windows and macOS. Gameplay testing is pending on both.",
  },
  toonstruck: {
    he: "התרגום הקהילתי עדיין חלקי ומיועד למהדורת GOG. הוא דורש גרסת ScummVM שתומכת בעברית במשחק. החבילה הוכנה ל־Windows ול־macOS ובדיקות משחק בשתיהן עדיין ממתינות.",
    en: "The community translation is unfinished and targets the GOG edition. It requires a ScummVM build with Hebrew support for Toonstruck. The package was prepared for Windows and macOS. Gameplay testing is pending on both.",
  },
  "kathy-rain-dc": {
    he: "התרגום הניסיוני מיועד למהדורת Director’s Cut ב־Windows בלבד וטרם נבדק במשחק. לפי המתרגמים, כדאי לעבור זמנית לאנגלית בחידת הקלטת בפרק 2 ובחידת המנעול האחרונה.",
    en: "The experimental patch targets Director’s Cut on Windows only. Gameplay testing is pending. The translators recommend switching to English temporarily for the chapter 2 tape puzzle and the final lock puzzle.",
  },
};
