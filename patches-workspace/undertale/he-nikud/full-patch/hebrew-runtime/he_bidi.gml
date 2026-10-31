// Logical character -> visual column and mirrored glyph. Caller destroys grid.
// [i,0]=visual col, [i,1]=glyph, [i,2]=resolved level; i is 1-based.
// [0,0]=length, [0,1]=paragraph level. Hebrew-containing segment defaults RTL.
var text = argument0;
var count = string_length(text);
var result = ds_grid_create(count + 1, 3);
var base = 0;
var typ, original, lev, order, i, j, k, t, previous, following;
for (i = 1; i <= count; i++) {
    t = he_bidi_type(string_char_at(text, i));
    typ[i] = t;
    original[i] = t;
    order[i] = i;
    if (t == 1 || t == 2) base = 1;
}
ds_grid_set(result, 0, 0, count);
ds_grid_set(result, 0, 1, base);
if (count == 0) return result;
// W1: nonspacing marks inherit preceding type. Catalog contains no niqqud.
previous = base;
for (i = 1; i <= count; i++) {
    if (typ[i] == 5) typ[i] = previous;
    previous = typ[i];
}
// W2/W3: Arabic contextual numbers and conversion to R.
previous = base;
for (i = 1; i <= count; i++) {
    t = typ[i];
    if (t == 3 && previous == 2) typ[i] = 4;
    if (t == 0 || t == 1 || t == 2) previous = t;
    if (t == 2) typ[i] = 1;
}
// W4: separators between matching numeric types.
for (i = 2; i < count; i++) {
    if (typ[i] == 6 && typ[i-1] == 3 && typ[i+1] == 3) typ[i] = 3;
    else if (typ[i] == 8 && (typ[i-1] == 3 || typ[i-1] == 4) && typ[i-1] == typ[i+1]) typ[i] = typ[i-1];
}
// UI tailoring: retain unary +/- with the following numeric run.
for (i = 1; i < count; i++) {
    if (typ[i] == 6 && (typ[i+1] == 3 || typ[i+1] == 4)) typ[i] = typ[i+1];
}
// W5: currency/percent runs adjacent to European numbers.
i = 1;
while (i <= count) {
    if (typ[i] == 7) {
        j = i;
        while (j <= count) { if (typ[j] != 7) break; j++; }
        previous = base; following = base;
        if (i > 1) previous = typ[i-1];
        if (j <= count) following = typ[j];
        if (previous == 3 || following == 3) for (k = i; k < j; k++) typ[k] = 3;
        i = j;
    } else i++;
}
// W6/W7.
previous = base;
for (i = 1; i <= count; i++) {
    if (typ[i] == 6 || typ[i] == 7 || typ[i] == 8) typ[i] = 9;
    if (typ[i] == 0 || typ[i] == 1) previous = typ[i];
    else if (typ[i] == 3 && previous == 0) typ[i] = 0;
}
// N0: matched ASCII brackets, including nested pairs. '<>' are mirrored
// under L4 but not treated as UAX9 paired brackets.
var stack, top, pairs, paircount, opens, closes, ch, match, op, closeat;
opens = "([{"; closes = ")]}"; top = 0; paircount = 0;
for (i = 1; i <= count; i++) {
    ch = string_char_at(text, i);
    if (string_pos(ch, opens) > 0) { stack[top] = i; top++; }
    else if (string_pos(ch, closes) > 0) {
        match = string_char_at(opens, string_pos(ch, closes));
        for (j = top - 1; j >= 0; j--) {
            if (string_char_at(text, stack[j]) == match) {
                pairs[paircount,0] = stack[j]; pairs[paircount,1] = i;
                paircount++; top = j; break;
            }
        }
    }
}
// Process pairs in opening order, as required by N0.
for (i = 0; i < paircount; i++) for (j = i + 1; j < paircount; j++) {
    if (pairs[j,0] < pairs[i,0]) {
        t = pairs[i,0]; pairs[i,0] = pairs[j,0]; pairs[j,0] = t;
        t = pairs[i,1]; pairs[i,1] = pairs[j,1]; pairs[j,1] = t;
    }
}
var hasbase, hasother, resolved;
for (k = 0; k < paircount; k++) {
    op = pairs[k,0]; closeat = pairs[k,1]; hasbase = 0; hasother = 0;
    for (i = op + 1; i < closeat; i++) {
        t = typ[i]; if (t == 3 || t == 4) t = 1;
        if (t == base) hasbase = 1;
        else if (t == 1-base) hasother = 1;
    }
    resolved = -1;
    if (hasbase) resolved = base;
    else if (hasother) {
        previous = base;
        for (i = op - 1; i >= 1; i--) {
            t = typ[i]; if (t == 3 || t == 4) t = 1;
            if (t == 0 || t == 1) { previous = t; break; }
        }
        resolved = previous;
    }
    if (resolved >= 0) { typ[op] = resolved; typ[closeat] = resolved; }
}
// N1/N2: neutrals follow equal neighbors, otherwise paragraph direction.
i = 1;
while (i <= count) {
    if (typ[i] == 9 || typ[i] == 10) {
        j = i;
        while (j <= count) { if (typ[j] != 9 && typ[j] != 10) break; j++; }
        previous = base; following = base;
        if (i > 1) previous = typ[i-1];
        if (j <= count) following = typ[j];
        if (previous == 3 || previous == 4) previous = 1;
        if (following == 3 || following == 4) following = 1;
        resolved = base;
        if (previous == following) resolved = previous;
        for (k = i; k < j; k++) typ[k] = resolved;
        i = j;
    } else i++;
}
// I1/I2, L1 trailing spaces, L2 descending-level reversal.
var maxlevel = base, level, lo, hi;
for (i = 1; i <= count; i++) {
    level = base;
    if (base == 0) { if (typ[i] == 1) level = 1; else if (typ[i] == 3 || typ[i] == 4) level = 2; }
    else if (typ[i] == 0 || typ[i] == 3 || typ[i] == 4) level = 2;
    lev[i] = level; maxlevel = max(maxlevel, level);
}
for (i = count; i >= 1; i--) { if (original[i] != 10) break; lev[i] = base; }
for (level = maxlevel; level >= 1; level--) {
    i = 1;
    while (i <= count) {
        if (lev[order[i]] >= level) {
            j = i; while (j <= count) { if (lev[order[j]] < level) break; j++; }
            lo = i; hi = j-1;
            while (lo < hi) { t = order[lo]; order[lo] = order[hi]; order[hi] = t; lo++; hi--; }
            i = j;
        } else i++;
    }
}
for (i = 1; i <= count; i++) {
    j = order[i]; ch = string_char_at(text, j);
    if (lev[j] mod 2 == 1) ch = he_mirror(ch);
    ds_grid_set(result, j, 0, i-1);
    ds_grid_set(result, j, 1, ch);
    ds_grid_set(result, j, 2, lev[j]);
}
return result;
