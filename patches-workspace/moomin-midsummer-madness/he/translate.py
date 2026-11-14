from pathlib import Path
import json,subprocess,shutil,re,collections,concurrent.futures
root=Path.cwd();base=root/'patches-workspace/moomin-midsummer-madness/he';tmp=root/'.tmp/moomin-luna';tmp.mkdir(parents=True,exist_ok=True)
rows=json.loads((base/'source.json').read_text(encoding='utf8'));glossary=(base/'character-glossary.md').read_text(encoding='utf8')
chunks=[];cur=[];weight=0
for idx,r in enumerate(rows):
 row=[idx,'/'.join(map(str,r['path'])),r['en']];cur.append(row);weight+=len(r['en'])+len(row[1])+40
 if weight>8500:chunks.append(cur);cur=[];weight=0
if cur:chunks.append(cur)
print('batches',len(chunks),'workers 3; using only source-aligned row-only outputs',flush=True)
def norm_tokens(s):
 s=re.sub(r'</(bold|italic|b|i)>','</>',s)
 return collections.Counter(re.findall(r'\{[^{}\n]+\}|<[^<>\n]{1,100}>',s))
def validate(f,part):
 s=re.sub(r'^```(?:json)?\s*|\s*```$','',f.read_text(encoding='utf8').strip());arr=json.loads(s);assert isinstance(arr,list) and len(arr)==len(part),(len(arr),len(part))
 for (idx,path,en),he in zip(part,arr):
  assert isinstance(he,str) and (not en.strip() or he.strip()),(idx,'empty')
  assert norm_tokens(en)==norm_tokens(he),(idx,'markup/placeholders',en,he)
  source_numbers=collections.Counter(re.findall(r'\d+',en)); output_numbers=collections.Counter(re.findall(r'\d+',he))
  if re.search(r'\b2D\b',en) and 'דו־ממד' in he: source_numbers=collections.Counter(x for x in source_numbers.elements() if x!='2')
  assert source_numbers==output_numbers,(idx,'digits',en,he)
 return arr
def run(i):
 part=chunks[i];f=tmp/f'{i:03}.json';err=''
 if f.exists():
  try:return [(r[0],h) for r,h in zip(part,validate(f,part))]
  except Exception as e:f.replace(tmp/f'{i:03}-invalid-existing.txt');err='Existing file failed: '+repr(e)
 for attempt in range(4):
  prompt='''Translate ONLY the entries inside the supplied rows into natural, fluent Israeli Hebrew for Moomin Midsummer Madness. Do not include context entries or add/remove/reorder entries. Return ONLY a JSON array of Hebrew strings in exactly the same order and count as the rows. Use the glossary, stable JSON path and scene clues. Preserve character identity and mood, clear puzzle clues, every placeholder, rich text tag, digit and meaningful linebreak. Translate localizable roles and headings; preserve real people and company names. For the label 2D, use דו־ממד. Catalog Hebrew stays in logical reading order, never visually reversed. Keep every translation nonempty where source is nonempty.\n'''+glossary+'\nRows [index, stable path, English]:\n'+json.dumps(part,ensure_ascii=False)
  res=subprocess.run([shutil.which('codex'),'exec','--ephemeral','-m','gpt-6-luna','-c','model_reasoning_effort="low"','-s','read-only','-o',str(f),'-'],input=prompt.encode('utf8'),stdout=subprocess.DEVNULL,stderr=(tmp/f'{i:03}.log').open('wb'))
  if res.returncode: err=f'Model call failed batch {i}';continue
  try:return [(r[0],h) for r,h in zip(part,validate(f,part))]
  except Exception as e:
   err='Previous result failed validation: '+repr(e); f.replace(tmp/f'{i:03}-invalid-{attempt}.txt')
 raise RuntimeError(err)
results={}; failures=[]
with concurrent.futures.ThreadPoolExecutor(max_workers=3) as pool:
 fut={pool.submit(run,i):i for i in range(len(chunks))}
 for done in concurrent.futures.as_completed(fut):
  i=fut[done]
  try:
   pairs=done.result();results.update(pairs)
   print(f'{i+1}/{len(chunks)} complete; {len(results)} translated',flush=True)
   draft=[dict(r,he=results[j]) for j,r in enumerate(rows) if j in results]
   (tmp/'partial-catalog.json').write_text(json.dumps(draft,ensure_ascii=False,indent=1),encoding='utf8')
  except Exception as e:
   failures.append((i,str(e)));print(f'{i+1}/{len(chunks)} failed: {e}',flush=True)
assert not failures,failures
assert len(results)==len(rows),(len(results),len(rows))
for i,r in enumerate(rows):r['he']=results[i]
(base/'catalog.json').write_text(json.dumps(rows,ensure_ascii=False,indent=1),encoding='utf8')
(base/'model-attribution.md').write_text('Draft translation: OpenAI Codex CLI, model gpt-6-luna, reasoning effort low, run in isolated read-only batches (up to three concurrent). All 4,692 player-facing text fields were drafted. Structural, markup, placeholder and number checks were run in build.py; entries remain editable in catalog.json.\n',encoding='utf8')
