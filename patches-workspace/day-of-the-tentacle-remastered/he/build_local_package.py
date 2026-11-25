#!/usr/bin/env python3
"""Package a local Remastered build as a reconstruction recipe."""
import argparse,hashlib,json,mmap,runpy,struct
from pathlib import Path
ROOT=Path(__file__).resolve().parent
VERSION = '0.1.1-ofl-kitor1'
FILENAME = f'DOTT-Remastered-Hebrew-{VERSION}.zip'
helper=runpy.run_path(str(ROOT.parents[1]/'tentacle/he/build_recipes.py'))
def h(b):return hashlib.sha256(b).hexdigest()
def main():
 ap=argparse.ArgumentParser();ap.add_argument('original',type=Path);ap.add_argument('translated',type=Path);ap.add_argument('--platform',choices=['osx','windows'],default='osx');a=ap.parse_args()
 report_root = a.translated.parent.parent if a.platform == 'windows' else a.translated.parents[3]
 report = json.loads((report_root / 'full-manifest.json').read_text())
 font = Path(report['font_source'])
 expected_font = ROOT / 'prototype/fonts/NotoSansHebrew.ttf'
 if font.resolve() != expected_font or h(font.read_bytes()) != '7ef36a2c3593758cdb622e1bdef4f84523e92fbc3ccc667438dd80ff54c2de88':
  ap.error('The package requires the pinned Noto Sans Hebrew font')
 with a.original.open('rb') as f,a.translated.open('rb') as g,mmap.mmap(f.fileno(),0,access=mmap.ACCESS_READ) as src,mmap.mmap(g.fileno(),0,access=mmap.ACCESS_READ) as dst:
  source_hash=h(src);target_hash=h(dst)
  original=helper['members'](src);translated=helper['members'](dst);start=struct.unpack_from('<I',src,20)[0]
  assert struct.unpack_from('<I',dst,20)[0]==start
  for pos in range(start,len(src),1024*1024):
   assert src[pos:min(pos+1024*1024,len(src))]==dst[pos:min(pos+1024*1024,len(src))],'Prototype unexpectedly modifies original archive data'
  ops=helper['delta'](src[:start],dst[:start],0);helper['copy'](ops,start,len(src)-start)
  position=len(src)
  for name,(off,size) in sorted(translated.items(),key=lambda x:x[1][0]):
   if off<len(src):continue
   assert off==position,(name,off,position)
   old,n=original[name];ops.extend(helper['delta'](src[old:old+n],dst[off:off+size],old));position+=size
  assert position==len(dst)
  recipe={'schema_version':1,'input_sha256':source_hash,'output_size':len(dst),'operations':ops}
  (ROOT/'recipes').mkdir(exist_ok=True);recipe_name='recipes/'+('windows-' if a.platform=='windows' else '')+'tenta.cle.json';p=ROOT/recipe_name;helper['save'](p,recipe)
  files={}
  for platform,archive in [(a.platform,'tenta.cle' if a.platform=='windows' else 'Dott.app/Contents/Resources/tenta.cle')]:
   files[platform]={'files':[{'path':archive,'operation':'modify','original_sha256':source_hash,'installed_sha256':target_hash,'source':{'kind':'reconstruct','path':recipe_name,'sha256':h(p.read_bytes()),'input':archive}}]}
  helper['save'](ROOT/f'patch.{a.platform}.candidate.json',{'schema_version':1,'scummvm_id':'day-of-the-tentacle-remastered','language':'he','version':VERSION,'filename':FILENAME,'platforms':files,'files':[]})
  provenance_path = ROOT / ('provenance.windows.json' if a.platform == 'windows' else 'provenance.json')
  provenance = json.loads(provenance_path.read_text()) if provenance_path.exists() else {}
  provenance.pop('redistribution', None)
  provenance.pop('runtime_validation', None)
  provenance.update(status='packaged_unverified', release_ready=False, version=VERSION,
                    input_sha256=source_hash, installed_sha256=target_hash,
                    source='prototype/build_full_hebrew.py', font_source=font.name,
                    font_sha256=h(font.read_bytes()), licensed_font_notice='licenses/FONT-OFL.txt')
  helper['save'](provenance_path, provenance)
  print('Local remastered recipe:',p.stat().st_size,'bytes')
if __name__=='__main__':main()
