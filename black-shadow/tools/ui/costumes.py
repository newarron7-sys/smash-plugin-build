"""Planche de tous les costumes du mod (rendus officiels), pour choisir lesquels porter.

usage : python3 costumes.py <sortie>
"""
import io
import json
import os
import re
import sys
import urllib.request
import zipfile

from PIL import Image, ImageDraw, ImageFont

OUT = sys.argv[1]
os.makedirs(OUT, exist_ok=True)
ua = {'User-Agent': 'Mozilla/5.0'}
get = lambda u: urllib.request.urlopen(urllib.request.Request(u, headers=ua), timeout=300).read()  # noqa: E731
p = json.loads(get('https://gamebanana.com/apiv11/Mod/591515/ProfilePage'))
imgs = []
pacs = set()
for f in p['_aFiles']:
    print('FICHIER', f['_sFile'], f.get('_nFilesize'))
    if not f['_sFile'].lower().endswith('.zip'):
        continue
    z = zipfile.ZipFile(io.BytesIO(get(f['_sDownloadUrl'])))
    for n in z.namelist():
        if '__MACOSX' in n:
            continue
        m = re.search(r'Fit(BlackShadow|\w+?)(\d\d)\.pac$', n)
        if m:
            pacs.add(n)
        if n.lower().endswith('.png') and 'render' in n.lower():
            imgs.append((n, Image.open(io.BytesIO(z.read(n))).convert('RGBA')))
print('MODELES', sorted(pacs))
print('RENDUS', [n for n, _ in imgs])


def key(t):
    n = t[0]
    num = re.findall(r'(\d+)', os.path.basename(n))
    return (0 if 'deathborn' not in n.lower() else 1, n.rsplit('/', 1)[0], int(num[0]) if num else 0)


imgs.sort(key=key)
W, H = 330, 360
cols = 5
rows = (len(imgs) + cols - 1) // cols
sheet = Image.new('RGB', (cols * W, 70 + rows * H), (22, 20, 34))
d = ImageDraw.Draw(sheet)
try:
    fb = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf', 26)
    f = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf', 20)
except OSError:
    fb = f = ImageFont.load_default()
d.text((12, 18), 'Costumes de Black Shadow dans le mod (choisis lesquels porter)', font=fb, fill=(240, 235, 225))
for i, (n, im) in enumerate(imgs):
    x, y = (i % cols) * W, 70 + (i // cols) * H
    bb = im.getbbox() or (0, 0, im.width, im.height)
    im = im.crop(bb)
    im.thumbnail((W - 20, H - 60))
    sheet.paste(im, (x + (W - im.width) // 2, y + 40), im)
    lab = ('Deathborn ' if 'deathborn' in n.lower() else 'Black Shadow ') + str(i + 1)
    d.text((x + 10, y + 8), lab, font=f, fill=(255, 210, 120))
sheet.save(os.path.join(OUT, 'costumes.png'))
print('planche', len(imgs), 'costumes')
