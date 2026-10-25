// Plain draw_text conversion. No writer commands are parsed. '#' and newlines
// keep line order. Fixed multi-space menu fields keep their original columns.
var text = string(argument0), count = string_length(text), hashebrew = 0, i;
if (count < 2) return text;
for (i = 1; i <= count; i++) if (he_bidi_type(string_char_at(text,i)) == 1) { hashebrew = 1; break; }
if (!hashebrew) return text;
var layout = he_layout(text,max(1,count),1);
var rows = ds_grid_get(layout,0,0), output = "", rowno, col, rowlength, glyphs;
for (rowno = 0; rowno < rows; rowno++) {
    rowlength = 0;
    for (i = 1; i <= count; i++) {
        col = ds_grid_get(layout,i,0);
        if (col >= 0 && ds_grid_get(layout,i,1) == rowno) {
            glyphs[col] = ds_grid_get(layout,i,2); rowlength = max(rowlength,col+1);
        }
    }
    for (col = 0; col < rowlength; col++) output += glyphs[col];
    if (rowno < rows-1) output += "#";
}
ds_grid_destroy(layout);
return output;
