"""Build local SoMI SE subtitle/font overrides without changing the source game.

Format reference: ShadowNate/MISETranslator. Arial output is local-use only.
"""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import struct
from bidi.algorithm import get_display
from PIL import Image, ImageDraw, ImageFont
from resources import Archive, STRIDE, replace_english, speech_rows

METRICS = 0x4260
LETTERS = ''.join(chr(c) for c in range(0x5d0,0x5eb))

def digest(data):
    return hashlib.sha256(data).hexdigest()


def font_candidate(data,png,ttf,slots,point_size):
    version,count,maximum = struct.unpack_from('<3I',data)
    if (version,count,maximum)!=(5,154,8482) or len(data)!=METRICS+count*16:
        raise ValueError('Unsupported font layout')
    original = Image.open(io.BytesIO(png)).convert('RGBA')
    records = list(struct.iter_unpack('<8h',data[METRICS:]))
    height = max(r[3]-r[1] for r in records)
    font = ImageFont.truetype(str(ttf),max(8,int(point_size*.85)))
    while font.getbbox(LETTERS)[3]-font.getbbox(LETTERS)[1] > height-4:
        if font.size<=1:
            raise ValueError('Font cannot fit glyph cell')
        font = ImageFont.truetype(str(ttf),font.size-1)
    top = font.getbbox(LETTERS)[1]
    x,y = 2,max(r[3] for r in records)+3
    additions = []
    for c in LETTERS:
        width = max(1,int(font.getlength(c)+1))
        if x+width+3 > original.width:
            x,y = 2,y+height+3
        additions.append((c,x,y,width))
        x += width+4
    new_height = original.height
    while y+height+3 > new_height:
        new_height *= 2
    atlas = Image.new('RGBA',(original.width,new_height))
    atlas.paste(original,(0,0))
    draw = ImageDraw.Draw(atlas)
    result = bytearray(data)
    struct.pack_into('<I',result,4,count+len(LETTERS))
    for i,(c,x,y,width) in enumerate(additions):
        draw.text((x+1,y+2-top),c,font=font,fill='white')
        struct.pack_into('<H',result,26+slots[c]*2,count+i)
        result.extend(struct.pack('<8h',x,y,x+width+2,y+height,0,width+2,width+2,0))
    bounds = (0,0,original.width,max(r[3] for r in records))
    if atlas.crop(bounds).tobytes()!=original.crop(bounds).tobytes() or result[METRICS:len(data)]!=data[METRICS:]:
        raise ValueError('Original glyphs or metrics changed')
    for c in range(maximum+1):
        if c not in slots.values() and result[26+c*2:28+c*2]!=data[26+c*2:28+c*2]:
            raise ValueError('Unrelated character mapping changed')
    output = io.BytesIO()
    atlas.save(output,format='PNG')
    return bytes(result),output.getvalue()


