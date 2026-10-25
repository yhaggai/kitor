// Integration fragment. Requires he_visual already imported or queued in the same group.
// No archive writes here. Root saves the assembled copy using the CLI -o option.
using System;
using System.IO;
using System.Linq;
using System.Collections.Generic;
using UndertaleModLib;
using UndertaleModLib.Compiler;
using UndertaleModLib.Models;
EnsureDataLoaded();
string renderRoot = Path.Combine(Environment.GetEnvironmentVariable("HEBREW_PATCH_ROOT"), "full-patch");
var renderVariants = new Dictionary<string,ushort> { {"draw_text",3}, {"draw_text_transformed",6}, {"draw_text_transformed_color",11}, {"draw_text_ext",5} };
// Snapshot originals before adding wrappers; never redirect wrapper->builtin calls.
var renderOriginals = Data.Code.Where(c=>!c.Name.Content.StartsWith("gml_Script_he_") && !c.Name.Content.StartsWith("gml_Object_obj_base_writer_") && c.Name.Content != "gml_Script_scr_drawtext_icons" && c.Name.Content != "gml_Script_scr_drawtext_icons_multiline").ToArray();
CodeImportGroup renderGroup = new(Data) { MainThreadAction = MainThreadAction, AutoCreateAssets = true };
foreach(var file in Directory.GetFiles(Path.Combine(renderRoot,"render-gml"),"*.gml").OrderBy(s=>s))
    renderGroup.QueueReplace(Path.GetFileNameWithoutExtension(file), File.ReadAllText(file));
foreach(var file in Directory.GetFiles(Path.Combine(renderRoot,"render-special"),"*.gml").OrderBy(s=>s))
    renderGroup.QueueReplace(Path.GetFileNameWithoutExtension(file), File.ReadAllText(file));
renderGroup.Import();
int renderCount = 0;
foreach(var c in renderOriginals) foreach(var ins in c.Instructions) {
    var name = ins.ValueFunction?.Name?.Content;
    if(ins.Kind != UndertaleInstruction.Opcode.Call || name == null || !renderVariants.ContainsKey(name)) continue;
    if(ins.ArgumentsCount != renderVariants[name]) throw new Exception("Unexpected draw-text arity in "+c.Name.Content+": "+name+" / "+ins.ArgumentsCount);
    var fn = Data.Functions.EnsureDefined("he_"+name, Data.Strings);
    if(fn == null) throw new Exception("Wrapper function not registered: he_"+name);
    ins.ValueFunction = fn;
    renderCount++;
}
ScriptMessage("Redirected "+renderCount+" original draw-text call instructions without recompiling their code entries.");
