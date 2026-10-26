import unittest

from align_dialogue import align


class AlignmentTests(unittest.TestCase):
    def entry(self, hebrew):
        return {'id': 'ACT_1_room_1_1',
                'review': ['classic_multisegment_speech_requires_alignment'],
                'sources': [{'voice_offset': 123, 'hebrew_raw': hebrew}]}

    def test_repeated_later_cue_does_not_steal_subtitle(self):
        rows = [{'id': f'ACT_1_room_1_{i+1}', 'text': s}
                for i, s in enumerate(['Hello', '*click*', 'Other line', '*click*'])]
        groups, pending, _ = align(rows, [self.entry(r'שלום\xFF\x03*קליק*')],
                                   {123: {b'Hello\xff\x03*click*'}})
        self.assertFalse(pending)
        self.assertEqual([t['index'] for t in groups[0]['targets']], [0, 1])

    def test_skipped_english_line_is_not_silently_matched(self):
        rows = [{'id': f'ACT_1_room_1_{i+1}', 'text': s}
                for i, s in enumerate(['Hello', 'Unexpected', 'Bye'])]
        groups, pending, _ = align(rows, [self.entry(r'שלום\xFF\x03להתראות')],
                                   {123: {b'Hello\xff\x03Bye'}})
        self.assertFalse(groups)
        self.assertEqual(pending[0]['reason'], 'english_sequence_not_unique')

    def test_translator_break_joins_only_when_full_english_matches(self):
        rows = [{'id': 'ACT_1_room_1_1', 'text': 'Hello there'}]
        groups, pending, _ = align(rows, [self.entry(r'שלום\xFF\x03לך')],
                                   {123: {b'Hello there'}})
        self.assertFalse(pending)
        self.assertEqual(groups[0]['targets'][0]['hebrew'], 'שלום\nלך')
        groups, pending, _ = align(rows, [self.entry(r'שלום\xFF\x03לך')],
                                   {123: {b'Hello elsewhere'}})
        self.assertFalse(groups)
        self.assertTrue(pending)


if __name__ == '__main__':
    unittest.main()
