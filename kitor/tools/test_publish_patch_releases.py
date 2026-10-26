import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import publish_patch_releases as publisher
from build_patch_packages import InvalidPackage


class PublishingTests(unittest.TestCase):
    def release(self):
        return {'scummvm_id': 'sq1', 'language': 'he', 'version': 'RC1-kitor2',
                'release_revision': 1, 'filename': 'patch.zip', 'checksum': 'a' * 64}

    def test_revision_and_immutable_metadata(self):
        catalog = {'schema_version': 1, 'catalog_revision': 1, 'patches': []}
        first = self.release()
        self.assertTrue(publisher.merge_release(catalog, first))
        self.assertFalse(publisher.merge_release(catalog, copy.deepcopy(first)))
        for change in [{'release_revision': 0}, {'release_revision': True},
                       {'version': 'different'}, {'checksum': 'b' * 64},
                       {'release_revision': 2}, {'filename': '../escape.zip'}]:
            with self.subTest(change=change), self.assertRaises(InvalidPackage):
                publisher.merge_release(catalog, dict(first, **change))
        second = dict(first, version='RC2-kitor1', release_revision=2)
        self.assertTrue(publisher.merge_release(catalog, second))
        with self.assertRaises(InvalidPackage):
            publisher.merge_release(catalog, first)

    def test_upload_failure_never_publishes_draft(self):
        with tempfile.TemporaryDirectory() as temp:
            artifact = Path(temp) / 'patch.zip'
            artifact.write_bytes(b'payload')
            draft = {'draft': True, 'prerelease': False, 'assets': []}
            def gh(*args, **kwargs):
                if args[:2] == ('release', 'upload'):
                    raise RuntimeError('upload failed')
                return json.dumps(draft)
            with patch.object(publisher, 'optional_api', return_value=draft), patch.object(publisher, 'gh', side_effect=gh) as command:
                with self.assertRaisesRegex(RuntimeError, 'upload failed'):
                    publisher.publish_asset('owner/repo', 'a' * 40, 'tag', artifact, 'notes')
                self.assertFalse(any(call.args[:2] == ('release', 'edit') for call in command.call_args_list))

    def test_existing_release_asset_is_never_overwritten(self):
        with tempfile.TemporaryDirectory() as temp:
            artifact = Path(temp) / 'patch.zip'
            artifact.write_bytes(b'new bytes')
            release = {'draft': False, 'prerelease': False, 'assets': [{'name': artifact.name}]}
            def gh(*args, **kwargs):
                if args[:2] == ('release', 'download'):
                    Path(args[args.index('--dir') + 1], artifact.name).write_bytes(b'old bytes')
                return ''
            with patch.object(publisher, 'optional_api', return_value=release), patch.object(publisher, 'gh', side_effect=gh) as command:
                with self.assertRaisesRegex(InvalidPackage, 'Immutable release'):
                    publisher.publish_asset('owner/repo', 'a' * 40, 'tag', artifact, 'notes')
                self.assertFalse(any(call.args[:2] == ('release', 'upload') for call in command.call_args_list))

    def test_published_incomplete_release_is_rejected(self):
        with tempfile.TemporaryDirectory() as temp:
            artifact = Path(temp) / 'patch.zip'
            artifact.write_bytes(b'payload')
            with patch.object(publisher, 'optional_api', return_value={'draft': False, 'prerelease': False, 'assets': []}):
                with self.assertRaisesRegex(InvalidPackage, 'incomplete'):
                    publisher.publish_asset('owner/repo', 'a' * 40, 'tag', artifact, 'notes')


if __name__ == '__main__':
    unittest.main()
