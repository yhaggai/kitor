using System;
using System.IO;
using System.Linq;
EnsureDataLoaded();
string root=Path.Combine(Environment.GetEnvironmentVariable("HEBREW_PATCH_ROOT"), "full-patch");
int fonts=0;
foreach(var f in Data.Fonts){
 var expected=File.ReadAllLines(Path.Combine(root,"generated-fonts",f.Name.Content+".all.csv")).Where(x=>x.Length>0).Select(x=>Array.ConvertAll(x.Split(';'),int.Parse)).ToArray();
 if(expected.Length!=f.Glyphs.Count)throw new Exception("Glyph count "+f.Name.Content);
 foreach(var a in expected){var g=f.Glyphs.Single(g=>g.Character==a[0]);if(g.SourceX!=a[1]||g.SourceY!=a[2]||g.SourceWidth!=a[3]||g.SourceHeight!=a[4]||g.Shift!=a[5]||g.Offset!=a[6])throw new Exception("Glyph metrics "+f.Name.Content);}
 for(int c=1488;c<=1514;c++)if(!f.Glyphs.Any(g=>g.Character==c))throw new Exception("Missing Hebrew letter");
 fonts++;
}
foreach(var n in new[]{"he_bidi","he_layout","he_visual","he_draw_text","he_draw_text_ext","he_draw_text_transformed","he_draw_text_transformed_color"})if(!Data.Code.Any(c=>c.Name.Content=="gml_Script_"+n))throw new Exception("Missing helper "+n);
int hooks=Data.Code.SelectMany(c=>c.Instructions).Count(i=>(i.ValueFunction?.Name?.Content??"").StartsWith("he_draw_text"));
if(hooks!=378)throw new Exception("Hook count "+hooks);
ScriptMessage("PASS: reloaded final archive; "+fonts+" complete Hebrew fonts; all glyph metrics match; helpers present; "+hooks+" draw hooks.");
