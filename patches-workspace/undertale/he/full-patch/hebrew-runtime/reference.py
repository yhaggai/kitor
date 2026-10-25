"""Executable Python equivalent of the restricted GML runtime, no dependencies.
Not a Unicode conformance suite. Logical source indices remain 1-based.
"""
import re
import unittest

MIRROR = str.maketrans('([{<>}])', ')]}><{[(')

def typ(c):
    cp = ord(c)
    if 1425 <= cp <= 1469 or cp in (1471,1473,1474,1476,1477,1479) or 768 <= cp <= 879: return 5
    if 1488 <= cp <= 1524 or cp == 8207: return 1
    if cp == 8206: return 0
    if 48 <= cp <= 57: return 3
    if 1632 <= cp <= 1641: return 4
    if 65 <= cp <= 90 or 97 <= cp <= 122 or 192 <= cp <= 687: return 0
    if 1536 <= cp <= 1791: return 2
    if cp in (32,9,160): return 10
    if c in '+-': return 6
    if cp in (36,37,162,163,165,8362,8364): return 7
    if c in ',./:': return 8
    return 9

def bidi(text):
    n = len(text)
    ts = [typ(c) for c in text]
    orig = ts[:]
    base = int(any(t in (1,2) for t in ts))
    previous = base
    for i,t in enumerate(ts):
        if t == 5: ts[i] = previous
        previous = ts[i]
    previous = base
    for i,t in enumerate(ts):
        if t == 3 and previous == 2: ts[i] = 4
        if t in (0,1,2): previous = t
        if t == 2: ts[i] = 1
    for i in range(1,n-1):
        if ts[i] == 6 and ts[i-1] == ts[i+1] == 3: ts[i] = 3
        elif ts[i] == 8 and ts[i-1] in (3,4) and ts[i-1] == ts[i+1]: ts[i] = ts[i-1]
    for i in range(n-1):
        if ts[i] == 6 and ts[i+1] in (3,4): ts[i] = ts[i+1]
    i = 0
    while i < n:
        if ts[i] != 7: i += 1; continue
        j = i
        while j < n and ts[j] == 7: j += 1
        if (ts[i-1] if i else base) == 3 or (ts[j] if j < n else base) == 3:
            ts[i:j] = [3]*(j-i)
        i = j
    previous = base
    for i,t in enumerate(ts):
        if t in (6,7,8): ts[i] = 9
        if ts[i] in (0,1): previous = ts[i]
        elif ts[i] == 3 and previous == 0: ts[i] = 0
    stack, pairs = [], []
    for i,c in enumerate(text):
        if c in '([{': stack.append(i)
        elif c in ')]}':
            want = '([{'[')]}'.index(c)]
            for j in range(len(stack)-1,-1,-1):
                if text[stack[j]] == want:
                    pairs.append((stack[j],i)); stack = stack[:j]; break
    strong = lambda t: 1 if t in (3,4) else t
    for lo,hi in sorted(pairs):
        inner = [strong(t) for t in ts[lo+1:hi]]
        resolved = None
        if base in inner: resolved = base
        elif 1-base in inner:
            resolved = next((strong(ts[j]) for j in range(lo-1,-1,-1) if strong(ts[j]) in (0,1)),base)
        if resolved is not None: ts[lo] = ts[hi] = resolved
    i = 0
    while i < n:
        if ts[i] not in (9,10): i += 1; continue
        j = i
        while j < n and ts[j] in (9,10): j += 1
        before = strong(ts[i-1]) if i else base
        after = strong(ts[j]) if j < n else base
        ts[i:j] = [before if before == after else base]*(j-i)
        i = j
    levels = [(1 if t == 1 else 2 if t in (3,4) else 0) if base == 0 else (2 if t in (0,3,4) else 1) for t in ts]
    for i in range(n-1,-1,-1):
        if orig[i] != 10: break
        levels[i] = base
    order = list(range(n))
    for level in range(max(levels,default=base),0,-1):
        i = 0
        while i < n:
            if levels[order[i]] < level: i += 1; continue
            j = i
            while j < n and levels[order[j]] >= level: j += 1
            order[i:j] = order[i:j][::-1]; i = j
    out = {}
    for col,i in enumerate(order): out[i+1] = (col,text[i].translate(MIRROR) if levels[i]%2 else text[i],levels[i])
    return out

