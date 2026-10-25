"""Synthetic CLI smoke checks; build validate_owned_games before running these."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


class DisposableValidationTests(unittest.TestCase):
    def setUp(self):
        default = Path(__file__).parent / 'lifecycle-harness/target/debug' / ('validate_owned_games.exe' if os.name == 'nt' else 'validate_owned_games')
        self.binary = Path(os.environ.get('KITOR_VALIDATOR_BINARY', str(default))).resolve()
        if not self.binary.is_file():
            self.skipTest('Build validate_owned_games or set KITOR_VALIDATOR_BINARY first')
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.game = self.root / 'owned'
        self.game.mkdir()
        (self.game / 'original.bin').write_bytes(b'original bytes')
        (self.game / 'untouched.save').write_bytes(b'user save')
        self.before = {p.name: p.read_bytes() for p in self.game.iterdir()}
        self.package = self.root / 'patches/fixture/he'
        self.package.mkdir(parents=True)
        (self.package / 'payload').write_bytes(b'patched bytes')
        self.manifest = {'schema_version': 1, 'scummvm_id': 'fixture', 'language': 'he', 'version': '1', 'filename': 'fixture.zip', 'platforms': {'windows': {}}, 'files': [
            {'path': 'original.bin', 'operation': 'modify', 'original_sha256': hashlib.sha256(b'original bytes').hexdigest(), 'source': {'kind': 'copy', 'path': 'payload', 'sha256': hashlib.sha256(b'patched bytes').hexdigest()}},
            {'path': 'added.bin', 'operation': 'create', 'source': {'kind': 'copy', 'path': 'payload', 'sha256': hashlib.sha256(b'patched bytes').hexdigest()}}
        ]}

    def run_validator(self, config=None):
        (self.package / 'patch.json').write_text(json.dumps(self.manifest))
        config_path = self.root / 'config.json'
        config_path.write_text(json.dumps(config if config is not None else {'fixture': str(self.game)}))
        report = self.root / 'report.json'
        args = [str(self.binary), str(config_path), str(report), '--workspace', str(self.root / 'patches')]
        if os.name != 'nt':
            args.append('--allow-non-windows')
        result = subprocess.run(args, capture_output=True, text=True)
        self.assertEqual(self.before, {p.name: p.read_bytes() for p in self.game.iterdir()})
        return result, json.loads(report.read_text())

    def test_real_copy_plan_uses_disposable_subset(self):
        result, report = self.run_validator()
        self.assertEqual(result.returncode, 0, result.stderr + "\n" + json.dumps(report, indent=2))
        self.assertEqual(report['results'][0]['status'], 'pass')
        self.assertEqual(report['results'][0]['details']['backedUpFiles'], 1)
        self.assertEqual(report['nativeWindows'], os.name == 'nt')
        self.assertEqual(report['summary']['passed'], 1)

    def test_wrong_original_fails_without_touching_owned_game(self):
        self.manifest['files'][0]['original_sha256'] = '0' * 64
        result, report = self.run_validator()
        self.assertEqual(result.returncode, 1)
        self.assertEqual(report['results'][0]['status'], 'fail', json.dumps(report, indent=2))
        self.assertIn('Unsupported original version: original.bin', report['results'][0].get('error', ''), json.dumps(report, indent=2))
        self.assertEqual(report['summary']['passed'], 0)

    def test_empty_config_does_not_report_success(self):
        result, report = self.run_validator({'fixture': ''})
        self.assertEqual(result.returncode, 1)
        self.assertEqual(report['summary'], {'passed': 0, 'failed': 0, 'skipped': 1})
        self.assertIn('No games validated', result.stderr)


if __name__ == '__main__':
    unittest.main()
