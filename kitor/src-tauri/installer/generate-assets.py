"""Generate installer artwork with Pillow and macOS system fonts."""
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent
FONT = '/System/Library/Fonts/Supplemental/Arial.ttf'
BOLD = '/System/Library/Fonts/Supplemental/Arial Bold.ttf'

def artwork(size):
    image = Image.new('RGB', size, '#111b2a')
    draw = ImageDraw.Draw(image)
    for y in range(size[1]):
        t = y / size[1]
        draw.line((0, y, size[0], y), fill=(17 + int(7*t), 27 + int(12*t), 42 + int(17*t)))
    return image, draw

def text(draw, xy, value, size, color='#f5eee0', bold=False):
    draw.text(xy, value, font=ImageFont.truetype(BOLD if bold else FONT, size), fill=color)

im, d = artwork((660, 440))
text(d, (42, 30), 'KITOR', 16, '#e6b86b', True)
text(d, (42, 65), 'Your next adventure starts here.', 29, bold=True)
text(d, (42, 112), 'Classic adventures. A little closer to home.', 16, '#b1bdcb')
d.line((42, 158, 618, 158), fill='#354254')
d.line((290, 254, 362, 254), fill='#e6b86b', width=3)
d.line((350, 244, 362, 254, 350, 264), fill='#e6b86b', width=3)
text(d, (133, 350), 'Drag Kitor into Applications to install.', 20)
text(d, (185, 391), 'Then open Kitor from Applications.', 15, '#b1bdcb')
im.save(ROOT / 'dmg-background.png')
im, d = artwork((164, 314))
text(d, (16, 28), 'KITOR', 28, '#e6b86b', True)
d.line((16, 78, 148, 78), fill='#59606b')
for y, value in [(104, 'Your next'), (130, 'adventure'), (156, 'starts here.')]:
    text(d, (16, y), value, 20, bold=True)
text(d, (16, 263), 'Play it in Hebrew.', 14, '#b1bdcb')
im.save(ROOT / 'windows-sidebar.bmp')
im, d = artwork((150, 57))
text(d, (14, 14), 'KITOR', 24, '#e6b86b', True)
im.save(ROOT / 'windows-header.bmp')
