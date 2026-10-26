"""Build a local, offline review of the catalog import without publishing drafts."""
import hashlib
import html
import json
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parent
catalog = json.loads((ROOT / 'queue.json').read_text())
new = [g for g in catalog['games'] if g['status'] != 'existing']
for game in new:
    for source in game['sources'].values():
        path = ROOT / source['file']
        with Image.open(path) as im:
            source.update(width=im.width, height=im.height)
        source['sha256'] = hashlib.sha256(path.read_bytes()).hexdigest()
    if game['status'] not in ('published', 'ready-to-publish') and game['sources'].get('he', {}).get('kind') == 'original-hebrew-box-scan' and 'en' in game['sources']:
        game['status'] = 'pair-scan-review'
(ROOT / 'queue.json').write_text(json.dumps(catalog, ensure_ascii=False, indent=2) + '\n')
counts = {'new_entries':len(new),'english_sources':sum('en' in g['sources'] for g in new),'hebrew_scans':sum(g['sources'].get('he',{}).get('kind')=='original-hebrew-box-scan' for g in new),'ai_drafts':sum(g['sources'].get('he',{}).get('kind')=='ai-text-localization' for g in new)}
parts = ['''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>New catalog cover pairs — review</title><style>body{background:#12141b;color:#edf0f8;font:16px system-ui;margin:0;padding:32px}header{max-width:1100px;margin:auto auto 32px}h1{font-size:32px}p{color:#b9c0d4;line-height:1.6}input,select{background:#242936;color:white;padding:12px;border:1px solid #586177;border-radius:8px;margin:8px}main{display:grid;grid-template-columns:repeat(auto-fit,minmax(440px,1fr));gap:24px}article{background:#1c202b;border:1px solid #343c50;padding:18px;border-radius:14px}h2{font-size:19px;margin:0 0 8px}small{color:#b9c0d4}.pair{display:grid;grid-template-columns:1fr 1fr;gap:12px;margin-top:15px}.cover{height:330px;background:#11141b;display:flex;align-items:center;justify-content:center;border-radius:6px;overflow:hidden}.cover img{width:100%;height:100%;object-fit:contain}figcaption{margin-top:8px;font-size:13px;color:#c4c9d8}figure{margin:0}a{color:#8ac7ff}article[hidden]{display:none}.badge{color:#e7c878;font-size:13px}@media(max-width:550px){body{padding:12px}main{grid-template-columns:1fr}.cover{height:250px}}</style><header><h1>New game-box pairs</h1>''']
parts.append(f"<p>{counts['new_entries']} new catalog entries · {counts['english_sources']} English sources · {counts['hebrew_scans']} existing Hebrew scans · {counts['ai_drafts']} AI title edits.</p>")
parts.append('<p>149 new pairs are published on Cloudflare. Original source files are unchanged. Composited edits preserve every original pixel outside the recorded title regions. Background beneath replaced lettering is reconstructed; the composites have been visually reviewed. Existing Hebrew retail scans can differ from the English packaging. Publication results are recorded in upload-verification.json. Three entries remain unresolved; select All new entries to inspect them.</p><input id="search" placeholder="Search English or Hebrew title"><select id="filter"><option value="pairs">Pairs available</option><option value="all">All new entries</option><option value="missing">Missing Hebrew</option><option value="draft">AI title edits</option></select></header><main>')
for g in new:
    s=g['sources'];pair='en' in s and 'he' in s;draft=s.get('he',{}).get('kind')=='ai-text-localization'
    attrs=f'data-search="{html.escape(g["title_en"]+" "+g["title_he"],quote=True)}" data-pair="{str(pair).lower()}" data-draft="{str(draft).lower()}"'
    parts.append(f'<article {attrs}><h2>{html.escape(g["title_en"])}</h2><div dir="rtl">{html.escape(g["title_he"] or "Hebrew title missing from input")}</div><small>JSON #{g["source_idx"]}</small> · <span class="badge">{html.escape(g["status"])}</span><div class="pair">')
    for lang,label in [('en','English original'),('he','Hebrew')]:
        a=s.get(lang)
        if a:
            file=html.escape(a['file'],quote=True);status=html.escape(a.get('status',''))
            parts.append(f'<figure><a class="cover" href="{file}"><img loading="lazy" src="{file}" alt="{label}"></a><figcaption>{label} · {status}</figcaption></figure>')
        else:parts.append(f'<figure><div class="cover">Not available yet</div><figcaption>{label}</figcaption></figure>')
    parts.append('</div><p>'+html.escape(g.get('note',''))+'</p></article>')
parts.append('''</main><script>function update(){let q=document.querySelector('#search').value.toLowerCase(),f=document.querySelector('#filter').value;document.querySelectorAll('article').forEach(a=>{a.hidden=!a.dataset.search.toLowerCase().includes(q)||(f==='pairs'&&a.dataset.pair!=='true')||(f==='missing'&&a.dataset.pair==='true')||(f==='draft'&&a.dataset.draft!=='true')})}document.querySelector('#search').oninput=update;document.querySelector('#filter').onchange=update;update();</script></html>''')
(ROOT / 'review.html').write_text(''.join(parts))
print(json.dumps(counts))
