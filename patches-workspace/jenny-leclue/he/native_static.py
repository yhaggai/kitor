"""Migrate menu/typewriter records only; restore obsolete Mac intro scene edits."""
import argparse,json,re,sys
from pathlib import Path
import UnityPy
BASE=Path(__file__).resolve().parent
sys.path.insert(0,str(BASE))
from native_dialogue import inputs,field
from layout import dialogue_display
from ui_assets import read_language,U32

def keys():
    return set(json.loads((BASE/'menu.json').read_text())) | {r['key'] for r in json.loads((BASE/'ui-catalog.json').read_text()) if r['key'].startswith('TMP_Typewriter')}

def build(source,output,stock=None):
    mapping,clusters,glyphs=inputs();rows={(r['object_id'],r['key']):r for r in json.loads((BASE/'ui-catalog.json').read_text())};selected=keys()
    env=UnityPy.load(str(source));win=source.name.endswith('.assets');shift=90 if win else 0
    objs={o.path_id:o for o in env.objects if win or o.assets_file.name=='resources.assets'}
    count=0
    for oid in range(14089,14124):
        obj=objs[oid-shift];raw=obj.get_raw_data()
        for lang in ['he','he-x-nikud']:
            start,end,records=read_language(raw,lang);entries=[]
            for key,old,alt in records:
                if key in selected:
                    row=rows[oid,key];texts=[]
                    for previous,column in [(old,'he'),(alt,'alternate_he')]:
                        source_text=row[column].replace('\\n','\n')
                        if lang=='he-x-nikud':
                            if '<pos=' in previous:
                                # Existing positioned glyphs already follow reading order.
                                text=re.sub(r'</?align[^>]*>|<pos=[^>]*>','',previous)
                            else:
                                text=dialogue_display(mapping.get(row[column],row[column]).replace('\\n','\n'),glyphs)
                        else:text=dialogue_display(source_text)
                        assert '<pos=' not in text
                        texts.append(text)
                    old,alt=texts;count+=1
                entries.append(field(key)+field(old)+field(alt))
            raw=raw[:start]+field(lang)+U32.pack(len(records))+b''.join(entries)+raw[end:]
        obj.set_raw_data(raw)
    if not win:
        assert stock,'Pristine Mac bundle required to restore original typewriter scene'
        pristine=UnityPy.load(str(stock));original={o.path_id:o for o in pristine.objects if o.assets_file.name=='level8'}
        restore={8274,8281,8284,8289,8290,8291,8295,8296,8303,8195}
        for o in env.objects:
            if o.assets_file.name=='level8' and o.path_id in restore:o.set_raw_data(original[o.path_id].get_raw_data())
    output.write_bytes(env.file.save() if win else env.file.save(packer='lz4'))
    print('Migrated',count,'menu/typewriter language records',flush=True)

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('source',type=Path);p.add_argument('output',type=Path);p.add_argument('--stock',type=Path);p.add_argument('--keys-source',type=Path);a=p.parse_args()
    if a.keys_source:a.keys_source.write_text('using System.Collections.Generic; static class NativeStaticKeys { public static readonly HashSet<string> Keys = new HashSet<string>{'+','.join(json.dumps(k) for k in sorted(keys()))+'}; }\n')
    build(a.source,a.output,a.stock)
