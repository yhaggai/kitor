#!/usr/bin/env python3
"""Extract/rebuild SQ1 EGA 2.2 text using the pinned Hebrew patch as a template.

Only Python's standard library is required. Never writes into an installed game.
Resource layout references and limitations are documented in README.md.
"""
import argparse
import base64
import bz2
from collections import Counter
import csv
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import struct
import tempfile

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[2]
KEY = b'Avis Durgan'
FIELDS = ['key', 'english', 'hebrew']
TOKENS = re.compile(r'%(?:v[0-9]*(?:\|[0-9]+)?|[gwsm0][0-9]*)')


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def load_tool(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def u16(data, offset=0):
    require(0 <= offset <= len(data) - 2, 'Truncated 16-bit field')
    return struct.unpack_from('<H', data, offset)[0]


def word(value):
    require(0 <= value <= 65535, 'AGI 16-bit size/offset limit exceeded')
    return struct.pack('<H', value)


def crypt(data):
    return bytes(value ^ KEY[i % len(KEY)] for i, value in enumerate(data))


def cstring(data, offset):
    require(0 <= offset < len(data), 'String offset outside resource')
    end = data.find(b'\0', offset)
    require(end >= 0, 'Unterminated string')
    return data[offset:end]


def resources(files, directory):
    table = files[directory]
    require(len(table) % 3 == 0, 'Invalid directory length')
    result = {}
    for number in range(len(table) // 3):
        entry = int.from_bytes(table[number * 3:number * 3 + 3], 'big')
        if entry == 0xffffff:
            continue
        volume, offset = entry >> 20, entry & 0xfffff
        data = files[f'VOL.{volume}']
        require(data[offset:offset + 2] == b'\x12\x34', 'Invalid AGI v2 resource header')
        require(data[offset + 2] == volume, 'Resource volume mismatch')
        size = u16(data, offset + 3)
        require(offset + 5 + size <= len(data), 'Truncated resource')
        result[number] = data[offset + 5:offset + 5 + size]
    return result


def messages(data):
    start = 2 + u16(data)
    require(start + 3 <= len(data), 'Missing message section')
    count = data[start]
    text_start = start + 3 + 2 * count
    # Original Sierra files exclude the count byte; the upstream compiler
    # includes it. Both layouts terminate at the resource boundary.
    end = len(data)
    require(u16(data, start + 1) in (end - start, end - start - 1),
            'Invalid message section size')
    require(text_start <= end, 'Invalid message section bounds')
    clear = data[:text_start] + crypt(data[text_start:end]) + data[end:]
    result = []
    for i in range(count):
        offset = u16(data, start + 3 + 2 * i)
        if offset:
            require(text_start <= start + 1 + offset < end, 'Invalid message pointer')
            result.append(cstring(clear[:end], start + 1 + offset))
        else:
            result.append(None)
    return result


def rebuild_messages(data, values):
    previous = messages(data)
    require(len(values) == len(previous), 'Message count changed')
    if values == previous:
        return data
    start = 2 + u16(data)
    size_bias = u16(data, start + 1) - (len(data) - start - 1)
    offsets, strings = bytearray(), bytearray()
    for value in values:
        if value is None:
            offsets += word(0)
        else:
            offsets += word(2 + len(values) * 2 + len(strings))
            strings += value + b'\0'
    section = bytes([len(values)]) + word(2 + len(offsets) + len(strings) + size_bias)
    result = data[:start] + section + offsets + crypt(strings)
    require(len(result) <= 65535, 'Logic resource exceeds AGI v2 size limit')
    return result


def objects(data):
    clear = crypt(data)
    size = u16(clear)
    require(size % 3 == 0 and size + 3 <= len(clear), 'Invalid inventory table')
    return clear, [cstring(clear, 3 + u16(clear, 3 + i * 3)) for i in range(size // 3)]


def rebuild_objects(data, values):
    clear, previous = objects(data)
    require(len(values) == len(previous), 'Object count changed')
    if values == previous:
        return data
    table = bytearray(clear[:3 + len(values) * 3])
    strings = bytearray()
    for i, value in enumerate(values):
        table[3 + i * 3:5 + i * 3] = word(len(table) - 3 + len(strings))
        strings += value + b'\0'
    return crypt(table + strings)


def description(data):
    offset = u16(data, 3)
    return cstring(data, offset) if offset else None


def rebuild_description(data, value):
    previous = description(data)
    if value == previous:
        return data
    # Append instead of shifting sprite offsets or touching cel data.
    require(value is not None, 'Cannot remove description slot')
    result = data[:3] + word(len(data)) + data[5:] + value + b'\0'
    require(len(result) <= 65535, 'View resource exceeds AGI v2 size limit')
    return result


def text_entries(files):
    return resource_text_entries(resources(files, 'LOGDIR'), files['OBJECT'], resources(files, 'VIEWDIR'))


def resource_text_entries(logics, inventory, views):
    entries = {}
    for logic, data in logics.items():
        for number, value in enumerate(messages(data), 1):
            entries[f'logic.{logic:03d}.message.{number:03d}'] = value
    for number, value in enumerate(objects(inventory)[1]):
        entries[f'object.{number:03d}.name'] = value
    for number, data in views.items():
        value = description(data)
        if value is not None:
            entries[f'view.{number:03d}.description'] = value
    return entries


def baseline(original):
    """Reconstruct and hash-verify the existing patch in temporary storage."""
    template_path = ROOT / 'text-template.json'
    manifest = (json.loads(template_path.read_text(encoding='utf-8'))['baseline_manifest']
                if template_path.exists() else json.loads((ROOT / 'patch.json').read_text()))
    reader = load_tool('sq1_bsdiff', REPO / 'kitor/src-tauri/src/services/apply_bsdiff.py')
    originals, translated = {}, {}
    with tempfile.TemporaryDirectory(prefix='sq1-baseline-') as temporary:
        for entry in manifest['files']:
            name, source = entry['path'], entry['source']
            asset = ROOT / source['path']
            require(sha(asset.read_bytes()) == source['sha256'], f'Changed baseline asset: {name}')
            if source['kind'] == 'bsdiff':
                data = (original / name).read_bytes()
                require(sha(data) == entry['original_sha256'], f'Unsupported original: {name}')
                originals[name] = data
                target = Path(temporary) / name
                reader.apply_patch(original / name, asset, target)
                data = target.read_bytes()
                require(sha(data) == entry['installed_sha256'], f'Baseline reconstruction failed: {name}')
            else:
                require(source['kind'] == 'copy', 'Unsupported baseline operation')
                data = asset.read_bytes()
            translated[name] = data
    return manifest, originals, translated


def catalog_rows(originals, translated):
    english = text_entries(originals)
    return [dict(key=key, english=(english.get(key) or b'').decode('cp437'),
                 hebrew=(value or b'').decode('cp1255'))
            for key, value in text_entries(translated).items()]


def write_csv(path, rows):
    with path.open('w', encoding='utf-8', newline='') as output:
        writer = csv.DictWriter(output, FIELDS, lineterminator='\n')
        writer.writeheader()
        writer.writerows(rows)


def read_changes(path, originals, translated):
    return read_catalog(path, catalog_rows(originals, translated), text_entries(translated))


def read_catalog(path, rows, previous):
    expected = {row['key']: row for row in rows}
    values, changes = {}, []
    with path.open(encoding='utf-8-sig', newline='') as source:
        reader = csv.DictReader(source)
        require(reader.fieldnames == FIELDS, f'CSV columns must be {FIELDS}')
        for row in reader:
            key = row['key']
            require(key in expected and key not in values, f'Unknown or duplicate key: {key}')
            require(set(row) == set(FIELDS) and None not in row.values(), f'Malformed CSV row: {key}')
            require(row['english'] == expected[key]['english'], f'English changed: {key}')
            text = row['hebrew']
            require('\0' not in text and '\r' not in text, f'NUL/CR forbidden: {key}; use LF for line breaks')
            try:
                value = text.encode('cp1255')
            except UnicodeEncodeError as error:
                raise ValueError(f'Unsupported character for SQ1 CP1255: {key}: {error}') from error
            if text != expected[key]['hebrew']:
                require(Counter(TOKENS.findall(text)) == Counter(TOKENS.findall(expected[key]['hebrew'])),
                        f'AGI substitution tokens changed: {key}')
                require(previous[key] is not None, f'Cannot populate unused message slot: {key}')
                require(bool(text), f'Cannot blank an existing text: {key}')
                changes.append(key)
            values[key] = None if previous[key] is None else value
    require(values.keys() == expected.keys(), 'CSV must contain every key; missing rows are not deletions')
    return values, changes


def regenerate(translated, values, resource_maps=None):
    result = dict(translated)
    maps = {directory: dict(resource_maps[directory] if resource_maps is not None
                            else resources(translated, directory))
            for directory in ('LOGDIR', 'VIEWDIR')}
    volume = bytearray(result['VOL.3'])
    changed_resources = []
    for directory in ('LOGDIR', 'VIEWDIR'):
        table = bytearray(result[directory])
        for number, data in maps[directory].items():
            if directory == 'LOGDIR':
                new = rebuild_messages(data, [values[f'logic.{number:03d}.message.{i:03d}']
                                              for i in range(1, len(messages(data)) + 1)])
            else:
                key = f'view.{number:03d}.description'
                new = rebuild_description(data, values[key]) if key in values else data
            if new != data:
                require(len(volume) + len(new) + 5 <= 0x100000, 'VOL.3 exceeds supported 20-bit address space')
                table[number * 3:number * 3 + 3] = ((3 << 20) | len(volume)).to_bytes(3, 'big')
                volume += b'\x12\x34\x03' + word(len(new)) + new
                changed_resources.append(f'{directory}:{number}')
                maps[directory][number] = new
        result[directory] = bytes(table)
    result['VOL.3'] = bytes(volume)
    result['OBJECT'] = rebuild_objects(translated['OBJECT'],
                                      [values[f'object.{i:03d}.name']
                                       for i in range(len(objects(translated['OBJECT'])[1]))])
    require(resource_text_entries(maps['LOGDIR'], result['OBJECT'], maps['VIEWDIR']) == values,
            'Re-extracted output does not match CSV')
    return result, changed_resources


def prepare_template(original, output):
    """One-time extraction of compilation context; no original volume archives."""
    require(not output.exists(), 'Refusing to overwrite an existing template')
    manifest, originals, translated = baseline(original)
    encode = lambda data: base64.b64encode(data).decode('ascii')
    template = {
        'schema_version': 1,
        'baseline_manifest': manifest,
        'english': {row['key']: row['english'] for row in catalog_rows(originals, translated)},
        'original_tables': {name: encode(originals[name]) for name in ('LOGDIR', 'VIEWDIR', 'OBJECT')},
        'translated_tables': {name: encode(translated[name]) for name in ('LOGDIR', 'VIEWDIR', 'OBJECT')},
        'logics': {str(n): encode(data) for n, data in resources(translated, 'LOGDIR').items()},
        'views': {str(n): encode(data) for n, data in resources(translated, 'VIEWDIR').items()
                  if description(data) is not None},
    }
    output.write_text(json.dumps(template, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    return sha(output.read_bytes())


def prepare_package(source_root, output):
    """Default offline compiler used by the shared packager, including CI.

The immutable template retains patch resource structure. All active message,
inventory and description values are taken from translations.csv each time.
"""
    config = json.loads((source_root / 'build-source.json').read_text(encoding='utf-8'))
    require(config.get('schema_version') == 1 and config.get('compiler') == 'sq1-csv',
            'Unsupported SQ1 build-source configuration')
    template_path = source_root / 'text-template.json'
    require(not template_path.is_symlink(), 'Symlink text template refused')
    require(sha(template_path.read_bytes()) == config['template_sha256'], 'SQ1 template checksum mismatch')
    template = json.loads(template_path.read_text(encoding='utf-8'))
    require(template['schema_version'] == 1, 'Unsupported SQ1 template')
    manifest = json.loads((source_root / 'patch.json').read_text(encoding='utf-8'))
    pinned = template['baseline_manifest']
    for field in ('scummvm_id', 'language', 'platforms', 'files'):
        require(manifest[field] == pinned[field], f'SQ1 baseline {field} changed; regenerate the template intentionally')
    decode = lambda data: base64.b64decode(data, validate=True)
    originals = {name: decode(data) for name, data in template['original_tables'].items()}
    translated = {name: decode(data) for name, data in template['translated_tables'].items()}
    maps = {directory: {int(n): decode(data) for n, data in template[field].items()}
            for directory, field in [('LOGDIR', 'logics'), ('VIEWDIR', 'views')]}
    assets = {}
    for entry in pinned['files']:
        source = entry['source']
        asset = source_root / source['path']
        require(not asset.is_symlink(), 'Symlink baseline asset refused')
        data = asset.read_bytes()
        require(sha(data) == source['sha256'], f'Changed baseline asset: {entry["path"]}')
        assets[source['path']] = data
        name = entry['path']
        if name in originals:
            require(sha(originals[name]) == entry['original_sha256'], f'Invalid original template: {name}')
            require(sha(translated[name]) == entry['installed_sha256'], f'Invalid translated template: {name}')
        if name == 'VOL.3':
            translated[name] = data
    previous = resource_text_entries(maps['LOGDIR'], translated['OBJECT'], maps['VIEWDIR'])
    require(previous.keys() == template['english'].keys(), 'Template key coverage mismatch')
    rows = [dict(key=key, english=template['english'][key], hebrew=(data or b'').decode('cp1255'))
            for key, data in previous.items()]
    catalog = source_root / 'translations.csv'
    require(not catalog.is_symlink(), 'Symlink CSV refused')
    values, changes = read_catalog(catalog, rows, previous)
    regenerated, changed_resources = regenerate(translated, values, maps)
    require(not output.exists(), 'Output must be a new directory')
    output.mkdir(parents=True)
    reader = load_tool('sq1_portable_bsdiff', REPO / 'kitor/src-tauri/src/services/apply_bsdiff.py')
    with tempfile.TemporaryDirectory(prefix='sq1-portable-verify-') as temporary:
        temporary = Path(temporary)
        for entry in manifest['files']:
            name, source = entry['path'], entry['source']
            if name in regenerated and regenerated[name] != translated[name]:
                data = regenerated[name]
                assets[source['path']] = delta(originals[name], data) if name in originals else data
                source['sha256'] = sha(assets[source['path']])
                entry['installed_sha256'] = sha(data)
            asset = output / source['path']
            asset.parent.mkdir(parents=True, exist_ok=True)
            asset.write_bytes(assets[source['path']])
            if name in originals:
                (temporary / 'original').write_bytes(originals[name])
                reader.apply_patch(temporary / 'original', asset, temporary / 'decoded')
                require((temporary / 'decoded').read_bytes() == regenerated[name], f'Invalid compiled delta: {name}')
    (output / 'patch.json').write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    report = {'schema_version': 1, 'compiler': 'sq1-csv', 'catalog_sha256': sha(catalog.read_bytes()),
              'template_sha256': config['template_sha256'], 'changed_keys': changes,
              'changed_resources': changed_resources, 'text_entries': len(values),
              'reextraction_matches_csv': True, 'changed_table_deltas_verified': True}
    (output / 'build-report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    return report


def delta(old, new):
    """Simple deterministic BSDIFF40: aligned byte differences plus trailing data."""
    def number(n):
        return n.to_bytes(8, 'little')
    length = min(len(old), len(new))
    control = bz2.compress(number(length) + number(len(new) - length) + number(0))
    diff = bz2.compress(bytes((b - a) & 255 for a, b in zip(old, new)))
    extra = bz2.compress(new[length:])
    return b'BSDIFF40' + number(len(control)) + number(len(diff)) + number(len(new)) + control + diff + extra


def build(original, csv_path, output, version, revision):
    require(not output.resolve().is_relative_to(original.resolve()), 'Output cannot be inside the original game')
    manifest, originals, translated = baseline(original)
    require(re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9._-]*', version), 'Invalid version label')
    require(version != manifest['version'] and revision > manifest.get('release_revision', 0),
            'Use a new version label and a higher release revision')
    values, changes = read_changes(csv_path, originals, translated)
    regenerated, changed_resources = regenerate(translated, values)
    require(not output.exists() and not output.is_symlink(), 'Output must be a new directory')
    output.mkdir(parents=True)
    (output / 'payload').mkdir()
    reader = load_tool('sq1_verify_bsdiff', REPO / 'kitor/src-tauri/src/services/apply_bsdiff.py')
    with tempfile.TemporaryDirectory(prefix='sq1-verify-') as temporary:
        for entry in manifest['files']:
            name, source = entry['path'], entry['source']
            data = regenerated[name]
            if data == translated[name]:
                asset_data = (ROOT / source['path']).read_bytes()
            else:
                asset_data = delta(originals[name], data) if source['kind'] == 'bsdiff' else data
            asset = output / source['path']
            asset.write_bytes(asset_data)
            source['sha256'] = sha(asset_data)
            entry['installed_sha256'] = sha(data)
            if source['kind'] == 'bsdiff':
                target = Path(temporary) / name
                reader.apply_patch(original / name, asset, target)
                require(target.read_bytes() == data, f'Generated delta failed verification: {name}')
    manifest.update(version=version, release_revision=revision, filename=f'sq1-he-{version}.zip')
    (output / 'patch.json').write_text(json.dumps(manifest, indent=2) + '\n')
    report = {
        'schema_version': 1, 'catalog_sha256': sha(csv_path.read_bytes()),
        'baseline_manifest_sha256': sha((ROOT / 'patch.json').read_bytes()),
        'text_entries': len(values), 'changed_keys': changes,
        'changed_resources': changed_resources,
        'unchanged_catalog_is_byte_identical': regenerated == translated,
        'reextraction_matches_csv': True, 'all_delta_outputs_verified': True,
        'release_ready': False, 'native_windows': 'not_run', 'native_macos_gameplay': 'not_run',
        'scope': 'Logic messages, inventory names, view descriptions. Vocabulary/art/code preserved.',
    }
    (output / 'build-report.json').write_text(json.dumps(report, indent=2) + '\n')
    provenance = json.loads((ROOT / 'provenance.json').read_text())
    provenance.update(release_ready=False, csv_build=report)
    (output / 'provenance.json').write_text(json.dumps(provenance, indent=2, ensure_ascii=False) + '\n')
    packager = load_tool('sq1_packager', REPO / 'kitor/tools/build_patch_packages.py')
    packager.build(output / 'patch.json', output / 'packages')
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    extract = sub.add_parser('extract')
    extract.add_argument('--original', type=Path, required=True)
    extract.add_argument('--output', type=Path, required=True)
    compile_parser = sub.add_parser('build')
    compile_parser.add_argument('--original', type=Path, required=True)
    compile_parser.add_argument('--csv', type=Path, default=ROOT / 'translations.csv')
    compile_parser.add_argument('--output', type=Path, required=True)
    compile_parser.add_argument('--version', required=True)
    compile_parser.add_argument('--revision', type=int, required=True)
    args = parser.parse_args()
    if args.command == 'extract':
        require(not args.output.resolve().is_relative_to(args.original.resolve()),
                'Catalog output cannot be inside the original game')
        require(not args.output.exists(), 'Refusing to overwrite an existing catalog')
        _, originals, translated = baseline(args.original)
        rows = catalog_rows(originals, translated)
        write_csv(args.output, rows)
        print(json.dumps({'entries': len(rows), 'output': str(args.output)}))
    else:
        print(json.dumps(build(args.original, args.csv, args.output, args.version, args.revision), indent=2))


if __name__ == '__main__':
    main()
