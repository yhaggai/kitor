"""Export GMS1 English map entries to an editable bilingual JSON catalog.
GMS1 quoted strings contain literal backslashes; do not apply Python/JSON escape decoding.
Existing catalog edits are preserved on re-export.
"""
from pathlib import Path
import json,re
root=Path(__file__).resolve().parent
source=(root/'decompiled/CodeEntries/gml_Script_textdata_en.gml').read_text()
seeds={}
output=root/'translation-work/catalog-before-full-translation.json'
output.parent.mkdir(exist_ok=True)
previous={}
if output.exists():
    previous={e['key']:e for e in json.loads(output.read_text())['entries']}
start='global.text_data_en = ds_map_create();'
assert source.startswith(start)
p=len(start)
def whitespace():
    global p
    while p<len(source) and source[p].isspace():p+=1
def expect(s):
    global p
    whitespace()
    assert source.startswith(s,p),(p,source[p:p+100],s)
    p+=len(s)
def literal():
    global p
    whitespace();q=source[p];assert q in ('"',"'")
    p+=1;end=source.index(q,p);v=source[p:end];p=end+1
    return v
entries=[];seen=set()
while True:
    whitespace()
    if p==len(source):break
    expect('ds_map_add(global.text_data_en,')
    key=literal();expect(',');english=literal();whitespace()
    while source[p]=='+':
        p+=1;english+=literal();whitespace()
    expect(');')
    assert key not in seen,key
    seen.add(key)
    old=previous.get(key,{})
    hebrew=old.get('hebrew',seeds.get(key,''))
    entry={'key':key,'english':english,'hebrew':hebrew,'status':old.get('status','draft' if hebrew else 'untranslated')}
    if old.get('translator_note'):entry['translator_note']=old['translator_note']
    entries.append(entry)
assert set(seeds)<=seen
catalog={'instructions':'Edit hebrew, status, and translator_note. Keep keys and English unchanged. Write Hebrew in logical reading order and preserve game control commands and placeholders. This generated baseline records original game strings for validation.','source':'textdata_en from the original game.ios','entry_count':len(entries),'entries':entries}
output.write_text(json.dumps(catalog,ensure_ascii=False,indent=2)+'\n')
# Verify serialization preserves every source string and every seeded translation.
roundtrip=json.loads(output.read_text())
assert roundtrip['entries']==entries
assert all(next(e for e in entries if e['key']==k)['hebrew']==v for k,v in seeds.items() if k not in previous)
print(f'Exported {output}: {len(entries)} entries, {sum(e["status"] == "draft" for e in entries)} Hebrew drafts; existing edits and translator notes preserved.')
print(json.dumps(next(e for e in entries if e['key']=='SCR_TEXT_292'),ensure_ascii=False,indent=2))
