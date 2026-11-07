"""Apply reviewed name vowels without changing letters, controls, or layout tags."""
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
MARK = re.compile(r'[\u0591-\u05bd\u05bf-\u05c2\u05c4\u05c5\u05c7]')
TAG = re.compile(r'<[^>]*>')
CLUSTER = re.compile(r'[א-ת][\u0591-\u05bd\u05bf-\u05c2\u05c4\u05c5\u05c7]*|[^א-ת]')
NAMES = json.loads((ROOT / 'names.json').read_text())['names']


def normalize_names(text, english=None):
    # Project through rich text (including one <pos> tag per manuscript letter),
    # then replace only the original Hebrew clusters at their original offsets.
    spans = list(TAG.finditer(text))
    hidden = {i for m in spans for i in range(m.start(), m.end())}
    indices = [i for i, c in enumerate(text) if i not in hidden and not MARK.fullmatch(c)]
    plain = ''.join(text[i] for i in indices)
    edits = {}
    for row in NAMES:
        # These spellings also occur as ordinary Hebrew words. Require source evidence.
        if row['hebrew'] in {'דן', 'לס', 'איתן', 'טים', 'קלוד'}:
            if english is None or not re.search(r'\b' + re.escape(row['english']) + r'\b', english, re.I):
                continue
        pattern = r'(?<![א-ת])[ובכלמש]{0,2}(' + re.escape(row['hebrew']) + r')(?![א-ת׳])'
        for match in re.finditer(pattern, plain):
            for offset, cluster in enumerate(CLUSTER.findall(row['pointed'])):
                if not re.match('[א-ת]', cluster):
                    continue
                start = indices[match.start(1) + offset]
                end = start + 1
                while end < len(text) and MARK.fullmatch(text[end]):
                    end += 1
                edits[start] = (end, cluster)
    for start, (end, cluster) in sorted(edits.items(), reverse=True):
        text = text[:start] + cluster + text[end:]
    return text


def source_english():
    sources = {}
    for filename in ('catalog.json', 'ui-catalog.json'):
        for row in json.loads((ROOT.parent / filename).read_text()):
            for he, en in (('he', 'en'), ('alternate_he', 'alternate_en')):
                if row.get(he):
                    sources.setdefault(row[he], []).append(row.get(en, ''))
    return {key: '\n'.join(value) for key, value in sources.items()}


def normalize_mapping(mapping):
    sources = source_english()
    return {key: normalize_names(value, sources.get(key)) for key, value in mapping.items()}


def main():
    report = {}
    for filename in ('pointed.json', 'manuscript.json'):
        path = ROOT / filename
        old = json.loads(path.read_text())
        new = normalize_mapping(old)
        assert old.keys() == new.keys()
        sources = source_english()
        for key in old:
            assert MARK.sub('', old[key]) == MARK.sub('', new[key]), key
            assert normalize_names(new[key], sources.get(key)) == new[key], key
        path.write_text(json.dumps(new, ensure_ascii=False, indent=2) + '\n')
        report[filename] = sum(old[k] != new[k] for k in old)
    print(json.dumps(report))


if __name__ == '__main__':
    main()
