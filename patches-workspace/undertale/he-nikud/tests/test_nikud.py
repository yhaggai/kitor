import importlib.util
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from nikud import encode, glyph_map, strip_points
from vowelize_catalog import vowelize

spec = importlib.util.spec_from_file_location('reference', ROOT/'full-patch/hebrew-runtime/reference.py')
reference = importlib.util.module_from_spec(spec)
spec.loader.exec_module(reference)


class NikudTests(unittest.TestCase):
    def test_atomic_bidi_and_roundtrip(self):
        text = 'שָׁלוֹם 123 ABC!'
        mapping = glyph_map({'entries': [{'hebrew': text}]})
        encoded = encode(text, mapping)
        self.assertEqual(len(encoded), len(strip_points(text)))
        self.assertTrue(all(reference.typ(c) == 1 for c in mapping.values()))
        inverse = {v: k for k, v in mapping.items()}
        import unicodedata
        self.assertEqual(''.join(inverse.get(c, c) for c in encoded), unicodedata.normalize('NFD', text))
        self.assertIn('123', reference.visual(encoded))
        self.assertIn('ABC', reference.visual(encoded))

    def test_normalization_and_deterministic_map(self):
        a = {'entries': [{'hebrew': 'שָׁ'}, {'hebrew': 'שָׁ'}]}
        mapping = glyph_map(a)
        self.assertEqual(len(mapping), 1)
        self.assertEqual(encode('שָׁ', mapping), encode('שָׁ', mapping))
        self.assertEqual(mapping, glyph_map({'entries': list(reversed(a['entries']))}))

    def test_bad_clusters(self):
        for text in ('ָא', 'aָ', '\uf000'):
            with self.assertRaises(ValueError):
                glyph_map({'entries': [{'hebrew': text}]})

    def test_latin_accents_and_controls_unchanged(self):
        text = r'\E0* שָׁלוֹם café ^3/%%'
        mapping = glyph_map({'entries': [{'hebrew': text}]})
        result = encode(text, mapping)
        self.assertIn('café', result)
        self.assertTrue(result.startswith(r'\E0* '))
        self.assertTrue(result.endswith(' ^3/%%'))

    def test_wrapping_and_reveal_match_unpointed_letters(self):
        pointed = r'\E1* שָׁלוֹם^1 עוֹלָם/%%'
        mapping = glyph_map({'entries': [{'hebrew': pointed}]})
        a, meta_a = reference.layout(encode(pointed, mapping), 8)
        b, meta_b = reference.layout(strip_points(pointed), 8)
        self.assertEqual(meta_a, meta_b)
        for i in a:
            self.assertEqual({k:v for k,v in a[i].items() if k != 'char'},
                             {k:v for k,v in b[i].items() if k != 'char'})

    def test_only_transfer_points(self):
        text = r'\E0* שלום  &  עולם^3/%% \[שם]'
        result = vowelize(text, lambda _: 'שָׁלוֹם עוֹלָם')
        self.assertEqual(strip_points(result), text)
        self.assertIn(r'\[שם]', result)
        self.assertIn('  &  ', result)

    def test_model_cannot_change_letters(self):
        with self.assertRaises(ValueError):
            vowelize('שלום', lambda _: 'שָׁלוֹן')

    def test_existing_points_are_preserved(self):
        self.assertEqual(vowelize('שָׁלוֹם', lambda _: self.fail('model called')), 'שָׁלוֹם')


if __name__ == '__main__':
    unittest.main()
