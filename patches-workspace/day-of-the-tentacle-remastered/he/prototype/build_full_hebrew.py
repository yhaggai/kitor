"""Full mapped Hebrew dialogue candidate. Never patches Steam files.

Requires Pillow and local FriBidi. FTX layout reference: DoubleFine Explorer,
uDFExplorer_Base.pas. Font mapping/metrics layout inferred from owned assets.
"""
import argparse
import ctypes
import gzip
import json
from pathlib import Path
import plistlib
import struct
import subprocess

from PIL import Image, ImageDraw, ImageFont
from map_dialogue import extract_speech, rebuild, speech_rows, digest

from build_font_probe import patch_font, visual

BYTE_SLOTS = {chr(1488+i): 192+i for i in range(27)}


def prepare_text(text, widths, limit=620):
    # Wrap logical words first. Reorder each resulting line independently so
    # an LTR engine cannot move the end of a Hebrew sentence to its first row.
    text = text.replace('^', '...').replace('׳', "'")
    lines = []
    for paragraph in text.split('\n'):
        current = ''
        for word in paragraph.split():
            proposed = (current + ' ' + word).strip()
            if current and sum(widths.get(c, 24) for c in proposed) > limit:
                lines.append(current)
                current = word
            else:
                current = proposed
        lines.append(current)
    return '\n'.join(visual(line) if line else '' for line in lines)


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--source-app',required=True,type=Path)
    ap.add_argument('--output',required=True,type=Path)
    ap.add_argument('--font',type=Path,default=Path('/System/Library/Fonts/Supplemental/Arial Bold.ttf'))
    ap.add_argument('--item-labels',type=Path,help='Generated item-label UI resource')
    args=ap.parse_args()
    if args.output.exists():
        raise SystemExit('Output already exists; choose a fresh test directory')
    args.output.mkdir(parents=True)
    app=args.output/'DottHebrewFull.app'
    subprocess.run(['cp','-cR',str(args.source_app),str(app)],check=True)
    source=args.source_app/'Contents/Resources/tenta.cle'
    archive=app/'Contents/Resources/tenta.cle'
    with source.open('rb') as f:
        header=list(struct.unpack('<4sf8I',f.read(40)))
        f.seek(header[3]);entries=list(struct.iter_unpack('<5I',f.read(header[7])))
        f.seek(header[4]);names=[n.decode() for n in f.read(header[8]).split(b'\0')[:-1]]
    replacements={}
    families=['fonts/DotT_a-talk_']
    if args.item_labels:
        families += ['fonts/DotT_a-verb_', 'fonts/DotT_a-lilfnt_']
        replacements['localization/en.uitext.info']=args.item_labels.read_bytes()
    for name in names:
        if name.startswith(tuple(families)) and name.endswith('.font'):
            tex_name=name[:-5]+'.ftx'
            f,t,preview=patch_font(extract_speech(source,name),extract_speech(source,tex_name),args.font, extend=True)
            f=bytearray(f)
            for char, slot in BYTE_SLOTS.items():
                glyph=struct.unpack_from('<H', f, 26+ord(char)*2)[0]
                struct.pack_into('<H', f, 26+slot*2, glyph)
            f=bytes(f)
            replacements[name]=f;replacements[tex_name]=t
            if name=='fonts/DotT_a-talk_48.font':
                preview.convert('RGBA').save(args.output/'font-atlas-preview.png')
    data=extract_speech(source)
    rows=speech_rows(data)
    original_bytes=set(b for r in rows for b in r['text'].encode('utf-8', 'surrogateescape'))
    assert not (original_bytes & set(BYTE_SLOTS.values())), 'Slot conflicts with original dialogue'
    translations=speech_rows((Path(__file__).parent/'generated/en.speech.info.aligned-candidate').read_bytes())
    metric_font = replacements['fonts/DotT_a-talk_48.font']
    count, maximum = struct.unpack_from('<2I', metric_font, 4)
    start = len(metric_font) - count*16
    widths = {chr(c): struct.unpack_from('<H', metric_font,
              start + struct.unpack_from('<H', metric_font, 26+c*2)[0]*16 + 12)[0]
              for c in range(maximum+1)}
    probes=[]
    assert [r['id'] for r in rows] == [r['id'] for r in translations]
    for row, translated in zip(rows, translations):
        if row['text'] != translated['text']:
            text=translated['text']
            displayed=prepare_text(text, widths) if any('א'<=c<='ת' for c in text) else text
            probes.append({'id':row['id'],'hebrew':text,'visual':displayed})
            encoded=b''.join(bytes([BYTE_SLOTS[c]]) if c in BYTE_SLOTS else c.encode('utf-8', 'surrogateescape') for c in displayed)
            row['text']=encoded.decode('utf-8', 'surrogateescape')
    assert len(probes) == 5142
    replacements['localization/en.speech.info']=rebuild(data,rows)
    changes=[]
    with archive.open('r+b') as f:
        for name,data in replacements.items():
            index=names.index(name);old=entries[index]
            f.seek(0,2);offset=f.tell();f.write(data)
            new=(offset-header[5],old[1],len(data),len(data),0)
            f.seek(header[3]+index*20);f.write(struct.pack('<5I',*new))
            changes.append({'member':name,'original_entry':old,'test_entry':new,'sha256':digest(data)})
        f.seek(0,2);header[9]=f.tell()-header[5]
        f.seek(0);f.write(struct.pack('<4sf8I',*header))
    for name,data in replacements.items():
        assert extract_speech(archive,name)==data
    info=app/'Contents/Info.plist'
    with info.open('rb') as f:pl=plistlib.load(f)
    pl.update(CFBundleIdentifier='local.kitor.dott-hebrew-full',CFBundleName='DOTT Hebrew Full',CFBundleExecutable='DottHebrewFull')
    with info.open('wb') as f:plistlib.dump(pl,f)
    launcher=app/'Contents/MacOS/DottHebrewFull'
    launcher.write_text('#!/bin/sh\ncd "$(dirname "$0")/../../.." || exit 1\nexec ./DottHebrewFull.app/Contents/MacOS/Dott > hebrew-runtime.log 2>&1\n')
    launcher.chmod(0o755)
    (args.output/'steam_appid.txt').write_text('388210\n')
    save=args.output/'savedata';save.mkdir()
    (save/'settings.ini').write_text('[display]\nwindow_width=1280\nwindow_height=720\nmaximized=0\nfullscreen=0\n')
    report={'status':'full mapped dialogue runtime candidate, not release','source_mutations':[],
            'encoding':'single-byte C0-DA mapped to Hebrew; unused by original English dialogue','font_source':str(args.font),'font_pairs':sum(n.endswith('.font') for n in replacements),'item_labels':bool(args.item_labels),'probes':probes,'changes':changes,
            'limitations':['579 dialogue records remain English, including commentary.',
                          'Menus and dialogue choices remain untranslated; item labels included only with --item-labels.',
                          'Local system font; do not distribute.',
                          'Byte slots C0-DA replace unused accented mappings; other languages unsupported. Explicit wrapping needs scene review.']}
    (args.output/'full-manifest.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({'app':str(app.resolve()),'font_pairs':report['font_pairs'],'subtitle_probes':len(probes)}))


if __name__=='__main__':
    main()
