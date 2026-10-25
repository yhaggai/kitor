"""Validate catalog identity and Undertale control syntax without modifying translations.
Run: python3 validate_translation_catalog.py
Width warnings are conservative text-length hints, not font/pixel measurements.
"""
from pathlib import Path
from collections import Counter
import json,re,sys
ROOT=Path(__file__).resolve().parent
CONTROL=re.compile(r'\\\[[^\]]+\]|\\[EMFTSz*>][\s\S]|\\[\s\S]|\^[0-9]|[/%%]')
HEBREW=re.compile(r'[א-ת]')
POINTS=re.compile(r'[\u0591-\u05bd\u05bf-\u05c7]')
def validate(catalog,source):
    errors=[];warnings=[]
    entries=catalog['entries'];original=source['entries']
    if len(entries)!=len(original):errors.append({'kind':'entry_count','expected':len(original),'actual':len(entries)})
    if catalog['entry_count']!=len(entries):errors.append({'kind':'metadata_count'})
    for i,(e,s) in enumerate(zip(entries,original)):
        key=e.get('key');h=e.get('hebrew','');eng=e.get('english','')
        if (key,eng)!=(s['key'],s['english']):errors.append({'key':key,'kind':'source_changed','index':i})
        if not isinstance(h,str):errors.append({'key':key,'kind':'not_string'});continue
        status=e.get('status')
        if status not in ('draft','reviewed','unchanged'):errors.append({'key':key,'kind':'unfinished','status':status})
        if eng.strip() and not h.strip():errors.append({'key':key,'kind':'empty_translation'})
        if status=='unchanged' and h!=eng:errors.append({'key':key,'kind':'unchanged_mismatch'})
        a=Counter(CONTROL.findall(eng));b=Counter(CONTROL.findall(h))
        if a!=b:errors.append({'key':key,'kind':'control_mismatch','missing':dict(a-b),'added':dict(b-a)})
        if POINTS.search(h):warnings.append({'key':key,'kind':'hebrew_points_or_special_punctuation'})
        if any(c in h for c in '\u202a\u202b\u202c\u202d\u202e\u2066\u2067\u2068\u2069'):errors.append({'key':key,'kind':'bidi_control_character'})
        if e.get('translator_note'):warnings.append({'key':key,'kind':'translator_note','note':e['translator_note']})
        if (eng.count('&'),eng.count('#'))!=(h.count('&'),h.count('#')):warnings.append({'key':key,'kind':'linebreak_count','english':[eng.count('&'),eng.count('#')],'hebrew':[h.count('&'),h.count('#')]})
        # Only compare stateful command order; pauses and placeholders may move naturally in Hebrew.
        state=lambda text:[t for t in CONTROL.findall(text) if t.startswith('\\') and not t.startswith('\\[')]
        if state(eng)!=state(h):warnings.append({'key':key,'kind':'command_order','english':state(eng),'hebrew':state(h)})
        plain=CONTROL.sub('',h)
        width=max((len(s) for s in re.split('[&#\n]',plain)),default=0)
        if width>30:warnings.append({'key':key,'kind':'layout_width_hint','max_chars':width})
        if status!='unchanged' and re.search('[A-Za-z]',eng) and not HEBREW.search(h):warnings.append({'key':key,'kind':'no_hebrew_letters'})
    return {'entry_count':len(entries),'statuses':dict(Counter(e.get('status') for e in entries)),'errors':errors,'error_count':len(errors),'warning_count':len(warnings),'warning_categories':dict(Counter(w['kind'] for w in warnings)),'warnings':warnings,'scope':'Catalog syntax and source identity only. Drafts need editorial review; full-game font, RTL, dynamic-string and layout integration are not validated.'}
if __name__=='__main__':
    catalog=json.loads((ROOT/'translation-catalog.json').read_text())
    source=json.loads((ROOT/'translation-work/catalog-before-full-translation.json').read_text())
    report=validate(catalog,source)
    (ROOT/'translation-validation.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k not in ('errors','warnings')},ensure_ascii=False,indent=2))
    if report['errors']:print(json.dumps(report['errors'][:20],ensure_ascii=False,indent=2))
    sys.exit(bool(report['errors']))
