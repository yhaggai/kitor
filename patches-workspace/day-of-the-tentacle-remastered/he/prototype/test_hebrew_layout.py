import unittest
from build_full_hebrew import prepare_text


class LayoutTests(unittest.TestCase):
    def test_hebrew_punctuation(self):
        self.assertEqual(prepare_text('שלום!', {}), '!םולש')

    def test_numbers_and_english_keep_their_order(self):
        result = prepare_text('בדיקה ABC 123!', {})
        self.assertIn('ABC 123', result)
        self.assertTrue(result.endswith('הקידב'))

    def test_wrap_before_reordering(self):
        widths = {c: 1 for c in 'שלום עולם '}
        self.assertEqual(prepare_text('שלום עולם', widths, limit=5), 'םולש\nםלוע')

    def test_explicit_blank_line(self):
        self.assertEqual(prepare_text('שלום\n\nעולם', {}), 'םולש\n\nםלוע')


if __name__ == '__main__':
    unittest.main()
