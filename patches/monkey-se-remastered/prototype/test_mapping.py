import unittest
from map_dialogue import map_hebrew
from resources import STRIDE,replace_english,speech_rows


def record(name='GUY_1',english=b'Hello'):
    return bytes(range(16))+b''.join(s+b'\0'+b' '*(255-len(s)) for s in [english,b'French',b'German',b'Italian',b'Spanish'])+name.encode().ljust(32,b'\0')


def source(text):
    return (r'\xFF\x0A\x08\x00\xFF\x0A\x00\x00\xFF\x0A\x0A\x00\xFF\x0A\x00\x00'+text).encode('cp1255')


class MappingTests(unittest.TestCase):
    def test_preservation(self):
        original = record()+record('SECOND')
        changed = replace_english(original,{0:b'New text'})
        self.assertEqual(changed[:16],original[:16])
        self.assertEqual(changed[272:],original[272:])
        self.assertEqual(len(changed),len(original))
        self.assertEqual(speech_rows(changed)[0]['english'],'New text')
        self.assertEqual(replace_english(original,{}),original)

    def test_invalid_replacements(self):
        for text in [b'x'*256,b'a\0b']:
            with self.assertRaises(ValueError):
                replace_english(record(),{0:text})

    def test_invalid_records(self):
        for data in [record()[:-1],b'x'*STRIDE,b'']:
            with self.assertRaises(ValueError):
                speech_rows(data)

    def test_mapping_by_id(self):
        mapped,review = map_hebrew(record('OTHER')+record(),source('שלום'),b'00000008GUY_1\n')
        self.assertEqual([(r['index'],r['hebrew']) for r in mapped],[(1,'שלום')])
        self.assertEqual(review[0]['reason'],'unmapped')

    def test_withholds_conflicts_and_control_codes(self):
        for text,reason in [(source('שלום')+b'\n'+source('אחר'),'ambiguous'),(source(r'שלום\xFF\x03עוד'),'control_codes')]:
            mapped,review = map_hebrew(record(),text,b'00000008GUY_1\n')
            self.assertEqual(mapped,[])
            self.assertEqual(review[0]['reason'],reason)

if __name__=='__main__':
    unittest.main()
