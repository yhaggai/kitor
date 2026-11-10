#!/usr/bin/env python3
"""Disco Elysium CSV source of truth; no machine-translation service or draft model.
Local source extraction requires an owned game. Never installs into the Steam copy.
"""
import argparse,csv,hashlib,json,re,sys
from collections import Counter
from pathlib import Path
import UnityPy
ROOT=Path(__file__).resolve().parent
DEFAULT=Path.home()/'Library/Application Support/Steam/steamapps/common/Disco Elysium/disco.app/Contents/Resources/Data'
FIELDS=['key','english','hebrew']
def sha(s):return hashlib.sha256(s.encode()).hexdigest()
def writejson(p,obj):p.write_text(json.dumps(obj,ensure_ascii=False,indent=2)+'\n')
def readcsv():
 with (ROOT/'translations.csv').open(newline='',encoding='utf-8') as f:
  r=csv.DictReader(f);assert r.fieldnames==FIELDS,'CSV columns must be key,english,hebrew';rows=list(r)
 assert len({r['key'] for r in rows})==len(rows),'Duplicate CSV key'
 return rows
def writecsv(rows):
 with (ROOT/'translations.csv').open('w',newline='',encoding='utf-8') as f:
  w=csv.DictWriter(f,fieldnames=FIELDS);w.writeheader();w.writerows(rows)
def extract(data):
 previous=readcsv() if (ROOT/'translations.csv').exists() else None
 file=next(data.glob('StreamingAssets/aa/*/dialoguebundle*'))
 db=next(o.read_typetree() for o in UnityPy.load(str(file)).objects if o.type.name=='MonoBehaviour')
 actors={str(a['id']):{f['title']:f['value'] for f in a['fields']} for a in db['actors']}
 entries={};context={}
 for c in db['conversations']:
  cf={f['title']:f['value'] for f in c['fields']}
  for d in c['dialogueEntries']:
   f={f['title']:f['value'] for f in d['fields']}
   if not f.get('Dialogue Text'):continue
   key='Dialogue Text/'+f['Articy Id'];en=f['Dialogue Text']
   assert key not in entries or entries[key]==en,'Conflicting Articy ID'
   entries[key]=en;context.setdefault(key,[]).append({'conversation_id':c['id'],'conversation':cf.get('Title'),'entry_id':d['id'],'speaker_id':f.get('Actor'),'speaker':actors.get(f.get('Actor'),{}).get('Name'),'addressee_id':f.get('Conversant'),'outgoing':d['outgoingLinks'],'conditions':d['conditionsString'],'script':d['userScript']})
 lock= data/'StreamingAssets/AssetBundles/OSX/lockits/arabic'
 missing=[];mapped=set()
 for o in UnityPy.load(str(lock)).objects:
  if o.type.name!='MonoBehaviour':continue
  t=o.read_typetree()
  if t['m_Name'] not in ['GeneralLockitArabic','DialoguesLockitArabic']:continue
  for term in t['mSource']['mTerms']:
   if term['TermType']!=0:continue
   key=term['Term']
   if t['m_Name']=='GeneralLockitArabic' and len(term['Languages'])>1 and term['Languages'][1]:
    en=term['Languages'][1];assert key not in entries or entries[key]==en;entries[key]=en
   if key in entries:mapped.add(key)
   else:missing.append({'table':t['m_Name'],'key':key})
 initial={r['key']:r['he'] for r in json.loads((ROOT/'opening.json').read_text())['entries']}
 if previous is not None:
  assert {r['key']:r['english'] for r in previous}==entries,'Source build differs; explicit CSV migration required'
 else:
  writecsv([{'key':k,'english':v,'hebrew':initial.get(k,'')} for k,v in entries.items()])
 writejson(ROOT/'source-index.json',{'schema_version':1,'source_sha256':{k:sha(v) for k,v in entries.items()},'context':context,'unmapped_source_keys':sorted(set(entries)-mapped),'unresolved_localization_terms':missing,'conversations':len(db['conversations'])})
 writejson(ROOT/'actors.json',actors)
