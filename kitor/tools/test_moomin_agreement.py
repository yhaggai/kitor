import csv
import json
from pathlib import Path
import re
import unittest

BASE = Path(__file__).resolve().parents[2] / 'patches-workspace/moomin-midsummer-madness/he'


class AgreementTests(unittest.TestCase):
    def test_reviewed_corrections_are_in_the_canonical_catalog(self):
        with (BASE/'catalog.csv').open(encoding='utf-8-sig', newline='') as stream:
            rows = {row['stable_id']:row for row in csv.DictReader(stream)}
        report = json.loads((BASE/'agreement-review.json').read_text(encoding='utf-8'))
        self.assertEqual(report['corrected_rows'], len(report['corrections']))
        for correction in report['corrections']:
            row = rows[correction['stable_id']]
            self.assertEqual(row['en'], correction['en'])
            self.assertEqual(row['he_unpointed'], correction['after_unpointed'])
            self.assertEqual(row['he_pointed'], correction['after_pointed'])
            self.assertEqual(re.sub(r'[\u0591-\u05bd\u05bf\u05c1-\u05c2\u05c4-\u05c5\u05c7]', '', row['he_pointed']), row['he_unpointed'])

    def test_shared_woodie_label_does_not_merge_different_characters(self):
        traits = json.loads((BASE/'speaker-agreement.json').read_text(encoding='utf-8'))['speakers']
        context = json.loads((BASE/'speaker-context.json').read_text(encoding='utf-8'))
        self.assertEqual(set(traits), set(context['characters']))
        self.assertEqual(traits['character_bushkid']['gender'], 'masculine')
        self.assertEqual(traits['character_trainer']['gender'], 'feminine')
        self.assertEqual(traits['character_artist']['basis'], 'documented grammatical convention')
        self.assertEqual(traits['character_someone']['gender'], 'unspecified')
        self.assertTrue(all(trait['number'] == 'singular' for trait in traits.values()))

    def test_female_speaker_and_mixed_group_address_are_distinct(self):
        with (BASE/'catalog.csv').open(encoding='utf-8-sig', newline='') as stream:
            rows = list(csv.DictReader(stream))
        def text(scene, phrase):
            return next(row['he_unpointed'] for row in rows
                        if json.loads(row['path']) == [scene, 'phrases', phrase, 'phrase', 'text'])
        self.assertEqual(text('dialogue_chapter2_pier_cutscene_both_boarding_pier', 0), 'אני אוהבת ספינה כזאת!')
        self.assertIn('מצאתם', text('dialogue_chapter4_stage_repeatquest_mamma', 0))
        collectives = json.loads((BASE/'speaker-agreement.json').read_text(encoding='utf-8'))['collectives']
        self.assertEqual(collectives['police_chief_disguise']['role_number'], 'singular')
        self.assertEqual(collectives['police_chief_disguise']['participant_number'], 'plural')


if __name__ == '__main__':unittest.main()
