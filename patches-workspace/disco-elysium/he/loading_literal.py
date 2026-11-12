"""Replace Disco Elysium's hardcoded Arabic loading label in IL2CPP metadata."""
import struct

ARABIC_LOADING = "تحميل".encode()
HEBREW_LOADING = "טוען".encode()


def replace_loading_label(source: bytes) -> bytes:
    magic, version, table, table_size, data, data_size = struct.unpack_from("<6I", source)
    entry = struct.Struct("<II")
    if (
        magic != 0xFAB11BAF
        or version != 27
        or table < 24
        or table_size % entry.size
        or table + table_size > data
        or data + data_size > len(source)
    ):
        raise ValueError("Unsupported IL2CPP string literal table")

    literals = list(entry.iter_unpack(source[table:table + table_size]))
    if any(offset + size > data_size for size, offset in literals):
        raise ValueError("IL2CPP string literal is outside its data section")
    matches = [
        (index, offset)
        for index, (size, offset) in enumerate(literals)
        if source[data + offset:data + offset + size] == ARABIC_LOADING
    ]
    if len(matches) != 1:
        raise ValueError(f"Expected one Arabic loading literal, found {len(matches)}")

    index, offset = matches[0]
    if len(HEBREW_LOADING) > len(ARABIC_LOADING) or any(
        other != index and size and start < offset + len(ARABIC_LOADING) and offset < start + size
        for other, (size, start) in enumerate(literals)
    ):
        raise ValueError("Loading literal cannot be safely replaced in place")

    patched = bytearray(source)
    entry.pack_into(patched, table + index * entry.size, len(HEBREW_LOADING), offset)
    start = data + offset
    patched[start:start + len(ARABIC_LOADING)] = HEBREW_LOADING.ljust(len(ARABIC_LOADING), b"\0")
    return bytes(patched)
