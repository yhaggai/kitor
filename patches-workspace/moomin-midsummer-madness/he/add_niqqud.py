"""Add Hebrew niqqud locally, preserving wording, order, markup and stable paths."""
import json,re,unicodedata
from pathlib import Path
from collections import Counter
import numpy as np
from nakdimon import dataset,hebrew
from nakdimon.predict import load_cached_model,merge_unconditional
from nakdimon.config import MAIN_MODEL

BASE=Path(__file__).resolve().parent
TAGS=re.compile(r'<[^<>]*>|\{[^{}]*\}')
MARKS=re.compile(r'[\u0591-\u05bd\u05bf\u05c1-\u05c2\u05c4-\u05c5\u05c7]')
CLUSTER=re.compile(r'[\u05d0-\u05ea][\u0591-\u05bd\u05bf\u05c1-\u05c2\u05c4-\u05c5\u05c7]*')
LETTERS=re.compile(r'[\u05d0-\u05ea]')
def unmark(text): return MARKS.sub('',text)

def inject(text,dotted):
 text=unmark(text)
 clusters=CLUSTER.findall(unicodedata.normalize('NFD',dotted.replace('\ufeff','').replace(hebrew.RAFE,'')))
 expected=LETTERS.findall(TAGS.sub('',text))
 if [x[0] for x in clusters]!=expected:
  raise ValueError(f'Predictor changed Hebrew base letters: {text[:120]!r} -> {dotted[:120]!r}')
 it=iter(clusters);result=[];end=0
 for m in TAGS.finditer(text):
  result.append(LETTERS.sub(lambda _:next(it),text[end:m.start()]));result.append(m.group());end=m.end()
 result.append(LETTERS.sub(lambda _:next(it),text[end:]))
 result=''.join(result)
 if Counter(TAGS.findall(text))!=Counter(TAGS.findall(result)):raise ValueError('Markup changed')
 if re.sub(MARKS,'',result)!=text:raise ValueError(f'Wording changed: {text[:120]!r} -> {result[:120]!r}')
 return result

def main():
 source=json.loads((BASE/'catalog.json').read_text(encoding='utf8'));model=load_cached_model(MAIN_MODEL)
 selected=[];plain=[];rows=[]
 for i,row in enumerate(source):
  p=TAGS.sub('',unmark(row['he']))
  if LETTERS.search(p):rows.append((i,row));plain.append(p)
 output=[None]*len(source);batch_size=64
 for start in range(0,len(rows),batch_size):
  batch=rows[start:start+batch_size];datas=[]
  for _,row in batch:
   p=TAGS.sub('',unmark(row['he']));d=dataset.Data.from_text(list(hebrew.iterate_dotted_text(p)),512)
   if len(d)!=1:raise ValueError(f'Unexpected long entry split: {row["path"]}')
   datas.append(d)
  x=np.concatenate([d.normalized for d in datas]);n,d,s=model.run(None,{model.get_inputs()[0].name:x.astype(np.int32)})
  txt=np.concatenate([v.text for v in datas]);norm=np.concatenate([v.normalized for v in datas])
  dotted=merge_unconditional(txt,norm,np.argmax(n,axis=-1),np.argmax(d,axis=-1),np.argmax(s,axis=-1))
  for (idx,row),raw in zip(batch,dotted):output[idx]=inject(row['he'],raw)
  done=start+len(batch)
  if done%512==0 or done==len(rows):print(f'{done}/{len(rows)} Hebrew-bearing entries',flush=True)
 result=[{'path':r['path'],'en':r['en'],'he':output[i] if output[i] is not None else r['he']} for i,r in enumerate(source)]
 for i,(old,new) in enumerate(zip(source,result)):
  if unmark(new['he'])!=unmark(old['he']):raise ValueError(f'Base text changed at {old["path"]}')
  if Counter(TAGS.findall(old['he']))!=Counter(TAGS.findall(new['he'])):raise ValueError(f'Markup changed at {old["path"]}')
 (BASE/'catalog-niqqud.json').write_text(json.dumps(result,ensure_ascii=False,indent=1),encoding='utf8')
 print(f'Wrote pointed catalog: {len(result)} entries; {len(rows)} with Hebrew.',flush=True)
if __name__=='__main__':main()
