"""Map classic Hebrew speech to DOTT Remastered; never modifies an installation.

Python 3 standard library only. Archive layout reference: BLooperZ/remonstered
core/lpak.py. Speech pointers were inspected in the owned macOS game resource.
"""
import argparse
import collections
import hashlib
import json
from pathlib import Path
import re
import struct


def digest(data):
    return hashlib.sha256(data).hexdigest()


def extract_speech(archive, member='localization/en.speech.info'):
    with archive.open('rb') as f:
        magic, version, *h = struct.unpack('<4sf8I', f.read(40))
        if magic != b'KAPL' or version != 1.0:
            raise ValueError('Only inspected KAPL 1.0 archives are supported')
        f.seek(h[1])
        entries = list(struct.iter_unpack('<5I', f.read(h[5])))
        f.seek(h[2])
        names = f.read(h[6]).split(b'\0')
        assert len(names) == len(entries) + 1 and names[-1] == b''
        # Names are sequential; the stored name offsets exclude terminators.
        for name, entry in zip(names, entries):
            if name.decode() == member:
                offset, _, packed, size, compressed = entry
                assert not compressed and packed == size
                f.seek(h[3] + offset)
                data = f.read(size)
                assert len(data) == size
                return data
    raise ValueError(f'Resource missing: {member}')


def speech_layout(data):
    count, = struct.unpack_from('<I', data)
    # The native Windows build uses 32-bit counts/pointers; macOS uses 64-bit.
    wide = struct.unpack_from('<I', data, 4)[0] == 0
    return count, 8 if wide else 4, 40 if wide else 32, '<Q' if wide else '<I'


def speech_rows(data):
    count, header, stride, pointer = speech_layout(data)
    assert header + count * stride <= len(data)
    rows = []
    for i in range(count):
        base = header + i * stride
        strings = []
        for field in (24, stride - struct.calcsize(pointer)):
            ptr = base + field
            start = ptr + struct.unpack_from(pointer, data, ptr)[0]
            assert header + count * stride <= start < len(data)
            strings.append(data[start:data.index(0, start)].decode('utf-8', errors='surrogateescape'))
        rows.append({'id': strings[0], 'text': strings[1]})
    return rows


def rebuild(data, rows):
    count, header, stride, pointer = speech_layout(data)
    assert len(rows) == count
    original_end = header + count * stride
    for i in range(count):
        for field in (24, stride - struct.calcsize(pointer)):
            ptr = header + i * stride + field
            start = ptr + struct.unpack_from(pointer, data, ptr)[0]
            original_end = max(original_end, data.index(0, start) + 1)
    result = bytearray(data[:header + count * stride])
    for i, row in enumerate(rows):
        for field, key in ((24, 'id'), (stride - struct.calcsize(pointer), 'text')):
            ptr = header + i * stride + field
            struct.pack_into(pointer, result, ptr, len(result) - ptr)
            result.extend(row[key].encode('utf-8', errors='surrogateescape') + b'\0')
    # Windows ships unused trailing strings; retain them without interpreting them.
    result.extend(data[original_end:])
    return bytes(result)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, required=True)
    parser.add_argument('--scummworks', type=Path, required=True)
    parser.add_argument('--remonstered', type=Path, required=True)
    parser.add_argument('--output', type=Path, default=Path(__file__).parent / 'generated')
    args = parser.parse_args()
    data = extract_speech(args.archive)
    rows = speech_rows(data)
    assert rebuild(data, rows) == data, 'Unmodified resource must round-trip byte-for-byte'
    table = (args.remonstered / 'dott/monster.tbl').read_bytes()
    ids = {int(line[:8], 16): line[8:].decode() for line in table.splitlines()}
    source = (args.scummworks / 'tentacle/strings.txt').read_bytes()
    by_id = collections.defaultdict(list)
    unvoiced = []
    unknown_offsets = []
    prefix = re.compile(r'^(?:\\xFF\\x0A\\x[0-9A-Fa-f]{2}\\x[0-9A-Fa-f]{2}){4}')
    for number, raw in enumerate(source.splitlines(), 1):
        line = raw.decode('cp1255')
        if not re.search('[\u05d0-\u05ea]', line):
            continue
        match = prefix.match(line)
        if not match:
            unvoiced.append({'line': number, 'hebrew_raw': line})
            continue
        control = bytes(int(x, 16) for x in re.findall(r'\\x([0-9A-Fa-f]{2})', match[0]))
        offset = int.from_bytes(control[2:4] + control[6:8], 'little')
        item = {'line': number, 'hebrew_raw': line[match.end():], 'voice_offset': offset}
        if offset not in ids:
            unknown_offsets.append(item)
        else:
            by_id[ids[offset]].append(item)
    mapping, conflicts, unmapped = [], [], []
    candidate = [dict(row) for row in rows]
    for i, row in enumerate(rows):
        options = by_id.get(row['id'], [])
        unique = {x['hebrew_raw'] for x in options}
        if len(unique) > 1:
            conflicts.append({**row, 'options': options})
        elif unique:
            raw = next(iter(unique))
            # SCUMM line/page boundaries become newlines; leave unknown codes
            # and legacy punctuation untouched for explicit review.
            text = raw.replace(r'\xFF\x01', '\n').replace(r'\xFF\x03', '\n')
            review = []
            if r'\xFF\x03' in raw:
                review.append('classic_multisegment_speech_requires_alignment')
            if '\\x' in text:
                review.append('unconverted_control_code')
            if '^' in text:
                review.append('legacy_caret_punctuation')
            entry = {'id': row['id'], 'english': row['text'], 'hebrew': text,
                     'sources': options, 'review': review}
            mapping.append(entry)
            if not any(flag in review for flag in ('unconverted_control_code', 'classic_multisegment_speech_requires_alignment')):
                candidate[i]['text'] = text
        else:
            unmapped.append(row)
    built = rebuild(data, candidate)
    assert speech_rows(built) == candidate
    # All non-pointer metadata, IDs, and all untranslated entries stay intact.
    for i in range(len(rows)):
        _, header, stride, _ = speech_layout(data)
        base = header + i * stride
        assert built[base:base+24] == data[base:base+24]
    report = {
        'status': 'offline dialogue candidate; not installed or rendering-tested',
        'speech_sha256': digest(data), 'hebrew_source_sha256': digest(source),
        'voice_table_sha256': digest(table), 'candidate_sha256': digest(built),
        'speech_entries': len(rows), 'mapped_entries': len(mapping),
        'candidate_replacements': sum(a['text'] != b['text'] for a, b in zip(rows, candidate)),
        'conflicting_entries': len(conflicts), 'unmapped_entries': len(unmapped),
        'unmapped_commentary': sum(r['id'].startswith('CMM_') for r in unmapped),
        'unvoiced_hebrew_lines': len(unvoiced), 'unknown_voice_offsets': len(unknown_offsets),
        'review_flags': dict(collections.Counter(flag for r in mapping for flag in r['review'])),
        'checks': ['original resource byte-identical round-trip',
                   'candidate UTF-8 text and IDs round-trip', 'non-pointer metadata preserved'],
    }
    args.output.mkdir(parents=True, exist_ok=True)
    for name, obj in [('report', report), ('dialogue-map', mapping), ('conflicts', conflicts),
                      ('unmapped', unmapped), ('unvoiced-hebrew', unvoiced),
                      ('unknown-offsets', unknown_offsets)]:
        (args.output / (name + '.json')).write_text(json.dumps(obj, ensure_ascii=True, indent=2) + '\n')
    (args.output / 'en.speech.info.candidate').write_bytes(built)
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
