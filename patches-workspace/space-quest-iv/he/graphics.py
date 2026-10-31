"""Indexed SCI1.1 graphics and heap title overrides for the Steam CD release."""
import struct
from collections import Counter
from itertools import groupby

from bidi.algorithm import get_display
from PIL import Image, ImageDraw, ImageFont


def uint32(data, at):
    return struct.unpack_from('<I', data, at)[0]


def bitmap(data, header):
    width, height = struct.unpack_from('<HH', data, header)
    control, literal = struct.unpack_from('<II', data, header + 24)
    if not (control and literal):
        start = control or literal
        pixels = data[start:start+width*height]
    else:
        pixels = bytearray()
        while len(pixels) < width * height:
            run = data[control]
            control += 1
            if run < 128:
                pixels.extend(data[literal:literal+run])
                literal += run
            elif run < 192:
                pixels.extend([data[literal]] * (run-128))
                literal += 1
            else:
                pixels.extend([data[header+8]] * (run-192))
    assert len(pixels) == width * height
    return Image.frombytes('L', (width, height), bytes(pixels))


def pack_pixels(im, clear):
    control, literal = bytearray(), bytearray()
    for y in range(im.height):
        row = im.crop((0,y,im.width,y+1)).tobytes()
        pending = bytearray()
        def flush():
            for i in range(0,len(pending),127):
                portion = pending[i:i+127]
                control.append(len(portion))
                literal.extend(portion)
            pending.clear()
        for color, group in groupby(row):
            n = sum(1 for _ in group)
            if n < 3 and color != clear:
                pending.extend([color]*n)
                continue
            flush()
            for i in range(0,n,63):
                control.append((192 if color == clear else 128) + min(n-i,63))
                if color != clear:
                    literal.append(color)
        flush()
    return control, literal


def mask(text, width, height, stretch=False):
    font = ImageFont.truetype('C:/Windows/Fonts/arialbd.ttf', max(10, height*2), layout_engine=ImageFont.Layout.BASIC)
    visual = get_display(text,base_dir='R')
    box = font.getbbox(visual)
    image = Image.new('L', (box[2]-box[0], box[3]-box[1]))
    ImageDraw.Draw(image).text((-box[0],-box[1]),visual,font=font,fill=255)
    scale = min(width/image.width, height/image.height)
    size = (width,height) if stretch else (max(1,round(scale*image.width)),max(1,round(scale*image.height)))
    return image.resize(size,Image.Resampling.LANCZOS).point(lambda p:255 if p>=103 else 0)


