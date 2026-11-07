"""Trace the supplied logo as editable paths. Run with uv --with pillow --with vtracer."""
from pathlib import Path
from tempfile import TemporaryDirectory
import re
from PIL import Image, ImageFilter
import vtracer


def layer_steam(svg):
    """Separate this artwork's plume, preserving the trace and its paint order."""
    paths = re.findall(r'<path\b[^>]+/>', svg)
    # These layers belong to the supplied 45-path VTracer composition. Fail before
    # replacing the asset if a different source/tracer changes that composition.
    if (len(paths) != 45 or 'fill="#133A57"' not in paths[6]
            or 'translate(1470.1796875,70.765625)' not in paths[6]):
        raise ValueError('Logo trace changed; identify the steam layers before regenerating.')
    steam_indices = {2, 6, 7, 15, 16, 17, 34}
    header = re.search(r'<svg\b[^>]+>', svg)[0]
    title = '<title id="kitor-logo-title">קיטור — Kitor</title>'
    clips = ('<defs>\n'
             '<clipPath id="kitor-logo-chimney"><rect x="0" y="480" width="1774" height="407"/></clipPath>\n'
             '<clipPath id="kitor-logo-steam"><rect width="1774" height="480"/></clipPath>\n'
             '</defs>')
    # The orange silhouette spans both chimney and steam; split at the rim.
    chimney = '<g clip-path="url(#kitor-logo-chimney)">' + paths[1] + '</g>'
    steam = ('<g class="kt-brand-steam">\n'
             '<g clip-path="url(#kitor-logo-steam)">' + paths[1] + '</g>\n'
             + '\n'.join(paths[i] for i in sorted(steam_indices)) + '\n</g>')
    body = '\n'.join(path for i, path in enumerate(paths)
                     if i not in steam_indices and i not in {0, 1})
    return '\n'.join([header + title, clips, paths[0], chimney, steam, body, '</svg>', ''])


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
    trace_path = Path(tmp) / 'trace.svg'
    vtracer.convert_image_to_svg_py(str(input_path), str(trace_path), colormode='color', hierarchical='stacked', mode='spline', filter_speckle=30, color_precision=8, layer_difference=16, corner_threshold=60, length_threshold=4, max_iterations=10, splice_threshold=45, path_precision=2)
    svg = trace_path.read_text()
    svg = re.sub(r'<svg ([^>]+)>', lambda m: f'<svg {m[1]} viewBox="0 0 {source.width} {source.height}" role="img" aria-labelledby="kitor-logo-title">', svg, count=1)
    svg = layer_steam(svg)
    dest.write_text(svg)
    print(f'{dest}: {dest.stat().st_size:,} bytes, {svg.count("<path")} paths')
