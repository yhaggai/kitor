"""Extend exported original fonts; never writes the original game archive.
Run export_all_fonts.csx once, then this script after editing the catalog.
"""
from pathlib import Path
import csv, hashlib, json, math, statistics, unicodedata
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent
SOURCE = ROOT / 'fonts'
OUT = ROOT / 'generated-fonts'
CATALOG = ROOT.parent / 'translation-catalog.json'
OUT.mkdir(exist_ok=True)
REGULAR = '/System/Library/Fonts/Supplemental/Arial.ttf'
BOLD = '/System/Library/Fonts/Supplemental/Arial Bold.ttf'

def median(values):
    return int(round(statistics.median(values)))

def alpha_bounds(image, row):
    _, x, y, w, h, _, _ = row
    return image.crop((x, y, x+w, y+h)).getchannel('A').getbbox()

def read_glyphs(path):
    rows = [list(map(int, r)) for r in csv.reader(path.open(), delimiter=';')]
    if len({r[0] for r in rows}) != len(rows):
        raise ValueError(f'Duplicate source glyphs: {path}')
    return {r[0]: r for r in rows}

def metrics(name, image, glyphs):
    # Sans is lowercase in the English dialogue. Papyrus is uppercase.
    reference = 'acehmnorsux' if 'comicsans' in name else 'ABDEHKNOPRSUX'
    rows = [glyphs[ord(c)] for c in reference if ord(c) in glyphs]
    rows = [r for r in rows if alpha_bounds(image, r)]
    basis = 'ASCII lowercase' if 'comicsans' in name else 'ASCII uppercase'
    if not rows:
        rows = [r for r in glyphs.values() if r[0] > 127 and alpha_bounds(image, r)]
        basis = 'existing Japanese glyphs (font contains no ASCII letters)'
    if not rows:
        raise ValueError(f'Cannot derive font metrics: {name}')
    bounds = [alpha_bounds(image, r) for r in rows]
    top = median([b[1] for b in bounds])
    bottom = median([b[3] for b in bounds])
    height = max(3, bottom-top)
    shift = max(3, median([r[5] for r in rows]))
    width = max(2, median([b[2]-b[0] for b in bounds]))
    if basis.startswith('existing Japanese'):
        # Hebrew has the width of a Latin letter, not a square CJK character.
        shift = max(3, round(shift*0.62))
        width = max(2, shift-max(1, round(shift*0.1)))
    gap = max(1, shift-width)
    return dict(basis=basis, body_top=top, body_bottom=bottom,
                body_height=height, reference_ink_width=width,
                reference_advance=shift, gap=gap, cell_height=max(r[4] for r in glyphs.values()))

def render(ch, face, metric, antialias):
    # Baseline anchor preserves relative ascenders/descenders: lamed above the
    # Hebrew body, final kaf/nun/pe/tsadi below it. Fit to original line space.
    hebrew = '\u0590' <= ch <= '\u05ff'
    base = face.getbbox('ה' if hebrew else 'H', anchor='ls')
    box = face.getbbox(ch, anchor='ls')
    if not ch.isspace() and bytes(face.getmask(ch)) == bytes(face.getmask(chr(0x10ffff))):
        raise ValueError(f'Raster font is missing character: {ch!r} U+{ord(ch):04X}')
    raw = Image.new('L', (max(1,box[2]-box[0]), max(1,box[3]-box[1])))
    ImageDraw.Draw(raw).text((-box[0],-box[1]),ch,font=face,fill=255,anchor='ls')
    refchars = 'אבדהחטמנספשת' if hebrew else 'ABDEHKNOPRSUX'
    refwidth = statistics.median(face.getbbox(c,anchor='ls')[2]-face.getbbox(c,anchor='ls')[0] for c in refchars)
    sx = metric['reference_ink_width']/refwidth
    sy = metric['body_height']/max(1,base[3]-base[1])
    w = max(1,round(raw.width*sx))
    h = max(1,round(raw.height*sy))
    # Original fonts have no vertical glyph offset: transparent leading rows
    # encode the baseline. Keep lamed entirely inside the texture rectangle.
    top = max(0, round(metric['body_top']+(box[1]-base[1])*sy))
    h = min(h, max(1,metric['cell_height']-top))
    scaled = raw.resize((w,h),Image.Resampling.LANCZOS)
    if not antialias:
        scaled = scaled.point(lambda value:255 if value>=100 else 0)
    if raw.getbbox() and not scaled.getbbox():
        raise ValueError(f'Glyph vanished during rasterization: {ch!r}')
    cell = Image.new('RGBA',(w,max(1,top+h)))
    cell.paste(Image.new('RGBA',(w,h),'white'),(0,top),scaled)
    advance = max(1,w+metric['gap'])
    if ch.isspace():
        advance=metric['reference_advance'];cell=Image.new('RGBA',(1,1))
    return cell,advance

catalog=json.loads(CATALOG.read_text())
visible={c for e in catalog['entries'] for c in e.get('hebrew','')
         if c.isprintable() and unicodedata.category(c)!='Cf'}
