"""Reuse Classic Hebrew item names in the Remastered UI table."""
import argparse
import collections
import json
import struct
from pathlib import Path
from map_dialogue import digest
from build_font_probe import visual
from build_full_hebrew import BYTE_SLOTS

# Explicit choices among translations already present in the Classic source.
CHOICES = {
 'door':'דלת', 'toilet':'אסלה', 'path':'שביל', 'cabinet':'ארון',
 'trapped teeth':'שיניים לכודות', 'flag gun':'אקדח דגל', 'monitor':'מסך',
 'purple tentacle':'משוש סגול', 'green tentacle':'משוש ירוק',
 'camera':'מצלמה', 'play':'נגן', 'filthy old car':'מכונית מלוכלכת וישנה',
 'world domination plans':'תוכניות להשתלטות על העולם', 'extension cord':'כבל מאריך',
 'frozen hamster':'אוגר קפוא', 'smiling people':'אנשים מחייכים',
 'bernard':'ברנרד', 'laverne':'לאוורן', 'chattering teeth':'שיניים נוקשות',
 'wet soggy noodles':'אטריות ספוגות מים',
}


def ui_rows(data):
    wide = struct.unpack_from('<I',data,4)[0] == 0
    size = 8 if wide else 4
    pointer = '<Q' if wide else '<I'
    count,=struct.unpack_from(pointer,data)
    rows=[]
    for i in range(count):
        vals=[]
        for j in range(2):
            ptr=size+i*size*2+j*size
            start=ptr+struct.unpack_from(pointer,data,ptr)[0]
            assert size+count*size*2<=start<len(data)
            vals.append(data[start:data.index(0,start)])
        rows.append(vals)
    return rows


def rebuild_ui(rows, original=None):
    size = 4 if original is not None and struct.unpack_from('<I',original,4)[0] else 8
    pointer = '<I' if size == 4 else '<Q'
    result=bytearray(size+len(rows)*size*2)
    struct.pack_into(pointer,result,0,len(rows))
    original_end = len(result)
    for i,row in enumerate(rows):
        for j,value in enumerate(row):
            ptr=size+i*size*2+j*size
            if original is not None:
                start=ptr+struct.unpack_from(pointer,original,ptr)[0]
                original_end=max(original_end,original.index(0,start)+1)
            struct.pack_into(pointer,result,ptr,len(result)-ptr)
            result.extend(value+b'\0')
    if original is not None:
        result.extend(original[original_end:])
    return bytes(result)


def build(data, english, hebrew):
    eng=english.splitlines();heb=hebrew.splitlines()
    assert len(eng)==len(heb)==6645
    marker=b'\\xFF\\x0A'
    checked=0
    for e,h in zip(eng,heb):
        if e.startswith(marker):
            assert e[:64]==h[:64] or h==b'#REF!', 'Classic source order mismatch'
            checked+=1
    assert checked==5269
    mapping=collections.defaultdict(list)
    for line,(e,h) in enumerate(zip(eng,heb),1):
        e=e.replace(b'\\x0F', b'\x0f')
        if b'\\x' in e or b'\\x' in h or h==b'#REF!':continue
        text=h.decode('cp1255')
        if any('א'<=c<='ת' for c in text):
            mapping[e.decode('cp1252').casefold()].append({'line':line,'hebrew':text})
    rows=ui_rows(data)
    assert rebuild_ui(rows,data)==data
    changes=[]
    # Original table's item/person names, plus dynamic inventory-name variants.
    for i in [*range(323),*range(709,734)]:
        key,value=rows[i]
        assert key==value or key.replace(b'\x0f',b'\x99')==value
        name=key.decode('cp1252')
        sources=mapping[name.casefold()]
        options={s['hebrew'] for s in sources}
        chosen=CHOICES.get(name.casefold())
        if chosen is None and len(options)==1:chosen=next(iter(options))
        assert chosen in options, (i,name,options)
        displayed=visual(chosen)
        encoded=b''.join(bytes([BYTE_SLOTS[c]]) if c in BYTE_SLOTS else c.encode('ascii') for c in displayed)
        rows[i][1]=encoded
        changes.append({'index':i,'key':name,'hebrew':chosen,'visual':displayed,'classic_sources':sources})
    result=rebuild_ui(rows,data)
    assert ui_rows(result)==rows
    return result,{'items':len(changes),'unique_labels':len({c['key'] for c in changes}),
                   'classic_english_sha256':digest(english),'classic_hebrew_sha256':digest(hebrew),
                   'validated_voice_anchors':checked,'changes':changes}


if __name__=='__main__':
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--ui',type=Path,required=True)
    ap.add_argument('--english',type=Path,required=True)
    ap.add_argument('--hebrew',type=Path,required=True)
    ap.add_argument('--output',type=Path,required=True)
    args=ap.parse_args()
    data,report=build(args.ui.read_bytes(),args.english.read_bytes(),args.hebrew.read_bytes())
    args.output.mkdir(parents=True,exist_ok=True)
    (args.output/'en.uitext.info.items').write_bytes(data)
    (args.output/'item-label-map.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k!='changes'}))
