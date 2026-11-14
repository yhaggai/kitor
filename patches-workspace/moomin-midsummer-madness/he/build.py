"""Build logical Hebrew catalog into the game's active English text slots."""
from __future__ import annotations
import argparse, collections, csv, hashlib, json, re, textwrap
import unicodedata
from pathlib import Path
import UnityPy
from bidi import algorithm as bidi

TOKEN = re.compile(r'(<[^<>\n]{1,100}>|\{[^{}\n]+\})')
NUM = re.compile(r'\d+')
NIQQUD = re.compile(r'[\u0591-\u05bd\u05bf\u05c1-\u05c2\u05c4-\u05c5\u05c7]')
CLUSTER = re.compile(r'[\u05d0-\u05ea][\u0591-\u05bd\u05bf\u05c1-\u05c2\u05c4-\u05c5\u05c7]+')
UNIT = re.compile(r'[\u05d0-\u05ea][\u0591-\u05bd\u05bf\u05c1-\u05c2\u05c4-\u05c5\u05c7]*|.',re.S)
PUNCT = str.maketrans({'׳':"'",'״':'"','־':'-','–':'-','—':'-','…':'...','“':'"','”':'"','„':'"','‘':"'",'’':"'"})
GLYPH_FILE=Path(__file__).with_name('niqqud-glyphs.json')
GLYPHS={unicodedata.normalize('NFD',k):v for k,v in json.loads(GLYPH_FILE.read_text(encoding='utf-8')).items()} if GLYPH_FILE.exists() else {}
ADVANCE_FILE=Path(__file__).with_name('dialogue-font-advances.json')
ADVANCE_DATA=json.loads(ADVANCE_FILE.read_text(encoding='utf-8'))
DIALOGUE_ADVANCES={int(k):v for k,v in ADVANCE_DATA['advances'].items()}
DIALOGUE_WIDTH=float(ADVANCE_DATA['bubble_width_limit'])

def wrap_dialogue(paragraph, units, offset, symbols):
    """Greedy word wrap against the shipped dialogue TMP glyph advances."""
    def advance(i):
        ch=paragraph[i]
        if ch in symbols:
            token=symbols[ch]
            if token.startswith('<'): return 0.0
            # Runtime substitutions can vary; reserve a short name-sized slot.
            return 8*DIALOGUE_ADVANCES.get(ord('א'),40.0)
        unit=units[offset+i]
        return DIALOGUE_ADVANCES.get(ord(unit[0]),DIALOGUE_ADVANCES.get(ord(ch),40.0))
    words=list(re.finditer(r'\S+',paragraph))
    if not words:return [('',0,len(paragraph))]
    lines=[];start=0;end=0;width=0.0
    for match in words:
        ws=paragraph[end:match.start()]
        word=paragraph[match.start():match.end()]
        word_width=sum(advance(i) for i in range(match.start(),match.end()))
        spacer_width=sum(advance(i) for i in range(end,match.start()))
        if end>start and width+spacer_width+word_width>DIALOGUE_WIDTH:
            lines.append((paragraph[start:end].rstrip(),start,end))
            start=match.start();end=match.end();width=word_width
        else:
            width+=spacer_width+word_width;end=match.end()
    lines.append((paragraph[start:].rstrip(),start,len(paragraph)))
    return lines

