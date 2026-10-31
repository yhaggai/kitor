// he_layout(text,maxColumns[,plainMode]) -> owned ds_grid; destroy when replaced.
// Column i corresponds to original 1-based source character index.
// Fields: 0 visual column (-1 no glyph); 1 row; 2 display char;
// 3 kind (0 control/suppressed,1 glyph,2 icon,3 newline);
// 4 choice anchor (-1 normal,0 \>0,1 \>1); 5 width in cells.
// Metadata [0,0]=row count; [0,1]=max occupied columns; [0,2]=has Hebrew;
// [0,3]=first / or % source position (0 absent); [0,4]=source length.
var text = string(argument0), capacity = max(1,floor(argument1));
var plain = 0; if (argument_count > 2) plain = argument2;
var length = string_length(text), result = ds_grid_create(length+1,6);
var tokens = ds_grid_create(length+1,4);
var i, j, ch, code, param, tokenCount = 0, anchor = -1, kind, consumed, sourceindex;
for (i = 0; i <= length; i++) {
    ds_grid_set(result,i,0,-1); ds_grid_set(result,i,1,0);
    ds_grid_set(result,i,2,""); ds_grid_set(result,i,3,0);
    ds_grid_set(result,i,4,-1); ds_grid_set(result,i,5,0);
}
ds_grid_set(result,0,0,1); ds_grid_set(result,0,1,0);
ds_grid_set(result,0,2,0); ds_grid_set(result,0,3,0); ds_grid_set(result,0,4,length);
i = 1;
while (i <= length) {
    ch = string_char_at(text,i); consumed = 1; kind = 1; sourceindex = i;
    if (!plain && (ch == "/" || ch == "%")) { ds_grid_set(result,0,3,i); break; }
    if ((!plain && ch == "&") || (plain && ch == "#") || ord(ch) == 10 || ord(ch) == 13) {
        ds_grid_set(tokens,tokenCount,0,i); ds_grid_set(tokens,tokenCount,1,"");
        ds_grid_set(tokens,tokenCount,2,1); ds_grid_set(tokens,tokenCount,3,anchor);
        ds_grid_set(result,i,3,3); tokenCount++; anchor = -1;
        if (ord(ch) == 13 && i < length) if (ord(string_char_at(text,i+1)) == 10) consumed = 2;
        kind = 0;
    }
    else if (!plain && ch == "^" && i < length) {
        code = ord(string_char_at(text,i+1));
        if (code >= 48 && code <= 57) { consumed = 2; kind = 0; }
    }
    else if (!plain && ord(ch) == 92) {
        kind = 0; consumed = min(2,length-i+1);
        code = string_char_at(text,i+1);
        if (string_pos(code,"EMFTSz*>") > 0 && code != "") {
            consumed = min(3,length-i+1); param = string_char_at(text,i+2);
            if (code == ">") {
                anchor = 0; if (param == "1") anchor = 1;
                ds_grid_set(tokens,tokenCount,0,i); ds_grid_set(tokens,tokenCount,1,"");
                ds_grid_set(tokens,tokenCount,2,2); ds_grid_set(tokens,tokenCount,3,anchor); tokenCount++;
            }
            else if ((code == "*" || code == "z") && i+2 <= length) {
                // Map icon to the argument index used by base_writer's draw branch.
                ch = chr(65532); kind = 2; sourceindex = i+2;
            }
        }
    }
    if (kind > 0) {
        ds_grid_set(tokens,tokenCount,0,sourceindex); ds_grid_set(tokens,tokenCount,1,ch);
        ds_grid_set(tokens,tokenCount,2,0); ds_grid_set(tokens,tokenCount,3,anchor); tokenCount++;
        ds_grid_set(result,sourceindex,3,kind);
        if (he_bidi_type(ch) == 1) ds_grid_set(result,0,2,1);
    }
    i += consumed;
}
// Wrap each explicit line/choice segment at whitespace; oversized words hard-wrap.
var pos = 0, rowno = 0, segmentEnd, lineEnd, lastSpace, typ, segmentCapacity, gapstart, gapend;
anchor = -1;
while (pos < tokenCount) {
    typ = ds_grid_get(tokens,pos,2);
    if (typ == 1) {
        sourceindex = ds_grid_get(tokens,pos,0); ds_grid_set(result,sourceindex,1,rowno);
        rowno++; anchor = -1; pos++; continue;
    }
    if (typ == 2) { anchor = ds_grid_get(tokens,pos,3); pos++; continue; }
    segmentEnd = pos;
    while (segmentEnd < tokenCount) {
        if (ds_grid_get(tokens,segmentEnd,2) != 0) break;
        segmentEnd++;
    }
    segmentCapacity = capacity;
    // Choice rows encode cursor columns in repeated spaces. Never wrap those
    // fields: doing so would detach their labels from the game's selection.
    for (j = pos+1; j < segmentEnd-1; j++) {
        if (ds_grid_get(tokens,j,1) == " " && ds_grid_get(tokens,j-1,1) != " ") {
            gapstart = j; gapend = j;
            while (gapend < segmentEnd) { if (ds_grid_get(tokens,gapend,1) != " ") break; gapend++; }
            if (gapend-gapstart >= 2 && gapend < segmentEnd) segmentCapacity = max(capacity,segmentEnd-pos);
            j = gapend-1;
        }
    }
    while (pos < segmentEnd) {
        lineEnd = min(segmentEnd,pos+segmentCapacity);
        if (lineEnd < segmentEnd) {
            lastSpace = -1;
            for (j = pos+1; j < lineEnd; j++) if (ds_grid_get(tokens,j,1) == " ") lastSpace = j;
            if (lastSpace >= 0) lineEnd = lastSpace;
        }
        he_layout_line(result,tokens,pos,lineEnd-1,rowno,capacity,anchor,plain);
        pos = lineEnd;
        if (pos < segmentEnd) {
            // Suppress only whitespace consumed by a soft wrap. Source indices
            // and control timing remain unchanged in the original writer.
            while (pos < segmentEnd) { if (ds_grid_get(tokens,pos,1) != " ") break; ds_grid_set(result,ds_grid_get(tokens,pos,0),3,0); pos++; }
            if (pos < segmentEnd) rowno++;
        }
    }
}
ds_grid_set(result,0,0,rowno+1);
ds_grid_destroy(tokens);
return result;
