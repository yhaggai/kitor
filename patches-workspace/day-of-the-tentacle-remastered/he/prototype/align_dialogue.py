"""Align split subtitles using original English text as independent evidence."""
import argparse
import collections
import json
from pathlib import Path
import re

from map_dialogue import digest, extract_speech, rebuild, speech_rows


def normalized(text):
    # The classic font represents ellipses with a caret. Ignore whitespace only;
    # no fuzzy matching, word deletion, or guessed translation is allowed.
    text = text.encode('utf-8', errors='surrogateescape').decode('cp1252') if any(0xdc80 <= ord(c) <= 0xdcff for c in text) else text
    text = text.replace('^', '...').replace('\xfe\x08', ' ')
    text = text.replace('\x0f', '™').replace('macram‚', 'macramé')
    text = re.sub(r'\s+(?=\.\d)', '', text)
    text = re.sub(r'[`"\']', '', text)
    text = re.sub(r'\s*\.\.\.\s*', '...', text)
    return ' '.join(text.split())


def original_speech(data):
    decoded = bytes(value ^ 0x69 for value in data)
    marker = re.compile(b'(?:\xff\x0a..){4}', re.DOTALL)
    result = collections.defaultdict(set)
    for match in marker.finditer(decoded):
        control = match[0]
        offset = int.from_bytes(control[2:4] + control[6:8], 'little')
        end = decoded.find(b'\0', match.end())
        if end != -1:
            result[offset].add(decoded[match.end():end])
    return result


