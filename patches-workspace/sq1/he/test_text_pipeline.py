"""Synthetic format tests; optional owned-game integration via SQ1_ORIGINAL."""
import csv
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest
import zipfile

import text_pipeline as p


def logic(values, bias=0):
    offsets, strings = bytearray(), bytearray()
    for value in values:
        offsets += p.word(0 if value is None else 2 + len(values) * 2 + len(strings))
        if value is not None:
            strings += value + b'\0'
    return (p.word(2) + b'\x00\x00' + bytes([len(values)])
            + p.word(2 + len(offsets) + len(strings) + bias) + offsets + p.crypt(strings))


class FormatTests(unittest.TestCase):
    def test_variable_length_messages_and_null_slots(self):
        for bias in (0, 1):
            source = logic([b'one', None, b'', b'%m1'], bias)
            values = ['שלום, עולם!'.encode('cp1255'), None, b'', b'%m1']
            rebuilt = p.rebuild_messages(source, values)
            self.assertEqual(p.messages(rebuilt), values)
            self.assertEqual(source[:4], rebuilt[:4])
            self.assertEqual(p.rebuild_messages(source, p.messages(source)), source)

    def test_shared_string_offsets_are_safe_to_split(self):
        source = bytearray(logic([b'same', b'same']))
        source[9:11] = source[7:9]
        self.assertEqual(p.messages(source), [b'same', b'same'])
        self.assertEqual(p.messages(p.rebuild_messages(source, [b'changed', b'same'])),
                         [b'changed', b'same'])

    def test_bad_bounds_and_oversize_rejected(self):
        with self.assertRaises(ValueError):
            p.messages(logic([b'hello'])[:-1])
        with self.assertRaises(ValueError):
            p.rebuild_messages(logic([b'hello']), [b'x' * 65536])

    def test_objects_keep_locations_and_animation_limit(self):
        source = p.crypt(p.word(6) + b'\x11' + p.word(6) + b'\xff'
                         + p.word(10) + b'\x04' + b'one\0two\0')
        values = ['מפתח ארוך'.encode('cp1255'), b'two']
        result = p.rebuild_objects(source, values)
        clear, actual = p.objects(result)
        self.assertEqual(actual, values)
        self.assertEqual((clear[2], clear[5], clear[8]), (17, 255, 4))

    def test_view_keeps_sprite_bytes(self):
        source = b'\x01\x01\x01' + p.word(9) + b'CEL!' + b'old\0'
        result = p.rebuild_description(source, b'longer description')
        self.assertEqual(p.description(result), b'longer description')
        self.assertEqual(source[5:], result[5:len(source)])

    def test_bsdiff_growth_shrink_and_empty(self):
        reader = p.load_tool('test_sq1_bsdiff', p.REPO / 'kitor/src-tauri/src/services/apply_bsdiff.py')
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            for old, new in [(b'abc', b'longer output'), (b'long input', b'a'), (b'a', b''), (b'', b'new')]:
                (root / 'old').write_bytes(old)
                (root / 'delta').write_bytes(p.delta(old, new))
                reader.apply_patch(root / 'old', root / 'delta', root / 'new')
                self.assertEqual((root / 'new').read_bytes(), new)


