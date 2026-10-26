from pathlib import Path
from layout import write_layouts
root=Path(__file__).resolve().parents[1]/'.local/runtime-fixture'
write_layouts(['שלום עולם','שלום','כן','שלום 123'],root/'Data/Localizations/he/layouts.tsv')
print(root)
