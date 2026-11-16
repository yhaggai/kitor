"""Apply explicit pointed spellings for recurring character names and terms."""
import json,re,unicodedata
from pathlib import Path

BASE=Path(__file__).resolve().parent
MARKS='\u0591-\u05bd\u05bf\u05c1-\u05c2\u05c4-\u05c5\u05c7'
def strip(s):return re.sub(f'[{MARKS}]','',s)
def main():
 rows=json.loads((BASE/'catalog-niqqud.json').read_text(encoding='utf-8'))
 base=json.loads((BASE/'catalog.json').read_text(encoding='utf-8'))
 glossary=json.loads((BASE/'niqqud-glossary.json').read_text(encoding='utf-8'))
 for row in rows:
  text=unicodedata.normalize('NFD',row['he'])
  for plain,pointed in sorted(glossary.items(),key=lambda kv:-len(kv[0])):
   p=unicodedata.normalize('NFD',plain);q=unicodedata.normalize('NFD',pointed)
   pattern=r'(?<![\u05d0-\u05ea])((?:[ובכלמשה][\u0591-\u05bd\u05bf\u05c1-\u05c2\u05c4-\u05c5\u05c7]*)?)'+''.join(re.escape(c)+f'[{MARKS}]*' for c in p)+r'(?![\u05d0-\u05ea])'
   text=re.sub(pattern,lambda m:m.group(1)+q,text)
  row['he']=text
 # Verify the explicit glossary marks did not change any base text or row identity.
 if len(base)!=len(rows):raise ValueError('Catalog length mismatch')
 for old,new in zip(base,rows):
  if old['path']!=new['path'] or strip(old['he'])!=strip(new['he']):raise ValueError(f'Base text changed: {old["path"]}')
 (BASE/'catalog-niqqud.json').write_text(json.dumps(rows,ensure_ascii=False,indent=1),encoding='utf-8')
 print(f'Applied pointed glossary to {len(rows)} catalog entries; wording unchanged.')
if __name__=='__main__':main()
