#!/usr/bin/env python3
"""Build a private Arial prototype from CSV and owned, hash-pinned game data.
Never writes into either game installation. Generated payloads are binary deltas.
"""
import argparse, hashlib, json
from pathlib import Path
import UnityPy
import bsdiff4
import numpy as np
from PIL import Image, ImageDraw, ImageFont
from scipy.ndimage import distance_transform_edt
import text_pipeline as text
ROOT=Path(__file__).resolve().parent
VERSION='0.1.0-preview1'
def sha(data):return hashlib.sha256(data).hexdigest()
def font_bundle(source,translations,font_path):
 env=UnityPy.load(str(source));obj=next(o for o in env.objects if o.type.name=='MonoBehaviour');t=obj.read_typetree()
 texture=next(o for o in env.objects if o.type.name=='Texture2D').read()
 assert (t['m_AtlasWidth'],t['m_AtlasHeight'])==(4096,4096)
 atlas=Image.new('RGBA',(8192,4096),(255,255,255,0));atlas.paste(texture.image,(0,0))
 font=ImageFont.truetype(str(font_path),256);pad=20
 existing={c['m_Unicode'] for c in t['m_CharacterTable']}
 required=set(range(0x5d0,0x5eb))|{ord(ch) for value in translations.values() for ch in value if not ch.isspace()}
 added=sorted(required-existing)
 for i,code in enumerate(added):
  ch=chr(code);left,top,right,bottom=font.getbbox(ch,anchor='ls');w=right-left;h=bottom-top
  assert w+2*pad<=350 and h+2*pad<=400 and i<100,'Atlas capacity exceeded'
  mask=Image.new('L',(w+2*pad,h+2*pad));ImageDraw.Draw(mask).text((pad-left,pad-top),ch,font=font,fill=255,anchor='ls')
  inside=np.array(mask)>127;sdf=np.clip(128+(distance_transform_edt(inside)-distance_transform_edt(~inside))*127/pad,0,255).astype('uint8')
  rgba=Image.new('RGBA',mask.size,'white');rgba.putalpha(Image.fromarray(sdf));x=4096+(i%10)*350;y=(i//10)*400
  atlas.paste(rgba,(x,4096-y-rgba.height));index=10000+i
  t['m_GlyphTable'].append({'m_Index':index,'m_Metrics':{'m_Width':float(w),'m_Height':float(h),'m_HorizontalBearingX':float(left),'m_HorizontalBearingY':float(-top),'m_HorizontalAdvance':font.getlength(ch)},'m_GlyphRect':{'m_X':x+pad,'m_Y':y+pad,'m_Width':w,'m_Height':h},'m_Scale':1.0,'m_AtlasIndex':0})
  t['m_CharacterTable'].append({'m_ElementType':1,'m_Unicode':code,'m_GlyphIndex':index,'m_Scale':1.0})
 t['m_AtlasWidth']=8192;t['m_AtlasPopulationMode']=0;obj.save_typetree(t);texture.image=atlas;texture.save()
 for o in env.objects:
  if o.type.name=='Material':
   mt=o.read_typetree();mt['m_SavedProperties']['m_Floats']=[(k,8192 if k=='_TextureWidth' else v) for k,v in mt['m_SavedProperties']['m_Floats']];o.save_typetree(mt)
 output=env.file.save(packer='lz4')
 check=UnityPy.load(output);ft=next(o.read_typetree() for o in check.objects if o.type.name=='MonoBehaviour')
 assert required<={c['m_Unicode'] for c in ft['m_CharacterTable']}
 return output,len(added)
def lockit_bundle(source,translations):
 env=UnityPy.load(str(source));seen=set()
 for o in env.objects:
  if o.type.name!='MonoBehaviour':continue
  t=o.read_typetree()
  if t.get('m_Name') not in ['GeneralLockitArabic','DialoguesLockitArabic']:continue
  for term in t['mSource']['mTerms']:
   if term['Term'] in translations:term['Languages'][0]=translations[term['Term']];seen.add(term['Term'])
  o.save_typetree(t)
 assert seen==set(translations),'Translated keys missing in platform source'
 output=env.file.save(packer='lz4');actual={}
 for o in UnityPy.load(output).objects:
  if o.type.name=='MonoBehaviour':
   for term in o.read_typetree().get('mSource',{}).get('mTerms',[]):
    if term['Term'] in translations:actual[term['Term']]=term['Languages'][0]
 assert actual==translations
 return output

def build(mac,windows,font_path):
 report=text.validate();translations={r['key']:r['hebrew'] for r in text.readcsv() if r['hebrew']}
 manifest={'schema_version':1,'scummvm_id':'disco-elysium','language':'he','version':VERSION,'filename':f'Disco-Elysium-Hebrew-{VERSION}.zip','platforms':{}}
 fingerprints={};glyphs={}
 for platform,root,base in [('osx',mac,'Contents/Resources/Data/StreamingAssets/AssetBundles/OSX'),('windows',windows,'disco_Data/StreamingAssets/AssetBundles/Windows')]:
  assert root.is_dir();files=[]
  for kind in ['lockits','fonts']:
   rel=f'{base}/{kind}/arabic';source=root/rel;before=source.read_bytes()
   if kind=='lockits':after=lockit_bundle(source,translations)
   else:after,glyphs[platform]=font_bundle(source,translations,font_path)
   delta=bsdiff4.diff(before,after);assert bsdiff4.patch(before,delta)==after
   target=ROOT/'payload'/platform/(kind+'.bsdiff');target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(delta)
   files.append({'path':rel,'operation':'modify','original_sha256':sha(before),'installed_sha256':sha(after),'source':{'kind':'bsdiff','path':str(target.relative_to(ROOT)),'sha256':sha(delta),'input':rel}})
   fingerprints[f'{platform}/{kind}']={'source_sha256':sha(before),'output_sha256':sha(after),'delta_bytes':len(delta)}
  manifest['platforms'][platform]={'files':files}
 text.writejson(ROOT/'patch.preview.json',manifest)
 text.writejson(ROOT/'package-build.json',{'schema_version':1,'csv_sha256':sha((ROOT/'translations.csv').read_bytes()),'compiler_sha256':sha(Path(__file__).read_bytes()),'translated':report['translated'],'source_entries':report['entries'],'font':'Arial','font_sha256':sha(font_path.read_bytes()),'font_redistribution':'Not cleared; private prototype only, no public release','glyphs_added':glyphs,'platforms':fingerprints,'release_ready':False})
 print(json.dumps({'translated':report['translated'],'deltas':fingerprints}))
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--mac',type=Path,required=True);p.add_argument('--windows',type=Path,required=True);p.add_argument('--font',type=Path,required=True);a=p.parse_args();build(a.mac.resolve(),a.windows.resolve(),a.font.resolve())
