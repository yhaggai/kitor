// Ordinary draw-text strings only. Preserve numbers and single glyphs verbatim.
var he_s = argument0;
if (!is_string(he_s)) return he_s;
if (string_length(he_s) < 2) return he_s;
var he_has = false;
for (var he_i = 1; he_i <= string_length(he_s); he_i += 1) {
    var he_cp = ord(string_char_at(he_s, he_i));
    if ((he_cp >= 1488 && he_cp <= 1514) || (he_cp >= 61440 && he_cp <= 63487)) { he_has = true; break; }
}
if (!he_has) return he_s;
return he_visual(he_s);