@unittest.skipUnless(os.environ.get('SQ1_ORIGINAL'), 'Set SQ1_ORIGINAL to the supported owned game')
class OwnedGameTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.original = Path(os.environ['SQ1_ORIGINAL'])
        cls.manifest, cls.originals, cls.translated = p.baseline(cls.original)
        cls.rows = p.catalog_rows(cls.originals, cls.translated)

    def test_checked_in_catalog_compiles(self):
        values, changes = p.read_changes(p.ROOT / 'translations.csv', self.originals, self.translated)
        result, changed_resources = p.regenerate(self.translated, values)
        self.assertEqual(p.text_entries(result), values)

    def test_extracted_baseline_is_byte_identical(self):
        with tempfile.TemporaryDirectory() as td:
            catalog = Path(td) / 'baseline.csv'
            p.write_csv(catalog, self.rows)
            values, changes = p.read_changes(catalog, self.originals, self.translated)
            self.assertEqual(changes, [])
            result, changed_resources = p.regenerate(self.translated, values)
            self.assertEqual(result, self.translated)
            self.assertEqual(changed_resources, [])

    def test_edits_compile_and_package_without_changing_originals(self):
        edits = {
            'logic.000.message.002': 'מדים חדשים לבדיקה',
            'object.001.name': 'חפץ לבדיקה',
            'view.186.description': 'זוהי ערכת הישרדות לבדיקה.\nשורה נוספת, עם "מרכאות".',
        }
        rows = [dict(row, hebrew=edits.get(row['key'], row['hebrew'])) for row in self.rows]
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            catalog = root / 'edited.csv'
            # Sorting a spreadsheet must not change key-to-resource identity.
            p.write_csv(catalog, list(reversed(rows)))
            values, changes = p.read_changes(catalog, self.originals, self.translated)
            rebuilt, _ = p.regenerate(self.translated, values)
            expected = p.text_entries(self.translated)
            expected.update({key: value.encode('cp1255') for key, value in edits.items()})
            self.assertEqual(p.text_entries(rebuilt), expected)
            for name in ('PICDIR', 'SNDDIR', 'WORDS.TOK', 'WORDS.TOK.EXTENDED', 'agi-font-dos.bin'):
                self.assertEqual(rebuilt[name], self.translated[name])
            for number, old in p.resources(self.translated, 'LOGDIR').items():
                new = p.resources(rebuilt, 'LOGDIR')[number]
                self.assertEqual(old[:2 + p.u16(old)], new[:2 + p.u16(old)])
            for number, old in p.resources(self.translated, 'VIEWDIR').items():
                new = p.resources(rebuilt, 'VIEWDIR')[number]
                self.assertEqual(old[:3], new[:3])
                self.assertEqual(old[5:], new[5:len(old)])
            report = p.build(self.original, catalog, root / 'build', '1.0-csv-test', 1)
            self.assertEqual(set(report['changed_keys']), set(edits))
            reader = p.load_tool('integration_sq1_bsdiff', p.REPO / 'kitor/src-tauri/src/services/apply_bsdiff.py')
            # The default CI path has no original game: it must produce the same
            # game bytes as the independently reconstructed owned-game builder.
            source = root / 'portable-source'
            shutil.copytree(p.ROOT, source, ignore=shutil.ignore_patterns('__pycache__'))
            shutil.copyfile(catalog, source / 'translations.csv')
            packager = p.load_tool('integration_sq1_packager', p.REPO / 'kitor/tools/build_patch_packages.py')
            portable_archives = packager.build(source / 'patch.json', root / 'portable-packages')
            for artifact in portable_archives:
                with zipfile.ZipFile(artifact) as archive:
                    built = json.loads(archive.read('patch.json'))
                    for entry in built['files']:
                        payload = archive.read(entry['source']['path'])
                        if entry['source']['kind'] == 'bsdiff':
                            (root / 'delta').write_bytes(payload)
                            reader.apply_patch(self.original / entry['path'], root / 'delta', root / 'decoded')
                            payload = (root / 'decoded').read_bytes()
                        self.assertEqual(payload, rebuilt[entry['path']])
            for platform in ('windows', 'osx'):
                with zipfile.ZipFile(root / 'build/packages' / f'sq1-he-1.0-csv-test-{platform}.zip') as archive:
                    self.assertIsNone(archive.testzip())
                    package_manifest = json.loads(archive.read('patch.json'))
                    for entry in package_manifest['files']:
                        name, source = entry['path'], entry['source']
                        asset = archive.read(source['path'])
                        self.assertEqual(p.sha(asset), source['sha256'])
                        if source['kind'] == 'bsdiff':
                            (root / 'delta').write_bytes(asset)
                            reader.apply_patch(self.original / name, root / 'delta', root / 'decoded')
                            actual = (root / 'decoded').read_bytes()
                        else:
                            actual = asset
                        self.assertEqual(actual, rebuilt[name])
            # Reproducible packages from the same inputs.
            p.build(self.original, catalog, root / 'repeat', '1.0-csv-test', 1)
            for archive in (root / 'build/packages').iterdir():
                self.assertEqual(archive.read_bytes(), (root / 'repeat/packages' / archive.name).read_bytes())
        # No install took place; the supported source files remain untouched.
        for name, data in self.originals.items():
            self.assertEqual((self.original / name).read_bytes(), data)

    def test_bad_catalogs_rejected(self):
        with tempfile.TemporaryDirectory() as td:
            catalog = Path(td) / 'invalid.csv'
            variants = [self.rows[:-1], self.rows + [self.rows[0]]]
            for key, value in [('english', 'changed'), ('hebrew', '😀'), ('hebrew', 'oops %m99')]:
                rows = [dict(row) for row in self.rows]
                rows[1][key] = value
                variants.append(rows)
            rows = [dict(row) for row in self.rows]
            for row in rows:
                if row['key'] == 'logic.000.message.029':
                    row['hebrew'] = row['hebrew'].replace('%s', '')
            variants.append(rows)
            for rows in variants:
                p.write_csv(catalog, rows)
                with self.assertRaises(ValueError):
                    p.read_changes(catalog, self.originals, self.translated)


if __name__ == '__main__':
    unittest.main()
