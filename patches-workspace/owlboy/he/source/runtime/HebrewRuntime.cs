// Original patch code. The game remains responsible for timing, audio and effects.
using System;
using System.IO;
using System.Text;
using System.Reflection;
using System.Collections.Generic;
using Owlboy;
using Owlboy.ScreenManagement.Renderers;
using Microsoft.Xna.Framework;

namespace OwlboyHebrew {
 public static class Runtime {
  private static SpeechBubbleSprite bubble;
  private static SpeechBubbleRenderer renderer;
  // The build-time patcher replaces this stub with a direct game method call.
  private static void OriginalDraw(Font font,ScaledSpriteBatch batch,string text,Vector2 pos,Color tint,float scale,bool onScreen,int offset,int maxWidth,int lineHeight,bool oldCoordinates,bool pad) {
#if RUNTIME_TESTS
   font.HebrewOriginalDraw(batch,text,pos,tint,scale,onScreen,offset,maxWidth,lineHeight,oldCoordinates,pad);
#else
   throw new InvalidOperationException("Unlinked Hebrew runtime");
#endif
  }
  private static readonly FieldInfo spacingField = typeof(Font).GetField("horizontalSpacing", BindingFlags.Instance|BindingFlags.NonPublic);
  private static readonly FieldInfo offsetField = typeof(Font).GetField("offsetY", BindingFlags.Instance|BindingFlags.NonPublic);
  private static readonly Dictionary<string,Layout> layouts = new Dictionary<string,Layout>();
  private static bool loaded;
  private sealed class Layout { public int[] order; public string visual; }
  public static void Begin(SpeechBubbleRenderer r, Sprite parent) { renderer=r; bubble=parent as SpeechBubbleSprite; }
  public static void End() { bubble=null; renderer=null; }
  public static bool HasHebrew(string s) { if(s==null)return false; foreach(char c in s) if(c>='\u05d0' && c<='\u05ea') return true; return false; }
  // Annotate renderer-created substrings, preserving their absolute logical indices.
  // The marker uses no mapped glyphs, so the game's measurement ignores it.
  public static string Slice(string s,int start,int count) {
   string value=s.Substring(start,count);
   if(bubble==null || !HasHebrew(bubble.storedText))return value;
   var b=new StringBuilder();int index=start;
   foreach(string line in value.Split('\n')) {
    if(b.Length>0)b.Append('\n');
    b.Append('\ue000');foreach(char digit in index.ToString(System.Globalization.CultureInfo.InvariantCulture))b.Append((char)('\ue100'+digit-'0'));
    b.Append('\ue001');b.Append(line);index+=line.Length+1;
   }
   return b.ToString();
  }
  private static string Decode(string s,ref int start) {
   if(s.Length==0 || s[0]!='\ue000')return s;
   int end=s.IndexOf('\ue001');if(end<2)return s;
   int value=0;for(int i=1;i<end;i++){int digit=s[i]-'\ue100';if(digit<0||digit>9)return s;value=checked(value*10+digit);}
   start=value;return s.Substring(end+1);
  }
  private static void Load() {
   if(loaded)return;loaded=true;
   string path=Path.Combine("Data","Localizations","he","layouts.tsv");
   if(!File.Exists(path))throw new FileNotFoundException("Hebrew layout data is missing. Reinstall the Hebrew patch.",path);
   foreach(string record in File.ReadAllLines(path,Encoding.UTF8)) {
    string[] fields=record.Split('\t');if(fields.Length!=3)continue;
    string logical=Encoding.UTF8.GetString(Convert.FromBase64String(fields[0]));
    string visual=Encoding.UTF8.GetString(Convert.FromBase64String(fields[1]));
    string[] indexes=fields[2].Split(',');int[] order=new int[indexes.Length];
    for(int i=0;i<indexes.Length;i++)order[i]=int.Parse(indexes[i]);
    if(order.Length!=logical.Length || visual.Length!=logical.Length)throw new InvalidDataException("Invalid Hebrew layout length");
    bool[] seen=new bool[order.Length];foreach(int i in order){if(i<0||i>=seen.Length||seen[i])throw new InvalidDataException("Invalid Hebrew layout permutation");seen[i]=true;}
    layouts[logical]=new Layout{order=order,visual=visual};
   }
  }
  private static Layout Resolve(string line) {
   Load();Layout result;if(layouts.TryGetValue(line,out result))return result;
   // Dynamic values: RTL paragraph with preserved Latin/number runs.
   // Static translation lines use the full Unicode BiDi maps generated at build time.
   var order=new List<int>();for(int i=line.Length-1;i>=0;i--){if(IsLtr(line[i])){int end=i;while(i>0&&IsLtr(line[i-1]))i--;for(int j=i;j<=end;j++)order.Add(j);}else order.Add(i);}
   var text=new StringBuilder();foreach(int i in order){char c=line[i];text.Append(c=='('?')':c==')'?'(':c=='['?']':c==']'?'[':c);}
   return new Layout{order=order.ToArray(),visual=text.ToString()};
  }
  private static bool IsLtr(char c){return c>='0'&&c<='9'||c>='A'&&c<='Z'||c>='a'&&c<='z'||c=='.'||c==':'||c=='/'||c=='%';}
  private static float Advance(Font font,char c,float spacing){int i=font.GetCharIndex(c);return i<0?0:font.sourceRects[i].Width+spacing;}
  public static bool TryDraw(Font font,ScaledSpriteBatch batch,string text,Vector2 pos,Color tint,float scale,bool onScreen,int offset,int maxWidth,int lineHeight,bool oldCoordinates,bool pad) {
   if(string.IsNullOrEmpty(text))return false;
   string full=bubble==null?null:bubble.storedText;
   bool marked=text[0]=='\ue000';
   bool speech=HasHebrew(full) && (marked || (bubble.displayText!=null && bubble.displayText.StartsWith(text,StringComparison.Ordinal)));
   if(!speech && !HasHebrew(text))return false;
   float spacing=(float)spacingField.GetValue(font);
   float unit=scale*(font.highres&&!onScreen?0.5f:1f);
   int effectiveLineHeight=lineHeight<0?(font.highres?font.Height:Owlboy.Localization.Localizer.Lang.langConfig.LineSpacing):lineHeight;
   float maxLineWidth=0;
   int centering= speech && renderer!=null && renderer.CenterText && !renderer.RenderTextSeparately ? font.GetTextWidth(full)/2-font.GetTextWidth(bubble.displayText)/2 : 0;
   if(speech)foreach(string line in full.Split('\n')){float w=0;foreach(char c in line)w+=Advance(font,c,spacing);maxLineWidth=Math.Max(maxLineWidth,w);}
   int nextStart=0;int localLine=0;
   foreach(string encodedLine in text.Split('\n')) {
    int start=nextStart;string shown=Decode(encodedLine,ref start);
    string reference=shown;int within=0;
    if(speech) {
     if(start<0||start>full.Length)throw new InvalidDataException("Hebrew speech index out of range");
     int lineStart=start==0?0:full.LastIndexOf('\n',start-1)+1;int lineEnd=full.IndexOf('\n',start);if(lineEnd<0)lineEnd=full.Length;
     reference=full.Substring(lineStart,lineEnd-lineStart);within=start-lineStart;
     if(within+shown.Length>reference.Length || reference.Substring(within,shown.Length)!=shown)throw new InvalidDataException("Hebrew speech slice mismatch");
    }
    if(reference.Length>0){
     Layout map=Resolve(reference);float[] positions=new float[reference.Length];char[] glyphs=new char[reference.Length];float width=0;
     for(int i=0;i<map.order.Length;i++){int logical=map.order[i];positions[logical]=width;glyphs[logical]=map.visual[i];width+=Advance(font,map.visual[i],spacing);}
     float correction=0;
     if(speech && marked && within>0){float prefix=font.MeasureString(reference.Substring(0,within)).X;correction=(int)((prefix>0?prefix+2:0)*(renderer==null?1:renderer.textScale));}
     float alignment=speech?Math.Max(0,maxLineWidth-width)*unit:0;
     for(int i=0;i<shown.Length;i++) {
      int logical=within+i;float x=pos.X-centering-correction+alignment+positions[logical]*unit;
      float y=pos.Y+localLine*effectiveLineHeight*scale*(font.highres?0.5f:1f)-(localLine>0?(int)offsetField.GetValue(font):0);
      int remaining=maxWidth==int.MaxValue?int.MaxValue:maxWidth-(int)(x-pos.X);
      if(remaining<=0)continue;
      OriginalDraw(font,batch,glyphs[logical].ToString(),new Vector2(x,y),tint,scale,onScreen,offset,remaining,lineHeight,oldCoordinates,pad);
     }
    }
    nextStart=start+shown.Length+1;localLine++;
   }
   return true;
  }
 }
}
