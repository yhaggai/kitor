import sys,unittest,unicodedata,re
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
from niqqud_trial import SAMPLES,CLUSTER,tile
from build import catalog,TOKENS
from collections import Counter
class NiqqudTests(unittest.TestCase):
 def test_markup_preserved(self):
  old=catalog()
  for key,value in SAMPLES.items():self.assertEqual(Counter(TOKENS.findall(old[key])),Counter(TOKENS.findall(value)))
 def test_all_cluster_marks_fit(self):
  for value in SAMPLES.values():
   for m in CLUSTER.finditer(unicodedata.normalize('NFD',value)):
    t=tile(m.group(),16);self.assertIsNotNone(t.getbbox());self.assertLessEqual(t.getbbox()[3],16)
if __name__=='__main__':unittest.main()

class CompositeFontTests(unittest.TestCase):
 def test_narrow_letter_vowels_keep_distinct_dots(self):
  image=tile('ו\u05b6',19)
  self.assertGreaterEqual(image.width,5)
  self.assertEqual(sum(image.getpixel((x,12))[3]>0 for x in range(image.width)),2)
 def test_shuruk_dot_is_separate_from_stem(self):
  image=tile('ו\u05bc',19)
  self.assertTrue(image.getpixel((0,7))[3]);self.assertFalse(image.getpixel((1,7))[3]);self.assertTrue(image.getpixel((2,7))[3])
 def test_maqaf_is_not_a_combining_mark(self):
  self.assertEqual(CLUSTER.findall('א\u05b7־ב\u05b8'),['א\u05b7','ב\u05b8'])
 def test_descender_clearance(self):
  image=tile('ך\u05b8',19)
  self.assertTrue(image.getpixel((2,17))[3]);self.assertFalse(image.getpixel((2,14))[3])

class FontPreservationTests(unittest.TestCase):
 def test_existing_rectangles_and_pixels_are_unchanged(self):
  import tempfile,json,io
  from PIL import Image
  from niqqud_trial import extend_composites
  with tempfile.TemporaryDirectory() as tmp:
   folder=Path(tmp)/'SpriteFont1';folder.mkdir()
   old=Image.new('RGBA',(8,16),(1,2,3,255));old.save(folder/'SpriteFont1.png')
   (folder/'charmap.txt').write_text('א');(folder/'info.txt').write_text('1.5\n0\n16\n0 0 5 16\n')
   files,widths=extend_composites(folder,{'א\u05b7':'\ue200'})
   new=Image.open(io.BytesIO(files['SpriteFont1.png']))
   self.assertEqual(new.crop((0,0,8,16)).tobytes(),old.tobytes())
   self.assertTrue(files['info.txt'].decode().splitlines()[3].startswith('0 0 5 16,'))
   self.assertEqual(widths['א'],6.5)
   self.assertEqual(files['info.txt'].decode().splitlines()[2],'20')

class FullCatalogTests(unittest.TestCase):
 def test_complete_and_exact_original_text(self):
  from niqqud_trial import load_pointed
  self.assertEqual(set(load_pointed()),set(catalog()))
 def test_all_full_catalog_marks_have_visible_glyphs(self):
  from niqqud_trial import load_pointed
  for value in load_pointed().values():
   for m in CLUSTER.finditer(unicodedata.normalize('NFD',value)):
    self.assertIsNotNone(tile(m.group(),19).getbbox())
