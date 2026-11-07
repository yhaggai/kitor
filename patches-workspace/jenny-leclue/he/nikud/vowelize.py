"""Generate the Jenny LeClue logical-order draft locally with nakdimon==0.2.1."""
import json
import re
from pathlib import Path

from nakdimon.predict import predict
from names import normalize_mapping

ROOT = Path(__file__).resolve().parent
BASE = ROOT.parent
LETTER = re.compile('[א-ת]')
CLUSTER = re.compile(r'[א-ת][\u05b0-\u05bd\u05bf\u05c1\u05c2\u05c7]*')
PROTECTED = re.compile(r'<[^>]+>|@(?:VS|VM|VL|S|P)[\d.]+|\{[^}]+\}|\[(?!lua\(RandomLine)[^\]]+\]', re.I)


def point_batch(lines):
    text = '\n'.join(lines)
    masked = PROTECTED.sub(lambda m: ' ' * len(m[0]), text)
    positions = list(CLUSTER.finditer(masked))
    prediction = CLUSTER.findall(predict(masked + '.', maxlen=256))
    if [m[0][0] for m in positions] != [s[0] for s in prediction]:
        raise ValueError('Nakdimon changed the Hebrew letters')
    parts = []
    end = 0
    for match, cluster in zip(positions, prediction):
        parts.extend((text[end:match.start()], match[0] if len(match[0]) > 1 else cluster))
        end = match.end()
    parts.append(text[end:])
    return ''.join(parts)


def main():
    rows = json.loads((BASE / 'catalog.json').read_text())
    menu = json.loads((BASE / 'menu.json').read_text())
    ui = json.loads((BASE / 'ui-catalog.json').read_text())
    strings = dict.fromkeys([r['he'] for r in rows] + list(menu.values()) +
                            [r[key] for r in ui for key in ('he', 'alternate_he')])
    lines = sorted({line for s in strings for line in s.split('\n') if LETTER.search(line)})
    pointed = {}
    for start in range(0, len(lines), 32):
        batch = lines[start:start + 32]
        pointed.update(zip(batch, point_batch(batch).split('\n'), strict=True))
    output = {s: '\n'.join(pointed.get(line, line) for line in s.split('\n')) for s in strings if LETTER.search(s)}
    output = normalize_mapping(output)
    (ROOT / 'pointed.json').write_text(json.dumps(output, ensure_ascii=False, indent=2) + '\n')


if __name__ == '__main__':
    main()
