"""Windows builder integration regression using a tiny synthetic game archive."""
import os
import json
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import unittest

from fontTools.ttLib import TTFont
from build import menu_aliases, movie_translations
from native_language import executable_operations, archive_operations, materialize
import pefile


class BuildTests(unittest.TestCase):
    @unittest.skipUnless(os.environ.get('KITOR_GRIM_APP'), 'Set KITOR_GRIM_APP for native source checks')
    def test_seventh_language_binary_layout(self):
        root = Path(os.environ['KITOR_GRIM_APP'])
        original = (root / 'GrimFandango.exe').read_bytes()
        updated = materialize(original, executable_operations(original))
        pe = pefile.PE(data=updated)
        section = pe.sections[-1]
        self.assertEqual(section.Name.rstrip(b'\0'), b'.hebrew')
        self.assertFalse(section.Characteristics & 0x20000000)  # No executable section.
        names = section.get_data()[:140]
        self.assertEqual([names[i:i+20].rstrip(b'\0') for i in range(0, 140, 20)],
                         [f'grim.{language}.tab'.encode() for language in ['en', 'de', 'es', 'fr', 'it', 'pt', 'he']])
        self.assertEqual(pe.get_data(0x575ae4 - pe.OPTIONAL_HEADER.ImageBase, 4), struct.pack('<f', 7))
        archive = (root / 'DATA000.LAB').read_bytes()
        result = materialize(archive, archive_operations(archive))
        # All preexisting resource bytes stay in place except the selected directory entry.
        self.assertGreater(len(result), len(archive))
        with self.assertRaisesRegex(ValueError, 'Unsupported native executable'):
            executable_operations(original[:-1])

    def test_movie_subtitles_are_source_guarded(self):
        source = 'intma01\tSorry for the wait, Mr. Flores.\r\n'
        supplement = json.loads(Path(__file__).with_name('movie-he.json').read_text(encoding='utf-8'))
        self.assertEqual(len(supplement), 401)
        self.assertEqual(movie_translations(source, supplement)['intma01'], 'סליחה על ההמתנה, מר פלורס.')
        with self.assertRaisesRegex(ValueError, 'source changed'):
            movie_translations(source.replace('Mr. Flores.', 'someone else.'), supplement)
        for entry in supplement.values():
            self.assertTrue(any('\u0590' <= c <= '\u05ff' for c in entry['he']))

    def test_menu_reuse_is_unambiguous_and_does_not_remap_dialogue(self):
        source = ('sytx240\t&Options\r\nsytx249\tOptions\r\n'
                  'sytx935\tOPTIONS\r\nutxt010\tOptions\r\n'
                  'dialogue1\tOptions\r\nsytx999\tOptions...\r\n')
        translations = {'sytx240': 'אפשרויות', 'sytx249': 'אפשרויות'}
        self.assertEqual(menu_aliases(source, translations), {
            'sytx935': ['sytx240', 'sytx249'], 'utxt010': ['sytx240', 'sytx249']})
        translations['sytx249'] = 'different translation'
        self.assertEqual(menu_aliases(source, translations), {})

    def test_native_resources_and_line_endings(self):
        font = Path(os.environ.get('WINDIR', 'C:/Windows')) / 'Fonts/arial.ttf'
        if not font.is_file():
            self.skipTest('Requires a local Hebrew Arial font; no font is redistributed.')
        builder = Path(__file__).with_name('build.py')
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            game = root / 'source'
            fonts = game / 'FontsHD'
            fonts.mkdir(parents=True)
            original = b'#MenusAndUI\r\nmenu\tNew Game\r\nline\tHello\r\n'
            (game / 'grim.en.tab').write_bytes(original)
            (game / 'resource.lab').write_bytes(b'unchanged-resource')
            (fonts / 'FuturaStd-Heavy.ttf').write_bytes(b'original-font-placeholder')
            (fonts / 'FUTURA40.LAF.txt').write_bytes(b'FuturaStd-Heavy.ttf 40px\r\n')
            translated = 'line\tשלום\r\n'.encode('cp1255')
            payload = b'RCNE' + bytes(c ^ 0xdd for c in translated)
            name = b'grim.tab\0'
            patch = root / 'datausr.lab'
            patch.write_bytes(b'LABN' + struct.pack('<III', 0, 1, len(name))
                              + struct.pack('<IIII', 0, 32 + len(name), len(payload), 0)
                              + name + payload)
            command = [sys.executable, str(builder), '--game', str(game), '--patch', str(patch),
                       '--font', str(font), '--resources', 'copy', '--output']
            output = root / 'output'
            subprocess.run(command + [str(output)], check=True, capture_output=True)
            text = (output / 'grim.en.tab').read_bytes()
            self.assertIn(b'menu\tNew Game\r\n', text)
            self.assertEqual(text.count(b'\n'), text.count(b'\r\n'))
            self.assertNotIn(b'Hello', text)
            self.assertEqual((game / 'grim.en.tab').read_bytes(), original)
            self.assertEqual((output / 'resource.lab').read_bytes(), b'unchanged-resource')
            self.assertEqual((output / 'FontsHD/FUTURA40.LAF.txt').read_bytes(),
                             b'HebrewPrototype.ttf 40px\r\n')
            with TTFont(output / 'FontsHD/FuturaStd-Heavy.ttf') as generated:
                self.assertFalse(set(text) - set(generated.getBestCmap()) - {9, 10, 13})
            rejected = subprocess.run(command + [str(game / 'nested')], capture_output=True)
            self.assertNotEqual(rejected.returncode, 0)
            self.assertFalse((game / 'nested').exists())


if __name__ == '__main__':
    unittest.main()
