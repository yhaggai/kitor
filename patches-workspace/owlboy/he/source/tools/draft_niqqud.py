"""Offline vowel-mark draft. Does not send game text to any external service.
Requires nakdimon==0.2.1. Contextual review against English is still necessary.
"""
import json,re,unicodedata
from pathlib import Path
from nakdimon.predict import predict
ROOT=Path(__file__).resolve().parents[1]
TAGS=re.compile(r'<[^>]*>|\[\d+\]')
LETTERS=re.compile(r'[א-ת][\u0591-\u05bd\u05bf\u05c1-\u05c2\u05c4-\u05c5\u05c7]*')
def draft(text):
 spoken,suffix=(text.split('|',1)+[None])[:2] if '|' in text else (text,None)
 plain=TAGS.sub('',spoken.replace('<nl>',' '))
 if not re.search('[א-ת]',plain):return text
 dotted=unicodedata.normalize('NFD',predict(plain,maxlen=512))
 clusters=LETTERS.findall(dotted);base=re.findall('[א-ת]',plain)
 if [c[0] for c in clusters]!=base:raise ValueError('Predictor changed Hebrew letters')
 it=iter(clusters);pieces=[];end=0
 for m in TAGS.finditer(spoken):
  pieces.append(re.sub('[א-ת]',lambda _:next(it),spoken[end:m.start()]));pieces.append(m.group());end=m.end()
 pieces.append(re.sub('[א-ת]',lambda _:next(it),spoken[end:]));result=''.join(pieces)
 if suffix is not None:result+='|'+suffix
 if re.sub('[\u0591-\u05bd\u05bf\u05c1-\u05c2\u05c4-\u05c5\u05c7]','',result)!=text:raise ValueError('Source preservation failed')
 return result
if __name__=='__main__':
 target=ROOT/'translations/niqqud';target.mkdir(exist_ok=True)
 for source in sorted((ROOT/'translations').glob('part-*.json')):
  if (target/source.name).exists():
   print('Skipping existing reviewed file:',source.name,flush=True);continue
  result={}
  for i,(key,text) in enumerate(json.loads(source.read_text()).items()):
   result[key]=draft(text)
   if i%100==0:print(source.name,i,flush=True)
  (target/source.name).write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n');print('READY',source.name,flush=True)
