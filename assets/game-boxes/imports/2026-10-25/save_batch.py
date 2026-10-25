import json,pathlib,shutil,sys
r=pathlib.Path(__file__).resolve().parent;d=json.load(open(r/'queue.json'));p=json.load(open(r/'prompts.json'))
for a in json.load(open(r/sys.argv[1])):
 dest=r/'generated'/(a['id']+'-he.png');shutil.copy2(a['path'],dest);p[a['id']]=a['prompt']
 for g in d['games']:
  if g['game_id']==a['id']:
   g['sources']['he']={'file':str(dest.relative_to(r)),'kind':'ai-text-localization','tool':'built-in image_gen','status':'draft'};g['status']='pair-draft'
(r/'queue.json').write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n');(r/'prompts.json').write_text(json.dumps(p,ensure_ascii=False,indent=2)+'\n')
