import hashlib
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]


class UndertaleOptionsTests(unittest.TestCase):
    def test_only_regular_hebrew_is_available(self):
        registry = json.loads((ROOT/'kitor/patches/registry.json').read_text(encoding="utf-8"))
        game = next(g for g in registry['games'] if g['scummvm_id'] == 'undertale')
        self.assertEqual({p['language'] for p in game['patches']}, {'he'})
        manifests = []
        for folder, language in [('he', 'he')]:
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
        withdrawn = json.loads((ROOT/'patches-workspace/undertale/he-nikud/provenance.json').read_text(encoding="utf-8"))
        self.assertFalse(withdrawn['release_ready'])
        self.assertEqual(withdrawn['status'], 'withdrawn')


if __name__ == '__main__':
    unittest.main()
