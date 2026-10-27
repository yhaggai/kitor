"""Map AdventureBrew Hebrew speech to Special Edition voice IDs, read-only."""
import argparse
from collections import defaultdict
import hashlib
import json
from pathlib import Path
import re
from resources import speech_rows

PREFIX = re.compile(rb'^(?:\\xFF\\x0A\\x[0-9A-Fa-f]{2}\\x[0-9A-Fa-f]{2}){4}')

def map_hebrew(data, hebrew, table):
    ids = {}
    for line in table.splitlines():
        offset, name = int(line[:8],16), line[8:].decode('ascii')
        if offset in ids and ids[offset] != name:
            raise ValueError('Conflicting voice offset')
        ids[offset] = name
    candidates = defaultdict(list)
    for number, line in enumerate(hebrew.splitlines(),1):
        match = PREFIX.match(line)
        if not match:
            continue
        control = bytes(int(x,16) for x in re.findall(rb'\\x([0-9A-Fa-f]{2})',match[0]))
        offset = int.from_bytes(control[2:4]+control[6:8],'little')
        if offset in ids:
            candidates[ids[offset]].append({'source_line':number,'hebrew':line[match.end():].decode('cp1255')})
    mapped, review = [], []
    for row in speech_rows(data):
        options = candidates.get(row['id'],[])
        texts = {x['hebrew'] for x in options}
        reason = ('unmapped' if not texts else 'ambiguous' if len(texts)!=1
                  else 'control_codes' if '\\x' in next(iter(texts)) else None)
        if reason:
            review.append({**row,'reason':reason,'candidates':options})
        else:
            mapped.append({**row,'hebrew':next(iter(texts)), 'source_lines':[x['source_line'] for x in options]})
    return mapped, review


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    for name in ('game','hebrew','table','output'):
        ap.add_argument('--'+name,type=Path,required=True)
    args = ap.parse_args()
    if args.output.resolve().is_relative_to(args.game.resolve()):
        raise SystemExit('Output must be outside the game directory')
    data = (args.game/'audio/speech.info').read_bytes()
    hebrew, table = args.hebrew.read_bytes(), args.table.read_bytes()
    mapped, review = map_hebrew(data,hebrew,table)
    report = {'speech_records':len(speech_rows(data)),'mapped':len(mapped),'review':len(review),
              'speech_sha256':hashlib.sha256(data).hexdigest(),
              'hebrew_sha256':hashlib.sha256(hebrew).hexdigest(),
              'voice_table_sha256':hashlib.sha256(table).hexdigest(),'release_ready':False}
    args.output.mkdir(parents=True,exist_ok=True)
    for name,value in [('mapping.json',mapped),('review.json',review),('report.json',report)]:
        (args.output/name).write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(report,indent=2))

if __name__ == '__main__':
    main()