required=visible|{chr(c) for c in range(0x5d0,0x5eb)}
if any(ord(c)>65535 for c in required):
    raise ValueError('GameMaker glyphs use 16-bit character IDs; catalog includes unsupported supplementary characters')
reports=[]; previews=[]
for info in json.loads((SOURCE/'metadata.json').read_text()):
    name=info['name']; image=Image.open(SOURCE/f'{name}.png').convert('RGBA')
    glyphs=read_glyphs(SOURCE/f'{name}.csv'); metric=metrics(name,image,glyphs)
    facepath=BOLD if info['properties'].get('Bold')=='True' else REGULAR
    face=ImageFont.truetype(facepath,80)
    aa=info['properties'].get('AntiAliasing')!='0'
    missing=sorted(ord(c) for c in required if ord(c) not in glyphs)
    rendered=[];x=2;y=image.height+2;rowheight=0; additions=[]
    for code in missing:
        cell,shift=render(chr(code),face,metric,aa)
        if cell.width+4>image.width:raise ValueError(f'Glyph too wide: {name} {code}')
        if x+cell.width+2>image.width:
            x=2;y+=rowheight+2;rowheight=0
        additions.append([code,x,y,cell.width,cell.height,shift,0])
        rendered.append((cell,(x,y)));x+=cell.width+2;rowheight=max(rowheight,cell.height)
    newheight=2**math.ceil(math.log2(max(image.height,y+rowheight+2)))
    atlas=Image.new('RGBA',(image.width,newheight));atlas.paste(image,(0,0))
    for cell,position in rendered:atlas.paste(cell,position)
    assert atlas.crop((0,0,*image.size)).tobytes()==image.tobytes()
    assert len({r[0] for r in additions})==len(additions)
    assert not ({r[0] for r in additions}&glyphs.keys())
    complete=sorted([*glyphs.values(),*additions],key=lambda r:r[0])
    assert {ord(c) for c in required} <= {r[0] for r in complete}
    for code,gx,gy,gw,gh,shift,offset in complete:
        assert 0<=gx and 0<=gy and gx+gw<=atlas.width and gy+gh<=atlas.height
    atlas.save(OUT/f'{name}.png')
    for suffix,rows in [('',additions),('.all',complete)]:
        with (OUT/f'{name}{suffix}.csv').open('w') as f:csv.writer(f,delimiter=';').writerows(rows)
    reports.append(dict(name=name,original_size=list(image.size),generated_size=list(atlas.size),
        original_glyphs=len(glyphs),added_glyphs=len(additions),total_glyphs=len(complete),
        added_codepoints=missing,hebrew_letter_coverage=27,missing_required_codepoints=[],
        original_pixels_preserved=True,original_glyph_metrics_preserved=True,
        original_pixel_sha256=hashlib.sha256(image.tobytes()).hexdigest(),
        metrics=metric,hebrew_face=facepath,anti_alias=aa))
    # Preview uses imported glyph cells, not a separate font rendering path.
    display='שלום עולם אבגדהוזחטיךכלםמןנסעףפץצקרשת ״׳?!'
    added={r[0]:r for r in additions};allglyphs={**glyphs,**added};xx=0
    strip=Image.new('RGBA',(1000,max(r[4] for r in complete)+8),'black')
    for ch in display:
        row=allglyphs.get(ord(ch))
        if row is None:continue
        code,gx,gy,gw,gh,shift,offset=row
        glyph=atlas.crop((gx,gy,gx+gw,gy+gh));strip.alpha_composite(glyph,(xx+offset,2));xx+=shift
    previews.append((name,strip.crop((0,0,min(1000,max(1,xx)),strip.height))))
report={'font_count':len(reports),'catalog':str(CATALOG),'catalog_sha256':hashlib.sha256(CATALOG.read_bytes()).hexdigest(),
        'required_visible_codepoints':sorted(map(ord,required)),
        'fonts':reports,'limitations':['Hebrew uses Arial/Arial Bold adapted to existing pixel metrics, not bespoke Hebrew equivalents of Comic Sans/Papyrus.',
        'Tiny original 4x5 fonts have inherently limited Hebrew legibility; inspect in-game.',
        'Preview glyph order is logical for coverage checking; runtime bidi is handled by the renderer.']}
(OUT/'font-coverage.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
height=sum(max(28,im.height*2)+22 for _,im in previews)
sheet=Image.new('RGB',(1800,height),(24,24,24));draw=ImageDraw.Draw(sheet);y=0
for name,im in previews:
    draw.text((8,y),name,fill='white');y+=20
    im=im.resize((im.width*2,im.height*2),Image.Resampling.NEAREST)
    sheet.paste(im,(8,y));y+=max(28,im.height)+2
sheet.save(OUT/'font-preview.png')
print(json.dumps({'fonts':len(reports),'added_glyphs':sum(x['added_glyphs'] for x in reports),
                  'required_chars':len(required),'output':str(OUT),'all_original_pixels_preserved':True}))
