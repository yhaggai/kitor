import hashlib
import json
from pathlib import Path
import tempfile
import unittest
import zipfile

from build_patch_packages import build, validate, InvalidPackage


class PackageTests(unittest.TestCase):
    def test_release_manifests_support_both_ci_platforms(self):
        workspace = Path(__file__).resolve().parents[2] / 'patches-workspace'
        for path in workspace.glob('*/he/patch.json'):
            with self.subTest(manifest=str(path)):
                platforms = json.loads(path.read_text(encoding='utf-8'))['platforms']
                self.assertTrue({'windows', 'osx'} <= platforms.keys(),
                                'Single-platform packages must use a platform candidate manifest')

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / 'data').write_bytes(b'patch data')
        (self.root / 'unsafe-installer.sh').write_text('rm -rf game')
        self.manifest = self.root / 'patch.json'
        self.plan = {'schema_version': 1, 'scummvm_id': 'fixture', 'language': 'he',
                     'version': '1', 'filename': 'fixture.zip', 'platforms': {'windows': {}, 'osx': {}},
                     'files': [{'path': 'game.dat', 'operation': 'install', 'source': {
                         'kind': 'copy', 'path': 'data', 'sha256': hashlib.sha256(b'patch data').hexdigest()}}]}
        self.save()

    def save(self):
        self.manifest.write_text(json.dumps(self.plan))

    def test_only_references_are_packaged_reproducibly(self):
        first = build(self.manifest, self.root / 'first')
        second = build(self.manifest, self.root / 'second')
        for a, b in zip(first, second):
            self.assertEqual(a.read_bytes(), b.read_bytes())
            with zipfile.ZipFile(a) as archive:
                self.assertEqual(archive.namelist(), ['data', 'patch.json'])
                self.assertEqual(len(json.loads(archive.read('patch.json'))['platforms']), 1)

    def test_platform_archives_only_include_their_own_payloads(self):
        import copy
        (self.root / 'mac-data').write_bytes(b'mac')
        change = copy.deepcopy(self.plan['files'][0])
        change['source'].update(path='mac-data', sha256=hashlib.sha256(b'mac').hexdigest())
        self.plan['platforms']['osx'] = {'files': [change]}
        self.save()
        paths = build(self.manifest, self.root / 'output')
        with zipfile.ZipFile(paths[0]) as archive:
            self.assertIn('data', archive.namelist())
            self.assertNotIn('mac-data', archive.namelist())
        with zipfile.ZipFile(paths[1]) as archive:
            self.assertIn('mac-data', archive.namelist())
            self.assertNotIn('data', archive.namelist())

    def test_universal_preserves_platform_plans_and_filename(self):
        paths = build(self.manifest, self.root / 'output', universal=True)
        self.assertEqual([p.name for p in paths], ['fixture.zip'])
        with zipfile.ZipFile(paths[0]) as archive:
            self.assertEqual(json.loads(archive.read('patch.json')), self.plan)
            self.assertNotIn('unsafe-installer.sh', archive.namelist())

    def test_missing_or_corrupt_data_fails_before_outputs(self):
        (self.root / 'data').write_bytes(b'bad')
        with self.assertRaisesRegex(InvalidPackage, 'SHA256 mismatch'):
            build(self.manifest, self.root / 'output')
        self.assertFalse((self.root / 'output').exists())
        (self.root / 'data').unlink()
        with self.assertRaisesRegex(InvalidPackage, 'Missing asset'):
            validate(self.manifest, 'windows')

    def test_escape_and_platform_unsafe_paths_rejected(self):
        for path in ('../escape', '/absolute', 'C:/game', 'a\\b', 'NUL.txt', 'a/./b'):
            with self.subTest(path=path):
                self.plan['files'][0]['path'] = path
                self.save()
                with self.assertRaises(InvalidPackage):
                    validate(self.manifest, 'windows')

    def test_symlink_is_rejected(self):
        (self.root / 'data').rename(self.root / 'actual')
        try:
            (self.root / 'data').symlink_to(self.root / 'actual')
        except OSError:
            self.skipTest('Host does not permit symlink creation')
        with self.assertRaisesRegex(InvalidPackage, 'Symlink'):
            validate(self.manifest, 'osx')

    def test_colliding_targets_and_unsupported_status_fail(self):
        self.plan['files'].append(dict(self.plan['files'][0], path='GAME.DAT'))
        self.save()
        with self.assertRaisesRegex(InvalidPackage, 'Duplicate target'):
            validate(self.manifest, 'osx')
        self.plan['files'].pop()
        self.plan['status'] = 'unsupported'
        self.save()
        with self.assertRaisesRegex(InvalidPackage, 'Unsupported fields/status'):
            validate(self.manifest, 'osx')

    def test_single_platform_candidate_does_not_claim_macos(self):
        del self.plan['platforms']['osx']
        self.save()
        paths = build(self.manifest, self.root / 'output', platform='windows')
        self.assertEqual(len(paths), 1)
        self.assertTrue(paths[0].name.endswith('-windows.zip'))
        with self.assertRaises(InvalidPackage):
            build(self.manifest, self.root / 'output', universal=True, platform='windows')

    def test_licenses_travel_without_becoming_install_targets(self):
        (self.root / 'licenses').mkdir()
        (self.root / 'licenses/NOTICE.txt').write_text('redistribution notice')
        paths = build(self.manifest, self.root / 'output')
        with zipfile.ZipFile(paths[0]) as archive:
            self.assertIn('licenses/NOTICE.txt', archive.namelist())
            self.assertEqual(len(json.loads(archive.read('patch.json'))['platforms']['windows']['files']), 1)
        (self.root / 'licenses/bad.exe').write_bytes(b'binary')
        with self.assertRaisesRegex(InvalidPackage, 'License must be a text'):
            validate(self.manifest, 'osx')

    def test_legacy_md5_guards_are_validated(self):
        self.plan['files'][0]['original_md5'] = 'not-a-digest'
        self.save()
        with self.assertRaisesRegex(InvalidPackage, 'Invalid MD5'):
            validate(self.manifest, 'windows')
        self.plan['files'][0]['original_md5'] = 'a' * 32
        self.save()
        validate(self.manifest, 'windows')

    def test_noninstallable_provenance_fails(self):
        for status in ('blocked', 'no_patch_required', 'built_in'):
            (self.root / 'provenance.json').write_text(json.dumps({'status': status}))
            with self.assertRaisesRegex(InvalidPackage, 'noninstallable'):
                validate(self.manifest, 'windows')
        (self.root / 'provenance.json').write_text(json.dumps({'status': 'packaged_unverified'}))
        validate(self.manifest, 'windows')

    def test_both_platforms_required_before_writing(self):
        del self.plan['platforms']['osx']
        self.save()
        with self.assertRaisesRegex(InvalidPackage, 'Missing osx'):
            build(self.manifest, self.root / 'output')
        self.assertFalse((self.root / 'output').exists())

    def test_xdelta_tool_must_exist_and_match_hash(self):
        source = self.plan['files'][0]['source']
        source.update(kind='xdelta', input='original.dat', windows_tool='tool.exe', windows_tool_sha256='0' * 64)
        self.save()
        with self.assertRaisesRegex(InvalidPackage, 'Missing asset: tool.exe'):
            validate(self.manifest, 'windows')
        (self.root / 'tool.exe').write_bytes(b'tool')
        with self.assertRaisesRegex(InvalidPackage, 'SHA256 mismatch: tool.exe'):
            validate(self.manifest, 'osx')


if __name__ == '__main__':
    unittest.main()
