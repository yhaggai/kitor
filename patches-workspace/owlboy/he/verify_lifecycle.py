"""Exercise both package plans against disposable subsets using Kitor lifecycle."""
import hashlib,json,shutil,subprocess,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parent
GAME=Path('/Users/haggaiyaniv/Library/Application Support/Steam/steamapps/common/Owlboy/Owlboy.app/Contents/Resources')
TOOL=ROOT/'native/target/release/owlboy-font-test'
manifest=json.loads((ROOT/'patch.json').read_text())
report={}
for platform in ['osx','windows']:
 with tempfile.TemporaryDirectory(prefix='owlboy-font-') as temp:
  base=Path(temp);target=base/'game';target.mkdir();originals={}
  for f in manifest['files']:
   source=GAME/f['path']
   if source.exists():
    dest=target/f['path'];dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(source,dest);originals[f['path']]=source.read_bytes()
  (target/'untouched.txt').write_text('keep')
  command=[str(TOOL),'install',str(ROOT),str(target),str(base/'recovery'),platform]
  r=subprocess.run(command,check=True,capture_output=True,text=True)
  journal=Path(r.stdout.strip().split('Recovery journal: ')[1])
  assert len(list((journal/'originals').iterdir()))==len(originals)
  for f in manifest['files']:assert hashlib.sha256((target/f['path']).read_bytes()).hexdigest()==f['installed_sha256']
  (target/'added-later.txt').write_text('keep later')
  subprocess.run([str(TOOL),'uninstall',str(journal),str(target)],check=True)
  for f in manifest['files']:
   if f['path'] in originals:assert (target/f['path']).read_bytes()==originals[f['path']]
   else:assert not (target/f['path']).exists()
  assert (target/'untouched.txt').read_text()=='keep' and (target/'added-later.txt').read_text()=='keep later'
  report[platform]={'fixture_roundtrip':'passed','affected_files':len(manifest['files']),'backed_up_originals':len(originals),'native_gameplay':'not tested by this script'}
(ROOT/'lifecycle-validation.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
