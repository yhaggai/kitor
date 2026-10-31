using System;
using System.IO;
using System.Linq;
using System.Collections.Generic;
using System.Text.Json;
using UndertaleModLib.Util;
EnsureDataLoaded();
string output = Path.Combine(Environment.GetEnvironmentVariable("HEBREW_PATCH_ROOT"), "full-patch", "fonts");
Directory.CreateDirectory(output);
var metadata = new List<object>();
using (var worker = new TextureWorker()) {
 foreach (var f in Data.Fonts) {
  string name=f.Name.Content;
  worker.ExportAsPNG(f.Texture, Path.Combine(output,name+".png"));
  using(var w=new StreamWriter(Path.Combine(output,name+".csv")))
   foreach(var g in f.Glyphs) w.WriteLine($"{g.Character};{g.SourceX};{g.SourceY};{g.SourceWidth};{g.SourceHeight};{g.Shift};{g.Offset}");
  var props=new Dictionary<string,string>();
  foreach(var prop in f.GetType().GetProperties()) {
    if(prop.GetIndexParameters().Length!=0) continue;
    var value=prop.GetValue(f);
    if(value!=null && (value is string || value.GetType().IsPrimitive || value.GetType().IsEnum || prop.Name=="DisplayName")) props[prop.Name]=value.ToString();
  }
  metadata.Add(new {name, glyphCount=f.Glyphs.Count, properties=props});
 }
}
File.WriteAllText(Path.Combine(output,"metadata.json"),JsonSerializer.Serialize(metadata,new JsonSerializerOptions {WriteIndented=true}));
ScriptMessage("Exported "+Data.Fonts.Count+" fonts. No game archive written.");
