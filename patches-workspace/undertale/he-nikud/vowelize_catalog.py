"""Local draft vowelization: pip install nakdimon==0.2.1 (Python 3.12+).

Never trust model output to preserve game syntax: transfer only its marks onto
the original Hebrew letters after checking exact consonant identity.
"""
import argparse
import hashlib
import importlib.metadata
import json
from pathlib import Path
import re

from nikud import MARKS, strip_points

ROOT = Path(__file__).resolve().parent
CONTROL = re.compile(r'\\\[[^\]]*\]|\\[EMFTSz*>][\s\S]|\\[\s\S]|\^[0-9]')
LETTER = re.compile('[א-ת]')


def vowelize(text, predict):
    if any(c in MARKS for c in text):
        return text  # Preserve previously edited vowelization.
    masked = CONTROL.sub(lambda m: ' ' * len(m[0]), text)
    positions = [m.start() for m in LETTER.finditer(masked)]
    if not positions:
        return text
    # Independent catalog entries need sentence boundaries, not unknown NUL
    # tokens that can cause spurious vowels on the preceding final letter.
    model_text = masked.replace('\0', '.\n').replace('&', '\n').replace('#', '\n')
    model_text = model_text.replace('/', '.').replace('%', ' ')
    predicted = predict(model_text + '.')
    pointed = re.findall(r'[א-ת][\u05b0-\u05bd\u05bf\u05c1\u05c2\u05c7]*', predicted)
    if ''.join(strip_points(c) for c in pointed) != ''.join(text[i] for i in positions):
        raise ValueError('Model changed Hebrew letters; refusing to apply output')
    replacement = dict(zip(positions, pointed))
    result = ''.join(replacement.get(i, c) for i, c in enumerate(text))
    if strip_points(result) != text:
        raise ValueError('Non-nikud text changed')
    return result


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--catalog', type=Path, default=ROOT / 'translation-catalog.json')
    a = p.parse_args()
    from nakdimon.predict import predict as model_predict
    def predict(text):
        return model_predict(text, maxlen=256)
    original = a.catalog.read_bytes()
    catalog = json.loads(original)
    cache = {}; changed = 0
    pending = list(dict.fromkeys(e['hebrew'] for e in catalog['entries']
                   if e['status'] != 'unchanged' and LETTER.search(e['hebrew'])
                   and not any(c in MARKS for c in e['hebrew'])))
    for start in range(0, len(pending), 32):
        batch = pending[start:start+32]
        # Transfer preserves every separator, including newlines inside entries.
        combined = '\0'.join(batch)
        result = vowelize(combined, predict).split('\0')
        if len(result) != len(batch):
            raise ValueError('Batch boundaries changed')
        cache.update(zip(batch, result))
        print(f'Vowelized {min(start+32,len(pending))}/{len(pending)} unique strings', flush=True)
    for i, entry in enumerate(catalog['entries']):
        text = entry['hebrew']
        if entry['status'] == 'unchanged' or not LETTER.search(text):
            continue
        if text not in cache:
            cache[text] = vowelize(text, predict)
        if cache[text] != text:
            entry['hebrew'] = cache[text]
            entry['status'] = 'draft'
            entry['nikud_status'] = 'machine_draft'
            changed += 1
    catalog['nikud'] = {'engine': 'nakdimon', 'version': importlib.metadata.version('nakdimon'),
                        'source_sha256': hashlib.sha256(original).hexdigest(),
                        'review_status': 'machine_draft', 'changed_entries': changed}
    temp = a.catalog.with_suffix('.tmp')
    temp.write_text(json.dumps(catalog, ensure_ascii=False, indent=2) + '\n')
    temp.replace(a.catalog)
    print(f'Vowelized {changed} entries; linguistic review required.')


if __name__ == '__main__':
    main()
