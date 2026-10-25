"""Repack inspected Graft/VPatch data as BSDIFF40 without executing installers.

Graft: bsdiff crate 0.2.1 interleaved controls. VPatch: MD5 mode, copy/literal
blocks and timestamp metadata. Publisher variants and compatibility hashes are
returned explicitly; this tool never chooses a game directory or modifies one.
"""
import bz2
import struct
def num(n):return (abs(n)|(1<<63 if n<0 else 0)).to_bytes(8,'little')
def emit(ctrl,diff,extra,size):
 normalized=bytearray()
 for i in range(0,len(ctrl),24):
  record=ctrl[i:i+24];x,y=struct.unpack('<QQ',record[:16]);z=int.from_bytes(record[16:],'little');z=-(z&((1<<63)-1)) if z>>63 else z
  if x+y==0:
   if normalized:
    prev=int.from_bytes(normalized[-8:],'little');prev=-(prev&((1<<63)-1)) if prev>>63 else prev;normalized[-8:]=num(prev+z)
   elif z:normalized+=record
  else:normalized+=record
 c,d,e=map(bz2.compress,[normalized,diff,extra]);return b'BSDIFF40'+num(len(c))+num(len(d))+num(size)+c+d+e
def rawbsd(raw):
 c=bytearray();d=bytearray();e=bytearray();pos=size=0
 while pos<len(raw):
  head=raw[pos:pos+24];assert len(head)==24;x,y,z=struct.unpack('<QQQ',head);assert x<2**63 and y<2**63;pos+=24
  assert pos+x+y<=len(raw);c+=head;d+=raw[pos:pos+x];e+=raw[pos+x:pos+x+y];pos+=x+y;size+=x+y
 assert pos==len(raw);return emit(c,d,e,size)
def vpat(raw):
 assert raw[:4]==b'VPAT';count=int.from_bytes(raw[4:8],'little');assert count&0x80000000;pos=8;variants=[]
 for _ in range(count&0xffffff):
  n=int.from_bytes(raw[pos:pos+4],'little');orig=raw[pos+4:pos+20].hex();final=raw[pos+20:pos+36].hex();length=int.from_bytes(raw[pos+36:pos+40],'little');pos+=40;end=pos+length
  c=bytearray();d=bytearray();e=bytearray();old=size=0
  for i in range(n):
   op=raw[pos];pos+=1
   if op==255:pos+=8;continue
   assert op in [1,2,3,5,6,7];width={1:1,2:2,3:4,5:1,6:2,7:4}[op];length=int.from_bytes(raw[pos:pos+width],'little');pos+=width;assert length>0
   if op<=3:
    offset=int.from_bytes(raw[pos:pos+4],'little');pos+=4;c+=num(0)+num(0)+num(offset-old);c+=num(length)+num(0)+num(0);d+=bytes(length);old=offset+length
   else:c+=num(0)+num(length)+num(0);e+=raw[pos:pos+length];pos+=length
   size+=length
  assert pos==end,(pos,end);variants.append((orig,final,emit(c,d,e,size)))
 assert pos==len(raw);return variants

if __name__ == "__main__":
    import argparse, json
    from pathlib import Path
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("format", choices=["graft", "vpatch"])
    p.add_argument("input", type=Path)
    p.add_argument("output", type=Path)
    p.add_argument("--variant", type=int, default=0)
    args=p.parse_args()
    if args.format == "graft":
        output=rawbsd(args.input.read_bytes())
    else:
        variants=vpat(args.input.read_bytes())
        original, installed, output=variants[args.variant]
        print(json.dumps({"original_md5":original,"installed_md5":installed,"variant":args.variant}))
    args.output.write_bytes(output)
