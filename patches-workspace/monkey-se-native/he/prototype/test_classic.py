import struct
import unittest

from build_classic import classic_font, translate, translation_map
from build_candidate import ENCODING, encode


class ClassicTests(unittest.TestCase):
    def test_speech_boundaries_and_unknown_text_survive(self):
        self.assertEqual(translate(b'Hello\xff\x03Unknown', {b'Hello': b'HE'}), b'HE\xff\x03Unknown')

    def test_repeated_phrases_use_an_existing_consistent_translation(self):
        rows = [dict(text='Same', hebrew='שלום'), dict(text='Same', hebrew='עולם')]
        self.assertEqual(translation_map(rows)[b'Same'], encode('שלום'))

    def test_classic_pause_and_spacing_match_native_ellipsis(self):
        mapping = translation_map([dict(text='Well...  hello', hebrew='שלום')])
        self.assertEqual(translate(b'Well^ hello', mapping), encode('שלום'))

    def test_bitmap_slots_alias_existing_hebrew_glyphs(self):
        data = bytearray(8 + 25 + 256 * 4 + 128)
        struct.pack_into('<H', data, 8 + 23, 256)
        for code in range(224, 251):
            struct.pack_into('<I', data, 8 + 25 + code * 4, 1028 + code - 224)
        result = classic_font(data)
        for char, slot in ENCODING.items():
            expected = struct.unpack_from('<I', data, 8 + 25 + char.encode('cp1255')[0] * 4)[0]
            self.assertEqual(struct.unpack_from('<I', result, 25 + slot * 4)[0], expected)


if __name__ == '__main__':
    unittest.main()
