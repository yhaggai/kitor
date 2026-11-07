"""Migrate legacy visual-order dialogue fields to logical Hebrew for native RTL.

Run after nikud/build_assets.py. UI/manuscript assets are intentionally untouched.
The generated C# table must be compiled into the matching HebrewShortcut.dll.
"""
import argparse,json,re,struct,sys
from pathlib import Path
import UnityPy
BASE=Path(__file__).resolve().parent
sys.path.insert(0,str(BASE/'nikud'))
from names import normalize_mapping
from layout import display, dialogue_display as logical
POINTED=re.compile(r'[א-ת][\u05b0-\u05bd\u05bf\u05c1\u05c2\u05c7]+')
def inputs():
    mapping=normalize_mapping(json.loads((BASE/'nikud/pointed.json').read_text()))
    manuscript=normalize_mapping(json.loads((BASE/'nikud/manuscript.json').read_text()))
    clusters=sorted({c for t in [*mapping.values(),*manuscript.values()] for c in POINTED.findall(t)})
    return mapping,clusters,{c:chr(0xe000+i) for i,c in enumerate(clusters)}
def field(t):
    b=t.encode();return struct.pack('<I',len(b))+b+bytes(-len(b)%4)
def migrate(source,output):
    mapping,clusters,glyphs=inputs();env=UnityPy.load(str(source));win=source.name.endswith('.assets');shift=90 if win else 0
    objs={o.path_id:o for o in env.objects if win or o.assets_file.name=='resources.assets'}
    groups={}
    for row in json.loads((BASE/'catalog.json').read_text()):
        oid=int(row['id'].split(':')[0][6:])-shift;text=row['he'];dialogue=row['field']=='Dialogue Text'
        for lang,value,gs in [('he',text,None),('he-x-nikud',mapping.get(text,text),glyphs if text in mapping else None)]:
            prefix=field(lang if dialogue else 'Menu Text '+lang)
            old=prefix+field(display(value,dialogue,gs));new=prefix+field(logical(value,gs))
            if old!=new:
                pairs=groups.setdefault(oid,{})
                if old in pairs:assert pairs[old]==new,'Ambiguous legacy text'
                pairs[old]=new
    count=0
    for oid,pairs in groups.items():
        raw=objs[oid].get_raw_data()
        missing=[k for k,v in pairs.items() if k not in raw and v not in raw]
        assert not missing,(oid,len(missing),repr(missing[0][:120]) if missing else '')
        pattern=re.compile(b'|'.join(map(re.escape,pairs)))
        raw,n=pattern.subn(lambda m:pairs[m[0]],raw);count+=n;objs[oid].set_raw_data(raw)
    output.write_bytes(env.file.save() if win else env.file.save(packer='lz4'))
    print('Migrated',count,'Hebrew dialogue/choice fields',flush=True)
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('source',type=Path);p.add_argument('output',type=Path);p.add_argument('--glyph-source',type=Path);a=p.parse_args()
    if a.glyph_source:
        _,clusters,_=inputs();a.glyph_source.write_text('static class NativeGlyphs { public static readonly string[] Clusters = new string[] {'+','.join(json.dumps(c,ensure_ascii=True) for c in clusters)+'}; }\n')
    migrate(a.source,a.output)
