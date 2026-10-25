"""Read and CRC-check inline-data VPK v1 archives, without executing contents."""
from pathlib import Path
import struct
import zlib


def files(path):
    data = Path(path).read_bytes()
    magic, version, tree_size = struct.unpack_from('<III', data)
    if magic != 0x55AA1234 or version != 1:
        raise ValueError('Expected VPK v1')
    position = 12

    def string():
        nonlocal position
        end = data.index(0, position)
        value = data[position:end].decode('utf-8')
        position = end + 1
        return value

    while True:
        extension = string()
        if not extension:
            break
        while True:
            folder = string()
            if not folder:
                break
            while True:
                name = string()
                if not name:
                    break
                crc, preload_size, archive, offset, length, terminator = struct.unpack_from('<IHHIIH', data, position)
                position += 18
                if terminator != 0xFFFF or (archive != 0x7FFF and length):
                    raise ValueError('Unsupported external VPK data or corrupt entry')
                preload = data[position:position + preload_size]
                position += preload_size
                content = preload + data[12 + tree_size + offset:12 + tree_size + offset + length]
                if len(content) != preload_size + length or zlib.crc32(content) != crc:
                    raise ValueError('VPK entry length/CRC mismatch')
                yield ('' if folder == ' ' else folder + '/') + name + '.' + extension, content
    if position != 12 + tree_size:
        raise ValueError('VPK directory size mismatch')


if __name__ == '__main__':
    import sys
    for name, content in files(sys.argv[1]):
        print(name, len(content))
