"""Build four Hebrew TMP faces sharing the original Arabic atlas and materials.
Run after prototype.py; only modifies the isolated test copy. Keeps rollback bundles.
"""
import copy, hashlib, json, sys
from pathlib import Path
sys.path.insert(0,'/private/tmp/disco-he-python')
import numpy as np
from scipy.ndimage import distance_transform_edt
from PIL import Image,ImageDraw,ImageFont
import UnityPy
ROOT=Path(__file__).resolve().parent
SOURCE=Path.home()/'Library/Application Support/Steam/steamapps/common/Disco Elysium/disco.app/Contents/Resources/Data/StreamingAssets/AssetBundles/OSX'
DEST=Path('/private/tmp/disco-he-test.app/Contents/Resources/Data/StreamingAssets/AssetBundles/OSX')
STAGE=ROOT/'font-stage';STAGE.mkdir(exist_ok=True)
BACKUP=ROOT/'font-rollback';BACKUP.mkdir(exist_ok=True)
for relative in ['fonts/arabic','lockits/arabic']:
 backup=BACKUP/relative;backup.parent.mkdir(parents=True,exist_ok=True)
 if not backup.exists():backup.write_bytes((DEST/relative).read_bytes())
def ptr(n):return {'m_FileID':0,'m_PathID':n}
def hashname(s):
 h=0
 for c in s:h=((h<<5)+h)^ord(c.upper());h&=0xffffffff
 return h if h<2**31 else h-2**32
