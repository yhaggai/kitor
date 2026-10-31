using System;
using UndertaleModLib.Models;
EnsureDataLoaded();
if ((Data.GeneralInfo.Info & UndertaleGeneralInfo.InfoFlags.SteamEnabled) == 0 || Data.GeneralInfo.SteamAppID != 391540) throw new Exception("Steam settings missing");
ScriptMessage("PASS: Steam enabled, app ID 391540.");
