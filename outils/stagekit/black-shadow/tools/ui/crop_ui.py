"""Images d'interface de Black Shadow découpées dans les rendus officiels du mod P+ (dossier « Black Shadow Raw
Renders », 2103x2196) et son icône de stock HD, aux tailles d'Ultimate.

usage : python3 crop_ui.py <dossier des images du mod (ou "telecharger")> <dims.json> <sortie>
"""
import io
import json
import os
import sys
import urllib.request
import zipfile

from PIL import Image, ImageDraw, ImageFont

SRC, DIMS, OUT = sys.argv[1:4]
os.makedirs(os.path.join(OUT, 'png'), exist_ok=True)

RENDER = 'Black Shadow Raw Renders/0.png'  # costume 1 (celui du skin converti)
STOCK = 'HD Stocks/0_STC_HD.png'


def load(rel):
    if SRC != 'telecharger':
        return Image.open(os.path.join(SRC, rel)).convert('RGBA')
    global ZIP
    if 'ZIP' not in globals():
        ua = {'User-Agent': 'Mozilla/5.0'}
        get = lambda u: urllib.request.urlopen(urllib.request.Request(u, headers=ua), timeout=300).read()  # noqa: E731
        p = json.loads(get('https://gamebanana.com/apiv11/Mod/591515/ProfilePage'))
        ZIP = zipfile.ZipFile(io.BytesIO(get(p['_aFiles'][0]['_sDownloadUrl'])))
    n = next(x for x in ZIP.namelist() if x.endswith(rel) and '__MACOSX' not in x)
    return Image.open(io.BytesIO(ZIP.read(n))).convert('RGBA')


DEFAULT = {'chara_0': (128, 128), 'chara_1': (512, 512), 'chara_2': (64, 64), 'chara_3': (3028, 2036),
           'chara_4': (224, 224), 'chara_6': (512, 256), 'chara_7': (456, 300)}
dims = dict(DEFAULT)
try:
    for k, v in json.load(open(DIMS)).items():
        if 'width' in v and k in dims:
            dims[k] = (v['width'], v['height'])
except (OSError, ValueError):
    pass

R = load(RENDER)  # 2103x2196, visage vers (1030, 370)


def crop(cx, cy, w, size):
    """découpe centrée en (cx, cy), de largeur w (pixels du rendu), au rapport de `size`, redimensionnée"""
    h = w * size[1] / size[0]
    box = (round(cx - w / 2), round(cy - h / 2), round(cx + w / 2), round(cy + h / 2))
    return R.crop(box).resize(size, Image.LANCZOS)


imgs = {
    'chara_0': crop(1025, 400, 600, dims['chara_0']),          # record : visage
    'chara_6': crop(1030, 370, 620, dims['chara_6']),          # Final Smash : les yeux
    'chara_7': crop(1030, 378, 640, dims['chara_7']),          # grille : masque de face (remonté un peu)
}
# portrait en combat : comme dans le jeu, la tête dans un losange (carré incliné) au centre, transparente autour ;
# seul le haut (les cornes) peut dépasser du losange. Repère : chara_4 de Bandana Dee, contenu dans (46,19)-(204,192).
W4, H4 = dims['chara_4']
head = crop(1030, 400, 900, (W4, H4))
from PIL import ImageChops  # noqa: E402
mask = Image.new('L', (W4 * 4, H4 * 4), 0)
md = ImageDraw.Draw(mask)
cx, cy, r = W4 * 0.5 * 4, H4 * 0.6 * 4, W4 * 0.32 * 4
import math  # noqa: E402
pts = [(cx + r * math.cos(math.radians(a + 20)), cy + r * math.sin(math.radians(a + 20))) for a in (0, 90, 180, 270)]
md.polygon(pts, fill=255)
md.rectangle([W4 * 0.12 * 4, 0, W4 * 0.8 * 4, cy - r * 0.35], fill=255)
mask = mask.resize((W4, H4), Image.LANCZOS)
head.putalpha(ImageChops.multiply(head.split()[3], mask))
imgs['chara_4'] = head
# sélection : buste en grand, personnage en bas à droite
imgs['chara_1'] = crop(1000, 760, 1500, dims['chara_1'])
# VS / résultats : personnage entier en bas à gauche
W3, H3 = dims['chara_3']
c3 = Image.new('RGBA', (W3, H3), (0, 0, 0, 0))
h = int(H3 * 0.98)
body = R.resize((round(R.width * h / R.height), h), Image.LANCZOS)
c3.alpha_composite(body, (int(W3 * 0.06), H3 - h))
imgs['chara_3'] = c3
# icône de stock : celle du mod
imgs['chara_2'] = load(STOCK).resize(dims['chara_2'], Image.LANCZOS)

for k, im in imgs.items():
    im.save(os.path.join(OUT, 'png', '%s_ganon_00.png' % k))
    print('image', k, im.size)

# planche de contrôle
NAMES = {'chara_0': 'record', 'chara_1': 'sélection', 'chara_2': 'icône de stock', 'chara_3': 'VS / résultats',
         'chara_4': 'portrait en combat', 'chara_6': 'Final Smash', 'chara_7': 'icône de la grille'}
try:
    f = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf', 14)
    fb = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf', 20)
except OSError:
    f = fb = ImageFont.load_default()
cell = 300
sheet = Image.new('RGB', (cell * 4 + 50, 60 + 2 * (cell + 40)), (24, 22, 36))
d = ImageDraw.Draw(sheet)
d.text((10, 15), 'Interface de Black Shadow (rendus officiels du mod)', font=fb, fill=(235, 225, 200))
for j, k in enumerate(['chara_7', 'chara_1', 'chara_3', 'chara_2', 'chara_0', 'chara_4', 'chara_6']):
    im = imgs[k]
    x, y = 10 + (j % 4) * (cell + 10), 60 + (j // 4) * (cell + 40)
    d.text((x, y), '%s (%s, %dx%d)' % (NAMES[k], k, im.size[0], im.size[1]), font=f, fill=(235, 225, 200))
    t = im.copy()
    t.thumbnail((cell, cell - 24))
    bg = Image.new('RGBA', t.size, (60, 56, 84, 255))
    bg.alpha_composite(t)
    sheet.paste(bg.convert('RGB'), (x, y + 22))
sheet.save(os.path.join(OUT, 'planche_interface.png'))
print('planche ok')
