"""Prevent newly registered commands from silently escaping the application ACL."""
import json
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[1] / 'src-tauri'


class ApplicationSecurityTests(unittest.TestCase):
    def test_every_registered_command_has_an_explicit_main_window_permission(self):
        handlers = set(re.findall(r'commands::\w+::(\w+)', (ROOT / 'src/lib.rs').read_text()))
        declared = set(re.findall(r'"(\w+)"', (ROOT / 'build.rs').read_text()))
        self.assertEqual(handlers, declared)
        capability = json.loads((ROOT / 'capabilities/default.json').read_text())
        self.assertEqual(capability['windows'], ['main'])
        self.assertNotIn('remote', capability)
        granted = {p for p in capability['permissions'] if isinstance(p, str) and p.startswith('allow-')}
        self.assertEqual(granted, {'allow-' + c.replace('_', '-') for c in handlers})
        self.assertFalse(any(isinstance(p, str) and p.startswith('fs:') for p in capability['permissions']))

    def test_production_csp_and_asset_scope_are_restricted(self):
        security = json.loads((ROOT / 'tauri.conf.json').read_text())['app']['security']
        csp = dict(part.strip().split(' ', 1) for part in security['csp'].split(';'))
        self.assertEqual(csp['script-src'], "'self'")
        self.assertEqual(csp['object-src'], "'none'")
        self.assertEqual(csp['base-uri'], "'none'")
        self.assertNotIn('https:', csp['connect-src'])
        self.assertNotIn('$APPDATA/**', security['assetProtocol']['scope'])


if __name__ == '__main__':
    unittest.main()
