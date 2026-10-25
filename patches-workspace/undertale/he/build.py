"""Build from a matching, legally installed original macOS or Windows game archive."""
from pathlib import Path
import argparse,os,hashlib,subprocess,sys,shutil
ROOT=Path(__file__).resolve().parent
EXPECTED={'macOS':'91a8acf1f40bd7c45d0bdb978de55187e7fb2cd76a1e2cda41c078dd44d3565e', 'Windows':'b718f8223a5bb31979ffeed10be6140c857b882fc0d0462b89d6287ae38c81c7'}
def main():
 p=argparse.ArgumentParser();p.add_argument('--original',required=True,type=Path);p.add_argument('--utmt',required=True,type=Path);a=p.parse_args()
 original=a.original.resolve();cli=a.utmt.resolve();f=ROOT/'full-patch';out=ROOT/'build';out.mkdir(exist_ok=True)
 digest=hashlib.sha256(original.read_bytes()).hexdigest()
 platform=next((name for name,value in EXPECTED.items() if digest==value),None)
 if platform is None:p.error('Original archive version mismatch; use an unmodified supported game.ios/data.win or your verified backup.')
 env=dict(os.environ,HEBREW_PATCH_ROOT=str(ROOT))
 def run(args):subprocess.run([str(x) for x in args],cwd=ROOT,env=env,check=True)
 names=['gml_Script_textdata_en','gml_Object_obj_base_writer_Create_0','gml_Object_obj_base_writer_Draw_0','gml_Object_obj_asriel_body_Draw_0','gml_Script_scr_drawtext_icons','gml_Script_scr_drawtext_icons_multiline']
 args=[cli,'dump',original,'-o',ROOT/'decompiled']
 for n in names:args+=['-c',n]
 run(args)
 special=f/'render-decompiled/CodeEntries';special.mkdir(parents=True,exist_ok=True)
 for n in names[3:]:shutil.copy2(ROOT/'decompiled/CodeEntries'/f'{n}.gml',special/f'{n}.gml')
 run([sys.executable,ROOT/'extract_source_catalog.py'])
 run([sys.executable,ROOT/'validate_translation_catalog.py'])
 run([cli,'load',original,'-s',f/'export_all_fonts.csx'])
 for n in ['generate_fonts.py','build_translation_script.py','build_writer.py','build_special_render.py']:run([sys.executable,f/n])
 target=out/('data.hebrew-steam.win' if platform=='Windows' else 'game.hebrew-steam.ios')
 run([cli,'load',original,'-s',f/'apply_fonts.csx','-s',f/'apply_runtime.csx','-s',f/'render_hooks.csx','-o',target,'--overwrite'])
 run([cli,'load',target,'-s',f/'verify_full.csx','-s',f/'verify_steam.csx'])
 print('Built:',target)
if __name__=='__main__':main()
