"""Install the checked-in macOS or Windows delta in place, or restore a verified backup."""
from pathlib import Path
import argparse,datetime,hashlib,json,os,shutil,tempfile
from delta import apply_patch
ROOT=Path(__file__).resolve().parent
MAC_DEFAULT=Path.home()/'Library/Application Support/Steam/steamapps/common/Undertale/UNDERTALE.app'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def replace_verified(source,target,expected):
 fd,name=tempfile.mkstemp(prefix='.hebrew-',dir=target.parent);os.close(fd);tmp=Path(name)
 try:
  shutil.copy2(source,tmp)
  if sha(tmp)!=expected:raise ValueError('Staged file checksum mismatch')
  os.chmod(tmp,target.stat().st_mode)
  with tmp.open('rb') as f:os.fsync(f.fileno())
  os.replace(tmp,target)
 finally:tmp.unlink(missing_ok=True)
def windows_default():
 candidates=[]
 if os.name=='nt':
  import winreg
  try:
   with winreg.OpenKey(winreg.HKEY_CURRENT_USER,r'Software\Valve\Steam') as key:candidates.append(Path(winreg.QueryValueEx(key,'SteamPath')[0]))
  except OSError:pass
 for variable in ['PROGRAMFILES(X86)','PROGRAMFILES']:
  if os.environ.get(variable):candidates.append(Path(os.environ[variable])/'Steam')
 for root in list(candidates):
  vdf=root/'steamapps/libraryfolders.vdf'
  if vdf.exists():
   import re
   for path in re.findall(r'"path"\s*"([^"]+)"',vdf.read_text(encoding='utf-8')):candidates.append(Path(path.replace('\\\\','\\')))
 for root in candidates:
  folder=root/'steamapps/common/Undertale'
  if (folder/'data.win').exists():return folder
 return Path(os.environ.get('PROGRAMFILES(X86)',r'C:\Program Files (x86)'))/'Steam/steamapps/common/Undertale'
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--game-app',type=Path);p.add_argument('--game-dir',type=Path);p.add_argument('--platform',choices=['macos','windows'],default='windows' if os.name=='nt' else 'macos');p.add_argument('--rollback',type=Path);p.add_argument('--backup-dir',type=Path);a=p.parse_args()
 if a.rollback:
  b=a.rollback.resolve();m=json.loads((b/'manifest.json').read_text());target=Path(m['target']);original=b/m.get('backup_file','game.original.ios')
  if sha(original)!=m['source_sha256']:raise ValueError('Backup checksum mismatch')
  if sha(target)==m['source_sha256']:print('Already restored.');return
  if sha(target)!=m['patched_sha256']:raise ValueError('Installed archive changed; refusing to overwrite another mod or update.')
  replace_verified(original,target,m['source_sha256']);print('Original restored. Saves untouched.');return
 if a.platform=='windows':
  target=(a.game_dir or windows_default()).expanduser().resolve()/'data.win'
  patch_dir=ROOT/'patches/windows'
 else:
  target=(a.game_app or MAC_DEFAULT).expanduser().resolve()/'Contents/Resources/game.ios'
  patch_dir=ROOT/'patches'
 if not target.is_file():raise ValueError('Game archive not found: '+str(target)+'. Use --game-dir on Windows or --game-app on macOS.')
 m=json.loads((patch_dir/'manifest.json').read_text());delta=patch_dir/'hebrew-steam.bsdiff'
 if sha(target)==m['patched_sha256']:print('This Hebrew patch is already installed.');return
 if sha(target)!=m['source_sha256']:raise ValueError('Unsupported or already modified game version; nothing changed.')
 if sha(delta)!=m['patch_sha256']:raise ValueError('Patch checksum mismatch')
 parent=(a.backup_dir or ((Path(os.environ.get('LOCALAPPDATA',str(Path.home())))/'Undertale Hebrew/backups') if a.platform=='windows' else Path.home()/'Library/Application Support/Undertale Hebrew/backups')).expanduser();parent.mkdir(parents=True,exist_ok=True)
 b=Path(tempfile.mkdtemp(prefix=datetime.datetime.now().strftime('%Y%m%d-%H%M%S-'),dir=parent))
 backup_file='data.original.win' if a.platform=='windows' else 'game.original.ios'
 original=b/backup_file;shutil.copy2(target,original)
 if sha(original)!=m['source_sha256']:raise ValueError('Backup checksum mismatch')
 m=dict(m,target=str(target),backup_file=backup_file);(b/'manifest.json').write_text(json.dumps(m,indent=2)+'\n')
 # Apply into a separate temporary file before replacing the installed archive.
 with tempfile.TemporaryDirectory(prefix='undertale-hebrew-') as temp:
  output=Path(temp)/'patched.ios';apply_patch(original,delta,output)
  if sha(output)!=m['patched_sha256']:raise ValueError('Patched archive checksum mismatch')
  if sha(target)!=m['source_sha256']:raise ValueError('Game changed during installation; stopped.')
  replace_verified(output,target,m['patched_sha256'])
 print('Installed Hebrew with Steam integration. Keep language set to English.')
 print('Backup:',b)
 print('Restore by dragging this backup folder onto Restore Original Windows.cmd.' if a.platform=='windows' else 'Rollback: python3 '+repr(str(ROOT/'install.py'))+' --rollback '+repr(str(b)))
 print('Saves unchanged. Restart the game through Steam when ready.')
if __name__=='__main__':
 try:main()
 except (OSError,ValueError) as error:raise SystemExit(str(error))