def prepare(text,widths,slots,limit=600):
    text = text.replace('^','...').replace('׳',"'")
    lines = []
    for paragraph in text.splitlines():
        line = ''
        for word in paragraph.split():
            proposal = (line+' '+word).strip()
            if line and sum(widths.get(c,24) for c in proposal)>limit:
                lines.append(line)
                line = word
            else:
                line = proposal
        lines.append(line)
    displayed = '\n'.join(get_display(line,base_dir='R') for line in lines)
    return b''.join(bytes([slots[c]]) if c in slots else c.encode('cp1252') for c in displayed)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    for name in ('game','mapping','output'):
        ap.add_argument('--'+name,type=Path,required=True)
    default_font = (Path(os.environ.get('WINDIR','C:/Windows'))/'Fonts/arialbd.ttf'
                    if os.name=='nt' else Path('/System/Library/Fonts/Supplemental/Arial Bold.ttf'))
    ap.add_argument('--font',type=Path,default=default_font)
    args = ap.parse_args()
    if args.output.exists() or args.output.resolve().is_relative_to(args.game.resolve()):
        raise SystemExit('Choose a fresh output directory outside the source game')
    archive = Archive(args.game/'Monkey1.pak')
    fonts = {name:archive.read(name) for name in archive.members if name.startswith('fonts/') and name.endswith('.font')}
    speech = (args.game/'audio/speech.info').read_bytes()
    original_rows = speech_rows(speech)
    mappings = json.loads(args.mapping.read_text(encoding='utf-8'))
    seen = set()
    for row in mappings:
        i = row['index']
        if i in seen or not 0<=i<len(original_rows) or any(row[k]!=original_rows[i][k] for k in ('id','english')):
            raise ValueError('Mapping does not match source game, or duplicates a record')
        seen.add(i)
    used = set(b for row in original_rows for b in row['english'].encode('cp1252'))
    ui = (args.game/'localization/uiText.info').read_bytes()
    if len(ui)%1536:
        raise ValueError('Unsupported UI layout')
    used.update(b for i in range(0,len(ui),1536) for b in ui[i+256:i+512].split(b'\0')[0])
    empty = [c for c in range(128,256) if c not in used and all(struct.unpack_from('<H',f,26+c*2)[0]==0 for f in fonts.values())]
    if len(empty)<len(LETTERS):
        raise ValueError('Insufficient unused glyph slots')
    slots = dict(zip(LETTERS,empty))
    args.output.mkdir(parents=True)
    changes = []
    metric_font = None
    for name,data in fonts.items():
        tex = name[:-5]+'.png'
        size = int(Path(name).stem.rsplit('_',1)[1])
        f,png = font_candidate(data,archive.read(tex),args.font,slots,size)
        if name=='fonts/MinisterT_bo_48.font':
            metric_font = f
        for member,output in [(name,f),(tex,png)]:
            path = args.output/member
            path.parent.mkdir(parents=True,exist_ok=True)
            path.write_bytes(output)
            changes.append({'path':member,'original_sha256':digest(archive.read(member)),'sha256':digest(output)})
    if metric_font is None:
        raise ValueError('Missing reference subtitle font')
    widths = {}
    for c in set(chr(i) for i in range(256))|set(LETTERS):
        glyph = struct.unpack_from('<H',metric_font,26+slots.get(c,ord(c))*2)[0]
        widths[c] = struct.unpack_from('<h',metric_font,METRICS+glyph*16+12)[0]
    replacements,rejected = {},[]
    for row in mappings:
        try:
            encoded = prepare(row['hebrew'],widths,slots)
            if len(encoded)>255:
                raise ValueError('255-byte field overflow')
            replacements[row['index']] = encoded
        except (ValueError,UnicodeEncodeError) as error:
            rejected.append({**row,'reason':str(error)})
    candidate = replace_english(speech,replacements)
    for i in range(len(original_rows)):
        start = i*STRIDE
        if candidate[start:start+16]!=speech[start:start+16] or candidate[start+272:start+STRIDE]!=speech[start+272:start+STRIDE]:
            raise ValueError('Non-English field changed')
        if i not in replacements and candidate[start:start+STRIDE]!=speech[start:start+STRIDE]:
            raise ValueError('Unmapped record changed')
    (args.output/'audio').mkdir()
    (args.output/'audio/speech.info').write_bytes(candidate)
    changes.append({'path':'audio/speech.info','original_sha256':digest(speech),'sha256':digest(candidate)})
    report = {'translated':len(replacements),'unchanged':len(original_rows)-len(replacements),'font_pairs':len(fonts),
              'byte_slots':slots,'changes':changes,'runtime_verified':False,'release_ready':False,
              'bidi':'python-bidi 0.6.7, base_dir=R',
              'limitations':['Local Arial-derived output: do not redistribute','Loose font overrides unverified',
                             'English text language only','Menus and item labels unchanged','Wrapping and timing unverified']}
    for name,value in [('rejected.json',rejected),('manifest.json',report)]:
        (args.output/name).write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:report[k] for k in ('translated','unchanged','font_pairs','runtime_verified')},indent=2))

if __name__=='__main__':
    main()
