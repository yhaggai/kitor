"""Build an edition-checked Hebrew patch from the user's own game. Never installs."""
import argparse,base64,csv,hashlib,io,json,re,shutil,subprocess,sys
from pathlib import Path
from collections import Counter
from layout import plain,write_layouts
from fonts import extend,title,TITLES,png
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[1]
VERSION='0.2.0-beta.2'
TOKENS=re.compile(r'<[^>]*>|\[\d+\]')
NORMALIZE=str.maketrans({'־':'-','–':'-','—':'-','…':'...','׳':"'",'״':'"','‘':"'",'’':"'",'“':'"','”':'"','\u00a0':' '})
def digest(data):return hashlib.sha256(data).hexdigest()
def catalog():
 result={}
 for path in sorted((ROOT/'translations').glob('part-*.json')):
  data=json.loads(path.read_text(encoding='utf-8'))
  if set(data)&set(result):raise ValueError('Duplicate entry ids')
  result.update(data)
 return result

def wrap(text,widths,limit=210):
 # Keep all timing/effect tokens in logical order. Only add line-break tokens.
 output=[];width=0;last_space=None;tokens=re.findall(r'<[^>]*>|\[\d+\]|.',text)
 for token in tokens:
  if token=='<nl>':output.append(token);width=0;last_space=None;continue
  output.append(token)
  if TOKENS.fullmatch(token):continue
  width+=widths.get(token,6)
  if token==' ':last_space=len(output)-1
  if width>limit and last_space is not None:
   output[last_space]='<nl>';width=sum(widths.get(t,6) for t in output[last_space+1:] if not TOKENS.fullmatch(t));last_space=None
 return ''.join(output)

def compile_runtime(game,out):
 sdk=subprocess.check_output(['dotnet','--list-sdks'],text=True).strip().splitlines()[-1]
 version,location=sdk.split(' [');csc=Path(location.rstrip(']'))/version/'Roslyn/bincore/csc.dll'
 refs=[game/name for name in ['Owlboy.exe','FNA.dll','GamedevUtility.dll']]
 for name in ['mscorlib.dll','System.dll','System.Core.dll']:
  path=game/name
  if not path.exists() and sys.platform=='win32':
   import os
   path=Path(os.environ['WINDIR'])/'Microsoft.NET/Framework/v4.0.30319'/name
  if not path.exists():raise ValueError('Missing .NET Framework reference: '+str(path))
  refs.append(path)
 command=['dotnet',str(csc),'-nologo','-target:library','-nostdlib+','-langversion:7.3','-out:'+str(out/'OwlboyHebrew.dll'),str(ROOT/'runtime/HebrewRuntime.cs')]+['-reference:'+str(p) for p in refs]
 subprocess.run(command,check=True)
 subprocess.run(['dotnet','build',str(ROOT/'patcher/Patcher.csproj'),'-c','Release','--nologo'],check=True)
 subprocess.run(['dotnet',str(ROOT/'patcher/bin/Release/net9.0/Patcher.dll'),str(game/'Owlboy.exe'),str(out/'OwlboyHebrew.dll'),str(out/'Owlboy.exe')],check=True)