def validate(release=False):
 rows=readcsv();idx=json.loads((ROOT/'source-index.json').read_text());errors=[]
 if {r['key'] for r in rows}!=set(idx['source_sha256']):errors.append('CSV source key inventory changed')
 for r in rows:
  if sha(r['english'])!=idx['source_sha256'].get(r['key']):errors.append('Changed English source: '+r['key'])
  if not r['hebrew']:continue
  for pattern in [r'<[^>]+>',r'\{[^{}]+\}',r'\[[^\]]*\d[^\]]*\]']:
   if Counter(re.findall(pattern,r['english']))!=Counter(re.findall(pattern,r['hebrew'])):errors.append('Protected token mismatch: '+r['key'])
  if r['english'].count('*')!=r['hebrew'].count('*'):errors.append('Emphasis mismatch: '+r['key'])
  if re.search('[\u202a-\u202e\u2066-\u2069]',r['hebrew']):errors.append('Manual bidi control: '+r['key'])
 done=[r for r in rows if r['hebrew']];unmapped=idx['unmapped_source_keys']
 report={'entries':len(rows),'translated':len(done),'remaining':len(rows)-len(done),'source_words':sum(len(r['english'].split()) for r in rows),'coverage_percent':round(100*len(done)/len(rows),3),'unmapped_source_keys':len(unmapped),'unresolved_localization_terms':len(idx['unresolved_localization_terms']),'errors':errors,'release_ready':False}
 writejson(ROOT/'csv-validation.json',report)
 if errors:raise ValueError('\n'.join(errors[:20]))
 if release:raise ValueError('Release blocked until full translation, source mapping, platforms and native lifecycle/gameplay checks are complete')
 return report

def build(data,output):
 report=validate();rows=readcsv();translations={r['key']:r['hebrew'] for r in rows if r['hebrew']}
 source=data/'StreamingAssets/AssetBundles/OSX/lockits/arabic'
 output=output.resolve();assert output!=source.resolve() and data.resolve() not in output.parents,'Cannot build inside source game'
 env=UnityPy.load(str(source));changed=set()
 for o in env.objects:
  if o.type.name!='MonoBehaviour':continue
  t=o.read_typetree();dirty=False
  if t['m_Name'] not in ['GeneralLockitArabic','DialoguesLockitArabic']:continue
  for term in t['mSource']['mTerms']:
   if term['Term'] in translations:
    term['Languages'][0]=translations[term['Term']];changed.add(term['Term']);dirty=True
  if dirty:o.save_typetree(t)
 missing=set(translations)-changed
 assert not missing,f'Translated keys missing from localization bundle: {sorted(missing)[:10]}'
 output.parent.mkdir(parents=True,exist_ok=True);output.write_bytes(env.file.save())
 check=UnityPy.load(str(output));actual={}
 for o in check.objects:
  if o.type.name=='MonoBehaviour':
   for term in o.read_typetree().get('mSource',{}).get('mTerms',[]):
    if term['Term'] in translations:actual[term['Term']]=term['Languages'][0]
 assert actual==translations,'Serialized translations differ'
 print(json.dumps({'output':str(output),'translated_keys':len(changed),'sha256':hashlib.sha256(output.read_bytes()).hexdigest()}))
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('command',choices=['extract','validate','status','next','build','release-check']);p.add_argument('--data',type=Path,default=DEFAULT);p.add_argument('--output',type=Path,default=ROOT/'build/lockits/arabic');p.add_argument('--conversation',type=int);p.add_argument('--limit',type=int,default=30);a=p.parse_args()
 if a.command=='extract':extract(a.data);print(json.dumps(validate()))
 elif a.command in ['validate','status','release-check']:print(json.dumps(validate(a.command=='release-check')))
 elif a.command=='build':build(a.data,a.output)
 else:
  idx=json.loads((ROOT/'source-index.json').read_text());batch=[]
  for r in readcsv():
   ctx=idx['context'].get(r['key'],[])
   if not r['hebrew'] and (a.conversation is None or any(c['conversation_id']==a.conversation for c in ctx)):
    batch.append(dict(r,context=ctx))
    if len(batch)>=a.limit:break
  print(json.dumps(batch,ensure_ascii=False,indent=2))
