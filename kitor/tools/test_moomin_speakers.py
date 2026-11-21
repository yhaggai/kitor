import copy
import importlib.util
import json
from pathlib import Path
import unittest

SOURCE = Path(__file__).resolve().parents[2] / 'patches-workspace/moomin-midsummer-madness/he/merge_speakers.py'
spec = importlib.util.spec_from_file_location('moomin_speakers', SOURCE)
speakers = importlib.util.module_from_spec(spec)
spec.loader.exec_module(speakers)


class SpeakerTests(unittest.TestCase):
    def test_character_reference_overrides_misleading_scene_name_without_editing_text(self):
        path = ['dialogue_moomin', 'phrases', 0, 'phrase', 'text']
        row = {'path':json.dumps(path), 'stable_id':'fixture', 'speaker':'Moomintroll',
               'en':'Hello!', 'he_pointed':'שָׁלוֹם!', 'he_unpointed':'שלום!'}
        previous = copy.deepcopy(row)
        database = {'dialogue_moomin':{'phrases':[{'character_card':'little_my',
                     'emotion':'happy', 'phrase':{'text':'Hello!'}}]},
                    'little_my':{'character_name':{'text':'Little My'}}}
        context = speakers.merge_speakers(database, [row])
        self.assertEqual(row, dict(previous, speaker='Little My'))
        self.assertEqual(context['phrases']['fixture']['character_card'], 'little_my')
        self.assertEqual(context['phrases']['fixture']['emotion'], 'happy')
        self.assertEqual(speakers.merge_speakers(database, [row])['speaker_labels_changed'], 0)

    def test_unknown_character_fails_instead_of_guessing(self):
        row = {'path':json.dumps(['scene','phrases',0,'phrase','text']), 'stable_id':'fixture', 'en':'Hello!'}
        database = {'scene':{'phrases':[{'character_card':'missing','phrase':{'text':'Hello!'}}]}}
        with self.assertRaises(KeyError):speakers.merge_speakers(database, [row])

    def test_persisted_context_matches_every_csv_speaker(self):
        import csv
        context = json.loads(SOURCE.with_name('speaker-context.json').read_text(encoding='utf-8'))
        self.assertEqual(len(context['phrases']), 3866)
        with SOURCE.with_name('catalog.csv').open(encoding='utf-8-sig', newline='') as stream:
            rows = {row['stable_id']:row for row in csv.DictReader(stream)}
        for identity, phrase in context['phrases'].items():
            self.assertEqual(rows[identity]['speaker'], context['characters'][phrase['character_card']])
        key = json.dumps(['dialogue_chapter2_pier_cutscene_both_boarding_pier','phrases',0,'phrase','text'],separators=(',',':'))
        self.assertEqual(context['phrases'][key]['character_card'], 'character_little_my')
        self.assertEqual(context['phrases'][key]['emotion'], 'happy')


if __name__ == '__main__':unittest.main()
