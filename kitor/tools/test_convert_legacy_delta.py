import hashlib
import importlib.util
from pathlib import Path
import struct
import tempfile
import unittest
import convert_legacy_delta as convert
spec=importlib.util.spec_from_file_location('apply_bsdiff',Path(__file__).parents[1]/'src-tauri/src/services/apply_bsdiff.py')
reader=importlib.util.module_from_spec(spec);spec.loader.exec_module(reader)
class ConversionTests(unittest.TestCase):
 def apply(self,old,patch):
  with tempfile.TemporaryDirectory() as d:
   d=Path(d);(d/'old').write_bytes(old);(d/'patch').write_bytes(patch);reader.apply_patch(d/'old',d/'patch',d/'out');return (d/'out').read_bytes()
 def test_graft_interleaved_negative_seek(self):
  old=b'abcdefghij';raw=convert.num(3)+convert.num(2)+convert.num(-2)+bytes([1,2,3])+b'XY'+convert.num(2)+convert.num(1)+convert.num(0)+bytes([4,5])+b'!'
  self.assertEqual(self.apply(old,convert.rawbsd(raw)),b'bdfXYfh!')
 def test_vpatch_copy_literal_and_backward_seek(self):
  old=b'abcdefghij';blocks=bytes([5,2])+b'XY'+bytes([1,3])+struct.pack('<I',5)+bytes([1,2])+struct.pack('<I',1)+bytes([255])+bytes(8)
  expected=b'XYfghbc';raw=b'VPAT'+struct.pack('<I',0x80000001)+struct.pack('<I',4)+hashlib.md5(old).digest()+hashlib.md5(expected).digest()+struct.pack('<I',len(blocks))+blocks
  a,b,patch=convert.vpat(raw)[0];self.assertEqual(a,hashlib.md5(old).hexdigest());self.assertEqual(b,hashlib.md5(expected).hexdigest());self.assertEqual(self.apply(old,patch),expected)
 def test_initial_seek_only_control(self):
  raw=convert.num(0)+convert.num(0)+convert.num(4)+convert.num(3)+convert.num(0)+convert.num(0)+bytes(3)
  self.assertEqual(self.apply(b'abcdefghij',convert.rawbsd(raw)),b'efg')
 def test_truncated_data_rejected(self):
  for raw in [bytes(23),convert.num(4)+convert.num(0)+convert.num(0)+b'X']:
   with self.assertRaises((AssertionError,struct.error)):convert.rawbsd(raw)
  with self.assertRaises((AssertionError,IndexError)):convert.vpat(b'VPAT'+struct.pack('<I',0x80000001))
if __name__=='__main__':unittest.main()
