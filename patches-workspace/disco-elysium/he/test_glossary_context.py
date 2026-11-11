import unittest
import glossary_context as g

class Selection(unittest.TestCase):
    def setUp(self):
        self.glossary = {'entries': [
            {'english': 'Revachol', 'hebrew': 'רבאשול', 'voice': 'long irrelevant note'},
            {'english': 'Kim Kitsuragi', 'hebrew': 'קים', 'speaker_id': 395, 'voice': 'Dry', 'gender': 'masculine'},
            {'english': 'Seol', 'hebrew': 'סאול', 'aliases': ['Seolite'], 'localized_forms': {'language': 'סאולית'}},
            {'english': 'Joyce', 'hebrew': 'ג׳ויס', 'speaker_id': 100, 'voice': 'Formal'}]}
    def test_alias_and_addressee_selection(self):
        rows = [{'english': 'A Seolite.', 'context': [{'speaker_id': 395, 'addressee_id': 100}]}]
        by_name = {e['english']: e for e in g.select(self.glossary, rows)}
        self.assertEqual(by_name['Joyce']['voice'], 'Formal')
        self.assertEqual(by_name['Seol']['localized_forms']['language'], 'סאולית')
        self.assertNotIn('voice', by_name['Revachol'])
    def test_word_boundaries_and_unrelated_exclusion(self):
        result = g.select(self.glossary, [{'english': 'Joyceful Seolites', 'context': []}])
        self.assertNotIn('Joyce', [e['english'] for e in result])
        self.assertNotIn('Seol', [e['english'] for e in result])
    def test_audit_accepts_language_forms_and_flags_missing_term(self):
        self.assertEqual(g.audit(self.glossary, [{'key': 'a', 'english': 'Seolite', 'hebrew': 'סאולית'}]), [])
        flags = g.audit(self.glossary, [{'key': 'b', 'english': 'Seolite', 'hebrew': 'משהו אחר'}])
        self.assertEqual(flags[0]['key'], 'b')
    def test_canonical_unchanged(self):
        import copy
        before = copy.deepcopy(self.glossary)
        g.render(self.glossary, [{'english': 'Seolite', 'context': []}])
        self.assertEqual(before, self.glossary)
if __name__ == '__main__':
    unittest.main()
