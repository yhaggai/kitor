"""Export editable unpointed/pointed translations to the canonical UTF-8 CSV."""
import csv,json,re
from pathlib import Path

BASE=Path(__file__).resolve().parent
SPEAKERS={
 'moomintroll':'Moomintroll','littlemy':'Little My','snorkmaiden':'Snorkmaiden',
 'snufkin':'Snufkin','mymble':'Mymble','mamma':'Moominmamma','pappa':'Moominpappa',
 'fillyjonk':'Fillyjonk','misabel':'Misabel','emma':'Emma','parkkeeper':'Park Keeper',
 'policeman':'Policeman','groke':'The Groke'
}
FIELDS=['stable_id','path','category','scene','speaker','en','he_unpointed','he_pointed']
MARKS=re.compile(r'[\u0591-\u05bd\u05bf\u05c1-\u05c2\u05c4-\u05c5\u05c7]')
def main():
 unpointed=json.loads((BASE/'catalog.json').read_text(encoding='utf-8'))
 pointed=json.loads((BASE/'catalog-niqqud.json').read_text(encoding='utf-8'))
 if len(unpointed)!=len(pointed):raise ValueError('Catalog lengths differ')
 out=BASE/'catalog.csv'
 with out.open('w',encoding='utf-8-sig',newline='') as f:
  w=csv.DictWriter(f,fieldnames=FIELDS,quoting=csv.QUOTE_MINIMAL,lineterminator='\n');w.writeheader()
  for old,new in zip(unpointed,pointed):
   if old['path']!=new['path'] or old['en']!=new['en']:raise ValueError('Stable source ID mismatch')
   path=old['path'];key=str(path[0]);low=key.lower()
   if low.startswith('dialogue_'):category='dialogue';scene=key[len('dialogue_'):]
   elif low.startswith('journal_'):category='journal';scene=key[len('journal_'):]
   elif 'credit' in low:category='credits';scene=key
   elif low.startswith('character_'):category='character';scene=key[len('character_'):]
   elif low.startswith('strings_'):category='ui';scene=key[len('strings_'):]
   elif low.startswith('constants'):category='system';scene=key
   else:category='content';scene=key
   speaker=''
   if category=='dialogue':
    for token,label in sorted(SPEAKERS.items(),key=lambda kv:-len(kv[0])):
     if re.search(r'(?:^|_)'+re.escape(token)+r'(?:_|$)',low):speaker=label;break
   stable=json.dumps(path,ensure_ascii=False,separators=(',',':'))
   w.writerow({'stable_id':stable,'path':json.dumps(path,ensure_ascii=False,separators=(',',':')),
               'category':category,'scene':scene,'speaker':speaker,'en':old['en'],
               'he_unpointed':MARKS.sub('',old['he']),'he_pointed':new['he']})
 print(f'Wrote {len(unpointed)} UTF-8-BOM rows to {out}')
if __name__=='__main__':main()
