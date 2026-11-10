"""Build an isolated Hebrew opening-scene probe; never modifies the Steam copy."""
import copy, hashlib, json, sys
from pathlib import Path
sys.path.insert(0, '/private/tmp/disco-he-python')
import numpy as np
from scipy.ndimage import distance_transform_edt
from PIL import Image, ImageDraw, ImageFont
import UnityPy
ROOT=Path(__file__).resolve().parent
SOURCE=Path.home()/'Library/Application Support/Steam/steamapps/common/Disco Elysium/disco.app/Contents/Resources/Data'
DEST=Path('/private/tmp/disco-he-test.app/Contents/Resources/Data')
BASE=Path('StreamingAssets/AssetBundles/OSX')
# translations.csv is the sole editable translation source.
from text_pipeline import build, readcsv
translations={r['key']:r['hebrew'] for r in readcsv() if r['hebrew']}
staged=ROOT/'build/lockits/arabic'
build(SOURCE,staged)
(DEST/BASE/'lockits/arabic').write_bytes(staged.read_bytes())
replacements=translations
changed=list(translations)
# Preserve all original glyphs; append Hebrew in an enlarged atlas.
e=UnityPy.load(str(SOURCE/BASE/'fonts/arabic'))
fo=next(o for o in e.objects if o.type.name=='MonoBehaviour');t=fo.read_typetree()
to=next(o for o in e.objects if o.type.name=='Texture2D');tex=to.read()
atlas=Image.new('RGBA',(8192,4096),(255,255,255,0));atlas.paste(tex.image,(0,0))
font=ImageFont.truetype('/System/Library/Fonts/Supplemental/Arial.ttf',256)
pad=20
existing={c['m_Unicode'] for c in t['m_CharacterTable']}
required=set(range(0x5d0,0x5eb)) | {ord(ch) for text in translations.values() for ch in text if not ch.isspace()}
added=sorted(required-existing)
for i,code in enumerate(added):
 ch=chr(code);left,top,right,bottom=font.getbbox(ch,anchor='ls');w=right-left;h=bottom-top
 mask=Image.new('L',(w+2*pad,h+2*pad));ImageDraw.Draw(mask).text((pad-left,pad-top),ch,font=font,fill=255,anchor='ls')
 inside=np.array(mask)>127
 sdf=np.clip(128+(distance_transform_edt(inside)-distance_transform_edt(~inside))*127/pad,0,255).astype('uint8')
 rgba=Image.new('RGBA',mask.size,'white');rgba.putalpha(Image.fromarray(sdf))
 x=4096+(i%10)*350;y=(i//10)*400
 atlas.paste(rgba,(x,4096-y-rgba.height))
 index=10000+i
 t['m_GlyphTable'].append({'m_Index':index,'m_Metrics':{'m_Width':float(w),'m_Height':float(h),'m_HorizontalBearingX':float(left),'m_HorizontalBearingY':float(-top),'m_HorizontalAdvance':font.getlength(ch)},'m_GlyphRect':{'m_X':x+pad,'m_Y':y+pad,'m_Width':w,'m_Height':h},'m_Scale':1.0,'m_AtlasIndex':0})
 t['m_CharacterTable'].append({'m_ElementType':1,'m_Unicode':code,'m_GlyphIndex':index,'m_Scale':1.0})
t['m_AtlasWidth']=8192;t['m_AtlasPopulationMode']=0
fo.save_typetree(t);tex.image=atlas;tex.save()
for o in e.objects:
 if o.type.name=='Material':
  mt=o.read_typetree()
  mt['m_SavedProperties']['m_Floats']=[(key,8192 if key=='_TextureWidth' else value) for key,value in mt['m_SavedProperties']['m_Floats']]
  o.save_typetree(mt)
(DEST/BASE/'fonts/arabic').write_bytes(e.file.save())
# Reopen and verify the serialized artifacts.
check=UnityPy.load(str(DEST/BASE/'fonts/arabic')); ft=next(o.read_typetree() for o in check.objects if o.type.name=='MonoBehaviour')
assert all(c in {v['m_Unicode'] for v in ft['m_CharacterTable']} for c in range(0x5d0,0x5eb))
check=UnityPy.load(str(DEST/BASE/'lockits/arabic'));actual={}
for o in check.objects:
 if o.type.name=='MonoBehaviour':
  for term in o.read_typetree()['mSource']['mTerms']:
   if term['Term'] in replacements:actual[term['Term']]=term['Languages'][0]
assert actual=={k:replacements[k] for k in changed}
report={'steam_build':'23980936','language_slot':'Arabic','text_order':'logical Hebrew; native RTL confirmed for earlier Arial opening by user screenshots','changed_keys':changed,'hebrew_glyphs_added':len(added),'font':'Arial Regular 256px; provisional renderer test','source_sha256':{str(p):hashlib.sha256((SOURCE/p).read_bytes()).hexdigest() for p in [BASE/'fonts/arabic',BASE/'lockits/arabic']},'renderer_verified':False}
(ROOT/'validation.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
