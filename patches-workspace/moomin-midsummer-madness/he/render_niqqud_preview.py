"""Render a labeled preview from the rebuilt Pangolin/Balsamiq SDF atlases."""
import argparse,json,sys,unicodedata
from pathlib import Path
from PIL import Image,ImageDraw
import UnityPy
BASE=Path(__file__).resolve().parent
sys.path.insert(0,str(BASE))
import build

def draw_line(canvas,atlas,font,text,baseline,ycolor):
    chars={c.m_Unicode:c for c in font.m_CharacterTable};glyphs={g.m_Index:g for g in font.m_GlyphTable}
    rendered=build.visual(text,'font_preview');cursor=35
    for ch in rendered:
        cp=ord(ch);entry=chars.get(cp)
        if entry is None:raise ValueError(f'Missing preview glyph U+{cp:04X}')
        g=glyphs[entry.m_GlyphIndex];r=g.m_GlyphRect;m=g.m_Metrics
        if r.m_Width and r.m_Height:
            # Convert Unity's bottom-up raw Alpha8 atlas rows to display orientation.
            tile=atlas.crop((r.m_X,r.m_Y,r.m_X+r.m_Width,r.m_Y+r.m_Height)).transpose(Image.Transpose.FLIP_TOP_BOTTOM)
            mask=tile.point(lambda a:255 if a>=128 else 0)
            x=round(cursor+m.m_HorizontalBearingX);y=round(baseline-m.m_HorizontalBearingY)
            ink=Image.new('RGBA',mask.size,ycolor);canvas.alpha_composite(Image.composite(ink,Image.new('RGBA',mask.size,(0,0,0,0)),mask),(x,y))
        cursor+=m.m_HorizontalAdvance
    return cursor

def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('bundle',type=Path)
    ap.add_argument('--before',type=Path,required=True)
    ap.add_argument('--out',type=Path,default=BASE/'niqqud-font-preview.png')
    a=ap.parse_args()
    envs=[UnityPy.load(a.before.read_bytes()),UnityPy.load(a.bundle.read_bytes())]
    names=['Pangolin-Regular SDF MAIN','OpenDyslexic-Regular_SDF','BalsamiqSans-Regular_SDF','BalsamiqSans-Bold_SDF']
    img=Image.new('RGBA',(1800,760),'white');d=ImageDraw.Draw(img);color=(31,55,82,255)
    d.text((20,10),'Offline TMP atlas comparison: English / previous Hebrew / updated Hebrew',fill='black')
    for j,label in enumerate(['English','Previous Hebrew','Updated Hebrew']):d.text((j*600+20,35),label,fill='black')
    for i,name in enumerate(names):
        y=65+i*170
        d.text((20,y),name,fill='black')
        for col,(env,text) in enumerate([(envs[0],'Little My! Moomintroll'),(envs[0],'מַאי הַקְּטַנָּה! מוּמִינְטְרוֹל'),(envs[1],'מַאי הַקְּטַנָּה! מוּמִינְטְרוֹל')]):
            obj=next(o for o in env.objects if o.type.name=='MonoBehaviour' and o.peek_name()==name)
            font=obj.read();tex=next(o for o in env.objects if o.path_id==font.m_AtlasTextures[0].path_id).read()
            atlas=Image.frombytes('L',(tex.m_Width,tex.m_Height),tex.image_data)
            panel=Image.new('RGBA',(1200,190),'white')
            draw_line(panel,atlas,font,text,85,color)
            draw_line(panel,atlas,font,'סְנוּפְקִין וְסְנוֹרְקָה' if col else 'Snufkin and Snorkmaiden',165,color)
            img.alpha_composite(panel.resize((600,95),Image.Resampling.LANCZOS),(col*600,y+25))
    img.save(a.out);print(a.out)
if __name__=='__main__':main()
