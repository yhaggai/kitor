"""Merge explicit GameDatabase speaker references without editing translations."""
import argparse
import csv
import hashlib
import io
import json
from pathlib import Path


def merge_speakers(database, rows):
    context = {}
    characters = {}
    changed = 0
    for row in rows:
        path = json.loads(row['path'])
        if len(path) < 5 or path[1] != 'phrases' or path[-2:] != ['phrase', 'text']:
            continue
        scene = database[path[0]]
        phrase = scene['phrases'][path[2]]
        if phrase['phrase']['text'] != row['en']:
            raise ValueError(f'Source text changed at {row["stable_id"]}')
        card = phrase['character_card']
        character = database[card]
        speaker = character['character_name']['text']
        if not speaker:raise ValueError(f'Empty character name: {card}')
        changed += row['speaker'] != speaker
        row['speaker'] = speaker
        characters[card] = speaker
        context[row['stable_id']] = {
            'character_card':card, 'emotion':phrase.get('emotion', ''),
            'event':phrase.get('event', ''),
        }
    return {'characters':characters, 'phrases':context, 'speaker_labels_changed':changed}


def main():
    import UnityPy
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('bundle', type=Path)
    parser.add_argument('--catalog', type=Path, default=Path(__file__).with_name('catalog.csv'))
    parser.add_argument('--context', type=Path, default=Path(__file__).with_name('speaker-context.json'))
    args = parser.parse_args()
    blob = args.bundle.read_bytes()
    env = UnityPy.load(blob)
    assets = [o for o in env.objects if o.type.name == 'TextAsset' and o.peek_name() == 'GameDatabase']
    if len(assets) != 1:raise ValueError('Expected exactly one GameDatabase')
    database = json.loads(assets[0].read().m_Script)
    with args.catalog.open(encoding='utf-8-sig', newline='') as stream:
        reader = csv.DictReader(stream, strict=True)
        fields, rows = reader.fieldnames, list(reader)
    result = merge_speakers(database, rows)
    result['source_bundle_sha256'] = hashlib.sha256(blob).hexdigest()
    output = io.StringIO(newline='')
    writer = csv.DictWriter(output, fieldnames=fields, lineterminator='\n')
    writer.writeheader();writer.writerows(rows)
    args.catalog.write_text(output.getvalue(), encoding='utf-8-sig', newline='')
    args.context.write_text(json.dumps(result, ensure_ascii=False, indent=1)+'\n', encoding='utf-8')
    print(f'Merged {len(result["phrases"])} explicit speakers; corrected {result["speaker_labels_changed"]} labels')


if __name__ == '__main__':
    main()