def label(im, rect, text, ink, background=None, height=None):
    x,y,w,h = rect
    if background is not None:
        im.paste(background,(x,y,x+w,y+h))
    tile=mask(text,w,min(height or h,h))
    im.paste(ink,(x+(w-tile.width)//2,y+(h-tile.height)//2),tile)


def view_edit(n,l,f,im):
    w,h=im.size
    if n==947:
        headings={2:'מהירות',3:'עוצמה',4:'פירוט',5:'המשחק מושהה',6:'קול פעיל',7:'טקסט פעיל'}
        buttons={2:'שמירה',3:'טעינה',4:'מחדש',5:'יציאה',8:'המשך',9:'כתוביות',10:'דיבור',11:'משולב',12:'שמע'}
        if l==0 and f in headings or (l,f)==(13,1):
            label(im,(0,0,w,h),headings.get(f,'מהירות'),10 if l==13 else 0,5,height=8)
        elif l in buttons and f in (0,1):
            label(im,(3,2,w-6,h-5),buttons[l],34 if f else 0,3 if f else 4,height=8)
        else:return False
    elif n==972:
        if (l,f)==(0,0):
            label(im,(4,6,22,9),'רגע',22,50,height=8)
        elif (l,f) in ((0,1),(1,0)):
            label(im,(0,4,w,7),'רגע',7,60 if l else 0,height=6)
        else:return False
    elif n==150:
        actions=['דיסק לקריאה','כתיבה לדיסק','קריאה מהקרן','כתיבה לקרן']
        if (l,f)==(0,0):
            # Each caption lies inside a raised button on the left of the screen.
            for i,caption in enumerate(actions):
                label(im,(6,5+i*19,43,14),caption,0,5,height=8)
        elif 2<=l<=5 and f==0:
            label(im,(3,2,w-6,h-4),actions[l-2],34,3,height=9)
        else:return False
    elif n==710 and l<7:
        roles=[('הפקה בפועל','קן ויליאמס'),('ניהול אמנותי','ביל דייוויס'),('עיצוב המשחק','סקוט מרפי','ומארק קרואו'),('הפקת התקליטור','דן קארבר'),('עיצוב אמנותי','מארק קרואו'),('תכנות ראשי','סקוט מרפי','ודאג אולדפילד'),('הלחנה','מארק סייברט','וקן אלן')]
        words=roles[l]
        im.paste(63,(0,0,w,h))
        label(im,(0,0,w,8),words[0],12,height=7)
        band=(h-8)//(len(words)-1)
        for i,name in enumerate(words[1:]):
            label(im,(0,8+i*band,w,band),name,12,height=band)
    elif n==393 and (l,f)==(0,0):
        label(im,(0,0,w,8),'ברוכים הבאים',34,219,height=7)
        label(im,(0,9,w,27),'הרץ נהדר',34,219,height=20)
        label(im,(0,36,w,9),'חטיבה של בלאץ',48,219,height=7)
    elif n==394 and (l,f)==(0,0):
        bg=Counter(im.tobytes()).most_common(1)[0][0]
        fg=Counter(im.tobytes()).most_common(2)[1][0]
        label(im,(0,0,w,9),'באקמאסטר',fg,bg)
        label(im,(0,h-9,w,9),'נא להכניס כרטיס',fg,bg)
    elif n==610 and l==0:
        colors=Counter(im.crop((7,1,w-4,h-1)).tobytes()).most_common(2)
        label(im,(7,1,w-11,h-2),'בר',colors[1][0],colors[0][0],height=9)
    else:return False
    return True


def patch_view(n,data):
    result=bytearray(data)
    header_size=int.from_bytes(data[:2],'little')+2
    loop_size,cel_size=data[12:14]
    for l in range(data[2]):
        base=header_size+l*loop_size
        if data[base]!=255:continue
        for f in range(data[base+2]):
            header=uint32(data,base+12)+f*cel_size
            im=bitmap(data,header)
            if not view_edit(n,l,f,im):continue
            control,literal=pack_pixels(im,data[header+8])
            struct.pack_into('<II',result,header+24,len(result),len(result)+len(control))
            result.extend(control+literal)
            assert bitmap(result,header).tobytes()==im.tobytes()
    return bytes(result)


def title(im):
    # Restore the original radial background under the English logo by
    # interpolating its palette ramp horizontally between surviving samples.
    for y in range(21,110):
        row=list(im.crop((0,y,im.width,y+1)).tobytes())
        valid=[x for x,p in enumerate(row) if p>=160]
        for a,b in zip(valid,valid[1:]):
            for x in range(a+1,b):
                im.putpixel((x,y),round(row[a]+(row[b]-row[a])*(x-a)/(b-a)))
    for text,x,y,w,h in [('מסע',53,24,213,39),('בחלל',30,68,260,39)]:
        face=mask(text,w,h,stretch=True)
        im.paste(130,(x+2,y+2),face)
        for yy in range(h):
            for xx in range(w):
                if face.getpixel((xx,yy)):
                    im.putpixel((x+xx,y+yy),min(157,131+(xx+x)//15+yy//10))
    return im


def subtitle(im):
    im.paste(255,(0,0,im.width,im.height))
    for text,x,y,w,h in [("רוג'ר וילקו",31,10,258,69),('וקורעי הזמן',12,98,296,64)]:
        tile=mask(text,w,h,stretch=True)
        for shift in range(5,-1,-1):
            im.paste(32 if shift>1 else 58,(x+shift,y+shift),tile)
        for yy in range(h):
            im.paste(66+round(48*yy/(h-1)),(x,y+yy),tile.crop((0,yy,w,yy+1)))
    return im


# Three corners describe each lettering panel in native picture coordinates.
# Colors are sampled from that picture's own SCI palette.
SIGNS = {
    15:[((142,66),(181,66),(140,82),'בר',(47,85,147),(200,222,251))],
    9900:[((142,66),(181,66),(140,82),'בר',(47,85,147),(200,222,251))],
    375:[((84,48),(167,15),(88,66),'ארקייד',(62,91,16),(134,246,44))],
    386:[((0,17),(116,17),(0,32),'דרושים',(230,223,200),(178,22,13)),
         ((6,54),(54,54),(6,80),'חובה ללבוש\nחולצה ונעליים',(201,206,204),(22,26,36))],
    396:[((50,5),(273,5),(50,33),'באקמאסטר 2000',(87,78,68),(244,155,40))],
    398:[((17,28),(130,2),(22,50),'מציאות!',(43,32,33),(250,245,221))],
    609:[((103,3),(181,3),(103,47),'טייני\nחלליות משומשות',(75,213,211),(21,22,25))],
    614:[((42,50),(211,50),(42,69),'דרואידים זה אנחנו',(222,45,215),(15,13,15)),
         ((108,126),(144,126),(108,135),'סגור',(205,205,205),(30,30,30))],
    700:[((6,4),(144,4),(6,16),'קלאק אגרס',(148,17,9),(255,245,214)),
         ((6,17),(144,17),(6,26),'סימולטור עופות מתקדם',(137,24,16),(255,243,227))],
    701:[((2,22),(151,22),(2,37),'ערכת בנייה במשבצות',(250,242,225),(232,49,53))],
    702:[((9,20),(148,20),(9,44),'בום',(62,62,137),(250,246,238))],
    704:[((4,6),(150,6),(4,31),'איפה חיימי ליפשיץ?\nולמי אכפת בכלל?',(36,77,58),(245,245,217))],
    709:[((4,23),(151,5),(7,54),'דקרון דני',(226,11,142),(255,248,224)),
         ((6,135),(153,135),(6,160),'מעללים מלוכלכים בחושך\nבבגדים אלגנטיים',(138,19,111),(255,248,234))],
}


def draw_sign(im,data,sign):
    import math
    origin,horizontal,vertical,text,bg,fg=sign
    pal=data[uint32(data,28):]
    step=3 if pal[32] else 4
    start=pal[25]
    count=int.from_bytes(pal[29:31],'little')
    def nearest(rgb):
        def distance(i):
            pos=37+step*i+step-3
            return sum((a-b)**2 for a,b in zip(rgb,pal[pos:pos+3]))
        return start+min(range(count), key=distance)
    x,y=origin
    ux,uy=horizontal[0]-x,horizontal[1]-y
    vx,vy=vertical[0]-x,vertical[1]-y
    width,height=round(math.hypot(ux,uy)),round(math.hypot(vx,vy))
    panel=Image.new('L',(width,height),nearest(bg))
    lines=text.splitlines()
    for i,line in enumerate(lines):
        band=height//len(lines)
        label(panel,(1,i*band,width-2,band),line,nearest(fg),height=max(1,band-2))
    det=ux*vy-uy*vx
    a,b=width*vy/det,-width*vx/det
    e,f=-height*uy/det,height*ux/det
    coeff=(a,b,-a*x-b*y,e,f,-e*x-f*y)
    warped=panel.transform(im.size,Image.Transform.AFFINE,coeff)
    alpha=Image.new('L',panel.size,255).transform(im.size,Image.Transform.AFFINE,coeff)
    im.paste(warped,(0,0),alpha)


def patch_pic(n,data):
    h=uint32(data,32)
    im=bitmap(data,h)
    if n in SIGNS:
        for sign in SIGNS[n]:draw_sign(im,data,sign)
    elif n==1:title(im)
    elif n==600:subtitle(im)
    elif n==392:
        label(im,(100,5,115,15),'ברוכים הבאים',66,93)
        label(im,(18,38,285,77),'הרץ נהדר',34,93,height=54)
        label(im,(64,162,196,26),'חטיבה של בלאץ',48,93,height=17)
    elif n==614:
        label(im,(48,50,141,16),'דרואידים זה אנחנו',74,65,height=11)
        label(im,(107,126,36,9),'סגור',74,65,height=6)
    elif n==703:
        label(im,(34,33,102,32),'מסע בחלל',70,81,height=20)
        label(im,(38,93,99,16),'ספר רמזים',70,81,height=11)
    else:raise ValueError(n)
    control,literal=pack_pixels(im,data[h+8])
    start=uint32(data,h+24)
    palette,vector=uint32(data,28),uint32(data,16)
    result=bytearray(data[:start]+control+literal+data[palette:])
    delta=start+len(control)+len(literal)-palette
    struct.pack_into('<II',result,h+24,start,start+len(control))
    struct.pack_into('<I',result,28,palette+delta)
    struct.pack_into('<I',result,16,vector+delta)
    assert bitmap(result,h).tobytes()==im.tobytes()
    return bytes(result)
def patch_heap(data):
    # The original scripts point directly to these strings. Keep every address
    # and Roman-numeral glyph, and place each complete title in its own slot.
    titles = [
        (b'Space Quest ',b''),
        (b'\x1b - The Sarien Encounter', ('מסע בחלל B: הסאריאנים', {'B':27})),
        (b'\x1c - The Pirates of Pestulon', ('מסע בחלל C: שודדי פסטולון', {'C':28})),
        (b'\x1a - Roger Wilco and The Time Rippers', ("מסע בחלל A: רוג'ר וילקו וקורעי הזמן", {'A':26})),
        (b'\x1e - Latex Babes of Estros', ('מסע בחלל E: חתיכות הלטקס מאסטרוס', {'E':30})),
        (b"\x1d - Vohaul's Revenge \x1f", ('מסע בחלל D: נקמת ווהול F', {'D':29,'F':31})),
    ]
    out=bytearray(data)
    for source, replacement in titles:
        off=data.index(source+b'\0')
        if replacement:
            logical,glyphs=replacement
            replacement=get_display(logical.replace('מסע בחלל','ף'),base_dir='R').encode('cp1255').replace('ף'.encode('cp1255'),bytes([252]))
            for token,glyph in glyphs.items():replacement=replacement.replace(token.encode(),bytes([glyph]))
        if len(replacement)>len(source):
            raise ValueError((source,replacement))
        out[off:off+len(source)]=replacement.ljust(len(source),b'\0')
    return bytes(out)

HEAP_ROOMS = {0, 290, 371, 387, 391, 531, 544, 545, 700, 704, 707, 708}
HEAP_LABELS = {
    'Would you like to skip the introduction or watch the whole thing?': 'לדלג על הפתיחה\nאו לצפות בה עד הסוף?',
    'Skip it':'דילוג','Watch it':'צפייה','Restore a Game':'טעינת משחק',
    'Return to Top Menu':'לתפריט הראשי','Return to top menu':'לתפריט הראשי',
    'Return to Catalog Menu':'לתפריט הקטלוג','To Top Menu':'לתפריט הראשי',
    'Welcome to our Automated Catalog! Please select from the following departments:':'ברוכים הבאים לקטלוג האוטומטי! בחרו מחלקה:',
    'Sorry - dealers only!':'מצטערים, לסוחרים בלבד!',
    'Sorry - Due to delay in manufacturing, this item is not yet available.':'מצטערים, עקב עיכוב בייצור הפריט עדיין לא זמין.',
    'THANK YOU FOR SHOPPING Hz SO GOOD!':'תודה שקניתם\nבהרץ נהדר!',
    'Select the plug you wish to purchase.':'בחר את התקע שברצונך לקנות.',
    'Time to Format:':'זמן עד האתחול:', '** WARNING **':'** אזהרה **','Where To?':'לאן?',
    'VERY GOOD':'טוב מאוד','CRITICAL':'קריטי','DANGER':'סכנה','EXCELLENT':'מצוין','GOOD':'טוב','FAIR':'סביר',
    "'R' to Read Sign":'R לקריאת השלט','Squeeze me for a plop of mayonnaise.':'לחץ עליי למנת מיונז.',
    'Squeeze me for a plop of mustard.':'לחץ עליי למנת חרדל.','Squeeze me for a plop of catsup.':'לחץ עליי למנת קטשופ.',
    'Memory Free: 841,912,226 GBytes':'זיכרון פנוי: 841,912,226 GB','Memory Free: 2,451 KBytes':'זיכרון פנוי: 2,451 KB',
    'Initiating Formatting Sequence Value to 5000':'מתחיל באתחול. ערך התחלתי: 5000',
    'Take a good look, Roger:':"תסתכל היטב, רוג'ר:",'Remember this poor wretched soul...':'זכור את האומלל הזה...',
    '...for he is your SON!':'...כי הוא הבן שלך!', 'Order':'הזמנה','Exchange':'החלפה','Continue':'המשך',
    'Top Menu':'תפריט','Catalog':'קטלוג','Exit':'יציאה','Page':'עמוד','SQ IV Hint Book':'רמזי מסע בחלל 4',
}
def localize_heap(n,data,rows,encode):
    if n==0:return patch_heap(data)
    import re
    normalize=lambda s:' '.join(s.split())
    translations={normalize(r['source']):r['he'] for r in rows if r['resource']==n}
    translations.update(HEAP_LABELS)
    out=bytearray(data)
    for found in re.finditer(rb'(?<=\x00)[ -~\r\n]{2,}(?=\x00)',data[:int.from_bytes(data[:2],'little')]):
        key=normalize(found.group().decode())
        if not key or key not in translations:continue
        he=translations[key]
        if he==key:continue
        translated=encode(he)
        if len(translated)>len(found.group()):continue
        assert re.findall(rb'%[ds]',translated)==re.findall(rb'%[ds]',found.group())
        out[found.start():found.end()]=translated.ljust(len(found.group()),b'\0')
    return bytes(out)
