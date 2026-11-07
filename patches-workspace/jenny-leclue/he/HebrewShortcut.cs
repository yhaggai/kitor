using JLC;
using UnityEngine;
using TMPro;
using System.Collections.Generic;

public static class HebrewShortcut {
    static bool initialized;

    // The first publisher letter's English layout uses baked-in artwork and
    // disables Other_Langs. Unknown languages otherwise fall back to English.
    // Reuse its authored RTL layout; text lookup still uses the actual language.
    public static string LetterLayoutLanguage(string language, LocalisedLayout layout) {
        foreach (var item in layout.textItems) {
            if (!item.tmpro || item.tmpro.name != "TMP_Letter1F_Body1") continue;
            bool hebrew = language == "he" || language == "he-x-nikud";
            var alignment = layout.GetComponent<HebrewLetterAlignment>();
            if (!alignment && hebrew)
                alignment = layout.gameObject.AddComponent<HebrewLetterAlignment>();
            if (alignment) alignment.Apply(layout, hebrew);
            return hebrew ? "ar" : language;
        }
        return language;
    }

    // Called by the existing MainMenu.OnGUI hook; no overlay is drawn.
    public static void Draw(MainMenu menu) {
        if (initialized || !menu.menuCanvas || !menu.menuCanvas.enabled) return;
        var manager = JLCEngine.localisationManager;
        var settings = JLCEngine.settingsManager;
        if (!manager || !settings || manager.CurrentLanguage == null) return;
        var target = settings.FindLanguage("he");
        if (target == null) return;

        if (manager.CurrentLanguage.languageCode != target.languageCode)
            manager.ChangeLanguage(target);
        Options.optionsData.language = target.languageCode;
        Options.SaveToPrefs();
        PlayerPrefs.Save();
        // Allow subsequent changes through the game's own language settings.
        initialized = true;
    }
}

// Keep original alignments with the letter instance so switching languages
// restores them and unloading the scene releases the saved references.
public class HebrewLetterAlignment : MonoBehaviour {
    readonly Dictionary<TMP_Text, TextAlignmentOptions> originals =
        new Dictionary<TMP_Text, TextAlignmentOptions>();

    public void Apply(LocalisedLayout layout, bool hebrew) {
        foreach (var item in layout.textItems) {
            var text = item.tmpro;
            if (!text) continue;
            if (!originals.ContainsKey(text)) originals.Add(text, text.alignment);
            int original = (int)originals[text];
            // Replace only left horizontal alignment. Preserve vertical anchors
            // and intentionally centered graph labels or other captions.
            text.alignment = hebrew && (original & 0xff) == 1
                ? (TextAlignmentOptions)((original & ~0xff) | 4)
                : originals[text];
        }
    }
}
