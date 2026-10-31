import hashlib
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]


class UndertaleOptionsTests(unittest.TestCase):
    def test_both_options_match_their_own_packages(self):
        registry = json.loads((ROOT/'kitor/patches/registry.json').read_text(encoding="utf-8"))
        game = next(g for g in registry['games'] if g['scummvm_id'] == 'undertale')
        self.assertEqual({p['language'] for p in game['patches']}, {'he', 'he-x-nikud'})
        manifests = []
        for folder, language in [('he', 'he'), ('he-nikud', 'he-x-nikud')]:
            root = ROOT/'patches-workspace/undertale'/folder
            manifest = json.loads((root/'patch.json').read_text(encoding="utf-8"))
            patch = next(p for p in game['patches'] if p['language'] == language)
            self.assertEqual(manifest['language'], language)
            self.assertEqual(manifest['version'], patch['version'])
            self.assertEqual(manifest.get('release_revision', 0), patch.get('release_revision', 0))
            self.assertEqual(manifest['filename'], patch['filename'])
            for platform in ('windows', 'osx'):
                change, = manifest['platforms'][platform]['files']
                payload = root/change['source']['path']
                self.assertEqual(hashlib.sha256(payload.read_bytes()).hexdigest(), change['source']['sha256'])
            manifests.append(manifest)
        for platform in ('windows', 'osx'):
            plain, pointed = [m['platforms'][platform]['files'][0] for m in manifests]
            self.assertEqual(plain['original_sha256'], pointed['original_sha256'])
            self.assertNotEqual(plain['installed_sha256'], pointed['installed_sha256'])


if __name__ == '__main__':
    unittest.main()