def visual(text: str, table: str) -> str:
    """Convert logical Hebrew to the game's visual-order text, one wrapped line at a time."""
    text=text.translate(PUNCT)
    if not re.search('[א-ת]', text): return text
    # Preserve tag syntax and association while bidi reorders the visible glyphs.
    chunks=TOKEN.split(text); stack=[]; closures={}; plain=[]; styles=[]; units=[]; symbols={}; offset=0
    for token in chunks:
        if not token: continue
        if TOKEN.fullmatch(token) and token.startswith('</'):
            if stack:
                opening=stack.pop(); closures[opening]=token
            continue
        if TOKEN.fullmatch(token) and token.startswith('<'):
            if not re.fullmatch(r'<(?:bold|italic|b|i)>',token):
                sym=chr(0xE000+len(symbols));symbols[sym]=token;token=sym
                plain.append(token);styles.append(tuple(stack));units.append(token);continue
            if token.startswith('<'):
                stack.append(token);continue
        if TOKEN.fullmatch(token):
            sym=chr(0xE000+len(symbols));symbols[sym]=token;token=sym
            plain.append(token);styles.append(tuple(stack));units.append(token);continue
        for m in UNIT.finditer(token):
            unit=m.group();plain.append(unit[0]);units.append(unit);styles.append(tuple(stack))
    # A remaining open style is malformed input; retain it as text and let validation report it.
    source=''.join(plain); output=[]; paragraph_offset=0
    dialogue=table.startswith('dialogue_')
    # Dialogue bubbles have a much narrower live text box than journal/menu
    # surfaces. Keep visual-order lines within that measured bubble width and
    # use TMP's explicit <br> tag: SpeechBubble normalizes literal newlines to
    # spaces before TMP lays out the text, so newline wrapping here was lost.
    width=52 if table.startswith(('chapter_recap','journal_')) else 46
    for paragraph in source.split('\n'):
        if dialogue:
            lines=wrap_dialogue(paragraph,units,paragraph_offset,symbols)
        else:
            raw_lines=textwrap.wrap(paragraph,width=width,replace_whitespace=False,drop_whitespace=False,
                                    break_long_words=False,break_on_hyphens=False) or ['']
            lines=[];line_offset=0
            for raw_line in raw_lines:
                lines.append((raw_line,line_offset,line_offset+len(raw_line)))
                line_offset+=len(raw_line)
            if line_offset!=len(paragraph):raise ValueError('Non-dialogue wrapping lost source characters')
        for line,line_start,line_end in lines:
            state=bidi.get_empty_storage();state.update(base_level=1,base_dir='R')
            bidi.get_embedding_levels(line,state)
            for i,char in enumerate(state['chars']):
                source_index=paragraph_offset+line_start+i
                if char['ch']!=source[source_index]:raise ValueError('Wrapped visual line lost source alignment')
                char['style']=styles[source_index];char['unit']=units[source_index]
            bidi.explicit_embed_and_overrides(state);bidi.resolve_weak_types(state);bidi.resolve_neutral_types(state,False);bidi.resolve_implicit_levels(state,False);bidi.reorder_resolved_levels(state,False);bidi.apply_mirroring(state,False)
            active=(); rendered=[]
            for char in state['chars']:
                style=char['style'];common=0
                while common<min(len(active),len(style)) and active[common]==style[common]:common+=1
                for tag in reversed(active[common:]): rendered.append(closures.get(tag,'</>'))
                rendered.extend(style[common:])
                unit=char['unit']
                if len(unit)>1 and '\u05d0'<=unit[0]<='\u05ea':
                    key=unicodedata.normalize('NFD',unit)
                    if key not in GLYPHS:raise ValueError(f'No composite glyph for niqqud cluster {key!r}')
                    rendered.append(chr(GLYPHS[key]))
                else:rendered.append(symbols.get(char['ch'],char['ch']))
                active=style
            for tag in reversed(active):rendered.append(closures.get(tag,'</>'))
            output.append(''.join(rendered).rstrip())
        paragraph_offset+=len(paragraph)+1
    return ('<br>' if dialogue else '\n').join(output)

def encode_niqqud_clusters(text):
    def encode(match):
        key=unicodedata.normalize('NFD',match.group())
        if key not in GLYPHS:raise ValueError(f'No composite glyph for niqqud cluster {key!r}')
        return chr(GLYPHS[key])
    return CLUSTER.sub(encode,text)

