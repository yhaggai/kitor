import csv,json,tempfile,unittest
from pathlib import Path
import text_pipeline as p
class CsvContract(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.old=p.ROOT;p.ROOT=Path(self.tmp.name)
  self.rows=[{'key':'Dialogue Text/one','english':'A *word* {0}.','hebrew':'*מילה* {0}.'},{'key':'Dialogue Text/two','english':'Same,\nsource','hebrew':''}]
  p.writejson(p.ROOT/'source-index.json',{'source_sha256':{r['key']:p.sha(r['english']) for r in self.rows},'unmapped_source_keys':[],'unresolved_localization_terms':[]});p.writecsv(self.rows)
 def tearDown(self):p.ROOT=self.old;self.tmp.cleanup()
 def test_csv_roundtrip_and_coverage(self):
  self.assertEqual(p.readcsv(),self.rows);self.assertEqual(p.validate()['translated'],1)
 def test_duplicate_key_rejected(self):
  p.writecsv(self.rows+[self.rows[0]])
  with self.assertRaises(AssertionError):p.validate()
 def test_changed_source_rejected(self):
  self.rows[0]['english']='changed';p.writecsv(self.rows)
  with self.assertRaises(ValueError):p.validate()
 def test_lost_placeholder_rejected(self):
  self.rows[0]['hebrew']='*מילה*';p.writecsv(self.rows)
  with self.assertRaises(ValueError):p.validate()
 def test_lost_emphasis_rejected(self):
  self.rows[0]['hebrew']='מילה {0}';p.writecsv(self.rows)
  with self.assertRaises(ValueError):p.validate()
 def test_release_gate(self):
  with self.assertRaises(ValueError):p.validate(release=True)
if __name__=='__main__':unittest.main()
