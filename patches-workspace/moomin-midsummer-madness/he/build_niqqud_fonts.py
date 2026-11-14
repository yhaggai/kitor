"""Build composite TMP SDF glyphs for every niqqud cluster in catalog-niqqud.json."""
from __future__ import annotations
import argparse, copy, json, re, unicodedata
from pathlib import Path
import numpy as np
from scipy.ndimage import distance_transform_edt
from PIL import Image
import freetype, uharfbuzz as hb
import UnityPy

BASE=Path(__file__).resolve().parent
CLUSTER=re.compile(r'[\u05d0-\u05ea][\u0591-\u05bd\u05bf\u05c1-\u05c2\u05c4-\u05c5\u05c7]+')
FONT_NAMES=('Pangolin-Regular SDF MAIN','OpenDyslexic-Regular_SDF','BalsamiqSans-Regular_SDF','BalsamiqSans-Bold_SDF')
PUA_START=0xE200
PAD=12
SCALE=4

def sdf_tile(cluster, font_path, font_size, pad=PAD):
    # HarfBuzz applies the font's GPOS mark-to-base anchors; FreeType rasterizes
    # each shaped glyph ID. This avoids guessing mark positions or reversing them.
    face=freetype.Face(font_path);face.set_pixel_sizes(0,font_size*SCALE)
    blob=hb.Blob.from_file_path(font_path);hface=hb.Face(blob);hfont=hb.Font(hface)
    hfont.scale=(font_size*SCALE*64,font_size*SCALE*64);hb.ot_font_set_funcs(hfont)
    buf=hb.Buffer();buf.add_str(cluster);buf.direction='rtl';buf.script='Hebr';buf.language='he';hb.shape(hfont,buf)
    pen_x=pen_y=0;layers=[];left=top=10**9;right=bottom=-10**9
    for info,pos in zip(buf.glyph_infos,buf.glyph_positions):
        face.load_glyph(info.codepoint,freetype.FT_LOAD_RENDER|freetype.FT_LOAD_NO_HINTING)
        slot=face.glyph;bm=slot.bitmap
        origin_x=pen_x+pos.x_offset;origin_y=pen_y+pos.y_offset
        gx=round(origin_x/64)+slot.bitmap_left;gy=-round(origin_y/64)-slot.bitmap_top
        if bm.width and bm.rows:
            pitch=abs(bm.pitch);raw=np.frombuffer(bytes(bm.buffer),dtype=np.uint8).reshape((bm.rows,pitch))[:,:bm.width]
            if bm.pitch<0:raw=raw[::-1]
            layer=Image.fromarray(raw.copy(),'L');layers.append((gx,gy,layer))
            left=min(left,gx);top=min(top,gy);right=max(right,gx+bm.width);bottom=max(bottom,gy+bm.rows)
        pen_x+=pos.x_advance;pen_y+=pos.y_advance
    if right<=left or bottom<=top:raise ValueError(f'Empty HarfBuzz raster: {cluster!r}')
    w=max(1,(right-left+SCALE-1)//SCALE);h=max(1,(bottom-top+SCALE-1)//SCALE)
    oversampled=Image.new('L',(w*SCALE,h*SCALE),0)
    for gx,gy,layer in layers:
        tmp=Image.new('L',oversampled.size,0);tmp.paste(layer,(gx-left,gy-top));oversampled=Image.fromarray(np.maximum(np.asarray(oversampled),np.asarray(tmp)).astype(np.uint8),'L')
    mask=oversampled.resize((w,h),Image.Resampling.LANCZOS)
    bw=w+2*pad;bh=h+2*pad
    canvas=Image.new('L',(bw,bh),0);canvas.paste(mask,(pad,pad))
    a=np.asarray(canvas,dtype=np.uint8)
    inside=a>=128
    if not inside.any():raise ValueError(f'Empty raster: {cluster!r}')
    outside_dist=distance_transform_edt(~inside)
    inside_dist=distance_transform_edt(inside)
    # Match TMP's 8-bit signed distance convention (128 at outline, ~8 levels/pixel).
    sdf=np.clip(128.0+(inside_dist-outside_dist)*8.0,0,255).astype(np.uint8)
    # y is rasterized from top down; TMP bearing is measured upward from baseline.
    bearing_x=left//SCALE-pad
    bearing_y=-(top//SCALE-pad)
    return Image.fromarray(sdf,'L'),(bw,bh,bearing_x,bearing_y)

def clone_entry(proto):
    clone=copy.copy(proto)
    if hasattr(proto,'m_GlyphRect'):clone.m_GlyphRect=copy.copy(proto.m_GlyphRect)
    if hasattr(proto,'m_Metrics'):clone.m_Metrics=copy.copy(proto.m_Metrics)
    return clone

def append_font(env, font_obj, codes, font_path, output_prefix):
    font=font_obj.read()
    texture_obj=next(o for o in env.objects if o.path_id==font.m_AtlasTextures[0].path_id)
    texture=texture_obj.read()
    old_w,old_h=texture.m_Width,texture.m_Height
    old=Image.frombytes('L',(old_w,old_h),texture.image_data)
    # Pack in 1024-wide rows appended beneath the existing atlas.
    x=0;y=old_h;row_h=0;placements=[];tiles=[]
    basechars={c.m_Unicode:c.m_GlyphIndex for c in font.m_CharacterTable}
    glyph_by_index={g.m_Index:g for g in font.m_GlyphTable}
    # Match the actual unpointed Hebrew glyph height already present in this font.
    # The previous pass used a 44 px Arial raster that rendered substantially smaller.
    ref_heights=[glyph_by_index[basechars[cp]].m_GlyphRect.m_Height for cp in range(0x05d0,0x05eb)
                 if cp in basechars and basechars[cp] in glyph_by_index and glyph_by_index[basechars[cp]].m_GlyphRect.m_Height>0]
    ref_height=float(np.median(ref_heights)) if ref_heights else 38.0
    sample,_=sdf_tile('מ',font_path,44)
    sample_bbox=sample.point(lambda a:255 if a>=128 else 0).getbbox()
    sample_height=sample_bbox[3]-sample_bbox[1]
    font_size=max(44,round(44*ref_height/sample_height))
    print(f'{output_prefix}: {font.m_Name}: sizing Hebrew to {ref_height:.1f}px from Arial {font_size}px',flush=True)
    # Find ordinary Hebrew glyph metrics to keep advance and size tied to each base letter.
    for cluster,code in codes.items():
        tile,(w,h,bx,by)=sdf_tile(cluster,font_path,font_size)
        if x+w+1>old_w:x=0;y+=row_h+1;row_h=0
        if y+h>8192:raise ValueError('Composite glyph atlas exceeds 8192 px')
        tiles.append((x,y,tile));placements.append((cluster,code,x,y,w,h,bx,by))
        x+=w+1;row_h=max(row_h,h)
    new_h=max(old_h,y+row_h)
    atlas=Image.new('L',(old_w,new_h),0);atlas.paste(old,(0,0))
    # Texture2D image_data is Unity's bottom-up raw Alpha8 layout. Existing
    # glyphs are already in that convention; flip only new PIL tiles on write.
    for x,y,t in tiles:atlas.paste(t.transpose(Image.Transpose.FLIP_TOP_BOTTOM),(x,y))
    for cluster,code,x,y,w,h,bx,by in placements:
        base=ord(cluster[0]);base_idx=basechars.get(base)
        if base_idx is None or base_idx not in glyph_by_index:
            raise ValueError(f'{font.m_Name}: missing base Hebrew glyph {cluster[0]}')
        proto=glyph_by_index[base_idx];g=clone_entry(proto)
        g.m_Index=20000+(ord(code)-PUA_START)
        g.m_GlyphRect.m_X=x;g.m_GlyphRect.m_Y=y;g.m_GlyphRect.m_Width=w;g.m_GlyphRect.m_Height=h
        g.m_Metrics.m_Width=float(w);g.m_Metrics.m_Height=float(h)
        g.m_Metrics.m_HorizontalBearingX=float(bx);g.m_Metrics.m_HorizontalBearingY=float(by)
        g.m_Metrics.m_HorizontalAdvance=proto.m_Metrics.m_HorizontalAdvance
        g.m_Scale=1.0;g.m_AtlasIndex=0
        ch=clone_entry(font.m_CharacterTable[0]);ch.m_Unicode=ord(code);ch.m_GlyphIndex=g.m_Index;ch.m_Scale=1.0
        font.m_GlyphTable.append(g);font.m_CharacterTable.append(ch)
    font.m_AtlasHeight=new_h
    texture.m_Height=new_h;texture.m_CompleteImageSize=len(atlas.tobytes());texture.image_data=atlas.tobytes()
    texture_obj.save_typetree(texture)
    try:font_obj.save_typetree(font)
    except Exception:
        print('font-save diagnostic:',vars(font.m_GlyphTable[-1]),vars(font.m_CharacterTable[-1]),flush=True)
        raise
    print(f'{output_prefix}: {font.m_Name}: +{len(codes)} clusters, atlas {old_w}x{old_h} -> {old_w}x{new_h}')
    return True

def main():
    ap=argparse.ArgumentParser();ap.add_argument('font_bundle',type=Path);ap.add_argument('sharedassets',type=Path);ap.add_argument('--out',type=Path,required=True);a=ap.parse_args()
    rows=json.loads((BASE/'catalog-niqqud.json').read_text(encoding='utf-8'))
    clusters=sorted({unicodedata.normalize('NFD',m.group()) for r in rows for m in CLUSTER.finditer(r['he'])})
    codes={c:chr(PUA_START+i) for i,c in enumerate(clusters)}
    (BASE/'niqqud-glyphs.json').write_text(json.dumps({c:ord(v) for c,v in codes.items()},ensure_ascii=False,indent=1),encoding='utf-8')
    fonts=UnityPy.load(a.font_bundle.read_bytes());shared=UnityPy.load(a.sharedassets.read_bytes())
    for n in FONT_NAMES:
        fo=next(o for o in fonts.objects if o.type.name=='MonoBehaviour' and o.peek_name()==n)
        sf=next(o for o in shared.objects if o.type.name=='MonoBehaviour' and o.peek_name()==n)
        sf.serialized_type.node=fo.serialized_type.node
        arial='C:/Windows/Fonts/arialbd.ttf' if 'Bold' in n else 'C:/Windows/Fonts/arial.ttf'
        append_font(fonts,fo,codes,arial,'bundle')
        append_font(shared,sf,codes,arial,'sharedassets')
    a.out.mkdir(parents=True,exist_ok=True)
    (a.out/'fonts_assets_all.bundle').write_bytes(fonts.file.save(packer='lz4'))
    (a.out/'sharedassets0.assets').write_bytes(shared.file.save(packer='lz4'))
    print(f'Created map with {len(codes)} clusters and PUA range U+{PUA_START:04X}–U+{PUA_START+len(codes)-1:04X}')
if __name__=='__main__':main()