def layout(text,capacity,plain=False):
    capacity = max(1,int(capacity))
    result = {i:{'col':-1,'row':0,'char':'','kind':0,'anchor':-1,'width':0} for i in range(1,len(text)+1)}
    tokens = []
    anchor = -1; i = 0; stop = 0
    while i < len(text):
        c = text[i]; consumed = 1; kind = 1; source = i+1
        if not plain and c in '/%': stop = i+1; break
        if (not plain and c == '&') or (plain and c == '#') or c in '\r\n':
            tokens.append((i+1,'',1,anchor)); result[i+1]['kind'] = 3; anchor = -1; kind = 0
            if text[i:i+2] == '\r\n': consumed = 2
        elif not plain and c == '^' and i+1 < len(text):
            if text[i+1] in '0123456789': consumed = 2; kind = 0
        elif not plain and c == '\\':
            kind = 0; consumed = min(2,len(text)-i); code = text[i+1:i+2]
            if code and code in 'EMFTSz*>':
                consumed = min(3,len(text)-i); param = text[i+2:i+3]
                if code == '>':
                    anchor = int(param=='1'); tokens.append((i+1,'',2,anchor))
                elif code in '*z' and param:
                    c = '\ufffc'; kind = 2; source = i+3
        if kind:
            tokens.append((source,c,0,anchor)); result[source]['kind'] = kind
        i += consumed
    row = 0; pos = 0; anchor = -1
    while pos < len(tokens):
        src,c,kind,unused = tokens[pos]
        if kind == 1:
            result[src]['row'] = row; row += 1; anchor = -1; pos += 1; continue
        if kind == 2: anchor = unused; pos += 1; continue
        end = pos
        while end < len(tokens) and tokens[end][2] == 0: end += 1
        linecap = capacity
        entire = ''.join(t[1] for t in tokens[pos:end])
        if re.search(r'[^ ] {2,}[^ ]',entire): linecap = max(capacity,end-pos)
        while pos < end:
            limit = min(end,pos+linecap)
            if limit < end:
                spaces = [j for j in range(pos+1,limit) if tokens[j][1]==' ']
                if spaces: limit = spaces[-1]
            content = ''.join(t[1] for t in tokens[pos:limit])
            columns = bool(re.search(r'[^ ] {2,}[^ ]',content))
            rtl = any(typ(c)==1 for c in content)
            offset = max(0,capacity-len(content)) if rtl and not plain and not columns and anchor == -1 else 0
            # Match GML fixed leading/trailing/multiple whitespace fields.
            cursor = 0
            for m in re.finditer(r'^ +| {2,}| +$',content):
                if cursor < m.start():
                    for local,(col,glyph,level) in bidi(content[cursor:m.start()]).items():
                        x = result[tokens[pos+cursor+local-1][0]]; x.update(col=offset+cursor+col,row=row,char=glyph,anchor=anchor,width=1)
                for j in range(m.start(),m.end()):
                    result[tokens[pos+j][0]].update(col=offset+j,row=row,char=' ',anchor=anchor,width=1)
                cursor = m.end()
            if cursor < len(content):
                for local,(col,glyph,level) in bidi(content[cursor:]).items():
                    result[tokens[pos+cursor+local-1][0]].update(col=offset+cursor+col,row=row,char=glyph,anchor=anchor,width=1)
            pos = limit
            if pos < end:
                while pos < end and tokens[pos][1] == ' ':
                    result[tokens[pos][0]]['kind'] = 0; pos += 1
                if pos < end: row += 1
    return result, {'rows':row+1,'stop':stop}

def visual(text):
    if len(text)<2 or not any(typ(c)==1 for c in text): return text
    result,meta = layout(text,max(1,len(text)),True)
    return '#'.join(''.join(v['char'] for _,v in sorted(result.items(),key=lambda x:x[1]['col']) if v['col']>=0 and v['row']==row) for row in range(meta['rows']))

class RuntimeTests(unittest.TestCase):
    def test_hebrew(self): self.assertEqual(visual('שלום'), 'םולש')
    def test_latin_noop(self): self.assertEqual(visual('HP 10/20#100%'), 'HP 10/20#100%')
    def test_number_order(self): self.assertEqual(visual('שלום 123'), '123 םולש')
    def test_decimal(self): self.assertEqual(visual('מחיר 12.50'), '12.50 ריחמ')
    def test_numeric_sign(self): self.assertIn('-12',visual('ערך -12')) # see documented sign limit
    def test_latin_run(self): self.assertEqual(visual('שלום HP 123'), 'HP 123 םולש')
    def test_parentheses(self): self.assertEqual(visual('שלום (HP 10)'), '(HP 10) םולש')
    def test_nested(self): self.assertEqual(visual('(שלום [ABC])'), '([ABC] םולש)')
    def test_line_order(self): self.assertEqual(visual('שלום#עולם'), 'םולש#םלוע')
    def test_plain_punctuation(self): self.assertEqual(visual('שלום 50%'), '50% םולש')
    def test_choice_columns(self): self.assertEqual(visual('   כן        לא  '), '   ןכ        אל  ')
    def test_choice_no_wrap(self):
        r,m=layout('   כן        לא  ',5)
        self.assertEqual(m['rows'],1); self.assertEqual(r[4]['col'],4); self.assertEqual(r[14]['col'],14)
    def test_inline_controls(self):
        t='\\E1* שלום^1 עולם/%%'; r,m=layout(t,20)
        self.assertTrue(all(r[i]['col']==-1 for i in (1,2,3,10,11)))
        self.assertEqual(m['stop'],17); self.assertEqual(r[6]['col'],17)
    def test_icon(self):
        r,m=layout('שלום \\*Z פה',20)
        self.assertEqual(r[8]['kind'],2); self.assertGreaterEqual(r[8]['col'],0)
        self.assertEqual(r[6]['col'],-1)
    def test_anchor(self):
        r,m=layout('\\>0כן\\>1לא',20)
        self.assertEqual(r[4]['anchor'],0); self.assertEqual(r[9]['anchor'],1)
        self.assertEqual(r[4]['col'],1); self.assertEqual(r[9]['col'],1)
    def test_wrap(self):
        r,m=layout('שלום עולם יפה',6)
        self.assertEqual(m['rows'],3)
        self.assertTrue(all(v['col']<6 for v in r.values()))
    def test_logical_reveal(self):
        r,m=layout('שלום',10)
        self.assertEqual([r[i]['col'] for i in range(1,5)],[9,8,7,6])
    def test_terminator(self):
        r,m=layout('שלום/לא מצויר',20)
        self.assertTrue(all(r[i]['col']==-1 for i in range(5,14)))
    def test_color_does_not_split(self):
        r,m=layout('אב\\Rגד',4)
        self.assertEqual([r[i]['col'] for i in (1,2,5,6)],[3,2,1,0])
    def test_explicit_empty_lines(self): self.assertEqual(visual('שלום##עולם'), 'םולש##םלוע')
    def test_crlf(self): self.assertEqual(visual('שלום\r\nעולם'), 'םולש#םלוע')

if __name__=='__main__': unittest.main()
