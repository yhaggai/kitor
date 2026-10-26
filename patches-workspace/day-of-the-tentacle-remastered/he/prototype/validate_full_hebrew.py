"""Verify generated candidate resources against the owned original archive."""
import argparse
import gzip
import json
import struct
from pathlib import Path
from PIL import Image
from map_dialogue import extract_speech, speech_rows, digest
from build_full_hebrew import BYTE_SLOTS


def validate(source, output):
    target=output/'DottHebrewFull.app/Contents/Resources/tenta.cle'
    manifest=json.loads((output/'full-manifest.json').read_text())
    checked=0
    for item in manifest['changes']:
        name=item['member']
        candidate=extract_speech(target,name)
        assert digest(candidate)==item['sha256']
        if not name.endswith('.font'):
            continue
        original=extract_speech(source,name)
        assert struct.unpack_from('<I',candidate,4)[0]==187
        for c in range(8483):
            if c in BYTE_SLOTS.values() or 1488<=c<=1514:
                continue
            assert original[26+c*2:28+c*2]==candidate[26+c*2:28+c*2]
        images=[]
        for archive in (source,target):
            texture=extract_speech(archive,name[:-5]+'.ftx')
            w,h=struct.unpack_from('<HH',texture,4)
            images.append(Image.frombytes('LA' if texture[:4]==b'FTX2' else 'L',(w,h),gzip.decompress(texture[16:])))
        for i in range(160):
            a=struct.unpack_from('<4H2h2H',original,26+8483*2+i*16)
            b=struct.unpack_from('<4H2h2H',candidate,26+8483*2+i*16)
            assert a[4:]==b[4:]
            assert images[0].crop(a[:4]).tobytes()==images[1].crop(b[:4]).tobytes()
            checked+=1
    original_data=extract_speech(source)
    target_data=extract_speech(target)
    original_rows=speech_rows(original_data)
    target_rows=speech_rows(target_data)
    assert [r['id'] for r in original_rows]==[r['id'] for r in target_rows]
    for i in range(len(original_rows)):
        assert original_data[8+i*40:32+i*40]==target_data[8+i*40:32+i*40]
    unchanged=sum(a==b for a,b in zip(original_rows,target_rows))
    assert unchanged==579
    inverse={v:k for k,v in BYTE_SLOTS.items()}
    by_id={r['id']:r for r in target_rows}
    for probe in manifest['probes']:
        raw=by_id[probe['id']]['text'].encode('utf-8','surrogateescape')
        decoded=''.join(inverse.get(b,chr(b)) for b in raw)
        assert decoded==probe['visual']
    if manifest.get('item_labels'):
        from map_item_labels import ui_rows
        before=ui_rows(extract_speech(source,'localization/en.uitext.info'))
        after=ui_rows(extract_speech(target,'localization/en.uitext.info'))
        selected=set(range(323)) | set(range(709,734))
        assert len(before)==len(after)==807
        for i,(a,b) in enumerate(zip(before,after)):
            assert a[0]==b[0]
            if i not in selected:
                assert a==b
            else:
                assert a[1]!=b[1]
        assert not (set(b for i,r in enumerate(after) if i not in selected for b in r[1]) & set(BYTE_SLOTS.values())), 'Byte mapping collides with untouched UI'
    long=[r for r in manifest['probes'] if len(r['visual'].splitlines())>3]
    (output/'long-subtitle-review.json').write_text(json.dumps(long,ensure_ascii=False,indent=2)+'\n')
    report={'preserved_original_glyphs':checked,'verified_members':len(manifest['changes']),
            'unchanged_records':unchanged,'translated_records':len(manifest['probes']),
            'over_three_lines':len(long),'runtime_verification':'separate manual/UI check'}
    (output/'validation.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report))


if __name__=='__main__':
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--archive',type=Path,required=True)
    ap.add_argument('--output',type=Path,required=True)
    args=ap.parse_args()
    validate(args.archive,args.output)
