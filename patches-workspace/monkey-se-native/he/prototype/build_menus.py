"""Translate the native French UI slot, preserving IDs and other languages."""
import argparse
import json
from pathlib import Path
import struct
from build_candidate import encode


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--input', type=Path, required=True)
    a = p.parse_args()
    mapping = dict(line.split('\t', 1) for line in Path(__file__).with_name('menu-he.tsv').read_text(encoding='utf-8').splitlines() if line)
    rows = json.loads((a.input / 'menu-source.json').read_text(encoding='utf-8'))
    game = json.loads((a.input / 'mapping.json').read_text(encoding='utf-8'))['game']
    name = 'localization/uiText.info' if game == 1 else 'localization/en.uitext.info'
    original = (a.input / 'original' / name).read_bytes()
    data = bytearray(original)
    translated, missing = [], []
    for i, row in enumerate(rows):
        text = mapping.get(row['id'])
        if text is not None:
            value = encode(text.replace('\\n', '\n'))
            translated.append(row['id'])
        else:
            value = row['en'].encode('cp1252')
            if row['id'].startswith(('MENU_', 'OVERLAY_')):
                missing.append(row['id'])
        if game == 1:
            assert len(value) < 256, row['id']
            start = i * 1536 + 512
            data[start:start + 256] = value + bytes(256 - len(value))
            assert data[i * 1536:i * 1536 + 512] == original[i * 1536:i * 1536 + 512]
        else:
            ptr = 4 + i * 8 + 4
            struct.pack_into('<I', data, ptr, len(data) - ptr)
            data.extend(value + b'\0')
    target = a.input / 'payload' / (name if game == 1 else 'localization/fr.uitext.info')
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(data)
    report = dict(translated=len(translated), missing_menu_ids=missing,
                  source='menu-he.tsv: new Hebrew UI translation; dialogue uses AdventureBrew')
    (a.input / 'menu-report.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    print(json.dumps(report))


if __name__ == '__main__':
    main()
