"""Shape an entire Hebrew grapheme before rasterizing its glyph components."""
from functools import lru_cache
from pathlib import Path
import freetype
import uharfbuzz as hb
from PIL import Image, ImageChops


@lru_cache(maxsize=2)
def fonts(path):
    font = hb.Font(hb.Face(Path(path).read_bytes()))
    font.scale = (80 * 64, 80 * 64)
    hb.ot_font_set_funcs(font)
    face = freetype.Face(path)
    face.set_pixel_sizes(0, 80)
    return font, face


def shape(text, path):
    font, face = fonts(path)
    buffer = hb.Buffer()
    buffer.add_str(text)
    buffer.guess_segment_properties()
    hb.shape(font, buffer)
    parts = []; pen = 0
    for info, pos in zip(buffer.glyph_infos, buffer.glyph_positions):
        if info.codepoint == 0:
            raise ValueError(f'Missing shaped glyph: {text!r}')
        face.load_glyph(info.codepoint, freetype.FT_LOAD_RENDER)
        slot = face.glyph; bitmap = slot.bitmap
        x = round(pen + pos.x_offset / 64) + slot.bitmap_left
        y = -round(pos.y_offset / 64) - slot.bitmap_top
        if bitmap.width and bitmap.rows:
            pixels = bytes(bitmap.buffer)
            image = Image.frombytes('L', (bitmap.width, bitmap.rows), pixels, 'raw', 'L', bitmap.pitch)
            parts.append((x, y, image))
        pen += pos.x_advance / 64
    left = min(x for x, y, im in parts); top = min(y for x, y, im in parts)
    right = max(x + im.width for x, y, im in parts)
    bottom = max(y + im.height for x, y, im in parts)
    result = Image.new('L', (right-left, bottom-top))
    for x, y, im in parts:
        layer = Image.new('L', result.size); layer.paste(im, (x-left, y-top))
        result = ImageChops.lighter(result, layer)
    return result, (left, top, right, bottom)


def render_pointed(text, path, face, metric, antialias, render_base):
    raw, box = shape(text, path)
    base = face.getbbox('ה', anchor='ls')
    import statistics
    refwidth = statistics.median(face.getbbox(c, anchor='ls')[2]-face.getbbox(c, anchor='ls')[0]
                                 for c in 'אבדהחטמנספשת')
    sx = metric['reference_ink_width'] / refwidth
    sy = metric['body_height'] / max(1, base[3]-base[1])
    # Fit the complete cluster into the original cell, including upper/lower marks.
    # Keep the baseline transform uniform; never squash/crop marks independently.
    sy = min(sy, metric['cell_height'] / raw.height)
    top = max(0, round(metric['body_top'] + (box[1]-base[1])*sy))
    height = max(1, round(raw.height*sy))
    top = min(top, max(0, metric['cell_height']-height))
    width = max(1, round(raw.width*sx))
    scaled = raw.resize((width, height), Image.Resampling.LANCZOS)
    if not antialias:
        scaled = scaled.point(lambda v: 255 if v >= 80 else 0)
    if not scaled.getbbox():
        raise ValueError(f'Pointed glyph vanished: {text!r}')
    cell = Image.new('RGBA', (width, top+height))
    cell.paste(Image.new('RGBA', (width, height), 'white'), (0, top), scaled)
    _, advance = render_base(text[0], face, metric, antialias)
    return cell, advance
