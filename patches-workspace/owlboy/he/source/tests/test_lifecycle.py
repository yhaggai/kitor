import importlib.util
import json
import os
from pathlib import Path
import stat
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('manage', Path(__file__).resolve().parents[1] / 'tools/manage.py')
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)


class Lifecycle(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        root = Path(self.tmp.name).resolve()
        self.game, self.package, self.recovery = (root / x for x in ('game', 'package', 'recovery'))
        self.game.mkdir(); (self.package / 'payload').mkdir(parents=True)
        self.originals = {'Owlboy.exe': b'original executable', 'Data/existing': b'original data'}
        self.changed = {'Owlboy.exe': b'patched executable', 'Data/existing': b'new data', 'Data/he/new': b'new file'}
        for name, contents in self.originals.items():
            p = self.game / name; p.parent.mkdir(parents=True, exist_ok=True); p.write_bytes(contents)
        (self.game / 'untouched').write_bytes(b'keep me')
        os.chmod(self.game / 'Owlboy.exe', 0o755)
        self.plan = {'schema': 1, 'id': 'owlboy-hebrew', 'version': 'test', 'source_exe_sha256': m.digest(self.originals['Owlboy.exe']), 'files': []}
        for name, contents in self.changed.items():
            p = self.package / 'payload' / name; p.parent.mkdir(parents=True, exist_ok=True); p.write_bytes(contents)
            self.plan['files'].append({'path': name, 'original_sha256': m.digest(self.originals[name]) if name in self.originals else None, 'installed_sha256': m.digest(contents)})
        self.write_plan()

    def write_plan(self):
        (self.package / 'plan.json').write_text(json.dumps(self.plan))

    def install(self):
        return m.install(self.package, self.game, self.recovery)

    def assert_original(self):
        for name, data in self.originals.items():
            self.assertEqual((self.game / name).read_bytes(), data)
        self.assertFalse((self.game / 'Data/he/new').exists())
        self.assertEqual((self.game / 'untouched').read_bytes(), b'keep me')

    def test_selective_roundtrip_idempotent_mode_and_later_files(self):
        journal = self.install(); self.install()
        self.assertEqual(len(list((self.recovery / 'backups').iterdir())), 2)
        self.assertEqual((self.recovery / 'backups/000000').read_bytes(), self.originals['Owlboy.exe'])
        (self.game / 'Data/he/later').write_bytes(b'other mod')
        if os.name != 'nt':
            self.assertEqual(stat.S_IMODE((self.game / 'Owlboy.exe').stat().st_mode), 0o755)
        m.uninstall(journal, self.game); m.uninstall(journal, self.game)
        self.assert_original()
        self.assertEqual((self.game / 'Data/he/later').read_bytes(), b'other mod')
        if os.name != 'nt':
            self.assertEqual(stat.S_IMODE((self.game / 'Owlboy.exe').stat().st_mode), 0o755)

    def test_changed_after_install_refused_before_any_restore(self):
        journal = self.install()
        (self.game / 'Data/he/new').write_bytes(b'user changed')
        with self.assertRaises(ValueError): m.uninstall(journal, self.game)
        self.assertEqual((self.game / 'Owlboy.exe').read_bytes(), self.changed['Owlboy.exe'])
        with self.assertRaises(ValueError): self.install()

    def test_missing_and_corrupt_backup_refused(self):
        journal = self.install(); backup = self.recovery / 'backups/000001'
        backup.unlink()
        with self.assertRaises(ValueError): m.uninstall(journal, self.game)
        backup.write_bytes(b'corrupt')
        with self.assertRaises(ValueError): m.uninstall(journal, self.game)
        self.assertEqual((self.game / 'Owlboy.exe').read_bytes(), self.changed['Owlboy.exe'])

    def test_original_conflict(self):
        (self.game / 'Data/existing').write_bytes(b'other mod')
        with self.assertRaises(ValueError): self.install()
        self.assertEqual((self.game / 'Owlboy.exe').read_bytes(), self.originals['Owlboy.exe'])

    def test_payload_corruption(self):
        (self.package / 'payload/Owlboy.exe').write_bytes(b'corrupt')
        with self.assertRaises(ValueError): self.install()
        self.assert_original()

    def test_traversal_and_duplicate_refused(self):
        for bad in ('../escape', '/absolute', 'C:/escape', 'a\\b', 'a/../b', 'a//b', 'CON'):
            with self.subTest(path=bad):
                self.plan['files'][-1]['path'] = bad; self.write_plan()
                with self.assertRaises(ValueError): self.install()
                self.assert_original()
        self.plan['files'][-1]['path'] = 'OWLBOY.EXE'; self.write_plan()
        with self.assertRaises(ValueError): self.install()

    @unittest.skipUnless(hasattr(os, 'symlink'), 'No symlinks')
    def test_symlink_refusal(self):
        target = self.game / 'Data/existing'; target.unlink()
        try: target.symlink_to(self.game / 'untouched')
        except OSError: self.skipTest('Symlinks unavailable for this account')
        with self.assertRaises(ValueError): self.install()
        self.assertEqual((self.game / 'untouched').read_bytes(), b'keep me')

    def interrupt_after_first_replacement(self):
        real = m.atomic
        def interrupt(path, data, mode=None):
            real(path, data, mode)
            if path == self.game / 'Owlboy.exe':
                raise KeyboardInterrupt('simulated sudden interruption')
        with patch.object(m, 'atomic', side_effect=interrupt):
            with self.assertRaises(KeyboardInterrupt): self.install()
        return self.recovery / 'journal.json'

    def test_interrupted_install_retry(self):
        journal = self.interrupt_after_first_replacement()
        self.install()
        self.assertEqual(m.load(journal, self.game)['status'], 'installed')
        m.uninstall(journal, self.game); self.assert_original()

    def test_interrupted_install_rollback(self):
        journal = self.interrupt_after_first_replacement()
        m.uninstall(journal, self.game); self.assert_original()

    def test_interrupted_uninstall_retry(self):
        journal = self.install(); real = m.atomic
        def interrupt(path, data, mode=None):
            real(path, data, mode)
            if path == self.game / 'Owlboy.exe': raise OSError('interruption')
        with patch.object(m, 'atomic', side_effect=interrupt):
            with self.assertRaises(OSError): m.uninstall(journal, self.game)
        m.uninstall(journal, self.game); self.assert_original()

    def test_normal_failure_automatically_rolls_back_only_changed_files(self):
        real = m.atomic
        untouched_time = (self.game / 'Data/existing').stat().st_mtime_ns
        failed = False
        def fail_once(path, data, mode=None):
            nonlocal failed
            if path == self.game / 'Data/existing' and not failed:
                failed = True
                raise OSError('ordinary write failure')
            return real(path, data, mode)
        with patch.object(m, 'atomic', side_effect=fail_once):
            with self.assertRaises(OSError): self.install()
        self.assert_original()
        self.assertEqual((self.game / 'Data/existing').stat().st_mtime_ns, untouched_time)
        self.assertEqual(m.load(self.recovery / 'journal.json', self.game)['status'], 'uninstalled')

    def test_created_directories_removed_only_when_empty(self):
        journal = self.install()
        self.assertTrue((self.game / 'Data/he').is_dir())
        m.uninstall(journal, self.game)
        self.assertFalse((self.game / 'Data/he').exists())
        self.assertTrue((self.game / 'Data').is_dir())
        self.assertTrue(self.game.is_dir())

    def test_created_directory_with_later_nested_file_survives(self):
        journal = self.install()
        later = self.game / 'Data/he/unrelated/nested/file'
        later.parent.mkdir(parents=True); later.write_bytes(b'keep')
        m.uninstall(journal, self.game)
        self.assertEqual(later.read_bytes(), b'keep')
        self.assertFalse((self.game / 'Data/he/new').exists())

    def test_bad_source_hash(self):
        self.plan['source_exe_sha256'] = 'not-a-hash'; self.write_plan()
        with self.assertRaises(ValueError): self.install()
        self.assert_original()

    def test_journal_integrity_and_wrong_game(self):
        journal = self.install()
        data = json.loads(journal.read_text()); data['data']['records'][0]['mode'] = 0
        journal.write_text(json.dumps(data))
        with self.assertRaises(ValueError): m.uninstall(journal, self.game)

    def test_unowned_patched_files_refused(self):
        self.install()
        with self.assertRaises(ValueError): m.install(self.package, self.game, self.recovery.parent / 'other-recovery')


if __name__ == '__main__':
    unittest.main()
