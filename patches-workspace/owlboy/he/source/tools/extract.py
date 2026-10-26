"""Create a local source reference from an owned game; never commit the output."""
import argparse,csv,hashlib,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
ap=argparse.ArgumentParser();ap.add_argument('--game',required=True,type=Path);args=ap.parse_args()
contract=json.loads((ROOT/'catalog-contract.json').read_text(encoding='utf-8'));out=[]
for i,row in enumerate(list(csv.reader((args.game/'Data/Localizations/de.csv').open(encoding='utf-8-sig')))[1:]):
 if row[2]=='language_id':continue
 key=f'{i:04d}'
 if key not in contract or hashlib.sha256(row[2].encode()).hexdigest()!=contract[key]['sha256']:raise SystemExit('Edition mismatch: '+key)
 out.append({'id':key,'file':row[4],'line':row[5],'source':row[2]})
path=ROOT/'.local/source.json';path.parent.mkdir(exist_ok=True);path.write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(path)
