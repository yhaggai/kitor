// Font-only import. No code changes, archive output, or app mutation here.
using System;
using System.IO;
using System.Linq;
using UndertaleModLib.Util;
using UndertaleModLib.Models;
EnsureDataLoaded();
string fontRoot = Path.Combine(Environment.GetEnvironmentVariable("HEBREW_PATCH_ROOT"), "full-patch");
int fontsUpdated=0, glyphsAdded=0;
foreach (var path in Directory.GetFiles(Path.Combine(fontRoot,"fonts"),"*.csv").OrderBy(x=>x)) {
 string name=Path.GetFileNameWithoutExtension(path);
 var font=Data.Fonts.ByName(name);
 if(font==null) throw new Exception("Font missing from archive: "+name);
 // Verify original glyph metrics before extending; reject a mismatched/repatched archive.
 foreach(var line in File.ReadAllLines(path)) {
  var a=Array.ConvertAll(line.Split(';'),int.Parse);
  var originals=font.Glyphs.Where(g=>g.Character==a[0]).ToArray();
  if(originals.Length!=1) throw new Exception("Original glyph missing/duplicated: "+name+" "+a[0]);
  var g=originals[0];
  if(g.SourceX!=a[1] || g.SourceY!=a[2] || g.SourceWidth!=a[3] || g.SourceHeight!=a[4] || g.Shift!=a[5] || g.Offset!=a[6])
   throw new Exception("Original glyph metric mismatch: "+name+" "+a[0]);
 }
 string generated=Path.Combine(fontRoot,"generated-fonts");
 var additions=File.ReadAllLines(Path.Combine(generated,name+".csv")).Where(x=>x.Length>0).Select(x=>Array.ConvertAll(x.Split(';'),int.Parse)).ToArray();
 if(additions.Any(a=>font.Glyphs.Any(g=>g.Character==a[0]))) throw new Exception("Font already patched or contains duplicate addition: "+name);
 var texture=new UndertaleEmbeddedTexture();
 texture.Name=Data.Strings.MakeString("he_full_"+name);
 texture.TextureData.Image=GMImage.FromPng(File.ReadAllBytes(Path.Combine(generated,name+".png")));
 var image=texture.TextureData.Image;
 if(image.Width>ushort.MaxValue || image.Height>ushort.MaxValue) throw new Exception("Font texture too large: "+name);
 Data.EmbeddedTextures.Add(texture);
 var page=new UndertaleTexturePageItem();
 page.Name=Data.Strings.MakeString("he_full_"+name);
 page.SourceWidth=page.TargetWidth=page.BoundingWidth=(ushort)image.Width;
 page.SourceHeight=page.TargetHeight=page.BoundingHeight=(ushort)image.Height;
 page.TexturePage=texture;
 Data.TexturePageItems.Add(page);
 font.Texture=page;
 foreach(var a in additions) {
  if(a.Length!=7 || a[0]<0 || a[0]>ushort.MaxValue || a[1]+a[3]>image.Width || a[2]+a[4]>image.Height)
   throw new Exception("Invalid glyph addition: "+name);
  font.Glyphs.Add(new UndertaleFont.Glyph {
   Character=(ushort)a[0],SourceX=(ushort)a[1],SourceY=(ushort)a[2],
   SourceWidth=(ushort)a[3],SourceHeight=(ushort)a[4],Shift=(short)a[5],Offset=(short)a[6]
  });
 }
 var sorted=font.Glyphs.OrderBy(g=>g.Character).ToArray();
 if(sorted.Select(g=>g.Character).Distinct().Count()!=sorted.Length) throw new Exception("Duplicate merged glyphs: "+name);
 font.Glyphs.Clear();foreach(var glyph in sorted)font.Glyphs.Add(glyph);
 font.RangeStart=Math.Min(font.RangeStart,sorted.First().Character);
 font.RangeEnd=Math.Max(font.RangeEnd,(uint)sorted.Last().Character);
 foreach(int code in Enumerable.Range(0x5d0,27))
  if(!font.Glyphs.Any(g=>g.Character==code))throw new Exception("Missing Hebrew glyph: "+name);
 fontsUpdated++;glyphsAdded+=additions.Length;
}
if(fontsUpdated!=20)throw new Exception("Expected 20 font imports, got "+fontsUpdated);
ScriptMessage("Full Hebrew fonts applied in memory: "+fontsUpdated+" fonts, "+glyphsAdded+" additions. Original glyph metrics retained.");
