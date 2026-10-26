"""Organize visually checked new pairs without modifying source bytes."""
import hashlib,json,shutil
from pathlib import Path
from PIL import Image
R=Path(__file__).resolve().parent
A=R.parent.parent
q=json.loads((R/'queue.json').read_text())
m=json.loads((A/'covers/manifest.json').read_text())
backup=R/'manifest-before-import.json'
if not backup.exists():backup.write_text(json.dumps(m,ensure_ascii=False,indent=2)+'\n')
known={g['game_id'] for g in m['games']}
verification={r['game_id']:r for r in json.loads((R/'pixel-verification.json').read_text())}
stage=R/'upload'/'covers';stage.mkdir(parents=True,exist_ok=True)
new=[]
for g in q['games']:
 s=g.get('sources',{})
 if g['status']=='existing':continue
 if g['source_idx'] in [81,98]:
  g['title_he']={81:'סתירה',98:'רוורסיה'}[g['source_idx']]
  g['title_he_provenance']='Proposed translation; Hebrew title was blank in input JSON.'
 if not all(x in s for x in ['en','he']):continue
 item={'game_id':g['game_id'],'title':g['title_en'],'title_he':g['title_he'],'source_idx':g['source_idx'],'import_date':'2026-10-25','covers':{}}
 for lang in ['en','he']:
  src=R/s[lang]['file'];data=src.read_bytes();sha=hashlib.sha256(data).hexdigest()
  with Image.open(src) as im:
   ext={'JPEG':'jpg','PNG':'png','WEBP':'webp','GIF':'gif'}.get(im.format,src.suffix.lstrip('.'));w,h=im.size
  key=f'covers/{g["game_id"]}/{lang}/{sha[:12]}.{ext}'
  dst=A/key;dst.parent.mkdir(parents=True,exist_ok=True);dst.write_bytes(data)
  staged=R/'upload'/key;staged.parent.mkdir(parents=True,exist_ok=True);staged.write_bytes(data)
  assert dst.read_bytes()==data
  v={'file':key,'source_file':str(src.relative_to(A)),'sha256':sha,'bytes':len(data),'width':w,'height':h,'object_key':key,'original_url':m['base_url']+'/'+key,'variant_urls':{str(size):m['base_url']+f'/cdn-cgi/image/width={size},fit=scale-down,quality=85,format=auto/'+key for size in [240,480,960]},'source_page':s[lang].get('source_page',s['en'].get('source_page')),'image_url':s[lang].get('url'),'kind':s[lang].get('kind','original-cover'),'source':'AI title localization and masked composite' if lang=='he' and g['game_id'] in verification else 'Original source image'}
  if lang=='he' and g['game_id'] in verification:
   v.update(derived_from=str((R/s['en']['file']).relative_to(A)),note='Original pixels are identical outside recorded title masks; background beneath replaced text is reconstructed.',pixel_verification=verification[g['game_id']])
  elif lang=='he':v['note']='Existing Hebrew retail scan; packaging may differ from the English edition.'
  item['covers'][lang]=v;s[lang]['status']='visually-reviewed';s[lang]['object_key']=key
 g['status']='ready-to-publish'
 if g['game_id'] not in known:m['games'].append(item);known.add(g['game_id'])
 else:m['games']=[item if old['game_id']==g['game_id'] else old for old in m['games']]
 new.append(item)
exceptions={65:('native-hebrew-source-needed','Original Hebrew game, no English original box located; do not invent an English original.'),100:('generation-blocked','Built-in image generation declined the cover edit. No retry or bypass performed.'),171:('original-cover-needed','Developer page provides a gameplay screenshot, but no usable original cover was located.')}
for g in q['games']:
 if g['source_idx'] in exceptions:g['status'],g['note']=exceptions[g['source_idx']]
m['updated_on']='2026-10-25'
(A/'covers/manifest.json').write_text(json.dumps(m,ensure_ascii=False,indent=2)+'\n')
shutil.copy2(A/'covers/manifest.json',stage/'manifest.json')
(R/'publish-manifest.json').write_text(json.dumps({'bucket':m['bucket'],'base_url':m['base_url'],'games':new},ensure_ascii=False,indent=2)+'\n')
(R/'queue.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n')
print(f'{len(new)} new pairs; {len(m["games"])} total pairs; {sum(x.stat().st_size for x in stage.rglob("*") if x.is_file())/1048576:.1f} MiB staged')
