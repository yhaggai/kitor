using System;
using System.Text;
using System.Text.RegularExpressions;
using JLC;
using TMPro;
using RTLTMPro;

// Apply Arabic's existing processing only to Hebrew dialogue/choices. Static
// labels retain their existing visual-order data and font configuration.
public static class NativeDialogue {
    static bool Hebrew {
        get { var manager=JLCEngine.localisationManager;
              if (!manager || manager.CurrentLanguage==null) return false;
              var lang=manager.CurrentLanguage.languageCode;
              return lang=="he" || lang=="he-x-nikud"; }
    }
    public static bool IsRtl(FontSet fonts) { return Hebrew || (fonts && fonts.isRightToLeft); }
    public static void Configure(TMP_Text text) {
        var manager=JLCEngine.localisationManager;
        if (!text || !manager || manager.CurrentLanguage==null) return;
        text.isRightToLeftText=IsRtl(manager.CurrentLanguage.fontSet);
    }
    // Map generated atlas symbols back to their source clusters before bidi.
    // Otherwise the Arabic processor classifies private-use symbols as Latin.
    public static void FixRTL(string text, FastStringBuilder output, bool farsi, bool tags, bool numbers) {
        if (Hebrew) {
            var decoded=new StringBuilder();
            foreach(char c in text) {
                int index=c-0xe000;
                decoded.Append(index>=0 && index<NativeGlyphs.Clusters.Length ? NativeGlyphs.Clusters[index] : c.ToString());
            }
            text=decoded.ToString();
        }
        RTLSupport.FixRTL(text,output,farsi,tags,numbers);
    }
    // Called after the game's FastStringBuilder.Reverse(). Keep each pointed
    // letter a single reveal unit, exactly as in the existing atlas.
    public static string Encode(string text) {
        if (!Hebrew) return text;
        return Regex.Replace(text,@"[א-ת][\u05b0-\u05bd\u05bf\u05c1\u05c2\u05c7]+", m => {
            int index=Array.BinarySearch(NativeGlyphs.Clusters,m.Value,StringComparer.Ordinal);
            return index>=0 ? ((char)(0xe000+index)).ToString() : m.Value;
        });
    }
}
