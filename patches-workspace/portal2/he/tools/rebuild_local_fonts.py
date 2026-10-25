"""Rebuild a Portal 2 VPK with four audited local font edits; no game execution.

Usage: python3 rebuild_local_fonts.py ORIGINAL_VPK OUTPUT_VPK
The result is experimental until native Windows rendering has been verified.
"""
import json
from pathlib import Path
import struct
import sys
import zlib
from vpk_reader import files


def rebuild(source, destination):
    entries = dict(files(source))
    font_entry = 'resource/basemodui_scheme.res'
    if entries[font_entry].count(b'"Tahoma"') != 4:
        raise ValueError('Expected exactly four Tahoma references in source scheme')
    entries[font_entry] = entries[font_entry].replace(b'"Tahoma"', b'"Courier New"')
    groups = {}
    body = bytearray()
    for name, content in sorted(entries.items()):
        path = Path(name)
        folder = ' ' if path.parent.as_posix() == '.' else path.parent.as_posix()
        groups.setdefault(path.suffix[1:], {}).setdefault(folder, {})[path.stem] = (zlib.crc32(content), len(body), len(content))
        body.extend(content)
    tree = bytearray()
    for extension, folders in groups.items():
        tree.extend(extension.encode() + b'\0')
        for folder, names in folders.items():
            tree.extend(folder.encode() + b'\0')
            for name, (crc, offset, length) in names.items():
                tree.extend(name.encode() + b'\0')
                tree.extend(struct.pack('<IHHIIH', crc, 0, 0x7FFF, offset, length, 0xFFFF))
            tree.append(0)
        tree.append(0)
    tree.append(0)
    destination.write_bytes(struct.pack('<III', 0x55AA1234, 1, len(tree)) + tree + body)
    if dict(files(destination)) != entries:
        raise ValueError('Rebuilt archive failed entry/CRC verification')
    return {'entries_verified': len(entries), 'edited_entry': font_entry, 'font_replacements': 4, 'all_other_entry_bytes_identical': True}


if __name__ == '__main__':
    print(json.dumps(rebuild(Path(sys.argv[1]), Path(sys.argv[2]))))
