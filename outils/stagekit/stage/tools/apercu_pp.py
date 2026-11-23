"""Aperçu du stage construit (vue de la caméra de jeu) : décor de Papetoon avec la même transformation que
papetoon_ys.py, triangles texturés (couleur au centre de leurs UV, peintre), et collisions du lvd construit par-dessus.
usage : python3 apercu_pp.py <STGPAPETOON.pac> <lvd construit> <sortie.png>"""
import os, subprocess, sys
import numpy as np
import yaml
from PIL import Image, ImageDraw

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import brawl_stage  # noqa: E402

PAC, LVD, OUT = sys.argv[1:4]
ECH, Y0 = 58.0 / 56.0, float(os.environ.get('PP_Y0', '-4.98'))   # comme papetoon_ys.py
SANS = {'stGimmickYoshiBumper', 'EffFoxReflectorStart', 'EffFoxReflector', 'EffFoxReflectorRef', 'Path'}
TEX, models = brawl_stage.load(PAC)
ARR = {n: np.asarray(im.convert('RGBA')) for n, im in TEX.items()}
W, H = 1600, 900
eye = np.array([0.0, 30.0, 190.0])
tgt = np.array([0.0, 20.0, 0.0])
f = tgt - eye; f /= np.linalg.norm(f)
r = np.cross(f, [0, 1, 0]); r /= np.linalg.norm(r)
u = np.cross(r, f)
foc = (H / 2) / np.tan(np.radians(40 / 2))


def proj(p):
    d = p - eye
    z = d @ f
    return np.c_[W / 2 + foc * (d @ r) / z, H / 2 - foc * (d @ u) / z], z


tris = []
for _, mn, ol in models:
    if mn in SANS:
        continue
    for o in ol:
        if o['name'] in ('Rail', 'MainShadow', 'PlatMidShadow', 'PlatLeftShadow', 'PlatRightShadow') or o['tex'] not in ARR:
            continue
        pos = o['pos'][:, [0, 2, 1]].astype(np.float64)
        pos[:, :, 1] -= Y0
        pos *= ECH
        T = ARR[o['tex']]
        for k in range(len(pos)):
            if o['uv'] is not None:
                c = o['uv'][k].mean(0)
                x = int((c[0] % 1) * (T.shape[1] - 1)); y = int((c[1] % 1) * (T.shape[0] - 1))
                col = T[y, x]
            else:
                col = np.array([170, 170, 170, 255])
            if col[3] < 60:
                continue
            tris.append((pos[k], tuple(int(v) for v in col[:3])))
img = Image.new('RGB', (W, H), (150, 170, 190))
dr = ImageDraw.Draw(img)
items = []
for p, col in tris:
    s, z = proj(p)
    if (z < 1).any():
        continue
    items.append((z.mean(), s, col))
items.sort(key=lambda t: -t[0])
for _, s, col in items:
    dr.polygon([tuple(v) for v in s], fill=col)
# collisions
tmp = OUT + '.yml'
subprocess.run(['yamlvd', LVD, tmp], check=True, capture_output=True)
d = yaml.safe_load(open(tmp))
for c in d['collisions']:
    v = [(q['x'], q['y']) for q in c['vertices']]
    pts = [tuple(proj(np.array([[x, y, 0.0]]))[0][0]) for x, y in v]
    dr.line(pts, fill=(255, 40, 40), width=4)
for b in d.get('blast_zone') or []:
    pass
img.save(OUT)
print('apercu', OUT, len(items), 'triangles')
