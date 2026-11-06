using JLC;
using UnityEngine;
public static class HebrewShortcut {
    public static void Draw(MainMenu menu) {
        if (!menu.menuCanvas || !menu.menuCanvas.enabled) return;
        var manager = JLCEngine.localisationManager;
        if (!manager || manager.CurrentLanguage == null) return;
        string current = manager.CurrentLanguage.languageCode;
        bool hebrew = current == "he";
        string previous = PlayerPrefs.GetString("JennyHebrewPreviousLanguage", "en");
        string next = hebrew ? previous : "he";
        var target = JLCEngine.settingsManager.FindLanguage(next);
        if (target == null) return;
        float scale = Mathf.Clamp(Screen.height / 540f, 0.7f, 4f);
        var rect = new Rect(24*scale, Screen.height-90*scale, 148*scale, 30*scale);
        var style = new GUIStyle(GUI.skin.button);
        style.fontSize = Mathf.RoundToInt(15*scale);
        if (!GUI.Button(rect, hebrew ? target.englishName : "Hebrew / תירבע", style)) return;
        if (!hebrew) PlayerPrefs.SetString("JennyHebrewPreviousLanguage", current);
        manager.ChangeLanguage(target);
        Options.optionsData.language = target.languageCode;
        Options.SaveToPrefs();
        PlayerPrefs.Save();
    }
}
