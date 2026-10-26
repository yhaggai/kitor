import hashlib
import json
from pathlib import Path
import re
import tempfile
import textwrap
import unittest

from prepare_patch_releases import prepare, release_metadata

WORKFLOWS = Path(__file__).resolve().parents[2] / '.github/workflows'
WORKFLOW = (WORKFLOWS / 'deploy-patches.yml').read_text()
SCRIPT = textwrap.dedent(WORKFLOW.split("          python3 - <<'PY'\n", 1)[1].rsplit('          PY', 1)[0])
NAMESPACE = {'__name__': 'publisher_test'}
exec(compile(SCRIPT, 'publisher', 'exec'), NAMESPACE)


class ReleaseSecurityTests(unittest.TestCase):
    def test_approved_package_reaches_publisher_as_data_only(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            package = root / 'workspace/game/he'
            package.mkdir(parents=True)
            (package / 'data').write_bytes(b'patch')
            manifest = {'schema_version': 1, 'scummvm_id': 'game', 'language': 'he',
                        'version': '1', 'filename': 'game.zip', 'platforms': {'windows': {}, 'osx': {}},
                        'files': [{'path': 'game.dat', 'operation': 'install', 'source': {
                            'kind': 'copy', 'path': 'data', 'sha256': hashlib.sha256(b'patch').hexdigest()}}]}
            (package / 'patch.json').write_text(json.dumps(manifest))
            prepare(root / 'workspace', root / 'unapproved')
            self.assertEqual(NAMESPACE['validated_releases'](root / 'unapproved'), [])
            (package / 'provenance.json').write_text(json.dumps({'release_ready': True}))
            prepare(root / 'workspace', root / 'approved')
            entries = NAMESPACE['validated_releases'](root / 'approved')
            self.assertEqual(entries[0]['filename'], 'game.zip')

    def test_publisher_rejects_paths_options_and_modified_assets(self):
        validate = NAMESPACE['validated_releases']
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'patch.zip').write_bytes(b'archive')
            entry = {'tag': 'patch-game-he-v1', 'filename': 'patch.zip',
                     'sha256': hashlib.sha256(b'archive').hexdigest()}
            index = root / 'releases.json'
            index.write_text(json.dumps([entry]))
            self.assertEqual(validate(root), [entry])
            for key, value in [('filename', '../patch.zip'), ('filename', '/tmp/patch.zip'),
                               ('filename', '--help.zip'), ('tag', '--help'),
                               ('tag', 'patch-$(command)'), ('sha256', '0' * 64)]:
                index.write_text(json.dumps([dict(entry, **{key: value})]))
                with self.assertRaises(ValueError):
                    validate(root)
            index.write_text(json.dumps([entry, entry]))
            with self.assertRaises(ValueError):
                validate(root)

    def test_build_metadata_cannot_control_paths_or_shell(self):
        good = {'scummvm_id': 'game', 'language': 'he', 'version': '1.0', 'filename': 'patch.zip'}
        self.assertEqual(release_metadata(good)['tag'], 'patch-game-he-v1.0')
        for key in good:
            with self.assertRaises(ValueError):
                release_metadata(dict(good, **{key: '../bad'}))

    def test_writer_does_not_checkout_or_execute_package_code(self):
        publisher = WORKFLOW.split('  publish:', 1)[1]
        self.assertNotIn('actions/checkout', publisher)
        self.assertNotIn('python kitor/', publisher)
        self.assertIn('needs: build-and-release-patches', publisher)
        build = WORKFLOW.split('  build-and-release-patches:', 1)[1].split('  publish:', 1)[0]
        self.assertIn('needs: validate', build)
        self.assertIn('contents: read', build)
        self.assertEqual(WORKFLOW.count('contents: write'), 1)
        self.assertIn('workflow_call:', (WORKFLOWS / 'validate-patch-packages.yml').read_text())
        for path in WORKFLOWS.glob('*.yml'):
            for action in re.findall(r'uses: ([^\s#]+)', path.read_text()):
                if not action.startswith('./'):
                    self.assertRegex(action, r'@\w{40}$')


if __name__ == '__main__':
    unittest.main()
