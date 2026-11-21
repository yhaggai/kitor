"""Ink spacing and stable composite IDs for rebuilt Hebrew fonts."""
import unicodedata
SIDE_BEARING = 2.0
LOWER_MARKS = tuple(range(0x05B0, 0x05BC)) + (0x05BD, 0x05C7)


def spaced_metrics(bearing_x, advance, ink_box):
    """Reserve clear ink margins without shrinking the existing advance."""
    left, _, right, _ = ink_box
    bearing_x += max(0.0, SIDE_BEARING - (bearing_x + left))
    return bearing_x, max(advance, bearing_x + right + SIDE_BEARING)


def allocate_cluster_codes(clusters, existing, start=0xE200, end=0xF000):
    """Keep published codes, including unused legacy clusters in save names."""
    result = {}
    for cluster, code in existing.items():
        key = unicodedata.normalize('NFD', cluster)
        if key in result or type(code) is not int or not start <= code < end:
            raise ValueError('Invalid or duplicate existing composite mapping')
        result[key] = code
    if len(set(result.values())) != len(result):
        raise ValueError('Duplicate composite code')
    next_code = max(result.values(), default=start-1) + 1
    for cluster in sorted({unicodedata.normalize('NFD', c) for c in clusters}):
        if cluster in result:continue
        if next_code >= end:raise ValueError('Composite private-use range exhausted')
        result[cluster] = next_code
        next_code += 1
    return result
