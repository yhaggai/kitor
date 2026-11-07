"""Trace the supplied logo as editable paths. Run with uv --with pillow --with vtracer."""
from pathlib import Path
from tempfile import TemporaryDirectory
import re
from PIL import Image, ImageFilter
import vtracer

assets = Path(__file__).resolve().parents[1] / 'src' / 'assets'
source = Image.open(assets / 'kitor-logo.png').convert('RGB')
# A fixed palette removes paper grain without altering the lettering or silhouette.
colors = ['fff0cf', '123b59', 'f47918', 'b94a0d', 'ffcd69', '6d300e', 'ffac4e']
palette = Image.new('P', (1, 1))
values = [int(color[i:i+2], 16) for color in colors for i in (0, 2, 4)]
palette.putpalette(values + values[:3] * (256 - len(colors)))
clean = source.filter(ImageFilter.MedianFilter(5)).quantize(palette=palette, dither=Image.Dither.NONE).convert('RGB')
with TemporaryDirectory() as tmp:
    input_path = Path(tmp) / 'clean.png'
    clean.save(input_path)
    dest = assets / 'kitor-logo.svg'
    vtracer.convert_image_to_svg_py(str(input_path), str(dest), colormode='color', hierarchical='stacked', mode='spline', filter_speckle=30, color_precision=8, layer_difference=16, corner_threshold=60, length_threshold=4, max_iterations=10, splice_threshold=45, path_precision=2)
    svg = dest.read_text()
    svg = re.sub(r'<svg ([^>]+)>', lambda m: f'<svg {m[1]} viewBox="0 0 {source.width} {source.height}" role="img" aria-labelledby="logo-title"><title id="logo-title">קיטור — Kitor</title>', svg, count=1)
    dest.write_text(svg)
    print(f'{dest}: {dest.stat().st_size:,} bytes, {svg.count("<path")} paths')
