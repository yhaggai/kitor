"""Offline vowel-mark draft. Does not send game text to any external service.
Requires nakdimon==0.2.1. Contextual review against English is still necessary.
"""
import argparse,csv,re,unicodedata
from pathlib import Path
from build import catalog
from nakdimon.predict import predict
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
 parser=argparse.ArgumentParser(description=__doc__)
 parser.add_argument('--output',type=Path,required=True,help='Write a separate CSV draft for review')
 args=parser.parse_args()
 with args.output.open('x',encoding='utf-8',newline='') as stream:
  writer=csv.writer(stream)
  writer.writerow(['key','hebrew','hebrew_niqqud'])
  for key,text in catalog().items():
   writer.writerow([key,text,draft(text)])
