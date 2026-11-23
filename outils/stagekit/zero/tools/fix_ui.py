"""Zero (skin sur Roy c00) : portrait en combat (chara_4, à côté des pourcentages) refait comme ceux d'Ultimate :
la tête (rendu P+ sans fond) dans un losange incliné, transparente autour ; seul le haut du casque dépasse.

usage : python3 fix_ui.py <dossier ui_src> <sortie png>
"""
import math, os, sys
from PIL import Image, ImageChops, ImageDraw

SRC, OUT = sys.argv[1:3]
res = Image.open(os.path.join(SRC, 'CSP_Result_HD_0000.png')).convert('RGBA')
W = H = 224
head = res.crop((95, -10, 305, 200)).resize((W, H), Image.LANCZOS)
S = 4
mask = Image.new('L', (W * S, H * S), 0)
d = ImageDraw.Draw(mask)
cx, cy, r = W * 0.5 * S, H * 0.6 * S, W * 0.36 * S
pts = [(cx + r * math.cos(math.radians(a + 20)), cy + r * math.sin(math.radians(a + 20))) for a in (0, 90, 180, 270)]
d.polygon(pts, fill=255)
d.rectangle([W * 0.12 * S, 0, W * 0.88 * S, cy - r * 0.35], fill=255)
mask = mask.resize((W, H), Image.LANCZOS)
head.putalpha(ImageChops.multiply(head.split()[3], mask))
os.makedirs(OUT, exist_ok=True)
head.save(os.path.join(OUT, 'chara_4_roy_00.png'))
print('chara_4 ok', head.size)
