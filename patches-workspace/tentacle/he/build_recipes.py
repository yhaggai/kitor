#!/usr/bin/env python3
"""Build source-copy recipes from an owned Steam archive and upstream Hebrew deltas.
Development only: needs xdelta3. Installed packages need only Kitor's native decoder.
Audio layout follows GPL-3.0-or-later remonstered (see licenses/ and provenance).
Never stores base-game audio or original resources in a distributable package.
"""
import argparse, hashlib, json, mmap, struct, subprocess, tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parent

def digest(b):return hashlib.sha256(b).hexdigest()
def save(p,x):p.write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
def members(a):
    assert a[:8]==b'KAPL\x00\x00\x80?','Requires KAPL 1.0'
    offsets=struct.unpack_from('<4I',a,8); sizes=struct.unpack_from('<4I',a,24)
    assert sizes[1]%20==0
    result={}
    names=a[offsets[2]:offsets[2]+sizes[2]].split(b'\0')
    for i,pos in enumerate(range(offsets[1],offsets[1]+sizes[1],20)):
        off,name,compressed,size,flag=struct.unpack_from('<5I',a,pos)
        assert not flag and compressed==size
        result[names[i].decode()]=(offsets[3]+off,size)
    return result

def sounds(a,base,size,prefix):
    magic,version,count,headers,names,data,mode=struct.unpack_from('<4s6I',a,base)
    assert magic==b'FSB5' and mode==11 and version in (0,1)
    start=base+(64 if version==0 else 60); p=start; offsets=[]
    for _ in range(count):
        raw=struct.unpack_from('<Q',a,p)[0];p+=8
        offsets.append(((raw>>6)&((1<<28)-1))*16)
        more=raw&1
        while more:
            c=struct.unpack_from('<I',a,p)[0];p+=4;more=c&1;p+=(c>>1)&0xffffff
    assert p==start+headers
    offsets.append(data); result={}; data_start=p+names
    assert data_start+data<=base+size
    for i in range(count):
        n=p+struct.unpack_from('<I',a,p+4*i)[0];e=a.find(b'\0',n,p+names)
        assert e>=n
        name=a[n:e].decode()
        if name.startswith(prefix):result[name[len(prefix):]]=(data_start+offsets[i],offsets[i+1]-offsets[i])
    return result

def literal(ops,b):
    if not b:return
    if ops and 'hex' in ops[-1]:ops[-1]['hex']+=b.hex()
    else:ops.append({'hex':b.hex()})
def copy(ops,offset,length):
    if not length:return
    if ops and 'offset' in ops[-1] and ops[-1]['offset']+ops[-1]['length']==offset:ops[-1]['length']+=length
    else:ops.append({'offset':offset,'length':length})

def delta(original,translated,base):
    # Index aligned blocks, then extend exact matches. Unmatched bytes are Hebrew
    # delta payload, never a complete base-game file. No fuzzy matching.
    block=16; index={}
    for p in range(0,len(original)-block+1,block):index.setdefault(original[p:p+block],p)
    ops=[];p=0;pending=bytearray()
    while p<len(translated):
        at=index.get(translated[p:p+block]) if p+block<=len(translated) else None
        if at is None:pending.append(translated[p]);p+=1;continue
        literal(ops,bytes(pending));pending.clear();n=block
        while at+n<len(original) and p+n<len(translated) and original[at+n]==translated[p+n]:n+=1
        copy(ops,base+at,n);p+=n
    literal(ops,bytes(pending));return ops

