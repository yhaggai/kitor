from pathlib import Path
from layout import write_layouts
root=Path(__file__).resolve().parents[1]/'.local/runtime-fixture'
write_layouts(['שלום עולם','שלום','כן','שלום 123'],root/'Data/Localizations/he/layouts.tsv')
print(root)

import base64
extra=root/'Data/Localizations/hn/layouts.tsv';extra.parent.mkdir(parents=True,exist_ok=True)
b64=lambda s:base64.b64encode(s.encode()).decode()
extra.write_text(b64('\ue200\ue201 123')+'\t'+b64('123 \ue201\ue200')+'\t3,4,5,2,1,0\n')