def read_catalog(path):
    if path.suffix.lower()!='.csv':
        rows=json.loads(path.read_text(encoding='utf-8'))
        return [{'path':r['path'],'en':r['en'],'he':r['he']} for r in rows]
    result=[];seen=set()
    with path.open(encoding='utf-8-sig',newline='') as f:
        reader=csv.DictReader(f)
        required={'stable_id','path','category','scene','speaker','en','he_unpointed','he_pointed'}
        if set(reader.fieldnames or ())!=required:raise ValueError(f'Unexpected catalog CSV columns: {reader.fieldnames}')
        for i,row in enumerate(reader,2):
            p=json.loads(row['path']);stable=json.dumps(p,ensure_ascii=False,separators=(',',':'))
            if stable!=row['stable_id']:raise ValueError(f'Stable ID/path mismatch at CSV row {i}')
            if stable in seen:raise ValueError(f'Duplicate stable ID at CSV row {i}: {stable}')
            seen.add(stable)
            if NIQQUD.sub('',unicodedata.normalize('NFD',row['he_pointed']))!=NIQQUD.sub('',unicodedata.normalize('NFD',row['he_unpointed'])):
                raise ValueError(f'Pointed text changed wording at CSV row {i}: {stable}')
            if collections.Counter(re.findall(r'\{[^{}\n]+\}|<[^<>\n]+>',row['he_unpointed']))!=collections.Counter(re.findall(r'\{[^{}\n]+\}|<[^<>\n]+>',row['he_pointed'])):
                raise ValueError(f'Markup/placeholders changed at CSV row {i}: {stable}')
            result.append({'path':p,'en':row['en'],'he':row['he_pointed'],'he_unpointed':row['he_unpointed']})
    return result

def pathset(root,path,value):
    x=root
    for key in path[:-1]: x=x[key]
    x[path[-1]]=value

def walk(x,path=()):
    if isinstance(x,dict):
        for k,v in x.items(): yield from walk(v,path+(k,))
    elif isinstance(x,list):
        for i,v in enumerate(x): yield from walk(v,path+(i,))
    elif isinstance(x,str) and path and path[-1]=='text': yield path,x

def markup_state(text):
    """Return opening rich-text tags after confirming all rich-text spans balance."""
    stack=[]
    for token in re.findall(r'<[^<>\n]{1,100}>',text):
        if not re.fullmatch(r'<(?:bold|italic|b|i)>|</(?:bold|italic|b|i)?>',token):
            continue
        if token.startswith('</'):
            if not stack: raise ValueError(f'Unmatched closing rich-text tag: {token}')
            opening=stack.pop()
            named=re.match(r'</(\w+)>',token)
            if named and re.match(r'<(\w+)',opening).group(1)!=named.group(1):
                raise ValueError(f'Mismatched rich-text tags: {opening} {token}')
        elif not token.endswith('/>'):
            stack.append(token)
    if stack: raise ValueError(f'Unclosed rich-text tags: {stack}')
    return collections.Counter(re.findall(r'<(bold|italic|b|i)>',text))

