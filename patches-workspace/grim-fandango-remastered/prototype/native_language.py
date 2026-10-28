"""Hash-guarded seventh-language adaptation for the tested native GOG build.

Recipes copy original game bytes locally; no full executable/archive is shipped.
Lua 3.1 serialization reference: ScummVM engines/grim/lua/lundump.cpp.
"""
import argparse
import hashlib
import json
import struct
from pathlib import Path
import pefile


def sha(data):
    return hashlib.sha256(data).hexdigest()


def operations(data, edits, base=0):
    result, cursor = [], 0
    for start, end, replacement in sorted(edits):
        if not cursor <= start <= end <= len(data):
            raise ValueError('Overlapping or out-of-range edit')
        if start > cursor:
            result.append({'offset': base + cursor, 'length': start - cursor})
        if replacement:
            result.append({'hex': replacement.hex()})
        cursor = end
    if cursor < len(data):
        result.append({'offset': base + cursor, 'length': len(data) - cursor})
    return result


def materialize(data, ops):
    return b''.join(bytes.fromhex(op['hex']) if 'hex' in op else
                    data[op['offset']:op['offset'] + op['length']] for op in ops)


def executable_operations(data):
    if sha(data) != 'b8a3a2e97cdcf8aa8dd345889cf380d44e2ff33cfc69737acf0d284cd37c8a06':
        raise ValueError('Unsupported native executable')
    pe = pefile.PE(data=data)
    base = pe.OPTIONAL_HEADER.ImageBase
    offset = lambda va: pe.get_offset_from_rva(va - base)
    align = lambda value, unit: (value + unit - 1) // unit * unit
    raw = align(len(data), pe.OPTIONAL_HEADER.FileAlignment)
    rva = pe.OPTIONAL_HEADER.SizeOfImage
    array = data[offset(0x5d8f68):offset(0x5d8f68) + 120] + b'grim.he.tab'.ljust(20, b'\0')
    # Writable data only: same six filenames, plus Hebrew. No injected code.
    size = align(len(array) + 260, pe.OPTIONAL_HEADER.FileAlignment)
    section = struct.pack('<8sIIIIIIHHI', b'.hebrew\0', size, rva, size, raw, 0, 0, 0, 0, 0xc0000040)
    header = pe.sections[-1].get_file_offset() + 40
    assert header + 40 <= pe.OPTIONAL_HEADER.SizeOfHeaders
    assert data[header:header + 40] == bytes(40)
    edits = [(header, header + 40, section),
             (pe.FILE_HEADER.get_field_absolute_offset('NumberOfSections'),
              pe.FILE_HEADER.get_field_absolute_offset('NumberOfSections') + 2,
              struct.pack('<H', pe.FILE_HEADER.NumberOfSections + 1)),
             (pe.OPTIONAL_HEADER.get_field_absolute_offset('SizeOfImage'),
              pe.OPTIONAL_HEADER.get_field_absolute_offset('SizeOfImage') + 4,
              struct.pack('<I', align(rva + size, pe.OPTIONAL_HEADER.SectionAlignment))),
             (offset(0x575ae4), offset(0x575ae4) + 4, struct.pack('<f', 7.0)),
             (len(data), len(data), bytes(raw - len(data)) + array.ljust(size, b'\0'))]
    # Existing HIGHLOW relocations still relocate these absolute operands.
    for va in (0x41b6d3, 0x41c3f6, 0x446354):
        assert data[offset(va):offset(va) + 4] == struct.pack('<I', 0x5d8f68)
        edits.append((offset(va), offset(va) + 4, struct.pack('<I', base + rva)))
    # Keep the optional command-line table-name terminator inside the new buffer.
    va = 0x41b6e9
    assert data[offset(va):offset(va) + 4] == struct.pack('<I', 0x5d906b)
    edits.append((offset(va), offset(va) + 4, struct.pack('<I', base + rva + 259)))
    return operations(data, edits)


def menu_edits(data):
    if sha(data) != '46c3e52b18b1fae8656021f23e8e21190154f5a9c851a28f8ea74112976b7901':
        raise ValueError('Unsupported native menu bytecode')
    position, target = 10, None

    def integer(size):
        nonlocal position
        value = int.from_bytes(data[position:position + size], 'big')
        position += size
        return value

    def string():
        nonlocal position
        size = integer(2)
        value = bytes(c ^ 255 for c in data[position:position + size]).rstrip(b'\0')
        position += size
        return value

    def function():
        nonlocal position, target
        integer(2)
        string()
        size_offset = position
        size = integer(4)
        code_offset = position
        position += size
        count_offset = position
        count = integer(2)
        constants = []
        for _ in range(count):
            tag = integer(1)
            constants.append(string() if tag == ord('S') else integer(4) if tag == ord('N') else None)
        constants_end = position
        if b'/sytx917/' in constants:
            assert target is None
            target = (size_offset, code_offset, size, count_offset, count, constants_end)
        for _ in range(integer(2)):
            integer(2)
            string()
        while integer(1) == ord('#'):
            integer(2)
            function()

    function()
    assert position == len(data) and target is not None
    size_offset, code, size, count_offset, count, end = target
    assert (size, count) == (675, 78)
    assert data[code + 405:code + 407] == b'\x03\x05'
    assignment = bytearray(data[code + 447:code + 455])
    assert assignment[3:7] == bytes([3, 5, 8, 53])
    assignment[4], assignment[6] = 6, 78
    # No branch crosses this straight-line section in the hash-pinned function.
    label = b'S' + struct.pack('>H', 7) + bytes(c ^ 255 for c in b'Hebrew\0')
    return [(size_offset, size_offset + 4, struct.pack('>I', size + 8)),
            (code + 406, code + 407, b'\x06'),
            (code + 455, code + 455, bytes(assignment)),
            (count_offset, count_offset + 2, struct.pack('>H', count + 1)),
            (end, end, label)]


def archive_operations(data):
    count, names_size = struct.unpack_from('<II', data, 8)
    names = data[16 * (count + 1):16 * (count + 1) + names_size]
    for index in range(count):
        record = 16 + index * 16
        name, start, size, _ = struct.unpack_from('<IIII', data, record)
        if names[name:].split(b'\0')[0].lower() == b'menu_prefs.lua':
            script = data[start:start + size]
            edits = menu_edits(script)
            updated = materialize(script, operations(script, edits))
            ops = operations(data, [(record + 4, record + 12, struct.pack('<II', len(data), len(updated)))])
            # Reuse original script ranges at the end of the archive; only edits are literal.
            return ops + operations(script, edits, start)
    raise ValueError('menu_prefs.lua missing from game archive')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--game', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    for name, transform in [('GrimFandango.exe', executable_operations), ('DATA000.LAB', archive_operations)]:
        original = (args.game / name).read_bytes()
        ops = transform(original)
        output = materialize(original, ops)
        recipe = {'schema_version': 1, 'input_sha256': sha(original), 'output_size': len(output), 'operations': ops}
        (args.output / (name + '.json')).write_text(json.dumps(recipe, indent=2) + '\n')
        print(name, sha(original), sha(output), len(output))


if __name__ == '__main__':
    main()
