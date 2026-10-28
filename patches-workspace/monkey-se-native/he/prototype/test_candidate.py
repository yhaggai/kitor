import contextlib
import io
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest
from unittest.mock import patch
import build_candidate
from inspect_resources import cstr, speech_rows
from package_candidate import delta


class CandidateTests(unittest.TestCase):
    def test_mi2_package_preserves_executable(self):
        package = Path(__file__).resolve().parents[3] / 'monkey2-se-native/he/patch.json'
        manifest = json.loads(package.read_text())
        self.assertEqual([f['path'] for f in manifest['files']], ['monkey2.pak'])

    def build(self, game, data):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        root = Path(temp.name)
        (root / 'original/fonts').mkdir(parents=True)
        name = 'audio/speech.info' if game == 1 else 'localization/en.speech.info'
        p = root / 'original' / name
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_bytes(data)
        rows = speech_rows(data, game)
        rows[0]['hebrew'] = 'שלום עולם!'
        (root / 'mapping.json').write_text(json.dumps(dict(game=game, rows=rows)), encoding='utf8')
        with patch.object(sys, 'argv', ['build', '--input', str(root), '--font', 'unused']), contextlib.redirect_stdout(io.StringIO()):
            build_candidate.main()
        return (root / 'payload' / (name if game == 1 else 'localization/fr.speech.info')).read_bytes()

    def test_mi1_english_lookup_survives_french_translation(self):
        original = bytearray(1328)
        original[16:22] = b'Hello\0'
        original[272:280] = b'Bonjour\0'
        original[1296:1300] = b'GUY\0'
        result = self.build(1, original)
        self.assertEqual(result[:272], original[:272])
        self.assertEqual(result[528:], original[528:])
        self.assertEqual(result[272:272 + len(build_candidate.encode('שלום עולם!'))], build_candidate.encode('שלום עולם!'))

    def test_mi2_preserves_original_lookup_pointer_and_speech_id(self):
        original = bytearray(36)
        struct.pack_into('<I', original, 0, 1)
        for field, value in ((20, b'GUY'), (24, b'Hello'), (28, b'Hello')):
            ptr = 4 + field
            struct.pack_into('<I', original, ptr, len(original) - ptr)
            original.extend(value + b'\0')
        result = self.build(2, original)
        self.assertEqual(result[:32], original[:32])
        self.assertEqual(result[36:len(original)], original[36:])
        self.assertEqual(speech_rows(result, 2)[0]['id'], 'GUY')
        self.assertNotEqual(speech_rows(result, 2)[0]['text'], 'Hello')

    def test_recipe_roundtrip_with_insertions_and_binary_data(self):
        source = bytes(range(256)) * 3
        target = source[:31] + b'Hebrew' + source[38:] + b'new data'
        ops = delta(source, bytearray(target), 0)
        actual = b''.join(bytes.fromhex(op['hex']) if 'hex' in op else source[op['offset']:op['offset'] + op['length']] for op in ops)
        self.assertEqual(actual, target)

    def test_menu_format_placeholder_survives_bidi(self):
        self.assertIn(b'%iKB', build_candidate.encode('נדרשים %iKB פנויים'))


if __name__ == '__main__':
    unittest.main()
