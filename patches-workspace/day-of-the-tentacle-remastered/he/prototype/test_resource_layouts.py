import struct
import unittest

from map_dialogue import rebuild, speech_rows
from map_item_labels import rebuild_ui, ui_rows


class ResourceLayoutTests(unittest.TestCase):
    def test_speech_layouts_preserve_metadata_and_trailing_data(self):
        for size, stride in [(4, 32), (8, 40)]:
            with self.subTest(pointer_size=size):
                fmt = '<I' if size == 4 else '<Q'
                source = bytearray(size + stride)
                struct.pack_into(fmt, source, 0, 1)
                source[size:size+24] = bytes(range(1, 25))
                for field, text in [(24, b'VOICE_ID'), (stride-size, b'Original')]:
                    ptr = size + field
                    struct.pack_into(fmt, source, ptr, len(source)-ptr)
                    source.extend(text+b'\0')
                source.extend(b'unreferenced trailing data\0')
                rows = speech_rows(source)
                self.assertEqual(rebuild(source, rows), source)
                rows[0]['text'] = 'A longer translated sentence'
                result = rebuild(source, rows)
                self.assertEqual(speech_rows(result), rows)
                self.assertEqual(result[size:size+24], source[size:size+24])
                self.assertTrue(result.endswith(b'unreferenced trailing data\0'))

    def test_ui_layouts_preserve_keys_and_trailing_data(self):
        for size in (4, 8):
            with self.subTest(pointer_size=size):
                fmt = '<I' if size == 4 else '<Q'
                source = bytearray(size*3)
                struct.pack_into(fmt, source, 0, 1)
                for index, text in enumerate([b'Clock', b'Clock']):
                    ptr = size+index*size
                    struct.pack_into(fmt, source, ptr, len(source)-ptr)
                    source.extend(text+b'\0')
                source.extend(b'unused\0')
                rows = ui_rows(source)
                self.assertEqual(rebuild_ui(rows, source), source)
                rows[0][1] = b'Longer label'
                result = rebuild_ui(rows, source)
                self.assertEqual(ui_rows(result), rows)
                self.assertTrue(result.endswith(b'unused\0'))
