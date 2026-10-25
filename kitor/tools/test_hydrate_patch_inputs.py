import hashlib
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from hydrate_patch_inputs import hydrate


class HydrationTests(unittest.TestCase):
    def test_verified_download_only_is_installed(self):
        data = b'pinned payload'
        spec = {'schema_version': 1, 'release_tag': 'pinned', 'assets': [
            {'asset': 'payload.bin', 'destination': 'patches/file.bin', 'size': len(data), 'sha256': hashlib.sha256(data).hexdigest()}]}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            def download(args, **kwargs):
                (Path(args[args.index('--dir') + 1]) / 'payload.bin').write_bytes(data)
            with patch('hydrate_patch_inputs.subprocess.run', side_effect=download) as fetch:
                hydrate(root, spec, 'owner/repo')
                self.assertEqual((root / 'patches/file.bin').read_bytes(), data)
                hydrate(root, spec, 'owner/repo')
                self.assertEqual(fetch.call_count, 1)

    def test_wrong_download_never_creates_destination(self):
        spec = {'schema_version': 1, 'release_tag': 'pinned', 'assets': [
            {'asset': 'payload.bin', 'destination': 'patches/file.bin', 'size': 3, 'sha256': '0' * 64}]}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            def download(args, **kwargs):
                (Path(args[args.index('--dir') + 1]) / 'payload.bin').write_bytes(b'bad')
            with patch('hydrate_patch_inputs.subprocess.run', side_effect=download), self.assertRaisesRegex(ValueError, 'SHA256'):
                hydrate(root, spec, 'owner/repo')
            self.assertFalse((root / 'patches/file.bin').exists())

    def test_escape_destination_rejected(self):
        spec = {'schema_version': 1, 'release_tag': 'pinned', 'assets': [
            {'asset': 'payload.bin', 'destination': '../escape', 'size': 3, 'sha256': '0' * 64}]}
        with tempfile.TemporaryDirectory() as directory, self.assertRaises(ValueError):
            hydrate(Path(directory), spec, 'owner/repo')


if __name__ == '__main__':
    unittest.main()
