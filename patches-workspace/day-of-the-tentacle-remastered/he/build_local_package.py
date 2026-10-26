#!/usr/bin/env python3
"""Package the existing local remaster prototype. Personal-use candidate only."""
import argparse,hashlib,json,mmap,runpy,struct
from pathlib import Path
ROOT=Path(__file__).resolve().parent
helper=runpy.run_path(str(ROOT.parents[1]/'tentacle/he/build_recipes.py'))
def h(b):return hashlib.sha256(b).hexdigest()
def main():
 ap=argparse.ArgumentParser();ap.add_argument('original',type=Path);ap.add_argument('translated',type=Path);a=ap.parse_args()
 with a.original.open('rb') as f,a.translated.open('rb') as g,mmap.mmap(f.fileno(),0,access=mmap.ACCESS_READ) as src,mmap.mmap(g.fileno(),0,access=mmap.ACCESS_READ) as dst:
  original=helper['members'](src);translated=helper['members'](dst);start=struct.unpack_from('<I',src,20)[0]
  assert struct.unpack_from('<I',dst,20)[0]==start
  assert src[start:]==dst[start:len(src)],'Prototype unexpectedly modifies original archive data'
  ops=helper['delta'](src[:start],dst[:start],0);helper['copy'](ops,start,len(src)-start)
  position=len(src)
  for name,(off,size) in sorted(translated.items(),key=lambda x:x[1][0]):
   if off<len(src):continue
   assert off==position,(name,off,position)
   old,n=original[name];ops.extend(helper['delta'](src[old:old+n],dst[off:off+size],old));position+=size
  assert position==len(dst)
  recipe={'schema_version':1,'input_sha256':h(src),'output_size':len(dst),'operations':ops}
  (ROOT/'recipes').mkdir(exist_ok=True);p=ROOT/'recipes/tenta.cle.json';helper['save'](p,recipe)
  files={}
  for platform,archive in [('osx','Dott.app/Contents/Resources/tenta.cle')]:
   files[platform]={'files':[{'path':archive,'operation':'modify','original_sha256':h(src),'installed_sha256':h(dst),'source':{'kind':'reconstruct','path':'recipes/tenta.cle.json','sha256':h(p.read_bytes()),'input':archive}}]}
  helper['save'](ROOT/'patch.osx.candidate.json',{'schema_version':1,'scummvm_id':'day-of-the-tentacle-remastered','language':'he','version':'0.1-local-items-kitor1','filename':'DOTT-Remastered-Hebrew-local-kitor1.zip','platforms':files,'files':[]})
  helper['save'](ROOT/'provenance.json',{'status':'packaged_unverified','release_ready':False,'version':'0.1-local-items-kitor1','source':'prototype/generated/full-hebrew-items-v2/full-manifest.json','input_sha256':h(src),'installed_sha256':h(dst),'redistribution':'Local personal-use only. Prototype uses Arial glyphs; do not publish. Classic translation announcement also prohibits redistribution.','limitations':['579 dialogue records remain English including commentary','Menus and dialogue choices remain untranslated','English language only','Windows runtime and native Mac installation validation pending'],'runtime':'Native remastered engine; never ScummVM'})
  print('Local remastered recipe:',p.stat().st_size,'bytes')
if __name__=='__main__':main()
