import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import zipfile

import build_download_packages as packages
from build_patch_packages import InvalidPackage, validate


class DownloadPackages(unittest.TestCase):
    def test_all_archives_preserve_supported_file_plans_and_assets(self):
        index = json.loads(packages.INDEX.read_text())
        with tempfile.TemporaryDirectory() as temp:
            results = packages.build_all(Path(temp))
            self.assertEqual(len(results), 10)
            for artifact, game, patch in results:
                with zipfile.ZipFile(artifact) as archive:
                    expected = set()
                    for platform, editions in index[game['scummvm_id']].items():
                        for edition, source in editions.items():
                            _, assets = validate(packages.ROOT / source, platform)
                            prefix = f'_platforms/{platform}/{edition}/'
                            expected.update(prefix + name for name in assets)
                            for name, data in assets.items():
                                actual = archive.read(prefix + name)
                                if name == 'patch.json':
                                    before, after = json.loads(data), json.loads(actual)
                                    self.assertEqual(after.pop('version'), patch['version'])
                                    self.assertEqual(after.pop('release_revision'), patch['release_revision'])
                                    self.assertEqual(after.pop('filename'), patch['filename'])
                                    for key in ('version', 'release_revision', 'filename'):
                                        before.pop(key, None)
                                    self.assertEqual(before, after)
                                else:
                                    self.assertEqual(actual, data)
                    self.assertEqual(set(archive.namelist()), expected)
                rebuilt, _ = packages.build(game, patch, index[game['scummvm_id']], Path(temp) / 'repeat')
                self.assertEqual(hashlib.sha256(artifact.read_bytes()).digest(), hashlib.sha256(rebuilt.read_bytes()).digest())

    def test_missing_platform_is_rejected(self):
        registry = json.loads(packages.REGISTRY.read_text())
        game = next(g for g in registry['games'] if g['scummvm_id'] == 'drifter')
        with tempfile.TemporaryDirectory() as temp:
            with self.assertRaises(InvalidPackage):
                packages.build(game, game['patches'][0], {}, Path(temp))

    def test_production_build_has_no_embedded_translation_index(self):
        build = (packages.ROOT / 'kitor/src-tauri/build.rs').read_text()
        self.assertNotIn('include_bytes!', build)
        self.assertNotIn('bundled-assets', build)
        for game in json.loads(packages.REGISTRY.read_text())['games']:
            for patch in game['patches']:
                self.assertNotEqual(patch['source_type'], 'bundled')
                self.assertTrue(patch['download_url'].startswith('https://'))

    def test_app_release_gate_rejects_mismatched_published_bytes(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            index, registry = root / 'index.json', root / 'registry.json'
            index.write_text('{"fixture": {}}')
            registry.write_text(json.dumps({'games': [{'scummvm_id': 'fixture', 'patches': [{
                'download_url': 'https://github.com/yhaggai/kitor/releases/download/tag/fixture.zip',
                'filename': 'fixture.zip', 'file_size': 4,
                'checksum': hashlib.sha256(b'good').hexdigest()
            }]}]}))
            def download(args, **kwargs):
                (Path(args[args.index('--dir') + 1]) / 'fixture.zip').write_bytes(b'bad!')
            with patch.object(packages, 'INDEX', index), patch.object(packages, 'REGISTRY', registry), \
                 patch.object(packages.subprocess, 'check_output', return_value='{"draft": false, "prerelease": false}'), \
                 patch.object(packages.subprocess, 'run', side_effect=download):
                with self.assertRaisesRegex(InvalidPackage, 'Published package differs'):
                    packages.verify_published()