def main():
    ap=argparse.ArgumentParser();ap.add_argument('archive',type=Path);ap.add_argument('--platform',choices=['osx','windows'],default='osx');args=ap.parse_args()
    with args.archive.open('rb') as f,mmap.mmap(f.fileno(),0,access=mmap.ACCESS_READ) as a,tempfile.TemporaryDirectory() as td:
        table=members(a);archive_hash=digest(a);outputs={}
        def emit(name,ops):
            h=hashlib.sha256();length=0
            for op in ops:
                b=bytes.fromhex(op['hex']) if 'hex' in op else a[op['offset']:op['offset']+op['length']]
                length+=len(b);h.update(b)
            recipe={'schema_version':1,'input_sha256':archive_hash,'output_size':length,'operations':ops}
            native=ROOT.parents[1]/f'day-of-the-tentacle-remastered/he/patch.{args.platform}.candidate.json'
            if native.exists():recipe['alternate_input_sha256']=[json.loads(native.read_text())['platforms'][args.platform]['files'][0]['installed_sha256']]
            path=ROOT/'recipes'/f'{name if args.platform == 'osx' else 'windows-'+name}.json';save(path,recipe)
            outputs[name]={'recipe_sha256':digest(path.read_bytes()),'sha256':h.hexdigest(),'size':length}
        for ext in ['000','001']:
            off,size=table['classic/en/tentacle.'+ext];original=a[off:off+size]
            inp=Path(td)/'original';out=Path(td)/'translated';inp.write_bytes(original)
            subprocess.run(['xdelta3','-f','-d','-s',str(inp),str(ROOT/'source/extracted/PATCHES'/('patch.'+ext)),str(out)],check=True)
            emit('tentacle.'+ext,delta(original,out.read_bytes(),off))
            if args.platform=='osx':
                direct={'schema_version':1,'input_sha256':digest(original),'output_size':out.stat().st_size,'operations':delta(original,out.read_bytes(),0)}
                save(ROOT/'recipes'/f'classic.{ext}.json',direct)

        sound={}
        for name,prefix in [('audio/iMUSEClient_SFX.fsb',''),('audio/iMUSEClient_VO.fsb','EN_')]:sound.update(sounds(a,*table[name],prefix))
        lines=(ROOT/'source/monster.tbl').read_text().splitlines();tags=(ROOT/'source/tags.tbl').read_text().splitlines()
        assert len(lines)==len(tags)
        index=bytearray();ops=[];pos=0
        for line,tag in zip(lines,tags):
            off,length=sound[line[8:]];t=bytes.fromhex(tag)
            index+=bytes.fromhex(line[:8])+struct.pack('>3I',pos,len(t),length)
            literal(ops,t);copy(ops,off,length);pos+=len(t)+length
        emit('monster.so3',[{'hex':(struct.pack('>I',len(index))+index).hex()}]+ops)
        save(ROOT/('recipe-build.json' if args.platform=='osx' else 'recipe-build.windows.json'),{'input_sha256':archive_hash,'input_size':len(a),'outputs':outputs})
        files=[]
        for name,info in outputs.items():
            files.append({'path':'hebrew/'+name,'operation':'create','installed_sha256':info['sha256'],'source':{'kind':'reconstruct','path':'recipes/'+(name if args.platform=='osx' else 'windows-'+name)+'.json','sha256':info['recipe_sha256'],'input':'tenta.cle'}})
        if args.platform=='osx':
            for f in files:f['source']['input']='Dott.app/Contents/Resources/tenta.cle'
        manifest_path=ROOT/'patch.remastered-source.json'
        manifest=json.loads(manifest_path.read_text()) if manifest_path.exists() else {'schema_version':1,'scummvm_id':'tentacle','language':'he','version':'2.0-beta2-kitor1','filename':'DOTT-Classic-From-Remastered-Hebrew-2.0-beta2-kitor1.zip','platforms':{},'files':[]}
        manifest['platforms'][args.platform]={'files':files}
        save(manifest_path,manifest)
        if args.platform=='osx':
            direct_files=[]
            for ext in ['000','001']:
                path=ROOT/'recipes'/f'classic.{ext}.json';recipe=json.loads(path.read_text())
                direct_files.append({'path':'tentacle.'+ext,'operation':'modify','original_sha256':recipe['input_sha256'],'installed_sha256':outputs['tentacle.'+ext]['sha256'],'source':{'kind':'reconstruct','path':f'recipes/classic.{ext}.json','sha256':digest(path.read_bytes()),'input':'tentacle.'+ext}})
            save(ROOT/'patch.json',dict(manifest,filename='DOTT-Classic-Hebrew-2.0-beta2-kitor1.zip',platforms={'windows':{},'osx':{}},files=direct_files))
        print(json.dumps(outputs,indent=2))
if __name__=='__main__':main()
