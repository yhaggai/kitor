"""Build Unicode BiDi maps while retaining logical character identities."""
import base64,re
from bidi import algorithm as bidi
TOKEN=re.compile(r'<(?:\d+|b)>|\[\d+\]')
def plain(text):return TOKEN.sub('',text.replace('<nl>','\n'))
def layout(line):
 s=bidi.get_empty_storage();s['base_level']=1;s['base_dir']='R'
 bidi.get_embedding_levels(line,s,False,False)
 for i,c in enumerate(s['chars']):c['logical_index']=i
 bidi.explicit_embed_and_overrides(s,False);bidi.resolve_weak_types(s,False)
 bidi.resolve_neutral_types(s,False);bidi.resolve_implicit_levels(s,False)
 bidi.reorder_resolved_levels(s,False);bidi.apply_mirroring(s,False)
 order=[c['logical_index'] for c in s['chars']];visual=''.join(c['ch'] for c in s['chars'])
 if sorted(order)!=list(range(len(line))):raise ValueError('Bidi controls are not allowed in the editable catalog')
 return visual,order

def write_layouts(strings,path):
 variants=[]
 for text in strings:
  variants.extend([text,text.split('|',1)[0].lstrip('*')])
 lines=sorted({line for text in variants for line in plain(text).split('\n') if line})
 b64=lambda x:base64.b64encode(x.encode()).decode()
 records=[]
 for line in lines:
  visual,order=layout(line);records.append('\t'.join([b64(line),b64(visual),','.join(map(str,order))]))
 path.parent.mkdir(parents=True,exist_ok=True);path.write_text('\n'.join(records)+'\n',encoding='utf-8')
 return len(lines)
