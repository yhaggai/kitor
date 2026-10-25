from pathlib import Path
p=Path(__file__).parent;src=p/'render-decompiled/CodeEntries';out=p/'render-special';out.mkdir(exist_ok=True)
name='gml_Object_obj_asriel_body_Draw_0';s=(src/(name+'.gml')).read_text();old='fullphrase = " " + mon_name + " ";';new='fullphrase = " " + he_render_visual(mon_name) + " ";';assert s.count(old)==1;s=s.replace(old,new);(out/(name+'.gml')).write_text(s)
name='gml_Script_scr_drawtext_icons';s=(src/(name+'.gml')).read_text();prefix=r'''// Hebrew-only path: reorder text and neutral icon tokens as one sentence.
var he_input = scr_replace_buttons_pc(argument[2]);
var he_has = false;
for (var he_i = 1; he_i <= string_length(he_input); he_i += 1) {
    var he_cp = ord(string_char_at(he_input, he_i));
    if (he_cp >= 1488 && he_cp <= 1514) { he_has = true; break; }
}
if (he_has) {
    var he_scale = 1;
    if (argument_count >= 4) he_scale = argument[3];
    var he_encoded = "";
    var he_count = 0;
    var he_icon;
    for (var he_i = 1; he_i <= string_length(he_input); he_i += 1) {
        if (string_copy(he_input, he_i, 2) == "\*") {
            he_icon[he_count] = scr_getbuttonsprite(string_char_at(he_input, he_i + 2), 0);
            he_encoded += chr(57344 + he_count);
            he_count += 1;
            he_i += 2;
        } else he_encoded += string_char_at(he_input, he_i);
    }
    var he_visual_text = he_render_visual(he_encoded);
    var he_y = argument[1];
    var he_start = 1;
    var he_len = string_length(he_visual_text);
    var he_lineheight = string_height("M");
    while (he_start <= he_len) {
        var he_end = he_start;
        while (he_end <= he_len && string_char_at(he_visual_text, he_end) != "#" && ord(string_char_at(he_visual_text, he_end)) != 10 && ord(string_char_at(he_visual_text, he_end)) != 13) he_end += 1;
        var he_run = "";
        var he_x = argument[0];
        he_run = "";
        for (var he_j = he_start; he_j < he_end; he_j += 1) {
            var he_char = string_char_at(he_visual_text, he_j);
            var he_k = ord(he_char) - 57344;
            if (he_k >= 0 && he_k < he_count) {
                if (string_length(he_run) > 0) { draw_text(he_x, he_y, he_run); he_x += string_width(he_run); he_run = ""; }
                if (he_icon[he_k] != -4) {
                    draw_sprite_ext(he_icon[he_k], 0, he_x, he_y, he_scale, he_scale, 0, c_white, 1);
                    he_x += (sprite_get_width(he_icon[he_k]) + 1) * he_scale;
                }
            } else he_run += he_char;
        }
        if (string_length(he_run) > 0) draw_text(he_x, he_y, he_run);
        he_y += he_lineheight;
        he_start = he_end + 1;
        if (ord(string_char_at(he_visual_text, he_end)) == 13 && ord(string_char_at(he_visual_text, he_start)) == 10) he_start += 1;
    }
    return 0;
}
''';(out/(name+'.gml')).write_text(prefix+s)

name='gml_Script_scr_drawtext_icons_multiline';s=(src/(name+'.gml')).read_text();prefix=r'''// Hebrew path shares the whole-sentence icon reorderer, including line breaks.
var he_input = string(argument[2]);
var he_has = false;
for (var he_i = 1; he_i <= string_length(he_input); he_i += 1) {
    var he_cp = ord(string_char_at(he_input, he_i));
    if (he_cp >= 1488 && he_cp <= 1514) { he_has = true; break; }
}
if (he_has) {
    if (argument_count >= 4) scr_drawtext_icons(argument[0], argument[1], argument[2], argument[3]);
    else scr_drawtext_icons(argument[0], argument[1], argument[2]);
    return 0;
}
''';(out/(name+'.gml')).write_text(prefix+s)
