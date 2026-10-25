"""BSDIFF40 application using only Python's standard library."""
import bz2
from pathlib import Path

def number(data):
    if len(data)!=8:raise ValueError('Truncated patch integer')
    value=int.from_bytes(data,'little')
    return -(value&((1<<63)-1)) if value&(1<<63) else value

def apply_patch(original,patch,output):
    old=Path(original).read_bytes();p=Path(patch).read_bytes()
    if p[:8]!=b'BSDIFF40' or len(p)<32:raise ValueError('Unsupported patch header')
    control_len,diff_len,size=(number(p[i:i+8]) for i in (8,16,24))
    if min(control_len,diff_len,size)<0 or 32+control_len+diff_len>len(p):raise ValueError('Invalid patch lengths')
    control=bz2.decompress(p[32:32+control_len]);diff=bz2.decompress(p[32+control_len:32+control_len+diff_len]);extra=bz2.decompress(p[32+control_len+diff_len:])
    result=bytearray();oldpos=dp=ep=cp=0
    while len(result)<size:
        if cp+24>len(control):raise ValueError('Truncated patch control block')
        x,y,z=(number(control[i:i+8]) for i in range(cp,cp+24,8));cp+=24
        if min(x,y)<0 or x+y==0 or len(result)+x+y>size or dp+x>len(diff) or ep+y>len(extra):raise ValueError('Invalid patch control instruction')
        end=oldpos+x
        # Bytes outside the old file contribute zero, as in the BSDIFF format.
        while oldpos<end:
            n=min(1024*1024,end-oldpos);block=bytearray(diff[dp:dp+n]);lo=max(0,-oldpos);hi=min(n,len(old)-oldpos)
            if hi>lo:block[lo:hi]=bytes((a+b)&255 for a,b in zip(block[lo:hi],old[oldpos+lo:oldpos+hi]))
            result.extend(block);oldpos+=n;dp+=n
        result.extend(extra[ep:ep+y]);ep+=y;oldpos+=z
    Path(output).write_bytes(result)

if __name__ == "__main__":
    import sys
    apply_patch(*sys.argv[1:])
