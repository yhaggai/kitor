<p align="center">
  <img src="docs/images/kitor-logo-animated.gif" width="640" height="320" alt="Kitor logo — a golden chimney with a curl of steam / סמל קיטור — ארובה זהובה עם סליל אדים" />
</p>

<h1 align="center">קיטור · Kitor</h1>

<p align="center">המשחקים שלכם. התרגומים של הקהילה. מקום אחד לכולם.<br />Your games. Community translations. One place for both.</p>

<p align="center"><a href="#hebrew">עברית</a> · <a href="#english">English</a> · <a href="https://github.com/yhaggai/kitor/releases">הורדות / Downloads</a></p>

<p align="center">
  <img src="docs/images/kitor-home.jpg" width="960" alt="Kitor Home showing Hebrew game covers in Classics, Detectives, and Thriller / מסך הבית של קיטור עם עטיפות בעברית בקטגוריות קלאסיקות, בלשים ומתח" />
</p>

<a id="hebrew"></a>
<div dir="rtl" lang="he">

## מה זה קיטור?

קיטור הוא יישום למחשב שמחבר בין ספריית המשחקים שלכם לבין תרגומים לעברית שנוצרו בקהילה. המטרה היא להפוך את הדרך מגילוי תרגום ועד למשחק בעברית לפשוטה ונגישה: למצוא משחק נתמך, להתקין את התרגום ולשחק מתוך מקום אחד.

הפרויקט מרכז משחקי הרפתקאות קלאסיים ומשחקים נוספים, ומשלב הפעלה באמצעות ScummVM או באמצעות המשחק עצמו, בהתאם לכותר ולגרסה. קיטור נועד להנגיש את עבודת המתרגמים; הקרדיט על כל תרגום שייך ליוצריו.

### מה אפשר לעשות?

- **לרכז את הספרייה:** לסרוק תיקיות ולזהות התקנות נתמכות, כולל משחקים מ־Steam ומ־GOG.
- **לגלות תרגומים:** לעיין בקטלוג המשחקים והתרגומים לעברית.
- **להתקין תרגום:** להוריד ולהחיל חבילת תרגום שמתאימה למשחק ולגרסה הנתמכת.
- **להסיר ולשחזר:** לגבות את הקבצים שהתרגום משנה ולעקוב אחרי קבצים שנוספו, כדי לאפשר הסרה ושחזור של התקנה שנוהלה בקיטור.
- **לשחק מתוך הספרייה:** להפעיל משחקים באמצעות מסלול ההפעלה שהוגדר להם.

### מתחילים לשחק

1. מוסיפים את תיקיית המשחק או סורקים התקנות קיימות.
2. בוחרים תרגום מתאים וקוראים את הדרישות וההנחיות שלו.
3. סוגרים את המשחק, מתקינים את התרגום ומפעילים מהספרייה.

צריך עותק משלכם של המשחק; קיטור אינו מספק את המשחקים עצמם. התמיכה ב־Windows וב־macOS, מהדורות המשחק ודרישות ההפעלה משתנות בין כותרים ותרגומים. יש לבדוק את פרטי המשחק באפליקציה; הופעה בקטלוג אינה מבטיחה שכל גרסה נבדקה במלואה.

### פיתוח ותרומה

להרצת היישום בפיתוח נדרשים Bun, Rust ודרישות הבנייה של Tauri למערכת ההפעלה שלכם. מריצים את הפקודות הבאות משורש המאגר:

</div>

```sh
bun install
cd kitor
bun run tauri dev
```

<div dir="rtl" lang="he">

פרטי פיתוח נוספים נמצאים ב[תיעוד היישום](kitor/README.md), [תיעוד האתר](website/README.md) ו[סביבת העבודה לתרגומים](patches-workspace/README.md).

אפשר לתרום באמצעות דיווחי תקלות, בדיקות תאימות, שיפורי נגישות, קוד ועדכוני קטלוג. בדיווח על בעיה כדאי לציין את מערכת ההפעלה, שם המשחק, החנות או המהדורה, גרסת התרגום והשלבים לשחזור. ניתן לפתוח [Issue](https://github.com/yhaggai/kitor/issues) או Pull Request.

תודה לקהילת [הרפתקאה עברית](https://www.facebook.com/groups/hebrewadventure), למתרגמים ולמתרגמות ולכל הקהילות שמנגישות משחקים בעברית, ולפרויקט ScummVM שמאפשר להריץ רבים ממשחקי ההרפתקאות הקלאסיים. המשחקים, האיורים והתרגומים שייכים לבעלי הזכויות שלהם; יש לכבד את תנאי השימוש של כל מקור.

</div>

---

<a id="english"></a>

## What is Kitor?

Kitor is a desktop app that connects your game library with community-made Hebrew translations. Its purpose is to make the journey from discovering a translation to playing in Hebrew simple and accessible: find a supported game, install its translation, and launch it from one place.

The project brings together classic adventures and other games, using ScummVM or the game's own runtime depending on the title and edition. Kitor helps people access translators' work; credit for each translation belongs to its creators.

### What can it do?

- **Organize your library:** scan folders and detect supported installations, including games from Steam and GOG.
- **Discover translations:** browse a catalog of games and Hebrew translation patches.
- **Install a translation:** download and apply a patch for a supported game and edition.
- **Uninstall and restore:** back up files affected by a patch and track added files, enabling removal and restoration of installations managed by Kitor.
- **Play from your library:** launch each game through its configured runtime.

### Start playing

1. Add your game folder or scan existing installations.
2. Choose a compatible translation and read its requirements and instructions.
3. Close the game, install the translation, and launch it from your library.

You need your own copy of the game; Kitor does not supply the games themselves. Windows and macOS support, game editions, and runtime requirements vary by title and patch. Check the game's details in the app; a catalog entry does not mean every version has been fully tested.

### Development and contributions

To run the desktop app in development, install Bun, Rust, and the Tauri build prerequisites for your operating system. From the repository root:

```sh
bun install
cd kitor
bun run tauri dev
```

See the [desktop app documentation](kitor/README.md), [website documentation](website/README.md), and [patch workspace guide](patches-workspace/README.md) for more details.

Contributions can include bug reports, compatibility testing, accessibility improvements, code, and catalog updates. When reporting a problem, include your operating system, game title, store or edition, patch version, and steps to reproduce it. Open an [issue](https://github.com/yhaggai/kitor/issues) or a pull request.

Thanks to the [הרפתקאה עברית (Hebrew Adventure)](https://www.facebook.com/groups/hebrewadventure) community, and to all translators and communities making games accessible in Hebrew, and to ScummVM for making so many classic adventures playable. Games, artwork, and translations belong to their respective rights holders; respect each source's terms of use.

## License and code signing

Original code by Haggai Yaniv is available under the [MIT license](LICENSE).
See [licensing scope and third-party exclusions](LICENSING.md); community
translations, game assets, and upstream components retain their own terms.

[Code signing policy](CODE_SIGNING_POLICY.md) · [Desktop privacy policy](PRIVACY.md)

SignPath application preparation is in progress; SignPath signing is not yet enabled.
