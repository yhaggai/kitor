// Internal: write one wrapped source-token line into layout result.
// result,tokens,start,end,row,maxColumns,anchor,plainMode
var result = argument0, tokens = argument1, first = argument2, last = argument3;
var rowno = argument4, capacity = argument5, anchor = argument6, plain = argument7;
var count = last-first+1;
if (count <= 0) return 0;
var text = "", i, j, k, ch, run, hascolumns = 0, rtl = 0;
for (i = first; i <= last; i++) text += ds_grid_get(tokens,i,1);
for (i = 1; i <= count; i++) {
    ch = string_char_at(text,i);
    if (he_bidi_type(ch) == 1) rtl = 1;
    if (ch == " " && i > 1 && i < count) {
        j = i; while (j <= count) { if (string_char_at(text,j) != " ") break; j++; }
        if (j-i >= 2 && j <= count && string_char_at(text,i-1) != " ") hascolumns = 1;
        i = j-1;
    }
}
var offset = 0;
if (rtl && !plain && !hascolumns && anchor == -1) offset = max(0,capacity-count);
// Preserve leading/trailing padding and every multiple-space column gap.
// Run text within each fixed field through bidi; never exchange field order.
i = 1;
while (i <= count) {
    ch = string_char_at(text,i);
    if (ch == " ") {
        j = i; while (j <= count) { if (string_char_at(text,j) != " ") break; j++; }
        if (i == 1 || j > count || j-i >= 2) {
            for (k = i; k < j; k++) {
                var src = ds_grid_get(tokens,first+k-1,0);
                ds_grid_set(result,src,0,offset+k-1); ds_grid_set(result,src,1,rowno);
                ds_grid_set(result,src,2," "); ds_grid_set(result,src,4,anchor);
                ds_grid_set(result,src,5,1);
            }
            i = j;
        }
    }
    if (i > count) break;
    var segmentstart = i;
    j = i;
    while (j <= count) {
        if (string_char_at(text,j) == " ") {
            k = j; while (k <= count) { if (string_char_at(text,k) != " ") break; k++; }
            if (k > count || k-j >= 2) break;
            j = k;
        } else j++;
    }
    if (j == segmentstart) continue;
    var segment = string_copy(text,segmentstart,j-segmentstart);
    var visual = he_bidi(segment);
    for (k = 1; k <= j-segmentstart; k++) {
        var sourceindex = ds_grid_get(tokens,first+segmentstart+k-2,0);
        ds_grid_set(result,sourceindex,0,offset+segmentstart-1+ds_grid_get(visual,k,0));
        ds_grid_set(result,sourceindex,1,rowno);
        ds_grid_set(result,sourceindex,2,ds_grid_get(visual,k,1));
        ds_grid_set(result,sourceindex,4,anchor);
        ds_grid_set(result,sourceindex,5,1);
    }
    ds_grid_destroy(visual);
    i = j;
}
ds_grid_set(result,0,1,max(ds_grid_get(result,0,1),offset+count));
return count;