fontenv=UnityPy.load(str(SOURCE/'fonts/arabic'))
fo=next(o for o in fontenv.objects if o.type.name=='MonoBehaviour');base=fo.read_typetree();asset=fo.assets_file
texo=next(o for o in fontenv.objects if o.type.name=='Texture2D');tex=texo.read()
atlas=Image.new('RGBA',(8192,4096),(255,255,255,0));atlas.paste(tex.image,(0,0))
ab=next(o for o in fontenv.objects if o.type.name=='AssetBundle');abt=ab.read_typetree()
# Existing name retained for menu fallback. Dialogue has separately addressable faces.
specs=[('NotoSansArabic SDF','Assistant',300,286,fo.path_id),('HebrewMenuBold SDF','Assistant',700,286,910000002),('HebrewDialogue SDF','FrankRuhlLibre',500,284,910000003),('HebrewDialogueBold SDF','FrankRuhlLibre',700,284,910000004)]
for face,(name,family,weight,size,oid) in enumerate(specs):
 t=copy.deepcopy(base);t['m_Name']=name;t['hashCode']=hashname(name);t['m_AtlasWidth']=8192;t['m_AtlasPopulationMode']=0;t['italicStyle']=12
 font=ImageFont.truetype(str(ROOT/'fonts'/f'{family}.ttf'),size);font.set_variation_by_axes([weight]);pad=20
 for i,code in enumerate(range(0x5d0,0x5eb)):
  ch=chr(code);left,top,right,bottom=font.getbbox(ch,anchor='ls');w=right-left;h=bottom-top
  assert w+2*pad<=350 and h+2*pad<=320,(name,ch,w,h)
  mask=Image.new('L',(w+2*pad,h+2*pad));ImageDraw.Draw(mask).text((pad-left,pad-top),ch,font=font,fill=255,anchor='ls')
  inside=np.array(mask)>127;sdf=np.uint8(np.clip(128+(distance_transform_edt(inside)-distance_transform_edt(~inside))*127/pad,0,255))
  glyph=Image.new('RGBA',mask.size,'white');glyph.putalpha(Image.fromarray(sdf))
  x=4096+(i%9)*350;y=(face*3+i//9)*320
  atlas.paste(glyph,(x,4096-y-glyph.height));index=10000+face*100+i
  t['m_GlyphTable'].append({'m_Index':index,'m_Metrics':{'m_Width':float(w),'m_Height':float(h),'m_HorizontalBearingX':float(left),'m_HorizontalBearingY':float(-top),'m_HorizontalAdvance':font.getlength(ch)},'m_GlyphRect':{'m_X':x+pad,'m_Y':y+pad,'m_Width':w,'m_Height':h},'m_Scale':1.0,'m_AtlasIndex':0})
  t['m_CharacterTable'].append({'m_ElementType':1,'m_Unicode':code,'m_GlyphIndex':index,'m_Scale':1.0})
 boldid=specs[1 if face<2 else 3][4]
 t['m_FontWeightTable'][7]['regularTypeface']=ptr(boldid)
 target=fo if face==0 else copy.copy(fo)
 if face:
  target.path_id=oid;asset.objects[oid]=target
  start=len(abt['m_PreloadTable']);deps=copy.deepcopy(abt['m_PreloadTable'][:6]);deps[-1]=ptr(oid);abt['m_PreloadTable'].extend(deps)
  abt['m_Container'].append(('assets/hebrew/'+name.lower().replace(' ','-')+'.asset',{'preloadIndex':start,'preloadSize':len(deps),'asset':ptr(oid)}))
 target.save_typetree(t)
tex.image=atlas;tex.save();ab.save_typetree(abt)
for o in fontenv.objects:
 if o.type.name=='Material':
  t=o.read_typetree();t['m_SavedProperties']['m_Floats']=[(k,8192 if k=='_TextureWidth' else v) for k,v in t['m_SavedProperties']['m_Floats']];o.save_typetree(t)
(STAGE/'fonts').mkdir(exist_ok=True);(STAGE/'fonts/arabic').write_bytes(fontenv.file.save())
# Read current prototype text and change only font routing.
lockenv=UnityPy.load(str(DEST/'lockits/arabic'));routes={}
for o in lockenv.objects:
 if o.type.name!='MonoBehaviour':continue
 t=o.read_typetree()
 if t['m_Name']!='FontsLockitArabic':continue
 for term in t['mSource']['mTerms']:
  if term['TermType']!=9:continue
  key=term['Term']
  if key in ['SinaNova-Medium SDF','Collage Dialogue SDF']:value='HebrewDialogue SDF'
  elif key=='SinaNova-Bold SDF':value='HebrewDialogueBold SDF'
  elif key=='Dobra-Bold SDF':value='HebrewMenuBold SDF'
  else:value='NotoSansArabic SDF'
  term['Languages'][0]=value;routes[key]=value
 o.save_typetree(t)
(STAGE/'lockits').mkdir(exist_ok=True);(STAGE/'lockits/arabic').write_bytes(lockenv.file.save())
# Validate actual serialized IDs, glyphs, mappings and unchanged non-font localization.
check=UnityPy.load(str(STAGE/'fonts/arabic'));byid={o.path_id:o for o in check.objects}
for name,family,weight,size,oid in specs:
 t=byid[oid].read_typetree();assert t['m_Name']==name
 assert set(range(0x5d0,0x5eb))<={c['m_Unicode'] for c in t['m_CharacterTable']}
 assert t['m_FontWeightTable'][7]['regularTypeface']['m_PathID'] in byid
original=UnityPy.load(str(DEST/'lockits/arabic'));staged=UnityPy.load(str(STAGE/'lockits/arabic'))
def tables(env):return {o.path_id:o.read_typetree() for o in env.objects if o.type.name=='MonoBehaviour'}
a,b=tables(original),tables(staged)
for oid,t in a.items():
 if t['m_Name']!='FontsLockitArabic':assert t==b[oid]
report={'faces':[{'name':n,'family':f,'weight':w,'raster_size':s,'path_id':i} for n,f,w,s,i in specs],'routes':routes,'italic':'TMP synthetic slant 12; no designed italic face','atlas':[8192,4096],'validation':'serialized glyphs, bold pointers, routes, unchanged text','in_game_verified':False,'rollback':str(BACKUP)}
# Install verified staged bytes atomically.
for relative in ['fonts/arabic','lockits/arabic']:
 dest=DEST/relative;temp=dest.with_name('arabic.he-staged');temp.write_bytes((STAGE/relative).read_bytes());temp.replace(dest)
(ROOT/'font-validation.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
