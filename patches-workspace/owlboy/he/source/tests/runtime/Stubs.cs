namespace Microsoft.Xna.Framework {
 public struct Vector2 {public float X,Y;public Vector2(float x,float y){X=x;Y=y;}}
 public struct Color {}
 public struct Rectangle {public int Width;}
}
namespace Owlboy {
 using Microsoft.Xna.Framework;
 public class Sprite {}
 public class SpeechBubbleSprite:Sprite {public string storedText,displayText;}
 public class ScaledSpriteBatch {}
 public class Font {
  private float horizontalSpacing=1;
  private int offsetY=0;
  public bool highres=false;public int Height=16;
  public Rectangle[] sourceRects=new Rectangle[65536];
  public List<(string text,float x,float y)> calls=new();
  public Font(){for(int i=0;i<65536;i++)sourceRects[i].Width=5;}
  public int GetCharIndex(char c)=>c;
  public int GetTextWidth(string s)=>s.Length*6;
  public Vector2 MeasureString(string s)=>new Vector2(s.Length*6,16);
  public void HebrewOriginalDraw(ScaledSpriteBatch b,string text,Vector2 pos,Color tint,float scale,bool screen,int offset,int max,int lineHeight,bool old,bool pad){calls.Add((text,pos.X,pos.Y));}
 }
}
namespace Owlboy.Localization {public class Config {public int LineSpacing=15;}public class Localizer {public static Localizer Lang=new();public Config langConfig=new();}}
namespace Owlboy.ScreenManagement.Renderers {public class SpeechBubbleRenderer {public float textScale=1;public bool CenterText=false,RenderTextSeparately=true;}}
