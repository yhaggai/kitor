!include nsDialogs.nsh
!include LogicLib.nsh

; Tauri includes this file before its welcome page, so this disclosure also
; precedes installation of the WebView2 prerequisite.
Page custom KitorPrivacyPage

Function KitorPrivacyPage
  ; Tauri's updater uses /P. Silent installations skip pages automatically.
  ${GetOptions} $CMDLINE "/P" $0
  ${IfNot} ${Errors}
    Abort
  ${EndIf}

  nsDialogs::Create 1018
  Pop $0
  ${If} $0 == error
    Abort
  ${EndIf}

  ; Windows language ID 1037 is Hebrew.
  ${If} $LANGUAGE == 1037
    nsDialogs::SetRTL 1
    ${NSD_CreateLabel} 0 0 100% 17u "פרטיות ב־Kitor"
    Pop $0
    ${NSD_CreateLabel} 0 22u 100% 85u "חיבור ל־Steam או ל־GOG והתחברות עם Google לצורך תרומות הם אופציונליים ונפתחים מתוך Kitor. אפשר להפעיל בהגדרות רענון אוטומטי של ספריות Steam ו־GOG.$\r$\n$\r$\nKitor בודקת עדכונים וקטלוג תרגומים ומציגה תמונות מרוחקות. אם WebView2 חסר, המתקין יוריד ויתקין אותו מ־Microsoft."
    Pop $0
    ${NSD_CreateLink} 0 115u 100% 14u "מדיניות הפרטיות המלאה"
  ${Else}
    ${NSD_CreateLabel} 0 0 100% 17u "Kitor privacy"
    Pop $0
    ${NSD_CreateLabel} 0 22u 100% 85u "Connecting to Steam or GOG and signing in with Google to contribute are optional. You can start these connections from Kitor and enable automatic Steam or GOG library refresh in Settings.$\r$\n$\r$\nKitor checks for app and translation catalog updates and displays remote artwork. Setup downloads and installs Microsoft's WebView2 runtime if it is missing."
    Pop $0
    ${NSD_CreateLink} 0 115u 100% 14u "Read the full privacy policy"
  ${EndIf}
  Pop $0
  ${NSD_OnClick} $0 KitorOpenPrivacyPolicy
  nsDialogs::Show
FunctionEnd

Function KitorOpenPrivacyPolicy
  Pop $0
  ExecShell "open" "https://github.com/yhaggai/kitor/blob/v1/PRIVACY.md"
FunctionEnd
