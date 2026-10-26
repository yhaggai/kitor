"""Extend locally owned atlases with original Hebrew pixel glyphs."""
import io,json
from pathlib import Path
from PIL import Image,ImageDraw,ImageFont
ROOT=Path(__file__).resolve().parents[1]
PATTERNS=json.loads((ROOT/'fonts/hebrew-5x7.json').read_text(encoding='utf-8'))
def png(image):
 out=io.BytesIO();image.save(out,format='PNG');return out.getvalue()
def glyph(c,height,factor=1):
 rows=PATTERNS[c];tile=Image.new('RGBA',(len(rows[0])*factor,height));d=ImageDraw.Draw(tile)
 top=(3 if c=='ל' else 5)*factor
 if height==14:top=1 if c=='ל' else 3
 for y,row in enumerate(rows):
  for x,v in enumerate(row):
   if v=='1':d.rectangle((x*factor,top+y*factor,(x+1)*factor-1,top+(y+1)*factor-1),fill='white')
 return tile

def extend(folder,required):
 name=folder.name;chars=(folder/'charmap.txt').read_text(encoding='utf-8');lines=(folder/'info.txt').read_text(encoding='utf-8').splitlines()
 if lines[0].startswith('OffsetY='):header=4
 else:header=3
 boxes=[list(map(int,r.split())) for r in lines[header].split(',')]
 if len(boxes)!=len(chars):raise ValueError('Unsupported font map: '+str(folder))
 original=Image.open(folder/(name+'.png')).convert('RGBA');height=boxes[0][3];factor=2 if height>=28 else 1
 extras=[];x=original.width+1
 for c in sorted(set(PATTERNS)|set(required)):
  if c in chars or c in '\n\r\t':continue
  if c in PATTERNS:tile=glyph(c,height,factor)
  elif c.isascii() and c.isprintable():
   # Pillow's embedded font supplies rare ASCII punctuation absent from the atlas.
   tile=Image.new('RGBA',(8*factor,height));ImageDraw.Draw(tile).text((0,2*factor),c,font=ImageFont.load_default(size=10*factor),fill='white')
   box=tile.getbbox()
   if not box:raise ValueError('Empty glyph '+repr(c))
   tile=tile.crop((0,0,box[2],height))
  else:raise ValueError('Unsupported glyph '+repr(c))
  assert tile.getbbox();boxes.append([x,0,tile.width,height]);extras.append((x,tile));x+=tile.width+1;chars+=c
 atlas=Image.new('RGBA',(x,height));atlas.paste(original,(0,0))
 for offset,tile in extras:atlas.paste(tile,(offset,0))
 assert all(x>=0 and y>=0 and x+w<=atlas.width and y+h<=atlas.height for x,y,w,h in boxes)
 metadata='\n'.join(lines[:header])+ '\n'+','.join(' '.join(map(str,r)) for r in boxes)+'\n'
 return {name+'.png':png(atlas),'charmap.txt':chars.encode(),'info.txt':metadata.encode()},dict(zip(chars,[b[2]+float(lines[header-3]) for b in boxes]))

def title(text):
 lines=text.split('\n');factor=3;width=max(sum((len(PATTERNS[c][0])+1 if c!=' ' else 4)*factor for c in line) for line in lines)
 out=Image.new('RGBA',(width+8,len(lines)*32+8))
 for row,line in enumerate(lines):
  tiles=[glyph(c,16).crop((0,3,len(PATTERNS[c][0]),14)) if c!=' ' else Image.new('RGBA',(3,11)) for c in line[::-1]]
  x=(out.width-sum((t.width+1)*factor for t in tiles))//2
  for tile in tiles:out.alpha_composite(tile.resize((tile.width*factor,tile.height*factor),Image.Resampling.NEAREST),(x,row*32));x+=(tile.width+1)*factor
 return png(out)
TITLES={'sprTittelVellie_58x37.png':'וולי','sprTitleAdvent_183x37.png':'אדוונט','sprTitleOwlTemple_117x48.png':'מקדש\nהינשופים','sprTittelSvevendeKontinent_232x37.png':'היבשת המרחפת','sprTittelPiratmoderskipet_175x45.png':'ספינת האם\nשל הפיראטים'}
