"""The normal packager must compile SQ1 CSV offline, without stale fallbacks."""
import csv
import hashlib
import json
from pathlib import Path
import shutil
import tempfile
import unittest
import zipfile

from build_patch_packages import build, validate, InvalidPackage

SOURCE = Path(__file__).resolve().parents[2] / 'patches-workspace/sq1/he'


class Sq1DefaultBuildTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.source = self.root / 'source'
        shutil.copytree(SOURCE, self.source, ignore=shutil.ignore_patterns('__pycache__'))
        self.manifest = self.source / 'patch.json'

    def edit(self, value):
        path = self.source / 'translations.csv'
        with path.open(encoding='utf-8', newline='') as source:
            rows = list(csv.DictReader(source))
        for row in rows:
            if row['key'] == 'logic.000.message.002':
                row['hebrew'] = value
        with path.open('w', encoding='utf-8', newline='') as output:
            writer = csv.DictWriter(output, ['key', 'english', 'hebrew'], lineterminator='\n')
            writer.writeheader()
            writer.writerows(rows)

    def test_default_platform_and_release_builds_compile_csv(self):
        self.edit('מדים חדשים לבדיקה')
        csv_hash = hashlib.sha256((self.source / 'translations.csv').read_bytes()).hexdigest()
        paths = build(self.manifest, self.root / 'platforms')
        release, = build(self.manifest, self.root / 'release', universal=True)
        self.assertEqual(len(paths), 2)
        for path in paths + [release]:
            with zipfile.ZipFile(path) as archive:
                report = json.loads(archive.read('build-report.json'))
                self.assertEqual(report['catalog_sha256'], csv_hash)
                self.assertIn('logic.000.message.002', report['changed_keys'])
                self.assertNotEqual(archive.read('payload/VOL.3'), (SOURCE / 'payload/VOL.3').read_bytes())
                self.assertEqual(archive.read('payload/VOL.0.bsdiff'), (SOURCE / 'payload/VOL.0.bsdiff').read_bytes())
        again, = build(self.manifest, self.root / 'again', universal=True)
        self.assertEqual(release.read_bytes(), again.read_bytes())

    def test_validate_also_compiles_and_never_caches_an_old_csv(self):
        _, before = validate(self.manifest, 'osx')
        self.edit('מדים שונים לבדיקה')
        _, after = validate(self.manifest, 'osx')
        self.assertNotEqual(before['payload/VOL.3'], after['payload/VOL.3'])
        self.assertNotEqual(before['build-report.json'], after['build-report.json'])

    def test_bad_or_missing_csv_fails_before_archives_are_written(self):
        self.edit('unsupported 😀')
        with self.assertRaisesRegex(ValueError, 'Unsupported character'):
            build(self.manifest, self.root / 'output')
        self.assertFalse((self.root / 'output').exists())
        (self.source / 'translations.csv').unlink()
        with self.assertRaises(FileNotFoundError):
            build(self.manifest, self.root / 'output')
        self.assertFalse((self.root / 'output').exists())

    def test_changed_template_and_unknown_compiler_rejected(self):
        path = self.source / 'text-template.json'
        path.write_bytes(path.read_bytes() + b' ')
        with self.assertRaisesRegex(ValueError, 'template checksum'):
            validate(self.manifest, 'windows')
        path = self.source / 'build-source.json'
        config = json.loads(path.read_text())
        config['compiler'] = '../../other.py'
        path.write_text(json.dumps(config))
        with self.assertRaisesRegex(InvalidPackage, 'Unsupported source compiler'):
            validate(self.manifest, 'windows')


if __name__ == '__main__':
    unittest.main()
