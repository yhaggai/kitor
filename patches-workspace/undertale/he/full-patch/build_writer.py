from pathlib import Path
ROOT=Path(__file__).resolve().parent
SRC=ROOT.parent/'decompiled/CodeEntries'
writer=(SRC/'gml_Object_obj_base_writer_Draw_0.gml').read_text()
header='''// Hebrew full-text alpha: cache layout once per resolved message and width.
// writingxend is an old wrapping threshold, not the last glyph's origin.
// Reserve the entire final cell and keep unbounded writers inside the viewport.
var he_right = min(writingxend, view_xview[view_current] + view_wview[view_current] - 12);
for (var he_kind = 0; he_kind < 2; he_kind++) {
    var he_object = obj_blconwdflowey;
    if (he_kind == 1) he_object = obj_blconsm;
    var he_bubble = instance_nearest(writingx, writingy, he_object);
    if (!instance_exists(he_bubble)) continue;
    var he_bx = he_bubble.x - sprite_get_xoffset(he_bubble.sprite_index) * he_bubble.image_xscale;
    var he_by = he_bubble.y - sprite_get_yoffset(he_bubble.sprite_index) * he_bubble.image_yscale;
    var he_bw = sprite_get_width(he_bubble.sprite_index) * he_bubble.image_xscale;
    var he_bh = sprite_get_height(he_bubble.sprite_index) * he_bubble.image_yscale;
    if (writingx >= he_bx && writingx < he_bx + he_bw && writingy >= he_by && writingy < he_by + he_bh) {
        he_right = min(he_right, he_bx + he_bw - 12);
    }
}
var he_columns = max(1, floor((he_right - writingx) / max(1, spacing)));
// Papyrus's fixed Latin cells leave large gaps around narrow Hebrew letters.
var he_proportional = myfont == fnt_papyrus && !vtext
    && string_pos("  ", originalstring) == 0
    && string_pos(chr(92) + ">", originalstring) == 0
    && string_pos(chr(92) + "C", originalstring) == 0;
if (!variable_instance_exists(id, "he_grid")) { he_grid = -1; he_cached_text = ""; he_cached_columns = -1; }
if (he_grid == -1 || he_cached_text != originalstring || he_cached_columns != he_columns || he_cached_proportional != he_proportional || he_cached_spacing != spacing) {
    if (he_grid != -1) ds_grid_destroy(he_grid);
    he_grid = he_layout(originalstring, he_columns);
    if (he_proportional && ds_grid_get(he_grid,0,2)) {
        var he_rows = ds_grid_get(he_grid,0,0);
        var he_stride = ds_grid_get(he_grid,0,1);
        var he_sources = ds_grid_create(he_stride,he_rows);
        ds_grid_clear(he_sources,0);
        for (var he_i = 1; he_i <= string_length(originalstring); he_i++) {
            var he_col = ds_grid_get(he_grid,he_i,0);
            if (he_col >= 0) ds_grid_set(he_sources,he_col,ds_grid_get(he_grid,he_i,1),he_i);
        }
        for (var he_row = 0; he_row < he_rows; he_row++) {
            var he_edge = 0;
            var he_previous_cell = -2;
            for (var he_cell = he_stride-1; he_cell >= 0; he_cell--) {
                var he_source = ds_grid_get(he_sources,he_cell,he_row);
                if (he_source == 0) continue;
                var he_char = ds_grid_get(he_grid,he_source,2);
                var he_advance = spacing;
                var he_code = ord(he_char);
                if (he_code >= 1488 && he_code <= 1514)
                    he_advance = min(spacing,max(1,ceil((ord(string_char_at("__PAPYRUS_ADVANCES__",he_code-1487))-64)*htextscale)));
                if (he_cell != he_previous_cell-1) he_edge = (he_cell+1)*spacing;
                he_edge -= he_advance;
                he_previous_cell = he_cell;
                ds_grid_set(he_grid,he_source,0,he_edge);
            }
        }
        ds_grid_destroy(he_sources);
    }
    he_cached_proportional = he_proportional;
    he_cached_spacing = spacing;
    he_cached_text = originalstring;
    he_cached_columns = he_columns;
}
var he_rtl = ds_grid_get(he_grid,0,2) && !vtext;
'''
# Encode the generated glyph advances in a string for the legacy GML runtime.
papyrus_metrics = {}
for row in (ROOT/'generated-fonts/fnt_papyrus.csv').read_text().splitlines():
    code, _, _, _, _, advance, _ = map(int, row.split(';'))
    papyrus_metrics[code] = advance
header = header.replace('__PAPYRUS_ADVANCES__', ''.join(chr(64+papyrus_metrics[code]) for code in range(1488,1515)))
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
    var he_xoffset = ds_grid_get(he_grid,{index},0);
    if (!he_proportional) he_xoffset *= spacing;
    {xvar} = he_origin + he_xoffset;
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
