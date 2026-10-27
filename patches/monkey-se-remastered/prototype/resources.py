"""Read owned KAPL resources and patch fixed-size SoMI SE speech records."""
from pathlib import Path
import struct

STRIDE = 0x530

class Archive:
    def __init__(self, path):
        self.path = Path(path)
        with self.path.open('rb') as f:
            header = f.read(40)
            if len(header) != 40:
                raise ValueError('Truncated archive')
            magic, version, _, entries, names, self.start, _, size, names_size, data_size = struct.unpack('<4sf8I', header)
            if magic != b'KAPL' or version != 1.0 or size % 20 or self.start + data_size != self.path.stat().st_size:
                raise ValueError('Unsupported archive layout')
            f.seek(entries)
            records = list(struct.iter_unpack('<5I', f.read(size)))
            f.seek(names)
            names_list = f.read(names_size).split(b'\0')
            if len(names_list) != len(records)+1 or names_list[-1]:
                raise ValueError('Archive name count mismatch')
            self.members = {}
            for raw, record in zip(names_list, records):
                name = raw.decode('ascii')
                if name in self.members or record[0]+record[2] > data_size:
                    raise ValueError('Invalid archive entry')
                self.members[name] = record

    def read(self, name):
        offset, _, packed, size, compressed = self.members[name]
        if compressed or packed != size:
            raise ValueError('Compressed resource unsupported')
        with self.path.open('rb') as f:
            f.seek(self.start+offset)
            result = f.read(size)
        if len(result) != size:
            raise ValueError('Truncated resource')
        return result


def speech_rows(data):
    if not data or len(data) % STRIDE:
        raise ValueError('Invalid speech table size')
    rows = []
    for i in range(len(data)//STRIDE):
        record = data[i*STRIDE:(i+1)*STRIDE]
        fields = [record[16+j*256:16+(j+1)*256] for j in range(5)]
        if any(b'\0' not in field for field in fields) or b'\0' not in record[-32:]:
            raise ValueError('Unterminated speech field')
        rows.append({'index': i, 'id': record[-32:].split(b'\0')[0].decode('ascii'),
                     'english': fields[0].split(b'\0')[0].decode('cp1252')})
    return rows


def replace_english(data, replacements):
    rows = speech_rows(data)
    result = bytearray(data)
    for index, text in replacements.items():
        if not 0 <= index < len(rows) or len(text)>255 or b'\0' in text:
            raise ValueError('Invalid replacement or field overflow')
        start = index*STRIDE+16
        result[start:start+256] = text+b'\0'+b' '*(255-len(text))
    return bytes(result)
