"""Extend the original UI font atlases, preserving every original glyph."""
import struct
from pathlib import Path
import numpy as np
from PIL import Image, ImageDraw, ImageFilter, ImageFont

U32 = struct.Struct('<I')


def table(raw, stride):
    for start in range(60, min(1500, len(raw)-4), 4):
        count = U32.unpack_from(raw, start)[0]
        chars = start + 4 + count * stride
        if (20 < count < 2000 and chars + 4 + count*16 < len(raw)
                and U32.unpack_from(raw, chars)[0] == count
                and U32.unpack_from(raw, chars+4)[0] == 1):
            return start, count, chars, chars+4+count*16
    raise ValueError('Unknown original UI font layout')


def extend(obj, texture, glyphs, windows, font_path):
    raw = obj.get_raw_data()
    stride = 52 if windows else 48
    start, count, chars, end = table(raw, stride)
    records = {U32.unpack_from(raw, start+4+i*stride)[0]:
               raw[start+4+i*stride:start+4+(i+1)*stride] for i in range(count)}
    characters = {U32.unpack_from(raw, chars+4+i*16+4)[0]:
                  U32.unpack_from(raw, chars+4+i*16+8)[0] for i in range(count)}
    _, hw, hh, hx, hy, ha = struct.unpack_from('<I5f', records[characters[ord('H')]])
    font = ImageFont.truetype(str(font_path), 150)
    stroke = 2
    l, t, r, b = font.getbbox('א', anchor='ls', stroke_width=stroke)
    scale = hh / (b-t)
    tex = texture.read()
    original = tex.image.convert('RGBA')
    width = height = 4096
    canvas = Image.new('RGBA', (width, height), (255,255,255,0))
    canvas.paste(original, (0,height-original.height))
    additions, char_additions = [], []
    x = y = row_height = 0
    symbols = [(chr(code), chr(code)) for code in range(0x5d0,0x5eb)] + list(glyphs.items())
    for index, (cluster, symbol) in enumerate(symbols):
        l,t,r,b = font.getbbox(cluster, anchor='ls', stroke_width=stroke)
        mask = Image.new('L', (r-l,b-t))
        draw = ImageDraw.Draw(mask)
        draw.text((-l,-t),cluster,font=font,fill=255,anchor='ls')
        # Strengthen the letter without merging adjacent nikud dots.
        draw.text((-l,-t),cluster[0],font=font,fill=255,anchor='ls',
                  stroke_width=stroke,stroke_fill=255)
        w,h = max(1,round(mask.width*scale)),max(1,round(mask.height*scale))
        tile = Image.new('L',(w+20,h+20))
        tile.paste(mask.resize((w,h),Image.Resampling.LANCZOS),(10,10))
        if x+tile.width > width:
            y += row_height
            x = row_height = 0
        assert y+tile.height <= height-original.height
        distance = np.zeros((tile.height,tile.width),dtype=np.int16)
        for radius in range(1,9):
            distance += np.asarray(tile.filter(ImageFilter.MinFilter(2*radius+1)))>127
            distance -= np.asarray(tile.filter(ImageFilter.MaxFilter(2*radius+1)))<=127
        rgba = Image.new('RGBA',tile.size,'white')
        rgba.putalpha(Image.fromarray((127+15*distance).astype('uint8')))
        canvas.paste(rgba,(x,y))
        gid = 13000+index
        advance = font.getlength(cluster[0])*scale
        additions.append(struct.pack('<I5f4ifi',gid,w,h,l*scale,-t*scale,advance,x+10,height-y-10-h,w,h,1.,0)+bytes(stride-48))
        char_additions.append(struct.pack('<IIIf',1,ord(symbol),gid,1.))
        x += tile.width
        row_height = max(row_height,tile.height)
    for i,(he,latin) in enumerate([('׳',"'"),('״','"'),('־','-')]):
        gid = 15000+i
        additions.append(U32.pack(gid)+records[characters[ord(latin)]][4:])
        char_additions.append(struct.pack('<IIIf',1,ord(he),gid,1.))
    dims = struct.pack('<II',*original.size)
    assert dims in raw[end:]
    total = U32.pack(count+len(additions))
    obj.set_raw_data(raw[:start]+total+raw[start+4:chars]+b''.join(additions)+total+
                     raw[chars+4:end]+b''.join(char_additions)+raw[end:].replace(dims,struct.pack('<II',width,height)))
    tex.set_image(canvas)
    tex.save()
    print('UI font',obj.path_id,'cap height',hh,'Hebrew scale',round(scale,3),flush=True)
    return canvas
