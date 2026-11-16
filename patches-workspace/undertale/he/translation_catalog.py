"""Read the editable UTF-8 translation catalog without changing its text."""
import csv
from pathlib import Path

CATALOG = Path(__file__).with_name('translation-catalog.csv')
FIELDS = ['key', 'english', 'hebrew', 'status', 'translator_note']


def read_catalog(path=CATALOG):
    with path.open(encoding='utf-8-sig', newline='') as stream:
        reader = csv.DictReader(stream, strict=True)
        if reader.fieldnames != FIELDS:
            raise ValueError(f'Unexpected translation catalog columns: {reader.fieldnames}')
        rows = list(reader)
    keys = set()
    for row in rows:
        if None in row or None in row.values() or not row['key'] or row['key'] in keys:
            raise ValueError(f"Malformed or duplicate translation row: {row['key']!r}")
        keys.add(row['key'])
    return {'entries': rows, 'entry_count': len(rows)}
