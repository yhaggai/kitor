import importlib.util
import json
from pathlib import Path
import unittest
from unittest.mock import patch

from build_patch_packages import validate

ROOT = Path(__file__).resolve().parents[2]
PACKAGE = ROOT / 'patches-workspace/moomin-midsummer-madness/he'
spec = importlib.util.spec_from_file_location('moomin_builder', PACKAGE / 'build_moomin_package.py')
builder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(builder)


class MoominPackageTests(unittest.TestCase):
    def test_catalog_updates_both_crcs_without_changing_other_bytes(self):
        catalog = b'prefix' + (123).to_bytes(4, 'little') + b'middle' + (456).to_bytes(4, 'little') + b'suffix'
        with patch.object(builder, 'bundle_crc', side_effect=[123, 789, 456, 1011]):
            actual = builder.update_catalog(catalog, [(b'old-db', b'new-db'), (b'old-font', b'new-font')])
        self.assertEqual(actual, b'prefix' + (789).to_bytes(4, 'little') + b'middle' + (1011).to_bytes(4, 'little') + b'suffix')

    def test_catalog_rejects_missing_duplicate_and_zero_crc(self):
        for catalog, crc in [(b'absent', 123), ((123).to_bytes(4, 'little') * 2, 123), (b'\0' * 4, 0)]:
            with self.subTest(crc=crc, catalog=catalog):
                with patch.object(builder, 'bundle_crc', side_effect=[crc, 789]):
                    with self.assertRaises(ValueError):
                        builder.update_catalog(catalog, [(b'old', b'new')])

    def test_current_build_metadata_and_embedded_package_agree(self):
        registry = json.loads((ROOT / 'kitor/patches/registry.json').read_text(encoding='utf8'))
        game = next(g for g in registry['games'] if g['scummvm_id'] == 'moomin-midsummer-madness')
        release = game['patches'][0]
        manifest, assets = validate(PACKAGE / 'patch.windows.candidate.json', 'windows')
        self.assertEqual(release['version'], manifest['version'])
        self.assertEqual(release['release_revision'], manifest['release_revision'])
        self.assertEqual(release['filename'], manifest['filename'])
        self.assertEqual(release['supported_builds'][0]['build_id'], '26803369')
        database = builder.FILES[0]
        self.assertEqual(manifest['platforms']['windows']['files'][0]['path'], database)
        self.assertIn(database, game['behavior']['detection']['layouts'][0]['required_files'])
        for edition in ('original', 'patched'):
            self.assertIn(database, game['behavior']['launch'][edition]['commands']['windows']['required_files'])
        products = json.loads((ROOT / 'kitor/src-tauri/data/steam_products.json').read_text(encoding='utf8'))
        product = next(p for p in products if p['app_id'] == 4463520)
        self.assertIn(database, product['required_files'])
        self.assertIn('26803369', product['cache_key'])
        bundled = json.loads((ROOT / 'kitor/patches/bundled-assets.json').read_text(encoding='utf8'))['moomin-midsummer-madness/windows']
        self.assertEqual(set(assets), set(bundled))
        for name, source in bundled.items():
            if name != 'patch.json':
                self.assertEqual(assets[name], (ROOT / source).read_bytes())


if __name__ == '__main__':
    unittest.main()
