"""Build a local-only font proof of concept from an owned Owlboy installation.
Original hand-drawn Hebrew glyphs. English assets remain local build inputs.
Requires Pillow; no game files are modified by this builder.
"""
import argparse, csv, hashlib, io, json
from pathlib import Path
from PIL import Image, ImageDraw
ROOT=Path(__file__).resolve().parent
# Seven-pixel body; lamed rises above it, final forms descend below it.
PATTERNS={
'א':['10001','10010','01010','00100','01010','10001','10001'],
'ב':['11110','00010','00010','00010','00010','00010','11111'],
'ג':['011','001','001','001','011','101','101'],
'ד':['11111','00010','00010','00010','00010','00010','00010'],
'ה':['11111','00001','00001','10001','10001','10001','10001'],
'ו':['11','01','01','01','01','01','01'],
'ז':['111','010','010','010','010','010','010'],
'ח':['11111','10001','10001','10001','10001','10001','10001'],
'ט':['10011','10101','10001','10001','10001','10001','01110'],
'י':['11','01','01'],
'ך':['11111','00001','00001','00001','00001','00001','00001','00001','00001'],
'כ':['11110','00001','00001','00001','00001','00001','11110'],
'ל':['10000','10000','11111','00001','00001','00010','00100','01000','01000'],
'ם':['11111','10001','10001','10001','10001','10001','11111'],
'מ':['10110','11001','10001','10001','10001','10001','10111'],
'ן':['11','01','01','01','01','01','01','01','01'],
'נ':['011','001','001','001','001','001','111'],
'ס':['11110','10001','10001','10001','10001','10001','01110'],
'ע':['10001','10001','10001','01001','01001','01010','11100'],
'ף':['11111','10001','11001','00001','00001','00001','00001','00001','00001'],
'פ':['11110','10001','11001','00001','00001','00001','11110'],
'ץ':['10001','10001','01010','00100','00010','00010','00010','00010','00010'],
'צ':['10001','10001','01010','00100','00010','00001','11111'],
'ק':['11111','00001','00001','10001','10001','10010','10000','10000','10000'],
'ר':['11110','00001','00001','00001','00001','00001','00001'],
'ש':['10101','10101','10101','10101','10101','11001','11111'],
'ת':['01111','01001','01001','01001','01001','01001','11001']}
TRANSLATIONS={
'Revered.':'מכובדים.', 'Exalted.':'נעלים.', 'Wise.':'חכמים.',
'This is what we owls aspire to be.':'כך אנו, הינשופים, שואפים להיות.',
'No need to look so nervous!':'אין צורך להילחץ!',
'This is a great moment for you!':'זהו רגע גדול בשבילך!',
"I'll be honest with you, Otus.":'אהיה כן איתך, אוטוס.',
'So today we\'ll be doing some flying.':'אז היום נתרגל תעופה.',
'Start new game':'משחק חדש','Continue':'המשך','Languages':'שפות',
'Options':'אפשרויות','Exit game':'יציאה','Select a language':'בחירת שפה'}
def sha(data):return hashlib.sha256(data).hexdigest()
def main():
 ap=argparse.ArgumentParser();ap.add_argument('resources',type=Path);args=ap.parse_args()
 source=args.resources.resolve();loc=source/'Data/Localizations';payload=ROOT/'payload';payload.mkdir(exist_ok=True)
 changes=[]
 def put(rel,data):
  dest=payload/rel;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(data)
  original=source/rel
  entry={'path':rel,'operation':'modify' if original.exists() else 'create','source':{'kind':'copy','path':'payload/'+rel,'sha256':sha(data)},'installed_sha256':sha(data)}
  if original.exists():entry['original_sha256']=sha(original.read_bytes())
  changes.append(entry)
 def png(im):
  b=io.BytesIO();im.save(b,format='PNG');return b.getvalue()
 # Dialogue uses locale-specific SpriteFont1. Other font IDs are hardcoded to en.
 for folder in sorted((loc/'en').iterdir()):
  if not (folder/'info.txt').exists():continue
  name=folder.name;chars=(folder/'charmap.txt').read_text();lines=(folder/'info.txt').read_text().splitlines()
  rects=[list(map(int,r.split())) for r in lines[3].split(',')]
  assert len(chars)==len(rects)
  original=Image.open(folder/(name+'.png')).convert('RGBA')
  h=rects[0][3];factor=2 if h>=28 else 1
  x=original.width+1;glyphs=[]
  for c,pattern in PATTERNS.items():
   w=len(pattern[0])*factor;tile=Image.new('RGBA',(w,h));d=ImageDraw.Draw(tile)
   top=(3 if c=='ל' else 5)*factor
   if h==14:top=(1 if c=='ל' else 3)
   for y,row in enumerate(pattern):
    for xx,v in enumerate(row):
     if v=='1':d.rectangle((xx*factor,top+y*factor,(xx+1)*factor-1,top+(y+1)*factor-1),fill='white')
   assert tile.getbbox() and tile.getbbox()[3]<=h
   rects.append([x,0,w,h]);glyphs.append((x,tile));x+=w+1;chars+=c
  atlas=Image.new('RGBA',(x,h));atlas.paste(original,(0,0))
  for gx,tile in glyphs:atlas.paste(tile,(gx,0))
  target='he' if name=='SpriteFont1' else 'en'
  prefix=f'Data/Localizations/{target}/{name}/'
  put(prefix+name+'.png',png(atlas));put(prefix+'charmap.txt',chars.encode())
  put(prefix+'info.txt',('\n'.join(lines[:3])+ '\n'+','.join(' '.join(map(str,r)) for r in rects)+'\n').encode())
  if name=='SpriteFont1': preview_atlas=atlas;preview_chars=chars;preview_rects=rects
 put('Data/Localizations/he/speechBubbleConfig.ini',b'[Config]\nLineSpacing=15\n')
 # Pure Hebrew + punctuation only: deliberately limited visual-order prototype.
 rows=list(csv.reader((loc/'de.csv').open(encoding='utf-8-sig')))
 translated=0
 for r in rows[1:]:
  if r[2]=='language_id':r[3]='he'
  else:
   logical=TRANSLATIONS.get(r[2]);r[3]=logical[::-1] if logical else r[2]
   translated+=logical is not None
 out=io.StringIO(newline='');csv.writer(out).writerows(rows);put('Data/Localizations/he.csv',out.getvalue().encode())
 master=list(csv.reader((loc/'Translations.csv').open(encoding='utf-8-sig')))
 for i,r in enumerate(master):r.insert(len(r)-2,'Hebrew prototype' if i==0 else 'he' if i==1 else '')
 out=io.StringIO(newline='');csv.writer(out).writerows(master);put('Data/Localizations/Translations.csv',out.getvalue().encode())
 flag=Image.new('RGBA',(24,16),'white');d=ImageDraw.Draw(flag);d.rectangle((0,2,23,4),fill='#2465af');d.rectangle((0,11,23,13),fill='#2465af')
 d.polygon([(12,5),(8,10),(16,10)],outline='#2465af');d.polygon([(12,11),(8,6),(16,6)],outline='#2465af')
 put('Data/Localizations/Flags/he.png',png(flag))
 manifest={'schema_version':1,'scummvm_id':'owlboy','language':'he','version':'0.1.0-font-prototype','filename':'owlboy-he-font-prototype.zip','platforms':{'windows':{},'osx':{}},'files':changes}
 (ROOT/'patch.json').write_text(json.dumps(manifest,indent=2)+'\n')
 (ROOT/'translations.json').write_text(json.dumps(TRANSLATIONS,ensure_ascii=False,indent=2)+'\n')
 preview=Image.new('RGBA',(380,145),'#182330');d=ImageDraw.Draw(preview)
 d.text((12,8),'OWLBOY / HEBREW FONT PROTOTYPE',fill='#a2c9db')
 samples=['אבגדהוזחטיכלמנסעפצקרשת','ךםןףץ','מכובדים.','כך אנו, הינשופים, שואפים להיות.','משחק חדש']
 for i,s in enumerate(samples):
  visual=s[::-1];width=sum(preview_rects[preview_chars.index(c)][2]+1.5 for c in visual)
  x=preview.width-15-width;y=28+i*22
  for c in visual:
   gx,gy,w,h=preview_rects[preview_chars.index(c)];tile=preview_atlas.crop((gx,gy,gx+w,gy+h));preview.alpha_composite(tile,(round(x),y));x+=w+1.5
 preview.resize((1140,435),Image.Resampling.NEAREST).save(ROOT/'font-preview.png')
 (ROOT/'provenance.json').write_text(json.dumps({'status':'experimental','release_ready':False,'edition':'Steam macOS installed build; Windows layout unverified','scope':'Font prototype, not a full translation','translated_rows':translated,'glyphs':27,'rtl':'Visual-order strings only; typewriter still LTR','install_root':'Owlboy.app/Contents/Resources on macOS; Owlboy.exe directory on Windows','source_exe_sha256':sha((source/'Owlboy.exe').read_bytes()),'native_validation':{'macOS':'pending','Windows':'not run'},'font_origin':'Original hand-drawn Hebrew bitmaps in build_prototype.py; original English atlas retained locally'},indent=2)+'\n')
 print(f'Built {len(changes)} files, {translated} translated rows, 27 Hebrew glyphs.')
if __name__=='__main__':main()
