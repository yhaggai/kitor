"""Build an optional hn locale from installed Hebrew; retain he byte-for-byte.
Install/uninstall with manage.py and a separate recovery journal. Remove this
add-on before uninstalling/upgrading the underlying Kitor Hebrew patch.
"""
import argparse,csv,io,json,re,unicodedata,base64
from pathlib import Path
from PIL import Image,ImageDraw
from build import compile_runtime,digest,wrap,catalog,TOKENS,NORMALIZE
from collections import Counter
from fonts import glyph,png
from layout import layout,plain
SAMPLES={
 '0001':'נֶעֱרָצִים.',
 '0002':'נִשְׂגָּבִים.',
 '0003':'חֲכָמִים.',
 '0004':'לְכָךְ אָנוּ, הַיַּנְשׁוּפִים, שׁוֹאֲפִים.',
 '0005':'בְּמֶשֶׁךְ מֵאוֹת שָׁנִים, <30>עָמַלְנוּ<nl>קָשׁוֹת כְּדֵי לִזְכּוֹת בַּכָּבוֹד<nl>שֶׁהַבְּרִיּוֹת רוֹחֲשׁוֹת לָנוּ.',
}
CLUSTER=re.compile('[\u05d0-\u05ea][\u0591-\u05bd\u05bf\u05c1-\u05c2\u05c4-\u05c5\u05c7]+')
def tile(cluster,height):
 base=glyph(cluster[0],height);w=max(5,base.width);out=Image.new('RGBA',(w,height));out.alpha_composite(base,((w-base.width)//2,0));marks_layer=Image.new('RGBA',out.size);d=ImageDraw.Draw(marks_layer);c=w//2
 # Pixel marks are attached to the letter tile, so reveal and BiDi keep them together.
 marks=set(cluster[1:]);dots=[]
 for mark in marks:
  if mark=='\u05b0':dots += [(c,12),(c,14)]
  elif mark=='\u05b4':dots += [(c,13)]
  elif mark=='\u05b5':dots += [(c-1,13),(c+1,13)]
  elif mark=='\u05b6':dots += [(c-1,12),(c+1,12),(c,14)]
  elif mark in ('\u05b7','\u05b8','\u05c7'):d.line((c-1,13,c+1,13),fill='white');dots += [(c,14)] if mark in ('\u05b8','\u05c7') else []
  elif mark in ('\u05b1','\u05b2','\u05b3'):
   dots += [(0,12),(0,14)]
   if mark=='\u05b1':dots += [(2,12),(4,12),(3,14)]
   else:
    d.line((2,13,4,13),fill='white')
    if mark=='\u05b3':dots.append((3,14))
  elif mark in ('\u05b9','\u05ba'):dots += [(0,1)]
  elif mark=='\u05bc':dots += [(0 if cluster[0]=='ו' else c,7)]
  elif mark=='\u05c1':dots += [(w-1,1)]
  elif mark=='\u05c2':dots += [(0,1)]
  elif mark=='\u05bd':d.line((0,12,0,14),fill='white')
  elif mark=='\u05bf':d.line((c-1,1,c+1,1),fill='white')
  elif mark=='\u05bb':dots += [(c-1,12),(c,13),(c+1,14)]
  else:raise ValueError('Unsupported mark '+repr(mark))
 for x,y in dots:d.point((max(0,min(w-1,x)),y),fill='white')
 if cluster[0] in 'ךקןץ':
  lower=marks_layer.crop((0,12,w,height));marks_layer.paste((0,0,0,0),(0,12,w,height));marks_layer.alpha_composite(lower,(0,15))
 out.alpha_composite(marks_layer)
 return out

def load_pointed():
 result={}
 for path in sorted((Path(__file__).resolve().parents[1]/'translations/niqqud').glob('part-*.json')):
  part=json.loads(path.read_text(encoding='utf-8'));assert not set(result)&set(part);result.update(part)
 base=catalog()
 if set(result)!=set(base):raise ValueError('Incomplete niqqud catalog')
 for key,text in result.items():
  if re.sub('[\u0591-\u05bd\u05bf\u05c1-\u05c2\u05c4-\u05c5\u05c7]','',text)!=base[key]:raise ValueError('Base text changed: '+key)
  if Counter(TOKENS.findall(text))!=Counter(TOKENS.findall(base[key])):raise ValueError('Markup changed: '+key)
 return result

def extend_composites(folder,codes):
 chars=(folder/'charmap.txt').read_text(encoding='utf-8');lines=(folder/'info.txt').read_text(encoding='utf-8').splitlines();header=4 if lines[0].startswith('OffsetY=') else 3
 boxes=[list(map(int,b.split())) for b in lines[header].split(',')]
 if folder.name=='SpriteFont1':lines[header-1]='20' # Measurement must match hn LineSpacing.
 old=Image.open(folder/(folder.name+'.png')).convert('RGBA');factor=2 if boxes[0][3]>=28 else 1
 height=max(old.height,19*factor);x=old.width+1;tiles=[]
 for cluster,code in codes.items():
  t=tile(cluster,19)
  if factor==2:t=t.resize((t.width*2,38),Image.Resampling.NEAREST)
  tiles.append((x,t));boxes.append([x,0,t.width,t.height]);chars+=code;x+=t.width+1
 if x>16384:raise ValueError('Atlas exceeds safe texture width')
 atlas=Image.new('RGBA',(x,height));atlas.paste(old,(0,0))
 for x,t in tiles:atlas.paste(t,(x,0))
 return {folder.name+'.png':png(atlas),'charmap.txt':chars.encode(),'info.txt':('\n'.join(lines[:header])+'\n'+','.join(' '.join(map(str,b)) for b in boxes)+'\n').encode()},dict(zip(chars,[b[2]+float(lines[header-3]) for b in boxes]))

def build(game,original,out,full=False):
 game=game.resolve();out.mkdir(parents=True,exist_ok=True);payload=out/'payload';changes=[]
 def put(path,data):
  target=payload/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(data);old=game/path
  changes.append(dict(path=path,original_sha256=digest(old.read_bytes()) if old.exists() else None,installed_sha256=digest(data)))
 loc=game/'Data/Localizations';rows=list(csv.reader((loc/'he.csv').open(encoding='utf-8-sig',newline='')))
 # Canonical order makes equivalent Unicode mark sequences share a glyph.
 samples={k:unicodedata.normalize('NFD',v.translate(NORMALIZE)) for k,v in (load_pointed() if full else SAMPLES).items()}
 clusters=sorted({m.group() for s in samples.values() for m in CLUSTER.finditer(s)})
 codes={cluster:chr(0xe200+i) for i,cluster in enumerate(clusters)};reverse={v:k for k,v in codes.items()}
 widths=None
 for folder in sorted((loc/'en').iterdir()):
  if not (folder/'info.txt').exists():continue
  source=loc/'he/SpriteFont1' if folder.name=='SpriteFont1' else folder
  files,metrics=extend_composites(source,codes)
  target='hn' if folder.name=='SpriteFont1' else 'en'
  for name,data in files.items():put(f'Data/Localizations/{target}/{folder.name}/{name}',data)
  if folder.name=='SpriteFont1':widths=metrics
 for path in (loc/'he').rglob('*'):
  if path.is_file() and 'SpriteFont1' not in path.parts and path.name not in ('layouts.tsv','speechBubbleConfig.ini'):put('Data/Localizations/hn/'+path.relative_to(loc/'he').as_posix(),path.read_bytes())
 for i,row in enumerate(rows[1:]):
  if row[2]=='language_id':row[3]='hn'
  elif f'{i:04d}' in samples:
   encoded=CLUSTER.sub(lambda m:codes[m.group()],samples[f'{i:04d}'])
   row[3]=encoded if row[4]=='default.ini' else wrap(encoded,widths)
 def csvbytes(data):
  b=io.StringIO(newline='');csv.writer(b).writerows(data);return b.getvalue().encode()
 put('Data/Localizations/hn.csv',csvbytes(rows))
 master=list(csv.reader((loc/'Translations.csv').open(encoding='utf-8-sig',newline='')))
 if any('hn' in r for r in master[:2]):raise ValueError('Trial already registered')
 for i,row in enumerate(master):row.insert(len(row)-2,('Hebrew (Niqqud beta)' if full else 'Hebrew (Niqqud trial)') if i==0 else 'hn' if i==1 else '')
 put('Data/Localizations/Translations.csv',csvbytes(master));put('Data/Localizations/hn/speechBubbleConfig.ini',b'[Config]\nLineSpacing=20\n')
 flag=Image.open(loc/'Flags/he.png').convert('RGBA');d=ImageDraw.Draw(flag);d.rectangle((105,30,140,60),fill='white');d.text((108,34),'N',fill='#2465af',stroke_width=1);d.ellipse((114,49,117,52),fill='#2465af');put('Data/Localizations/Flags/hn.png',png(flag))
 records=[];b64=lambda s:base64.b64encode(s.encode()).decode()
 for line in sorted({l for r in rows[2:] for variant in (r[3],r[3].split('|',1)[0].lstrip('*')) for l in plain(variant).split('\n') if l}):
  # Resolve direction using each cluster's Hebrew base; positions stay one-per-tile.
  skeleton=''.join(reverse[c][0] if c in reverse else c for c in line);visual,order=layout(skeleton)
  painted=''.join(line[j] if line[j] in reverse else visual[i] for i,j in enumerate(order))
  records.append('\t'.join([b64(line),b64(painted),','.join(map(str,order))]))
 put('Data/Localizations/hn/layouts.tsv',('\n'.join(records)+'\n').encode())
 compile_runtime(original,out,niqqud=True);put('OwlboyHebrew.dll',(out/'OwlboyHebrew.dll').read_bytes())
 plan=dict(schema=1,id='owlboy-hebrew',version='niqqud-beta-1' if full else 'niqqud-trial-1',source_exe_sha256=digest((game/'Owlboy.exe').read_bytes()),files=changes)
 (out/'plan.json').write_text(json.dumps(plan,indent=2)+'\n');(out/'samples.json').write_text(json.dumps(samples,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');(out/'clusters.json').write_text(json.dumps(codes,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
 print('Built separate hn language:',len(samples),'entries;',len(codes),'composite glyphs;',len(changes),'affected files')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--full',action='store_true');p.add_argument('--game',type=Path,required=True);p.add_argument('--original',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();build(a.game,a.original,a.out,a.full)
