from pathlib import Path
ROOT=Path(__file__).resolve().parent
SRC=ROOT.parent/'decompiled/CodeEntries'
writer=(SRC/'gml_Object_obj_base_writer_Draw_0.gml').read_text()
header='''// Hebrew full-text alpha: cache layout once per resolved message and width.
// writingxend is an old wrapping threshold, not the last glyph's origin.
// Reserve the entire final cell and keep unbounded writers inside the viewport.
var he_right = min(writingxend, view_xview[view_current] + view_wview[view_current] - 12);
var he_bubble = instance_nearest(writingx, writingy, obj_blconwdflowey);
if (instance_exists(he_bubble)) {
    var he_bx = he_bubble.x - sprite_get_xoffset(he_bubble.sprite_index) * he_bubble.image_xscale;
    var he_by = he_bubble.y - sprite_get_yoffset(he_bubble.sprite_index) * he_bubble.image_yscale;
    var he_bw = sprite_get_width(he_bubble.sprite_index) * he_bubble.image_xscale;
    var he_bh = sprite_get_height(he_bubble.sprite_index) * he_bubble.image_yscale;
    if (writingx >= he_bx && writingx < he_bx + he_bw && writingy >= he_by && writingy < he_by + he_bh) {
        he_right = min(he_right, he_bx + he_bw - 12);
    }
}
var he_columns = max(1, floor((he_right - writingx) / max(1, spacing)));
if (!variable_instance_exists(id, "he_grid")) { he_grid = -1; he_cached_text = ""; he_cached_columns = -1; }
if (he_grid == -1 || he_cached_text != originalstring || he_cached_columns != he_columns) {
    if (he_grid != -1) ds_grid_destroy(he_grid);
    he_grid = he_layout(originalstring, he_columns);
    he_cached_text = originalstring;
    he_cached_columns = he_columns;
}
var he_rtl = ds_grid_get(he_grid,0,2) && !vtext;
'''
# The original control interpreter/reveal loop remains authoritative.
writer=writer.replace('if (!vtext && myx > writingxend)','if (!he_rtl && !vtext && myx > writingxend)')
def position(index,xvar,yvar):
 return f'''if (he_rtl && ds_grid_get(he_grid,{index},3) > 0) {{
    var he_anchor = ds_grid_get(he_grid,{index},4);
    var he_origin = writingx;
    if (he_anchor >= 0) {{
        he_origin = 100;
        if (he_anchor == 1) he_origin = 196;
        else if (myfont == fnt_ja_comicsans_big) he_origin += 11;
        if (view_wview[view_current] == 640) he_origin *= 2;
        he_origin += view_xview[view_current];
    }}
    {xvar} = he_origin + ds_grid_get(he_grid,{index},0) * spacing;
    {yvar} = writingy + ds_grid_get(he_grid,{index},1) * vspacing;
}}'''
needle='        var letterx = myx;';assert writer.count(needle)==1
writer=writer.replace(needle,needle+'\n        if (he_rtl && ds_grid_get(he_grid,n,3) == 0) continue;\n'+position('n','letterx','myy')+'\n        if (he_rtl) myletter = ds_grid_get(he_grid,n,2);')
needle='                var spritey = myy;';assert writer.count(needle)==1
writer=writer.replace(needle,needle+'\n'+position('n','spritex','spritey'))
needle='                var sym_s = 862;';assert writer.count(needle)==1
writer=writer.replace(needle,needle+'\n'+position('n','myx','myy'))
(ROOT/'gml_Object_obj_base_writer_Draw_0.gml').write_text(header+writer)
create=(SRC/'gml_Object_obj_base_writer_Create_0.gml').read_text()
(ROOT/'gml_Object_obj_base_writer_Create_0.gml').write_text(create+'\nhe_grid = -1;\nhe_cached_text = "";\nhe_cached_columns = -1;\n')
(ROOT/'gml_Object_obj_base_writer_Destroy_0.gml').write_text('if (variable_instance_exists(id,"he_grid")) { if (he_grid != -1) { ds_grid_destroy(he_grid); he_grid = -1; } }\n')
print('Generated general base-writer RTL layout hooks and grid cleanup.')
