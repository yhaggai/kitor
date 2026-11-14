"""Extract stable logical text paths from a pristine Moomin database bundle."""
import argparse,json
from pathlib import Path
import UnityPy

def walk(value,path=()):
 if isinstance(value,dict):
  for k,v in value.items(): yield from walk(v,path+(k,))
 elif isinstance(value,list):
  for i,v in enumerate(value): yield from walk(v,path+(i,))
 elif isinstance(value,str) and path and path[-1]=='text' and value.strip(): yield path,value

def main():
 ap=argparse.ArgumentParser();ap.add_argument('bundle',type=Path);ap.add_argument('output',type=Path);a=ap.parse_args()
 env=UnityPy.load(a.bundle.read_bytes());objs=[o for o in env.objects if o.type.name=='TextAsset' and o.peek_name()=='GameDatabase']
 if len(objs)!=1:raise ValueError(f'Expected one GameDatabase TextAsset, found {len(objs)}')
 db=json.loads(objs[0].read().m_Script);rows=[{'path':list(path),'en':text} for path,text in walk(db)]
 a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_text(json.dumps(rows,ensure_ascii=False,indent=1),encoding='utf8')
 print(f'Extracted {len(rows)} player-facing text fields')
if __name__=='__main__':main()
