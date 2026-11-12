#!/usr/bin/env python3
"""Build the Hebrew patch from CSV and owned, hash-pinned game data.
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
from loading_literal import replace_loading_label
from language_display_name import replace_language_display_name
from interface_font import build_interface_font_recipe
ROOT=Path(__file__).resolve().parent
VERSION='0.1.2'
FALLBACK=ROOT/'NotoSans.ttf'
def sha(data):return hashlib.sha256(data).hexdigest()
def font_bundle(source,translations,font_path):
 env=UnityPy.load(str(source));obj=next(o for o in env.objects if o.type.name=='MonoBehaviour');t=obj.read_typetree()
 texture=next(o for o in env.objects if o.type.name=='Texture2D').read()
 assert (t['m_AtlasWidth'],t['m_AtlasHeight'])==(4096,4096)
 atlas=Image.new('RGBA',(8192,4096),(255,255,255,0));atlas.paste(texture.image,(0,0))
 fonts=[ImageFont.truetype(str(path),256) for path in (font_path,FALLBACK)]
 # Assistant defaults to ExtraLight, which makes orange dialogue choices hard to read.
 fonts[0].set_variation_by_axes([600])
 missing=[bytes(font.getmask(chr(0x10ffff))) for font in fonts]
 pad=20
 existing={c['m_Unicode'] for c in t['m_CharacterTable']}
 required=set(range(0x5d0,0x5eb))|{ord(ch) for value in translations.values() for ch in value if not ch.isspace()}
 added=sorted(required-existing)
 x,y,row_height=4096,0,0
 for i,code in enumerate(added):
  ch=chr(code)
  if ch=='▼':
   w,h,left,top,advance=144,128,0,-128,160
   mask=Image.new('L',(w+2*pad,h+2*pad))
   ImageDraw.Draw(mask).polygon([(pad,pad),(pad+w-1,pad),(pad+w//2,pad+h-1)],fill=255)
  else:
   font=next((f for f,m in zip(fonts,missing) if bytes(f.getmask(ch))!=m),None)
   if font is None:raise ValueError(f'Missing glyph U+{code:04X}')
   left,top,right,bottom=font.getbbox(ch,anchor='ls');w=right-left;h=bottom-top
   advance=font.getlength(ch)
   mask=Image.new('L',(w+2*pad,h+2*pad))
   ImageDraw.Draw(mask).text((pad-left,pad-top),ch,font=font,fill=255,anchor='ls')
  if x+mask.width>8192:
   x=4096;y+=row_height;row_height=0
  if y+mask.height>4096:raise ValueError('Atlas capacity exceeded')
  inside=np.array(mask)>127
  sdf=np.clip(128+(distance_transform_edt(inside)-distance_transform_edt(~inside))*127/pad,0,255).astype('uint8')
  rgba=Image.new('RGBA',mask.size,'white');rgba.putalpha(Image.fromarray(sdf))
  atlas.paste(rgba,(x,4096-y-rgba.height));index=10000+i
  t['m_GlyphTable'].append({'m_Index':index,'m_Metrics':{'m_Width':float(w),'m_Height':float(h),'m_HorizontalBearingX':float(left),'m_HorizontalBearingY':float(-top),'m_HorizontalAdvance':advance},'m_GlyphRect':{'m_X':x+pad,'m_Y':y+pad,'m_Width':w,'m_Height':h},'m_Scale':1.0,'m_AtlasIndex':0})
  t['m_CharacterTable'].append({'m_ElementType':1,'m_Unicode':code,'m_GlyphIndex':index,'m_Scale':1.0})
  x+=mask.width;row_height=max(row_height,mask.height)
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

def interface_image_bundle(source, translations, font_path):
 env=UnityPy.load(str(source))
 labels={
  'loading-panel-ar':translations['Buttons/LOAD_SCREEN_PANEL_TEXT'],
  'saving-panel-ar':'שומר',
  'button-reset-settings-ar':translations['Buttons/SETTINGS_BUTTON_RESET_SETTINGS_TEXT'],
 }
 button_labels={
  'continue-tight-ar':(232,511,translations['Buttons/ARCHETYPE_BUTTON_CONTINUE_TEXT']),
  'button_back-ar':(0,220,'הקודם'),
  'button_cancel-ar':(312,511,translations['Buttons/CONFIRMATION_BUTTON_CANCEL_TEXT']),
  'button_confirm-ar':(322,511,translations['Buttons/CONFIRMATION_BUTTON_CONFIRM_TEXT']),

 }
 rebuilt={}
 for obj in env.objects:
  name=obj.peek_name() if obj.type.name=='Texture2D' else None
  if name not in labels and name not in button_labels:continue
  texture=obj.read()
  original=texture.image.convert('RGBA')
  if name in button_labels:
   left,right,label=button_labels[name]
   panel=original.copy()
   draw=ImageDraw.Draw(panel)
   background=panel.getpixel((left+2,10))
   draw.rectangle((left,20,right-1,110),fill=background)
   font=ImageFont.truetype(str(font_path),57)
   font.set_variation_by_axes([500])
   alignment='lm' if name=='button_back-ar' else 'rm'
   x=left+35 if alignment=='lm' else right-37
   draw.text((x,61),label,font=font,anchor=alignment,fill='white',direction='rtl')
   texture.set_image(panel,target_format=4)
   texture.save()
   rebuilt[name]=panel.tobytes()
   continue
  alpha=original.getchannel('A')
  is_button=name=='button-reset-settings-ar'
  _,top,right,bottom=alpha.getbbox()
  if is_button:
   top,right,bottom=9,original.width-24,original.height-9
  font=ImageFont.truetype(str(font_path),34 if is_button else bottom-top)
  font.set_variation_by_axes([600])
  label=labels[name]
  label_box=font.getbbox(label,direction='rtl')
  label_width,label_height=label_box[2]-label_box[0],label_box[3]-label_box[1]
  x=right-label_width-label_box[0]
  y=(top+bottom-label_height)//2-label_box[1]
  if is_button:
   rgb=original.getpixel((16,original.height//2))[:3]
   panel=Image.new('RGBA',original.size,rgb)
   panel.putalpha(alpha)
   ImageDraw.Draw(panel).text((x,y),label,font=font,fill='white',direction='rtl')
  else:
   mask=Image.new('L',original.size)
   ImageDraw.Draw(mask).text((x,y),label,font=font,fill=alpha.getextrema()[1],direction='rtl')
   panel=Image.new('RGBA',original.size,'white')
   panel.putalpha(mask)
  texture.set_image(panel,target_format=4)
  texture.save()
  rebuilt[name]=panel.tobytes()
 expected=labels.keys()|button_labels.keys()
 assert rebuilt.keys()==expected,'Missing expected interface images'
 output=env.file.save(packer='lz4')
 actual={o.peek_name():o.read().image.convert('RGBA').tobytes() for o in UnityPy.load(output).objects if o.type.name=='Texture2D' and o.peek_name() in expected}
 assert actual==rebuilt
 return output

def interface_font_plan(root, data_root, platform, font_path):
 relative=(data_root/'sharedassets2.assets').as_posix()
 source=(root/relative).read_bytes()
 recipe,installed_sha=build_interface_font_recipe(source,font_path)
 target=ROOT/'payload'/platform/'interface-font.json'
 text.writejson(target,recipe)
 return {
  'path':relative,
  'operation':'modify',
  'original_sha256':recipe['input_sha256'],
  'installed_sha256':installed_sha,
  'source':{'kind':'reconstruct','path':str(target.relative_to(ROOT)),'sha256':sha(target.read_bytes()),'input':relative},
 }

def build(mac,windows,font_path):
 report=text.validate()
 rows={r['key']:r for r in text.readcsv()}
 translations={key:row['hebrew'] for key,row in rows.items() if row['hebrew']}
 overrides=json.loads((ROOT/'interface-text.json').read_text())
 for key,override in overrides.items():
  assert rows[key]['english']==override['english'],f'Interface source text changed: {key}'
  translations[key]=override['hebrew']
 manifest={'schema_version':1,'scummvm_id':'disco-elysium','language':'he','version':VERSION,'release_revision':3,'filename':f'Disco-Elysium-Hebrew-{VERSION}.zip','platforms':{}}
 fingerprints={};glyphs={}
 for platform,root,base in [('osx',mac,'Contents/Resources/Data/StreamingAssets/AssetBundles/OSX'),('windows',windows,'disco_Data/StreamingAssets/AssetBundles/Windows')]:
  assert root.is_dir();files=[]
  for kind in ['lockits','fonts','images']:
   rel=f'{base}/{kind}/arabic';source=root/rel;before=source.read_bytes()
   if kind=='lockits':after=lockit_bundle(source,translations)
   elif kind=='fonts':after,glyphs[platform]=font_bundle(source,translations,font_path)
   else:after=interface_image_bundle(source,translations,font_path)
   delta=bsdiff4.diff(before,after);assert bsdiff4.patch(before,delta)==after
   target=ROOT/'payload'/platform/(kind+'.bsdiff');target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(delta)
   files.append({'path':rel,'operation':'modify','original_sha256':sha(before),'installed_sha256':sha(after),'source':{'kind':'bsdiff','path':str(target.relative_to(ROOT)),'sha256':sha(delta),'input':rel}})
   fingerprints[f'{platform}/{kind}']={'source_sha256':sha(before),'output_sha256':sha(after),'delta_bytes':len(delta)}
  data_root=Path(base).parents[2]
  for kind,relative_path,transform in [
   ('loading-literal','il2cpp_data/Metadata/global-metadata.dat',replace_loading_label),
   ('language-display-name','resources.assets',replace_language_display_name),
  ]:
   rel=(data_root/relative_path).as_posix()
   before=(root/rel).read_bytes()
   after=transform(before)
   delta=bsdiff4.diff(before,after)
   assert bsdiff4.patch(before,delta)==after
   target=ROOT/'payload'/platform/(kind+'.bsdiff')
   target.write_bytes(delta)
   files.append({'path':rel,'operation':'modify','original_sha256':sha(before),'installed_sha256':sha(after),'source':{'kind':'bsdiff','path':str(target.relative_to(ROOT)),'sha256':sha(delta),'input':rel}})
   fingerprints[f'{platform}/{kind}']={'source_sha256':sha(before),'output_sha256':sha(after),'delta_bytes':len(delta)}
  font_plan=interface_font_plan(root,data_root,platform,font_path)
  files.append(font_plan)
  fingerprints[f'{platform}/interface-font']={'source_sha256':font_plan['original_sha256'],'output_sha256':font_plan['installed_sha256']}
  manifest['platforms'][platform]={'files':files}
 text.writejson(ROOT/'patch.json',manifest)
 text.writejson(ROOT/'package-build.json',{'schema_version':1,'csv_sha256':sha((ROOT/'translations.csv').read_bytes()),'compiler_sha256':sha(Path(__file__).read_bytes()),'translated':len(translations),'source_entries':report['entries'],'font':'Assistant and Noto Sans','font_sha256':sha(font_path.read_bytes()),'fallback_font_sha256':sha(FALLBACK.read_bytes()),'font_license':'SIL Open Font License 1.1','glyphs_added':glyphs,'platforms':fingerprints,'release_ready':False})
 print(json.dumps({'translated':len(translations),'deltas':fingerprints}))
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--mac',type=Path,required=True);p.add_argument('--windows',type=Path,required=True);p.add_argument('--font',type=Path,required=True);a=p.parse_args();build(a.mac.resolve(),a.windows.resolve(),a.font.resolve())