def align(rows, mapping, originals):
    proposals = collections.defaultdict(list)
    groups, unresolved = [], []
    for entry in mapping:
        if 'classic_multisegment_speech_requires_alignment' not in entry['review']:
            continue
        record = {'id': entry['id'], 'sources': entry['sources']}
        original = originals.get(entry['sources'][0]['voice_offset'], set())
        if len(original) != 1:
            unresolved.append({**record, 'reason': 'original_missing_or_ambiguous'})
            continue
        english = next(iter(original)).replace(b'\xff\x01', b' ').split(b'\xff\x03')
        hebrew = entry['sources'][0]['hebrew_raw'].split(r'\xFF\x03')
        hebrew = [s.replace(r'\xFE\x08', '') for s in hebrew]
        record.update(english=[s.decode('cp1252') for s in english], hebrew=hebrew)
        if len(english) == 1 and len(hebrew) > 1:
            # A translator-inserted page break can belong to a single HD subtitle.
            # Still require the complete original English to match that subtitle.
            hebrew = ['\n'.join(hebrew)]
        if len(english) != len(hebrew):
            unresolved.append({**record, 'reason': 'segment_count_differs'})
            continue
        if any(b'\xff' in text for text in english) or any('\\x' in s.replace(r'\xFF\x01', '') for s in hebrew):
            unresolved.append({**record, 'reason': 'unsupported_control_code'})
            continue
        # Restrict to the same actor/room/utterance ID namespace, then require
        # exactly one ordered assignment. Repeated dialogue elsewhere cannot match.
        scope = entry['id'].rsplit('_', 1)[0] + '_'
        targets = [(i, row) for i, row in enumerate(rows) if row['id'].rsplit('_', 1)[0] + '_' == scope]
        # A split utterance occupies consecutive subtitle records. This avoids
        # matching a repeated laugh or click from a later utterance in the scene.
        paths = []
        target_indices = {i for i, _ in targets}
        for start, row in targets:
            if row['id'] != entry['id']:
                continue
            indices = list(range(start, start + len(english)))
            if all(i in target_indices and normalized(rows[i]['text']) == normalized(text)
                   for i, text in zip(indices, record['english'])):
                paths.append(indices)
        if len(paths) != 1:
            unresolved.append({**record, 'reason': 'english_sequence_not_unique', 'possible_paths': len(paths)})
            continue
        indices = paths[0]
        group = {**record, 'targets': [{'index': i, 'id': rows[i]['id'], 'english': rows[i]['text'],
                                      'hebrew': h.replace(r'\xFF\x01', '\n').replace('^', '...')}
                                     for i, h in zip(indices, hebrew)]}
        groups.append(group)
        for target in group['targets']:
            proposals[target['index']].append({**target, 'source_group': entry['id']})
    return groups, unresolved, proposals


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', required=True, type=Path)
    parser.add_argument('--generated', type=Path, default=Path(__file__).parent / 'generated')
    args = parser.parse_args()
    def read(name):
        return json.loads((args.generated / (name + '.json')).read_text())
    data = extract_speech(args.archive)
    rows = speech_rows(data)
    assert digest(data) == read('report')['speech_sha256']
    initial = (args.generated / 'en.speech.info.candidate').read_bytes()
    assert digest(initial) == read('report')['candidate_sha256']
    initial_rows = speech_rows(initial)
    candidate = [dict(row) for row in initial_rows]
    original_data = extract_speech(args.archive, 'classic/en/tentacle.001')
    groups, unresolved, proposals = align(rows, read('dialogue-map'), original_speech(original_data))
    collisions = []
    accepted = set()
    blocked_groups = set()
    for index, options in proposals.items():
        texts = {x['hebrew'] for x in options}
        if len(texts) != 1 or (candidate[index]['text'] != rows[index]['text']
                              and normalized(candidate[index]['text']) not in {normalized(t) for t in texts}):
            collisions.append({'index': index, 'options': options})
            blocked_groups.update(x['source_group'] for x in options)
    for index, options in proposals.items():
        if any(x['source_group'] in blocked_groups for x in options):
            continue
        candidate[index]['text'] = options[0]['hebrew']
        accepted.add(index)
    overrides = json.loads(Path(__file__).with_name('alignment-overrides.json').read_text())
    unresolved_ids = {x['id'] for x in unresolved} | blocked_groups
    assert {x['source_group'] for x in overrides} == unresolved_ids
    override_targets = set()
    for override in overrides:
        indices = []
        for target in override['targets']:
            matches = [i for i, row in enumerate(rows) if row['id'] == target['id'] and row['text'] == target['english']]
            assert len(matches) == 1, f'Override source changed: {target["id"]}'
            index = matches[0]
            assert index not in override_targets
            override_targets.add(index)
            indices.append(index)
            candidate[index]['text'] = target['hebrew']
            accepted.add(index)
        assert indices == list(range(indices[0], indices[0] + len(indices)))
    built = rebuild(data, candidate)
    assert speech_rows(built) == candidate
    for i, row in enumerate(rows):
        assert row['id'] == candidate[i]['id']
        base = 8 + i * 40
        assert data[base:base+24] == built[base:base+24]
        if i not in accepted:
            assert candidate[i] == initial_rows[i]
    remaining = [{'index': i, **row} for i, row in enumerate(rows) if row == candidate[i]]
    report = {'status': 'offline alignment; engine rendering untested',
              'original_classic_sha256': digest(original_data), 'speech_sha256': digest(data),
              'candidate_sha256': digest(built), 'split_groups_aligned': len(groups) - len(blocked_groups),
              'split_groups_unresolved': len(unresolved) + len(blocked_groups),
              'explicitly_resolved_groups': len(overrides),
              'split_groups_remaining': len(unresolved_ids) - len(overrides),
              'aligned_subtitle_entries': len(accepted), 'collisions': len(collisions),
              'total_replacements': sum(a != b for a, b in zip(rows, candidate)),
              'remaining_entries': len(remaining),
              'remaining_commentary': sum(x['id'].startswith('CMM_') for x in remaining),
              'unresolved_reasons': dict(collections.Counter(x['reason'] for x in unresolved)),
              'checks': ['English sequence exact after documented typography normalization',
                         'equal segment counts or full-text-verified single-subtitle join', 'unique consecutive match within utterance',
                         'collision rejection', 'binary round-trip; IDs and metadata preserved',
                         'all non-aligned entries unchanged from initial candidate']}
    for name, value in [('alignment-report', report), ('aligned-groups', groups),
                        ('alignment-unresolved', unresolved), ('alignment-collisions', collisions),
                        ('remaining-after-alignment', remaining)]:
        (args.generated / (name + '.json')).write_text(json.dumps(value, ensure_ascii=True, indent=2) + '\n')
    (args.generated / 'en.speech.info.aligned-candidate').write_bytes(built)
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
