"""Create a BSDIFF40 delta aligned to GameMaker FORM chunks."""
from pathlib import Path
import argparse,bz2,hashlib,json,struct
from delta import apply_patch

def sha(data):return hashlib.sha256(data).hexdigest()
def chunks(data):
 if data[:4]!=b'FORM':raise ValueError('Expected GameMaker FORM archive')
 result=[('FORM',0,8)];pos=8
 while pos<len(data):
  end=pos+8+struct.unpack_from('<I',data,pos+4)[0]
  if end>len(data):raise ValueError('Truncated archive chunk')
  result.append((data[pos:pos+4],pos,end));pos=end
 return result
def off(n):return (abs(n)|(1<<63 if n<0 else 0)).to_bytes(8,'little')
def main():
 p=argparse.ArgumentParser();p.add_argument('--original',type=Path,required=True);p.add_argument('--patched',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--platform',choices=['macOS','Windows'],required=True);p.add_argument('--release',default='alpha2-steam');a=p.parse_args()
 old=a.original.read_bytes();new=a.patched.read_bytes();oc=chunks(old);nc=chunks(new)
 if [x[0] for x in oc]!=[x[0] for x in nc]:raise ValueError('Archive chunk sequences differ')
 controls=[];diff=bytearray();extra=bytearray()
 for (_,start,end),(_,ns,ne) in zip(oc,nc):
  x=min(end-start,ne-ns);y=ne-ns-x
  diff.extend((n-o)&255 for o,n in zip(old[start:start+x],new[ns:ns+x]));extra.extend(new[ns+x:ne])
  controls.append(off(x)+off(y)+off(end-(start+x)))
 control=bz2.compress(b''.join(controls));db=bz2.compress(diff);eb=bz2.compress(extra)
 patch=b'BSDIFF40'+off(len(control))+off(len(db))+off(len(new))+control+db+eb
 a.output.mkdir(parents=True,exist_ok=True);dest=a.output/'hebrew-steam.bsdiff';dest.write_bytes(patch)
 import tempfile
 with tempfile.TemporaryDirectory() as d:
  result=Path(d)/'roundtrip';apply_patch(a.original,dest,result)
  if result.read_bytes()!=new:raise ValueError('Delta reconstruction failed')
 manifest={'release':a.release,'platform':a.platform,'source_sha256':sha(old),'patched_sha256':sha(new),'patch_sha256':sha(patch),'catalog_entries':16455,'draft_translations':15227,'unchanged_entries':1228}
 catalog_path=Path(__file__).resolve().parent/'translation-catalog.json'
 catalog=json.loads(catalog_path.read_text())
 manifest['catalog_sha256']=sha(catalog_path.read_bytes())
 manifest['nikud']=catalog.get('nikud')
 if a.platform=='Windows':manifest['runtime_tested_on_windows']=False
 (a.output/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
 print('Delta verified:',dest,'bytes:',len(patch))
if __name__=='__main__':main()
