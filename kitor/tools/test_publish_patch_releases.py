import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import publish_patch_releases as publisher
from build_patch_packages import InvalidPackage


class PublishingTests(unittest.TestCase):
    def test_nikud_is_an_independent_release_track(self):
        catalog = {'schema_version': 1, 'catalog_revision': 1, 'patches': []}
        plain = dict(self.release(), scummvm_id='undertale')
        pointed = dict(plain, language='he-x-nikud', version='nikud1')
        publisher.merge_release(catalog, plain)
        publisher.merge_release(catalog, pointed)
        publisher.merge_release(catalog, dict(pointed, version='nikud2', release_revision=2))
        self.assertEqual(len(catalog['patches']), 2)
        self.assertEqual(next(p for p in catalog['patches'] if p['language'] == 'he'), plain)

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
            draft = {'id': 42, 'draft': True, 'prerelease': False, 'assets': []}
            def gh(*args, **kwargs):
                if args[:2] == ('release', 'upload'):
                    raise RuntimeError('upload failed')
                return json.dumps(draft)
            with patch.object(publisher, 'optional_api', return_value=draft), patch.object(publisher, 'gh', side_effect=gh) as command:
                with self.assertRaisesRegex(RuntimeError, 'upload failed'):
                    publisher.publish_asset('owner/repo', 'a' * 40, 'tag', artifact, 'notes')
                self.assertFalse(any('PATCH' in call.args for call in command.call_args_list))

    def test_existing_release_asset_is_never_overwritten(self):
        with tempfile.TemporaryDirectory() as temp:
            artifact = Path(temp) / 'patch.zip'
            artifact.write_bytes(b'new bytes')
            release = {'id': 42, 'draft': False, 'prerelease': False, 'assets': [{'name': artifact.name}]}
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

    def test_new_and_existing_drafts_publish_without_by_tag_readback(self):
        for resume in (False, True):
            with self.subTest(resume=resume), tempfile.TemporaryDirectory() as temp:
                artifact = Path(temp) / 'patch.zip'
                artifact.write_bytes(b'payload')
                release = {'id': 42, 'tag_name': 'tag', 'draft': True, 'prerelease': False, 'assets': []}
                def gh(*args, **kwargs):
                    if '--paginate' in args:
                        return json.dumps([[{'tag_name': 'unrelated'}], [release] if resume else []])
                    if 'POST' in args:
                        self.assertFalse(resume, 'Existing draft must be reused')
                        body = json.loads(kwargs['input'])
                        self.assertTrue(body['draft'])
                        self.assertEqual(body['target_commitish'], 'a' * 40)
                        return json.dumps(release)
                    if args[:2] == ('release', 'upload'):
                        release['assets'].append({'name': artifact.name, 'size': artifact.stat().st_size})
                        return ''
                    if 'PATCH' in args:
                        self.assertEqual(args[1], 'repos/owner/repo/releases/42')
                        self.assertEqual(json.loads(kwargs['input']), {'draft': False})
                        self.assertEqual(len(release['assets']), 1)
                        release['draft'] = False
                        return json.dumps(release)
                    self.assertEqual(args, ('api', 'repos/owner/repo/releases/42'))
                    return json.dumps(release)
                with patch.object(publisher, 'optional_api', return_value=None), patch.object(publisher, 'gh', side_effect=gh) as command:
                    publisher.publish_asset('owner/repo', 'a' * 40, 'tag', artifact, 'notes')
                    self.assertEqual(sum('POST' in c.args for c in command.call_args_list), int(not resume))
                    self.assertFalse(release['draft'])

    def test_duplicate_drafts_are_rejected(self):
        pages = [[{'tag_name': 'tag'}], [{'tag_name': 'tag'}]]
        with patch.object(publisher, 'optional_api', return_value=None), patch.object(publisher, 'gh', return_value=json.dumps(pages)):
            with self.assertRaisesRegex(InvalidPackage, 'Multiple releases'):
                publisher.find_release('owner/repo', 'tag')


if __name__ == '__main__':
    unittest.main()
