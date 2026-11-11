"""Select compact, source-backed glossary context; canonical glossary stays intact."""
import argparse
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
CORE = {'Revachol', 'Martinaise', 'Whirling-in-Rags', 'The Pale', 'Pale', 'RCM',
        'Wild Pines', 'Moralintern', 'Kim Kitsuragi', 'You'}

def names(entry):
    return [entry['english'], *[a for a in entry.get('aliases', []) if isinstance(a, str)]]

def matches(name, text):
    return bool(name and re.search(r'(?<!\w)' + re.escape(name) + r'(?!\w)', text, re.I))

def select(glossary, rows):
    text = '\n'.join(r['english'] for r in rows)
    ids = {str(c[k]) for r in rows for c in r.get('context', [])
           for k in ('speaker_id', 'addressee_id') if c.get(k) is not None}
    result = []
    for entry in glossary['entries']:
        actor = str(entry.get('speaker_id', '')) in ids
        relevant = actor or any(matches(n, text) for n in names(entry))
        if not relevant and entry['english'] not in CORE:
            continue
        item = {'english': entry['english'], 'hebrew': entry['hebrew']}
        # Alias/form mappings can change meaning; preserve them for selected entries.
        for key in ('aliases', 'localized_aliases', 'localized_forms', 'accepted_hebrew_forms'):
            if entry.get(key):
                item[key] = entry[key]
        if relevant:
            for key in ('gender', 'pronouns', 'forms_of_address', 'usage_note', 'notes'):
                if entry.get(key):
                    item[key] = entry[key]
        if actor:
            voice = entry.get('voice') or entry.get('luna_voice_draft', {}).get('note')
            if voice:
                item['voice'] = voice
            scene_notes = entry.get('scene_notes', {})
            scenes = {str(c['conversation_id']) for r in rows for c in r.get('context', []) if 'conversation_id' in c}
            selected = {k: v for k, v in scene_notes.items() if k in scenes}
            if selected:
                item['scene_notes'] = selected
        result.append(item)
    return result

def audit(glossary, rows):
    """Advisory only: Hebrew inflection and deliberate wordplay require review."""
    warnings = []
    for row in rows:
        if not row.get('hebrew'):
            continue
        for entry in glossary['entries']:
            if entry['english'] == 'You' or not any(matches(n, row['english']) for n in names(entry)):
                continue
            forms = [entry['hebrew'], *entry.get('accepted_hebrew_forms', []),
                     *entry.get('localized_forms', {}).values(), *entry.get('localized_aliases', {}).values()]
            if not any(f and f in row['hebrew'] for f in forms):
                warnings.append({'key': row['key'], 'term': entry['english'], 'expected_forms': forms,
                                 'status': 'review; inflection, wordplay or context may justify exception'})
    return warnings

def render(glossary, rows):
    selected = select(glossary, rows)
    full = json.dumps(glossary, ensure_ascii=False, separators=(',', ':'))
    compact = json.dumps(selected, ensure_ascii=False, separators=(',', ':'))
    return {'instructions': 'Use these canonical terms. Consult glossary.json by exact name only when context is ambiguous; do not load the full glossary routinely. Preserve reveal timing and contextual grammar. Flag deliberate terminology exceptions for review.',
            'glossary_sha256': hashlib.sha256(full.encode()).hexdigest(),
            'entries': selected,
            'measurement': {'full_glossary_characters': len(full), 'selected_characters': len(compact),
                            'character_reduction_percent': round(100 * (1 - len(compact) / len(full)), 2),
                            'note': 'Character counts, not token counts or verified cache savings.'}}

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('batch', type=Path)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--translations', type=Path, help='Optional translated batch CSV for advisory terminology checks')
    args = parser.parse_args()
    data = render(json.loads((ROOT / 'glossary.json').read_text()), json.loads(args.batch.read_text()))
    if args.translations:
        import csv
        with args.translations.open(newline='') as f:
            data['terminology_review'] = audit(json.loads((ROOT / 'glossary.json').read_text()), list(csv.DictReader(f)))
    text = json.dumps(data, ensure_ascii=False, indent=2) + '\n'
    if args.output:
        args.output.write_text(text)
    else:
        print(text, end='')
