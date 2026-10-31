import bz2,tempfile,unittest
from pathlib import Path
from delta import apply_patch

def integer(n):return (abs(n)|(1<<63 if n<0 else 0)).to_bytes(8,'little')
def patch(controls,diff,extra,size):
 c=bz2.compress(b''.join(integer(n) for row in controls for n in row));d=bz2.compress(diff)
 return b'BSDIFF40'+integer(len(c))+integer(len(d))+integer(size)+c+d+bz2.compress(extra)
class DeltaTests(unittest.TestCase):
 def run_patch(self,old,p):
  with tempfile.TemporaryDirectory() as d:
   d=Path(d);(d/'old').write_bytes(old);(d/'patch').write_bytes(p);apply_patch(d/'old',d/'patch',d/'out');return (d/'out').read_bytes()
 def test_difference_and_extra(self):
  self.assertEqual(self.run_patch(b'abc',patch([(3,1,0)],bytes([0,1,0]),b'!',4)),b'acc!')
 def test_negative_seek_and_outside_old(self):
  self.assertEqual(self.run_patch(b'abc',patch([(1,0,-2),(3,0,0)],b'\0\7\0\0',b'',4)),b'a\7ab')
 def test_reject_truncated_control(self):
  with self.assertRaises(ValueError):self.run_patch(b'abc',patch([],b'',b'',1))
 def test_reject_overrun(self):
  with self.assertRaises(ValueError):self.run_patch(b'abc',patch([(4,0,0)],b'\0'*4,b'',3))
if __name__=='__main__':unittest.main()
