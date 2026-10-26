"""Local-only font probe. Creates a fresh test app; never patches Steam files.

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

SLOTS = {chr(1488 + i): chr(65 + i) for i in range(27)}


def visual(text):
    lib = ctypes.CDLL('/opt/homebrew/lib/libfribidi.dylib')
    u32 = ctypes.c_uint32
    lib.fribidi_log2vis.argtypes = [ctypes.POINTER(u32), ctypes.c_int,
        ctypes.POINTER(u32), ctypes.POINTER(u32), ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p]
    lib.fribidi_log2vis.restype = ctypes.c_byte
    lines = []
    for line in text.split('\n'):
        source = (u32 * len(line))(*map(ord, line))
        output = (u32 * len(line))()
        direction = u32(0x111)
        assert lib.fribidi_log2vis(source, len(line), ctypes.byref(direction), output, None, None, None)
        lines.append(''.join(chr(c) for c in output))
    return '\n'.join(lines)


def patch_font(font_data, texture_data, ttf, *, extend=False):
    font = bytearray(font_data)
    version, count, maximum = struct.unpack_from('<3I', font)
    assert version == 5 and count == 160 and maximum == 8482
    metrics_start = len(font) - count * 16
    assert metrics_start == 26 + (maximum + 1) * 2
    magic, width, height, raw_size, packed_size = struct.unpack_from('<4sHHII', texture_data)
    assert magic in (b'FTX1', b'FTX2')
    raw = gzip.decompress(texture_data[16:])
    channels=2 if magic==b'FTX2' else 1
    assert len(raw) == raw_size == width * height * channels
    atlas = Image.frombytes('LA', (width, height), raw) if channels==2 else Image.merge('LA',(Image.new('L',(width,height),255), Image.frombytes('L',(width,height),raw)))
    records = [list(struct.unpack_from('<4H2h2H', font, metrics_start + i*16)) for i in range(count)]
    replacement = {struct.unpack_from('<H', font, 26 + ord(slot)*2)[0]: char for char, slot in SLOTS.items()}
    if extend:
        # Add independent glyph records; retain every existing character mapping.
        replacement = {}
        for char, template in SLOTS.items():
            index = struct.unpack_from('<H', font, 26 + ord(template)*2)[0]
            replacement[len(records)] = char
            struct.pack_into('<H', font, 26 + ord(char)*2, len(records))
            records.append(records[index].copy())
        count = len(records)
        struct.pack_into('<I', font, 4, count)
        font.extend(b'\0' * (27 * 16))
    cells = []
    for i, record in enumerate(records):
        x, y, right, bottom, bearing, ink_width, advance, flags = record
        cell = atlas.crop((x, y, right, bottom))
        if i in replacement:
            h = bottom-y
            size = max(3, int(h * .92))
            face = ImageFont.truetype(str(ttf), size)
            char = replacement[i]
            box = face.getbbox(char)
            pad = max(1, h//24)
            w = max(2, box[2]-box[0]+2*pad)
            mask = Image.new('L', (w, h))
            draw = ImageDraw.Draw(mask)
            draw.text((pad-box[0], max(pad, (h-(box[3]-box[1]))//2)-box[1]), char,
                      font=face, fill=255, stroke_width=pad, stroke_fill=255)
            luminance = Image.new('L', (w,h))
            ImageDraw.Draw(luminance).text((pad-box[0], max(pad, (h-(box[3]-box[1]))//2)-box[1]),
                                         char, font=face, fill=255)
            cell = Image.merge('LA', (luminance, mask))
            record[4:7] = [0, w, max(1,w-1)]
        cells.append(cell)
    # Repack all glyphs so narrow Latin slots do not squeeze Hebrew characters.
    positions = []
    x = y = row_h = 1
    for cell in cells:
        if x+cell.width+1 > width:
            x=1; y+=row_h+1; row_h=0
        positions.append((x,y)); x+=cell.width+1; row_h=max(row_h,cell.height)
    new_height = 1
    while new_height < y+row_h+1:
        new_height *= 2
    result = Image.new('LA',(width,new_height),(255,0))
    for i,(cell,(x,y)) in enumerate(zip(cells,positions)):
        result.paste(cell,(x,y))
        records[i][:4]=[x,y,x+cell.width,y+cell.height]
        struct.pack_into('<4H2h2H',font,metrics_start+i*16,*records[i])
    raw_result=result.tobytes() if channels==2 else result.getchannel('A').tobytes()
    compressed = gzip.compress(raw_result,mtime=0)
    texture = struct.pack('<4sHHII',magic,width,new_height,len(raw_result),len(compressed))+compressed
    return bytes(font),texture,result


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--source-app',required=True,type=Path)
    ap.add_argument('--output',required=True,type=Path)
    ap.add_argument('--font',type=Path,default=Path('/System/Library/Fonts/Supplemental/Arial Bold.ttf'))
    args=ap.parse_args()
    if args.output.exists():
        raise SystemExit('Output already exists; choose a fresh test directory')
    args.output.mkdir(parents=True)
    app=args.output/'DottFontProbe.app'
    subprocess.run(['cp','-cR',str(args.source_app),str(app)],check=True)
    source=args.source_app/'Contents/Resources/tenta.cle'
    archive=app/'Contents/Resources/tenta.cle'
    with source.open('rb') as f:
        header=list(struct.unpack('<4sf8I',f.read(40)))
        f.seek(header[3]);entries=list(struct.iter_unpack('<5I',f.read(header[7])))
        f.seek(header[4]);names=[n.decode() for n in f.read(header[8]).split(b'\0')[:-1]]
    replacements={}
    for name in names:
        if name.startswith('fonts/DotT_a-talk_') and name.endswith('.font'):
            tex_name=name[:-5]+'.ftx'
            f,t,preview=patch_font(extract_speech(source,name),extract_speech(source,tex_name),args.font)
            replacements[name]=f;replacements[tex_name]=t
            if name.endswith('_48.font'):
                preview.convert('RGBA').save(args.output/'font-atlas-preview.png')
    data=extract_speech(source)
    rows=speech_rows(data)
    translations=speech_rows((Path(__file__).parent/'generated/en.speech.info.aligned-candidate').read_bytes())
    probes=[]
    for i,row in enumerate(rows):
        if row['id'].startswith(('PUT_2_toxic-pipes_', 'GRT_2_toxic-pipes_', 'PUT_3_cu-purple_')):
            text=translations[i]['text'].replace('^','...')
            displayed=visual(text)
            encoded=''.join(SLOTS.get(c,c) for c in displayed)
            if row['id']=='PUT_2_toxic-pipes_1_1':
                encoded='test 123 | '+encoded
            assert encoded.isascii()
            probes.append({'id':row['id'],'hebrew':text,'visual':displayed,'encoded':encoded})
            row['text']=encoded
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
    pl.update(CFBundleIdentifier='local.kitor.dott-font-probe',CFBundleName='DOTT Font Probe',CFBundleExecutable='DottProbe')
    with info.open('wb') as f:plistlib.dump(pl,f)
    launcher=app/'Contents/MacOS/DottProbe'
    launcher.write_text('#!/bin/sh\ncd "$(dirname "$0")/../../.." || exit 1\nexec ./DottFontProbe.app/Contents/MacOS/Dott > probe-runtime.log 2>&1\n')
    launcher.chmod(0o755)
    (args.output/'steam_appid.txt').write_text('388210\n')
    save=args.output/'savedata';save.mkdir()
    (save/'settings.ini').write_text('[display]\nwindow_width=1280\nwindow_height=720\nmaximized=0\nfullscreen=0\n')
    report={'status':'diagnostic font/encoding probe, not gameplay release','source_mutations':[],
            'font_source':str(args.font),'font_pairs':(len(replacements)-1)//2,'probes':probes,'changes':changes,
            'limitations':['Uppercase Latin A-Z and [ are repurposed in subtitle fonts.',
                          'Only opening scene translated; later English capitals display Hebrew.',
                          'Local system font; do not distribute.','RTL preordered per explicit line; automatic wrapping unverified.']}
    (args.output/'probe-manifest.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({'app':str(app.resolve()),'font_pairs':report['font_pairs'],'subtitle_probes':len(probes)}))


if __name__=='__main__':
    main()
