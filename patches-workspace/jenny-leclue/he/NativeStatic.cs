using System;
using JLC;
using TMPro;
using UnityEngine;
using RTLTMPro;

// Native Arabic bidi for the menu/typewriter keys migrated to logical order.
// Other static screens keep their existing independent layout until migrated.
public static class NativeStatic {
    public static bool Hebrew {
        get { var m=JLCEngine.localisationManager;
            return m && m.CurrentLanguage!=null && (m.CurrentLanguage.languageCode=="he" || m.CurrentLanguage.languageCode=="he-x-nikud"); }
    }
    public static string Process(string text,string key) {
        if (!Hebrew || !NativeStaticKeys.Keys.Contains(key)) return text;
        var buffer=new FastStringBuilder(Math.Max(2048,text.Length*2));
        NativeDialogue.FixRTL(text,buffer,false,true,true);
        buffer.Reverse();
        return NativeDialogue.Encode(buffer.ToString());
    }
    public static void Refresh(LocalisedText label) {
        if(!NativeStaticKeys.Keys.Contains(label.key))return;
        var text=label.GetComponent<TMP_Text>();if(!text)return;
        var state=label.GetComponent<NativeStaticAlignment>();
        if(!state)state=label.gameObject.AddComponent<NativeStaticAlignment>();
        state.Apply(text,Hebrew);
        NativeDialogue.Configure(text);
    }
    // Dynamic option values bypass LocalisedText.OnChangeLanguage.
    // Numeric resolutions and Latin language names retain LTR rendering.
    public static void SetMenuText(TMP_Text text,string value) {
        if (Hebrew) {
            bool rtl=false;
            foreach(char c in value) if ((c>='\u0590' && c<='\u05ff') || (c>='\ue000' && c<='\uf8ff')) {rtl=true;break;}
            var state=text.GetComponent<NativeStaticAlignment>();
            if(!state)state=text.gameObject.AddComponent<NativeStaticAlignment>();
            state.Apply(text,rtl);
            text.isRightToLeftText=rtl;
        } else {
            var state=text.GetComponent<NativeStaticAlignment>();
            if(state)state.Apply(text,false);
            NativeDialogue.Configure(text);
        }
        text.SetText(value);
    }
    public static string TypewriterLanguage(string language) {
        return language=="he" || language=="he-x-nikud" ? "ar" : language;
    }
}
public class NativeStaticAlignment:MonoBehaviour {
    bool captured;TextAlignmentOptions original;
    public void Apply(TMP_Text text,bool hebrew) {
        if(!captured){original=text.alignment;captured=true;}
        int value=(int)original;
        // Preserve centering and vertical anchors; anchor RTL body text at right.
        text.alignment=hebrew && (value&255)==1 ? (TextAlignmentOptions)((value&~255)|4) : original;
    }
}
