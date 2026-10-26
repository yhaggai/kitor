import sys,json,re,unittest
from collections import Counter
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT/'tools'))
from build import catalog,wrap
from layout import layout,plain,write_layouts
class CatalogTests(unittest.TestCase):
 def test_complete_and_parseable(self):
  contract=json.loads((ROOT/'catalog-contract.json').read_text(encoding='utf-8'));texts=catalog()
  self.assertEqual(len(texts),4537);self.assertEqual(set(texts),set(contract))
  for key,text in texts.items():
   with self.subTest(id=key):
    self.assertTrue(isinstance(text,str) and text)
    self.assertEqual(Counter(re.findall(r'<[^>]*>|\[\d+\]',text)),Counter(contract[key]['tokens']))
    self.assertIsNone(re.search(r'<\d+>(?:<b>|\[\d+\])*<\d+>',text))
    self.assertIsNone(re.search('[\u0591-\u05bd\u05bf-\u05c7]',text))
    if contract[key]['branch']:self.assertEqual(text.split('|',1)[1],contract[key]['branch'])
 def test_bidi_maps(self):
  visual,order=layout('שלום 123 (ABC)')
  self.assertEqual(sorted(order),list(range(14)))
  self.assertIn('123',visual);self.assertIn('(ABC)',visual)
  for text in catalog().values():
   for line in plain(text).split('\n'):
    if line:
     visual,order=layout(line);self.assertEqual(sorted(order),list(range(len(line))))
 def test_wrapping_keeps_events(self):
  value='שלום <30><b>עולם<b> יפה מאוד [1]שלום'
  output=wrap(value,{c:6 for c in plain(value)},36)
  a=Counter(re.findall(r'<[^>]*>|\[\d+\]',value));b=Counter(re.findall(r'<[^>]*>|\[\d+\]',output));b.pop('<nl>',None)
  self.assertEqual(a,b);self.assertIn('<nl>',output)
if __name__=='__main__':unittest.main()