def build(game,out,allow_partial=False):
 game=game.resolve();out=out.resolve();out.mkdir(parents=True,exist_ok=True)
 if out==game or game in out.parents:raise ValueError('Build output must be outside the game')
 loc=game/'Data/Localizations'
 if (loc/'he.csv').exists():raise ValueError('Hebrew is already installed. Uninstall before building, or supply pristine local inputs.')
 compatibility=json.loads((ROOT/'compatibility.json').read_text(encoding='utf-8'))
 if digest((game/'Owlboy.exe').read_bytes()) not in compatibility['executable_sha256']:raise ValueError('Unsupported executable edition. No files changed.')
 source=list(csv.reader((loc/'de.csv').open(encoding='utf-8-sig',newline='')))
 if digest(json.dumps([r[2] for r in source[1:]],ensure_ascii=False,separators=(',',':')).encode())!=compatibility['source_catalog_sha256']:raise ValueError('Source catalog differs from the translated edition')
 translations=catalog();required={f'{i:04d}' for i in range(1,len(source)-1)}
 if required-set(translations) and not allow_partial:raise ValueError(f'Missing {len(required-set(translations))} translation entries')
 clean={};unchanged=[]
 for i,row in enumerate(source[1:]):
  if row[2]=='language_id':continue
  key=f'{i:04d}';text=translations.get(key,row[2]);text=text.translate(NORMALIZE)
  if not isinstance(text,str) or not text:raise ValueError('Empty translation '+key)
  if Counter(TOKENS.findall(row[2]))!=Counter(TOKENS.findall(text)):raise ValueError('Markup mismatch at '+key)
  if re.search(r'<\d+>(?:<b>|\[\d+\])*<\d+>',text):raise ValueError('Adjacent pause tags at '+key)
  if '|' in row[2] and row[2].split('|',1)[1]!=text.split('|',1)[-1]:raise ValueError('Branch identifier mismatch at '+key)
  if text==row[2]:unchanged.append(key)
  clean[key]=text
 chars=set(''.join(plain(s) for s in clean.values()))
 payload=out/'payload';payload.mkdir(exist_ok=True);changes=[]
 def put(relative,data):
  destination=payload/relative;destination.parent.mkdir(parents=True,exist_ok=True);destination.write_bytes(data)
  original=game/relative;changes.append({'path':relative,'original_sha256':digest(original.read_bytes()) if original.exists() else None,'installed_sha256':digest(data)})
 widths=None
 for folder in sorted((loc/'en').iterdir()):
  if not (folder/'info.txt').exists():continue
  files,metrics=extend(folder,chars);target='he' if folder.name=='SpriteFont1' else 'en'
  for name,data in files.items():put(f'Data/Localizations/{target}/{folder.name}/{name}',data)
  if folder.name=='SpriteFont1':widths=metrics
 widths[' ']=widths.get(' ',6)
 for i,row in enumerate(source[1:]):
  if row[2]=='language_id':row[3]='he'
  else:row[3]=wrap(clean[f'{i:04d}'],widths) if row[4]!='default.ini' else clean[f'{i:04d}']
 buffer=io.StringIO(newline='');csv.writer(buffer).writerows(source);put('Data/Localizations/he.csv',buffer.getvalue().encode())
 master=list(csv.reader((loc/'Translations.csv').open(encoding='utf-8-sig',newline='')))
 for i,row in enumerate(master):row.insert(len(row)-2,'Hebrew' if i==0 else 'he' if i==1 else '')
 buffer=io.StringIO(newline='');csv.writer(buffer).writerows(master);put('Data/Localizations/Translations.csv',buffer.getvalue().encode())
 put('Data/Localizations/he/speechBubbleConfig.ini',b'[Config]\nLineSpacing=15\n')
 # The language menu samples a fixed 142x89 rectangle, matching stock flags.
 flag=Image.new('RGBA',(142,89),'white');d=ImageDraw.Draw(flag);blue='#2465af'
 d.rectangle((0,11,141,22),fill=blue);d.rectangle((0,66,141,77),fill=blue)
 d.line([(71,28),(52,59),(90,59),(71,28)],fill=blue,width=3)
 d.line([(71,63),(52,32),(90,32),(71,63)],fill=blue,width=3)
 put('Data/Localizations/Flags/he.png',png(flag))
 for filename,text in TITLES.items():put('Data/Localizations/he/Titles/'+filename,title(text))
 layout_path=out/'layouts.tsv';count=write_layouts([r[3] for r in source[2:]],layout_path);put('Data/Localizations/he/layouts.tsv',layout_path.read_bytes())
 compile_runtime(game,out)
 for name in ['OwlboyHebrew.dll','Owlboy.exe']:put(name,(out/name).read_bytes())
 plan={'schema':1,'id':'owlboy-hebrew','version':VERSION,'source_exe_sha256':digest((game/'Owlboy.exe').read_bytes()),'files':changes}
 (out/'plan.json').write_text(json.dumps(plan,indent=2)+'\n',encoding='utf-8')
 # Kitor packages reuse the same explicit selective file plan on both platforms.
 files=[]
 for c in changes:
  item={'path':c['path'],'operation':'create' if c['original_sha256'] is None else 'modify','installed_sha256':c['installed_sha256'],'source':{'kind':'copy','path':'payload/'+c['path'],'sha256':c['installed_sha256']}}
  if c['original_sha256']:item['original_sha256']=c['original_sha256']
  files.append(item)
 manifest={'schema_version':1,'scummvm_id':'owlboy','language':'he','version':VERSION,'filename':'owlboy-hebrew.zip','platforms':{'windows':{},'osx':{}},'files':files}
 (out/'patch.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
 report={'entries':len(clean),'translated_entries':len(clean)-len(unchanged),'unchanged_ids':unchanged,'unicode_layouts':count,'affected_files':len(changes),'missing_entries':len(required-set(translations)),'source_exe_sha256':plan['source_exe_sha256'],'status':'beta; full playthrough and Windows gameplay pending'}
 (out/'validation.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps(report,indent=2))
 return report
if __name__=='__main__':
 ap=argparse.ArgumentParser();ap.add_argument('--game',type=Path,required=True);ap.add_argument('--out',type=Path,default=ROOT/'build/package');ap.add_argument('--allow-partial',action='store_true',help=argparse.SUPPRESS);args=ap.parse_args()
 build(args.game,args.out,args.allow_partial)
