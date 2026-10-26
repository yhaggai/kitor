#!/usr/bin/env python3
"""Add only files changed by ad-hoc signing a disposable patched Mac bundle.
Run codesign on a COPY after archive reconstruction; never sign the owned source.
"""
import argparse,hashlib,json,runpy
from pathlib import Path
ROOT=Path(__file__).resolve().parent
helper=runpy.run_path(str(ROOT.parents[1]/'tentacle/he/build_recipes.py'))
def h(b):return hashlib.sha256(b).hexdigest()
def main():
 ap=argparse.ArgumentParser();ap.add_argument('original',type=Path);ap.add_argument('signed_copy',type=Path);a=ap.parse_args()
 manifest_path=ROOT/'patch.osx.candidate.json';manifest=json.loads(manifest_path.read_text());files=manifest['platforms']['osx']['files'][:1];changes=[]
 for path in sorted(a.signed_copy.rglob('*')):
  if not path.is_file() or path.is_symlink():continue
  rel=path.relative_to(a.signed_copy)
  if rel.as_posix()=='Contents/Resources/tenta.cle':continue
  original=a.original/rel;assert original.is_file() and not original.is_symlink(),rel
  before=original.read_bytes();after=path.read_bytes()
  if before==after:continue
  name=f'recipes/signature-{len(changes)}.json';recipe={'schema_version':1,'input_sha256':h(before),'output_size':len(after),'operations':helper['delta'](before,after,0)}
  helper['save'](ROOT/name,recipe)
  files.append({'path':'Dott.app/'+rel.as_posix(),'operation':'modify','original_sha256':h(before),'installed_sha256':h(after),'source':{'kind':'reconstruct','path':name,'sha256':h((ROOT/name).read_bytes()),'input':'Dott.app/'+rel.as_posix()}})
  changes.append(rel.as_posix())
 manifest['platforms']['osx']['files']=files;helper['save'](manifest_path,manifest);print(json.dumps(changes,indent=2))
if __name__=='__main__':main()
