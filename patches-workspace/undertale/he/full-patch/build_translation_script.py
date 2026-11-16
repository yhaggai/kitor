from pathlib import Path
import json
import sys

ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT))
from translation_catalog import read_catalog
from validate_translation_catalog import validate
catalog=read_catalog()
source=json.loads((ROOT/'translation-work/catalog-before-full-translation.json').read_text())
result=validate(catalog,source)
assert not result['errors'],result['errors']
def gml(s):
    assert not any(c in s for c in '\n\r\0')
    if '"' not in s:return '"'+s+'"'
    if "'" not in s:return "'"+s+"'"
    return '('+' + \'"\' + '.join('"'+x+'"' for x in s.split('"'))+')'
lines=['global.text_data_en = ds_map_create();']
for e in catalog['entries']:
    lines.append('ds_map_add(global.text_data_en, '+gml(e['key'])+', '+gml(e['hebrew'])+');')
(ROOT/'full-patch/gml_Script_textdata_en.gml').write_text('\n'.join(lines)+'\n')
print('Generated',len(catalog['entries']),'translated text map entries')
