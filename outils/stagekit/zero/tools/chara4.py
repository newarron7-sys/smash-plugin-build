"""Zero (case zerorex) : portrait de combat (chara_4, à côté des pourcentages) au cadrage d'Ultimate.

v0.24 (demandé : il dépassait du losange sur le côté et en bas) : l'ancien était l'icône de P+ (240x240, son propre
losange, plus grand) réduite en 224x224. Celui-ci suit les portraits d'origine du jeu (mesurés sur des chara_4 au
style d'Ultimate, 162x162) :
- image de 162x162 ;
- le contenu reste dans le losange du jeu (coins mesurés : haut (99,-7), droite (157,100), bas (50,158),
  gauche (-8,51)) ; seul le haut à gauche peut dépasser (comme les chapeaux), le casque de Zero ;
- la tête (rendu P+ du résultat, sans fond) placée comme celle des persos du jeu : visage vers (86, 89).

usage : python3 chara4.py <dossier ui_src> <sortie.png> [aperçu.png]
"""
import os, sys
from PIL import Image, ImageChops, ImageDraw

SRC, OUT = sys.argv[1:3]
PREV = sys.argv[3] if len(sys.argv) > 3 else None
W = H = 162
res = Image.open(os.path.join(SRC, 'CSP_Result_HD_0000.png')).convert('RGBA')
# visage de Zero dans le rendu (384x480) : centre vers (190, 118) ; échelle -> casque d'environ 95 px de large
SC = 0.70
FACE_SRC = (190.0, 118.0)
FACE_DST = (86.0, 89.0)
big = res.resize((round(res.width * SC), round(res.height * SC)), Image.LANCZOS)
img = Image.new('RGBA', (W, H), (0, 0, 0, 0))
img.alpha_composite(big, (round(FACE_DST[0] - FACE_SRC[0] * SC), round(FACE_DST[1] - FACE_SRC[1] * SC)))

# losange du jeu + débord permis en haut à gauche (au-dessus du bord haut-gauche du losange)
T, R, B, L = (99, -7), (157, 100), (50, 158), (-8, 51)
S = 4
mask = Image.new('L', (W * S, H * S), 0)
d = ImageDraw.Draw(mask)
d.polygon([(x * S, y * S) for x, y in (T, R, B, L)], fill=255)
d.polygon([(x * S, y * S) for x, y in ((-8, 51), (99, -7), (99, -40), (-40, -40), (-40, 51))], fill=255)
mask = mask.resize((W, H), Image.LANCZOS)
img.putalpha(ImageChops.multiply(img.split()[3], mask))
os.makedirs(os.path.dirname(os.path.abspath(OUT)), exist_ok=True)
img.save(OUT)
print('chara_4', img.size, 'contenu', img.getbbox())

if PREV:
    # aperçu : le portrait sur le losange sombre du jeu, en grand, avec l'ancien à côté pour comparer
    def hud(p, size):
        base = Image.new('RGBA', (300, 260), (120, 70, 55, 255))
        dd = ImageDraw.Draw(base)
        ox, oy = 60, 50
        dd.polygon([(ox + x * 1.0, oy + y * 1.0) for x, y in (T, R, B, L)], fill=(25, 25, 30, 255), outline=(240, 240, 240, 255))
        dd.rectangle([ox + 120, oy + 120, 300, oy + 150], fill=(20, 20, 24, 255))
        pim = p.resize((size, size), Image.LANCZOS)
        base.alpha_composite(pim, (ox + (162 - size) // 2, oy + (162 - size) // 2))
        return base.resize((600, 520), Image.LANCZOS)
    old = None
    for c in (os.path.join(SRC, '..', 'ui_png', 'chara_4_zerorex_00.png'),):
        if os.path.exists(c):
            old = Image.open(c).convert('RGBA')
    sheet = Image.new('RGBA', (1240, 600), (30, 28, 40, 255))
    dr = ImageDraw.Draw(sheet)
    if old is not None:
        sheet.alpha_composite(hud(old, 162), (10, 60))
        dr.text((20, 20), 'AVANT (icone de P+ reduite)', fill=(255, 200, 200, 255))
    sheet.alpha_composite(hud(img, 162), (630, 60))
    dr.text((640, 20), 'APRES (cadrage des portraits du jeu, 162x162)', fill=(200, 255, 200, 255))
    sheet.save(PREV)
    print('apercu', PREV)
