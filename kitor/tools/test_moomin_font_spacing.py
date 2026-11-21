"""Offline regressions for Hebrew composite ink margins and mark clearance."""
import importlib.util
from pathlib import Path
import sys
import unittest
import unicodedata
from unittest.mock import patch

SOURCE = Path(__file__).resolve().parents[2] / 'patches-workspace/moomin-midsummer-madness/he'
sys.path.insert(0, str(SOURCE))
spec = importlib.util.spec_from_file_location('moomin_layout', SOURCE / 'font_layout.py')
fonts = importlib.util.module_from_spec(spec)
spec.loader.exec_module(fonts)


class FontSpacingTests(unittest.TestCase):
    def test_published_cluster_ids_survive_new_letters_and_unused_legacy_clusters(self):
        old = {'מָ':0xE200, 'שָׁ':0xE201}
        actual = fonts.allocate_cluster_codes(['בַּ', 'מָ'], old)
        for cluster, code in old.items():
            self.assertEqual(actual[unicodedata.normalize('NFD', cluster)], code)
        self.assertEqual(actual[unicodedata.normalize('NFD', 'בַּ')], 0xE202)
        self.assertEqual(fonts.allocate_cluster_codes(['מָ', 'בַּ'], actual), actual)

    def test_duplicate_or_out_of_range_published_codes_are_rejected(self):
        for old in ({'מָ':0xE200, 'בַּ':0xE200}, {'מָ':0xF000}):
            with self.assertRaises(ValueError):fonts.allocate_cluster_codes([], old)

    def test_overhanging_ink_gets_space_on_both_sides(self):
        bearing, advance = fonts.spaced_metrics(-15, 18, (12, 12, 36, 50))
        self.assertGreaterEqual(bearing + 12, 2)
        self.assertGreaterEqual(advance - (bearing + 36), 2)

    def test_wider_original_advance_is_preserved(self):
        self.assertEqual(fonts.spaced_metrics(-10, 60, (12, 12, 36, 50)), (-10, 60))

    def test_dialogue_wrap_reserves_composite_width(self):
        try:
            import build
        except ModuleNotFoundError as error:
            self.skipTest(f'Optional database dependencies unavailable: {error}')
        unit = 'מָ'
        code = build.GLYPHS[unit]
        with patch.object(build, 'DIALOGUE_ADVANCES', {ord('מ'):10, ord(' '):1, code:70}), patch.object(build, 'DIALOGUE_WIDTH', 100):
            lines = build.wrap_dialogue('מ מ', [unit, ' ', unit], 0, {})
        self.assertEqual([line[0] for line in lines], ['מ', 'מ'])

    @unittest.skipUnless(Path('C:/Windows/Fonts/arialbd.ttf').exists(), 'Windows Arial Bold raster check')
    def test_lower_vowel_moves_without_moving_the_letter(self):
        try:
            from build_niqqud_fonts import sdf_tile
        except ModuleNotFoundError as error:
            self.skipTest(f'Optional font raster dependencies unavailable: {error}')
        font = 'C:/Windows/Fonts/arialbd.ttf'
        original, old = sdf_tile('מַ', font, 67)
        adjusted, new = sdf_tile('מַ', font, 67, lower_mark_gap=3)
        self.assertEqual(old[2:], new[2:])
        self.assertEqual(old[0], new[0])
        self.assertEqual(new[1] - old[1], 3)
        # The top of the composite is the letter, unchanged by the vowel offset.
        self.assertEqual(original.crop((0, 0, old[0], 35)).tobytes(), adjusted.crop((0, 0, new[0], 35)).tobytes())


if __name__ == '__main__':
    unittest.main()