def build(original:Path,catalog:Path,destination:Path,report:Path):
    rows=read_catalog(catalog); by_path={tuple(r['path']):r for r in rows}
    if len(by_path)!=len(rows): raise ValueError('Duplicate stable paths in catalog')
    env=UnityPy.load(original.read_bytes()); assets=[o for o in env.objects if o.type.name=='TextAsset' and o.peek_name()=='GameDatabase']
    if len(assets)!=1: raise ValueError(f'Expected one GameDatabase TextAsset; found {len(assets)}')
    obj=assets[0];asset=obj.read();db=json.loads(asset.m_Script);seen=set();checks=[];expected_display={};translation_fields=0
    for path,source in walk(db):
        if path not in by_path: raise ValueError(f'Uncataloged player text: {path}')
        rec=by_path[path];seen.add(path)
        if rec['en']!=source: raise ValueError(f'Source mismatch: {path}')
        logical=rec['he'];display=visual(logical,str(path[0]))
        if re.search(r'<\w+>\s*</>',logical):
            raise ValueError(f'Empty styled span in logical Hebrew {path}')
        protected=r'\{[^{}\n]+\}|<sprite\s+[^<>]+/?>|<Eng>'
        if collections.Counter(re.findall(protected,source))!=collections.Counter(re.findall(protected,display)):
            raise ValueError(f'Placeholder/sprite mismatch {path}')
        original_tags=markup_state(source);display_tags=markup_state(display)
        if any(display_tags[tag]<count for tag,count in original_tags.items()):
            raise ValueError(f'Rich-text style lost in visual output {path}: {original_tags} != {display_tags}')
        source_nums=collections.Counter(re.findall(r'\d+',source));display_nums=collections.Counter(re.findall(r'\d+',display))
        if re.search(r'\b2D\b',source) and 'דו־ממד' in NIQQUD.sub('',logical):
            source_nums=collections.Counter(x for x in source_nums.elements() if x!='2')
        if source_nums!=display_nums: raise ValueError(f'Digit mismatch {path}: {source_nums} != {display_nums}')
        parent=db
        for key in path[:-1]: parent=parent[key]
        if 'data' in parent and 'en' in parent['data'] and 'translation' in parent['data']['en']:
            parent['data']['en']['translation']=display;translation_fields+=1
        pathset(db,path,display)
        expected_display[path]=display
    if seen!=set(by_path): raise ValueError(f'Unmatched catalog paths: {len(set(by_path)-seen)}')
    # Catch glyphs outside the patched Hebrew block; record other non-ASCII punctuation for review.
    unsupported=collections.Counter(ch for r in rows for ch in ''.join(x for x in r['he'].translate(PUNCT) if unicodedata.category(x)!='Mn') if ord(ch)>127 and not ('\u05d0'<=ch<='\u05ea') and ch not in '\n\r\t')
    asset.m_Script=json.dumps(db,ensure_ascii=False,separators=(',',':'))
    asset.save()
    blob=env.file.save(packer='lz4'); reopened=UnityPy.load(blob); final=[o for o in reopened.objects if o.type.name=='TextAsset' and o.peek_name()=='GameDatabase']
    finaldb=json.loads(final[0].read().m_Script); finalrows=list(walk(finaldb))
    if finaldb!=db:raise ValueError('Serialized GameDatabase differs from the catalog-built data')
    if len(finalrows)!=len(rows) or any(path not in expected_display or txt!=expected_display[path] for path,txt in finalrows):
        raise ValueError('Serialized visible text differs from the expected visual-order catalog')
    for path in expected_display:
        parent=finaldb
        for key in path[:-1]:parent=parent[key]
        if 'data' in parent and 'en' in parent['data'] and 'translation' in parent['data']['en']:
            if parent['data']['en']['translation']!=expected_display[path]:raise ValueError(f'English translation slot not serialized: {path}')
    destination.parent.mkdir(parents=True,exist_ok=True);destination.write_bytes(blob)
    report.write_text(json.dumps({'entries':len(rows),'covered':len(seen),'translation_fields':translation_fields,'serialized_visible_text_verified':True,'source_sha256':hashlib.sha256(original.read_bytes()).hexdigest(),'output_sha256':hashlib.sha256(blob).hexdigest(),'unsupported_nonascii':dict(unsupported),'db_fields':len(finalrows)},ensure_ascii=False,indent=2),encoding='utf8')
    print(f'Built {len(rows)} translated fields to {destination}; re-opened {len(finalrows)} text fields.')

if __name__=='__main__':
    ap=argparse.ArgumentParser();ap.add_argument('original',type=Path);ap.add_argument('catalog',type=Path);ap.add_argument('destination',type=Path);ap.add_argument('--report',type=Path,required=True);a=ap.parse_args();build(a.original,a.catalog,a.destination,a.report)
