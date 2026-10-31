var ch = argument0;
var left = "([{<";
var right = ")]}>";
var pos = string_pos(ch, left);
if (pos > 0) return string_char_at(right, pos);
pos = string_pos(ch, right);
if (pos > 0) return string_char_at(left, pos);
return ch;
