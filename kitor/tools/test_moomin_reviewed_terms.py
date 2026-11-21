import importlib.util
import csv
from pathlib import Path
import re
import unittest

SOURCE = Path(__file__).resolve().parents[2] / 'patches-workspace/moomin-midsummer-madness/he/reviewed_terms.py'
spec = importlib.util.spec_from_file_location('moomin_terms', SOURCE)
terms = importlib.util.module_from_spec(spec)
spec.loader.exec_module(terms)


class ReviewedTermTests(unittest.TestCase):
    def test_singular_plural_and_attached_prefixes(self):
        before = 'שֶׁמִּנַטֵּף הַשֶּׁמִנְטֵף לְשֶׁמְּנְטְפִים וְהַשְּׁמַנְטַפִּים'
        after = terms.apply_reviewed_terms(before)
        self.assertEqual(after, 'שְׁמַנְטַף הַשְּׁמַנְטַף לְשְׁמַנְטַפִים וְהַשְּׁמַנְטַפִים')
        self.assertEqual(after, terms.apply_reviewed_terms(after))
        strip = lambda value: re.sub(r'[\u0591-\u05bd\u05bf\u05c1-\u05c2\u05c4-\u05c5\u05c7]', '', value)
        self.assertEqual(strip(before), strip(after))

    def test_unrelated_text_and_markup_are_preserved(self):
        before = '<bold>שֶׁמִּנַטֵּף</>! {name} 42 מוּמִין'
        self.assertEqual(terms.apply_reviewed_terms(before), '<bold>שְׁמַנְטַף</>! {name} 42 מוּמִין')
        self.assertEqual(terms.apply_reviewed_terms('אכשמנטףא'), 'אכשמנטףא')

    def test_latest_user_name_has_a_persistent_pronunciation(self):
        self.assertEqual(terms.apply_reviewed_terms('לְאוּמְלֶלִית!'), 'לְאוּמְלָלִית!')

    def test_canonical_catalog_keeps_reviewed_readings_and_matching_consonants(self):
        with SOURCE.with_name('catalog.csv').open(encoding='utf-8-sig', newline='') as stream:
            rows = list(csv.DictReader(stream))
        self.assertEqual(len(rows), 4692)
        renamed = 0
        for row in rows:
            pointed, plain = row['he_pointed'], row['he_unpointed']
            self.assertEqual(terms.apply_reviewed_terms(pointed), pointed, row['stable_id'])
            self.assertEqual(re.sub(r'[\u0591-\u05bd\u05bf\u05c1-\u05c2\u05c4-\u05c5\u05c7]', '', pointed), plain, row['stable_id'])
            self.assertNotIn('מיסאבל', plain)
            self.assertNotIn('אומללת', plain)
            renamed += 'אומללית' in plain
        self.assertEqual(renamed, 31)


if __name__ == '__main__':
    unittest.main()
