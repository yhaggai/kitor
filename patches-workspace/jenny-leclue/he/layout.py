"""Logical dialogue for native RTL; visual layout only for static/legacy labels."""
import re
from bidi import algorithm

TOKENS = re.compile(r'(<[^>\n]+>|@(?:VS|VM|VL|S|P)[\d.]+|\[[^\]\n]+\]|\{[^}\n]+\})', re.I)
STYLE = re.compile(r'<(/?)(size|color|i|b|nobr|rotate)(?:[ =>]|$)', re.I)


def visual_line(chars):
    if not chars:
        return ''
    storage = algorithm.get_empty_storage()
    storage.update(base_level=1, base_dir='R')
    algorithm.get_embedding_levels(''.join(ch for ch, _, _ in chars), storage)
    for index, ch in enumerate(storage['chars']):
        ch['source'] = index
    for step in (algorithm.explicit_embed_and_overrides, algorithm.resolve_weak_types,
                 algorithm.resolve_neutral_types, algorithm.resolve_implicit_levels,
                 algorithm.reorder_resolved_levels, algorithm.apply_mirroring):
        step(storage, False)
    output = []
    active = ()
    for glyph in storage['chars']:
        _, styles, literal = chars[glyph['source']]
        common = 0
        while common < min(len(styles), len(active)) and active[common] == styles[common]:
            common += 1
        output.extend('</'+name+'>' for name, _ in reversed(active[common:]))
        output.extend(tag for _, tag in styles[common:])
        output.append(literal or glyph['ch'])
        active = styles
    output.extend('</'+name+'>' for name, _ in reversed(active))
    return ''.join(output)


def display(text, dialogue, pointed_glyphs=None):
    if not re.search('[א-ת]', text):
        return text
    random_line = re.fullmatch(r'\[lua\(RandomLine\("([^"]*)"\)\)\]', text)
    if random_line:
        options = [display(option, dialogue, pointed_glyphs) for option in random_line[1].split('|')]
        return '[lua(RandomLine("'+'|'.join(options)+'"))]'
    prefix = re.match(r'^(?:@(?:VS|VM|VL|S|P)[\d.]+)+', text, re.I)
    if prefix:
        return prefix[0] + display(text[prefix.end():], dialogue, pointed_glyphs)
    styles = []
    lines = []
    for line in text.split('\n'):
        chars = []
        for token in TOKENS.split(line):
            style = STYLE.match(token)
            if style:
                name = style[2].lower()
                if style[1]:
                    for idx in reversed(range(len(styles))):
                        if styles[idx][0] == name:
                            styles.pop(idx)
                            break
                else:
                    styles.append((name, token))
            elif TOKENS.fullmatch(token):
                chars.append(('\ufffc', tuple(styles), token))
            elif pointed_glyphs is not None:
                for cluster in re.findall(r"[א-ת][\u05b0-\u05bd\u05bf\u05c1\u05c2\u05c7]*|.", token):
                    chars.append((cluster[0], tuple(styles), pointed_glyphs[cluster] if len(cluster) > 1 else ""))
            else:
                chars.extend((ch, tuple(styles), '') for ch in token if not '\u0591' <= ch <= '\u05c7' or ch in '־׃')
        remaining = chars
        while len(remaining) > 44:
            cut = next((i for i in range(44, 0, -1) if remaining[i][0]==' '), None)
            if cut is None:
                break
            lines.append(visual_line(remaining[:cut]))
            remaining = remaining[cut+1:]
        lines.append(visual_line(remaining))
    result = '\n'.join(lines)
    return '<align=right>'+result+'</align>' if dialogue else result


def dialogue_display(text, pointed_glyphs=None):
    """Preserve logical order, authored line breaks, markup and timing positions.

    Native TMP layout owns wrapping and alignment. A pointed atlas cluster is
    one reveal unit; no reversal or fixed-width line splitting belongs here.
    """
    if pointed_glyphs is not None:
        return re.sub(r'[א-ת][\u05b0-\u05bd\u05bf\u05c1\u05c2\u05c7]+',
                      lambda match: pointed_glyphs[match[0]], text)
    return re.sub(r'[\u0591-\u05bd\u05bf-\u05c2\u05c4-\u05c7]', '', text)
