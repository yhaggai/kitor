"""Import a locally built Owlboy Hebrew package as game-dependent BSDIFF assets."""
import argparse, hashlib, json, shutil
from pathlib import Path
import bsdiff4
ROOT=Path(__file__).resolve().parent
p=argparse.ArgumentParser();p.add_argument('--package',type=Path,required=True);p.add_argument('--original',type=Path,required=True);a=p.parse_args()
hash=lambda b:hashlib.sha256(b).hexdigest()
plan=json.loads((a.package/'plan.json').read_text());changes=[]
for entry in plan['files']:
 name=entry['path'];data=(a.package/'payload'/name).read_bytes();assert hash(data)==entry['installed_sha256']
 base=name if entry['original_sha256'] else ('Data/Localizations/de.csv' if name=='Data/Localizations/he.csv' else name.replace('/he/SpriteFont1/','/en/SpriteFont1/') if '/he/SpriteFont1/' in name else None)
 change={'path':name,'operation':'modify' if entry['original_sha256'] else 'create','installed_sha256':hash(data)}
 if entry['original_sha256']:change['original_sha256']=entry['original_sha256']
 if base:
  old=(a.original/base).read_bytes()
  if entry['original_sha256']:assert hash(old)==entry['original_sha256']
  output='deltas/'+name+'.bsdiff';content=bsdiff4.diff(old,data)
  assert bsdiff4.patch(old,content)==data
  change['source']={'kind':'bsdiff','path':output,'input':base,'sha256':hash(content)}
 else:
  output='assets/'+name;content=data;change['source']={'kind':'copy','path':output,'sha256':hash(content)}
 dest=ROOT/output;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(content);changes.append(change)
manifest={'schema_version':1,'scummvm_id':'owlboy','language':'he','version':'0.2.0-beta.2-kitor1','filename':'owlboy-he-0.2.0-beta.2-kitor1-osx.zip','platforms':{'osx':{'files':changes}}}
(ROOT/'patch.osx.candidate.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(f'Imported {len(changes)} tracked file changes; macOS edition only.')
