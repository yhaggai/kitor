"""Local font specimen. Original English is reconstructed from shipped TMP atlases.
Candidate Hebrew uses the downloaded font files; this is not an in-game capture.
"""
from pathlib import Path
import json
import numpy as np
from PIL import Image, ImageDraw, ImageFont
import UnityPy
ROOT=Path(__file__).resolve().parent
src=Path.home()/'Library/Application Support/Steam/steamapps/common/Disco Elysium/disco.app/Contents/Resources/Data/StreamingAssets/AssetBundles/OSX/fonts/english'
env=UnityPy.load(str(src));objs={o.path_id:o for o in env.objects};fonts={}
for o in env.objects:
 if o.type.name=='MonoBehaviour':
  t=o.read_typetree()
  if 'm_CharacterTable' in t:fonts[t['m_Name']]=t
COL=(222,218,198,255)
def original(name,text,size):
 t=fonts[name];s=size/t['m_FaceInfo']['m_PointSize'];chars={c['m_Unicode']:c['m_GlyphIndex'] for c in t['m_CharacterTable']};glyphs={g['m_Index']:g for g in t['m_GlyphTable']}
 atlas=objs[t['m_AtlasTextures'][0]['m_PathID']].read().image.convert('RGBA');canvas=Image.new('RGBA',(1100,120));x=12;baseline=80
 for c in text:
  g=glyphs[chars[ord(c)]];r=g['m_GlyphRect'];m=g['m_Metrics'];pad=4
  if r['m_Width'] and r['m_Height']:
   box=(r['m_X']-pad,atlas.height-r['m_Y']-r['m_Height']-pad,r['m_X']+r['m_Width']+pad,atlas.height-r['m_Y']+pad)
   a=atlas.crop(box).getchannel('A');a=a.resize((max(1,round(a.width*s)),max(1,round(a.height*s))),Image.Resampling.LANCZOS)
   a=Image.fromarray(np.uint8(np.clip((np.asarray(a,dtype=float)-112)*255/32,0,255)))
   glyph=Image.new('RGBA',a.size,COL);glyph.putalpha(a)
   canvas.alpha_composite(glyph,(round(x+(m['m_HorizontalBearingX']-pad)*s),round(baseline-(m['m_HorizontalBearingY']+pad)*s)))
  x+=m['m_HorizontalAdvance']*s
 return canvas

def hebrew(family,text,size,weight=400,slant=False):
 if family=='Arial':path=Path('/System/Library/Fonts/Supplemental')/('Arial Bold.ttf' if weight>=600 else 'Arial.ttf')
 else:path=ROOT/'fonts'/f'{family}.ttf'
 f=ImageFont.truetype(str(path),size)
 if family!='Arial':f.set_variation_by_axes([weight])
 im=Image.new('RGBA',(1100,120));d=ImageDraw.Draw(im);d.text((12,80),text,font=f,fill=COL,anchor='ls',direction='rtl')
 if slant:im=im.transform(im.size,Image.Transform.AFFINE,(1,.20,-16,0,1,0),resample=Image.Resampling.BICUBIC)
 return im

def slanted(im):return im.transform(im.size,Image.Transform.AFFINE,(1,-.20,16,0,1,0),resample=Image.Resampling.BICUBIC)
W=1860;out=Image.new('RGB',(W,1170),'#161b1c');d=ImageDraw.Draw(out)
label=ImageFont.truetype('/System/Library/Fonts/Supplemental/Arial.ttf',21)
title=ImageFont.truetype('/System/Library/Fonts/Supplemental/Arial Bold.ttf',36)
d.text((36,28),'DISCO ELYSIUM / Hebrew typography study',font=title,fill='#f1e8d7')
d.text((36,82),'Font specimen, not an in-game capture. Original English reconstructed from game atlases.',font=label,fill='#b2b8b4')
xs=[160,720,1280]
for x,head in zip(xs,['Original English','Current Hebrew / Arial','Proposed Hebrew']):d.text((x,136),head,font=label,fill='#d68e68')
styles=[('Regular',400,False),('Bold',700,False),('Italic*',400,True),('Bold italic*',700,True)]
for n,(style,weight,italic) in enumerate(styles):
 y=190+n*145;d.text((30,y+36),style,font=label,fill='#aeb5b3')
 d.line((160,y+80,1815,y+80),fill='#333c3c')
 a=original('SinaNova-Bold SDF' if weight==700 else 'SinaNova-Medium SDF','Your conscience ferments in it.',36)
 if italic:a=slanted(a)
 b=hebrew('Arial','המצפון שלך תוסס בתוכה.',36,weight,italic)
 c=hebrew('FrankRuhlLibre','המצפון שלך תוסס בתוכה.',40,500 if weight==400 else 700,italic)
 for x,im in zip(xs,[a,b,c]):out.paste(im.crop((0,0,530,120)),(x,y),im.crop((0,0,530,120)))
d.text((30,805),'Menu',font=label,fill='#aeb5b3')
for x,im in zip(xs,[original('CoreSansES Light SDF','NEW GAME',52),hebrew('Arial','משחק חדש',52),hebrew('Assistant','משחק חדש',58,300)]):out.paste(im.crop((0,0,530,120)),(x,770),im.crop((0,0,530,120)))
notes=['Dialogue proposal: Frank Ruhl Libre Medium 500 / Bold 700, +11% nominal size versus Arial.',
       'Menu proposal: Assistant Light 300, +12% nominal size; no horizontal distortion.',
       '* Slanted samples are simulated oblique styles, not designed italics. Actual TMP emphasis needs an in-game check.',
       'Baselines are shared. Final font routing, line spacing, reveal and mixed-direction text remain to be tested.']
for i,n in enumerate(notes):d.text((36,930+i*42),n,font=label,fill='#b2b8b4')
out.save(ROOT/'typography-comparison.png')
print(ROOT/'typography-comparison.png')
