// Wrap logical input before visual reordering. Used only for draw_text_ext.
var he_s = argument0;
var he_width = argument1;
if (!is_string(he_s)) return he_s;
var he_has = false;
for (var he_i = 1; he_i <= string_length(he_s); he_i += 1) {
    var he_cp = ord(string_char_at(he_s, he_i));
    if (he_cp >= 1488 && he_cp <= 1514) { he_has = true; break; }
}
if (!he_has) return he_s;
if (he_width <= 0) return he_render_visual(he_s);
var he_out = "";
var he_line = "";
var he_space = 0;
for (var he_i = 1; he_i <= string_length(he_s); he_i += 1) {
    var he_ch = string_char_at(he_s, he_i);
    if (he_ch == "#" || ord(he_ch) == 10 || ord(he_ch) == 13) {
        he_out += he_render_visual(he_line) + "#";
        he_line = "";
        he_space = 0;
        if (ord(he_ch) == 13 && ord(string_char_at(he_s, he_i + 1)) == 10) he_i += 1;
    } else {
        if (string_length(he_line) > 0 && string_width(he_line + he_ch) > he_width) {
            if (he_space > 0) {
                he_out += he_render_visual(string_copy(he_line, 1, he_space - 1)) + "#";
                he_line = string_delete(he_line, 1, he_space);
            } else {
                he_out += he_render_visual(he_line) + "#";
                he_line = "";
            }
            he_space = 0;
        }
        if (he_ch != " " || string_length(he_line) > 0) {
            he_line += he_ch;
            if (he_ch == " ") he_space = string_length(he_line);
        }
    }
}
return he_out + he_render_visual(he_line);
