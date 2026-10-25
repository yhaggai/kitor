using System;
using System.IO;
using System.Linq;
using UndertaleModLib;
using UndertaleModLib.Models;
using UndertaleModLib.Compiler;
EnsureDataLoaded();
string root=Path.Combine(Environment.GetEnvironmentVariable("HEBREW_PATCH_ROOT"), "full-patch");
// Enable Steam integration in the generated macOS build.
Data.GeneralInfo.Info |= UndertaleGeneralInfo.InfoFlags.SteamEnabled;
Data.GeneralInfo.SteamAppID = 391540;
CodeImportGroup group=new(Data){MainThreadAction=MainThreadAction,AutoCreateAssets=true};
foreach(var p in Directory.GetFiles(Path.Combine(root,"hebrew-runtime"),"*.gml").OrderBy(x=>x))
 group.QueueReplace("gml_Script_"+Path.GetFileNameWithoutExtension(p),File.ReadAllText(p));
group.QueueReplace(Data.Code.ByName("gml_Script_textdata_en"),File.ReadAllText(Path.Combine(root,"gml_Script_textdata_en.gml")));
foreach(var name in new[]{"gml_Object_obj_base_writer_Draw_0","gml_Object_obj_base_writer_Create_0"})
 group.QueueReplace(Data.Code.ByName(name),File.ReadAllText(Path.Combine(root,name+".gml")));
var obj=Data.GameObjects.ByName("obj_base_writer");
group.QueueReplace(obj.EventHandlerFor(EventType.Destroy,Data),File.ReadAllText(Path.Combine(root,"gml_Object_obj_base_writer_Destroy_0.gml")));
group.Import();
ScriptMessage("Full Hebrew catalog and runtime layout scripts compiled.");
