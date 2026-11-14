"""Render a labeled preview from the rebuilt Pangolin/Balsamiq SDF atlases."""
import json,sys,unicodedata
from pathlib import Path
from PIL import Image,ImageDraw
import UnityPy
BASE=Path(__file__).resolve().parent
sys.path.insert(0,str(BASE))
import build

def draw_line(canvas,atlas,font,text,baseline,ycolor):
    chars={c.m_Unicode:c for c in font.m_CharacterTable};glyphs={g.m_Index:g for g in font.m_GlyphTable}
    rendered=build.visual(text,'dialogue_preview');cursor=35
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
    e=UnityPy.load(Path('.tmp/moomin-niqqud-fonttest/fonts_assets_all.bundle').read_bytes())
    samples=[('Pangolin Regular','Pangolin-Regular SDF MAIN','מוּמִינְטְרוֹל: מַאי הַקְּטַנָּה!'),('Balsamiq Bold','BalsamiqSans-Bold_SDF','סְנוּפְקִין וְסְנוֹרְקָה — הַשְּׁמַנְטַפִּים')]
    img=Image.new('RGBA',(1100,330),'white');d=ImageDraw.Draw(img);color=(31,55,82,255)
    for i,(label,name,text) in enumerate(samples):
        d.text((25,25+i*145),label,fill=(40,40,40,255))
        obj=next(o for o in e.objects if o.peek_name()==name);font=obj.read();tex=next(o for o in e.objects if o.path_id==font.m_AtlasTextures[0].path_id).read();atlas=Image.frombytes('L',(tex.m_Width,tex.m_Height),tex.image_data)
        end=draw_line(img,atlas,font,text,100+i*145,color)
        if end>1090:raise ValueError(f'Preview line too wide: {end}')
    dest=BASE/'niqqud-font-preview.png';img.save(dest);print(dest)
if __name__=='__main__':main()
