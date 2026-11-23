"""Stage de Brawl (STGxxx.pac) : liste ses modèles et fait un aperçu de face du décor (chaque triangle de la couleur
de sa texture), avec la zone de jeu et, en option, le sol d'un stage d'Ultimate pour comparer. Usage perso.

usage : python3 analyse_brawl_stage.py <.pac ou dossier> <sortie.png> <titre> [sol Ultimate : x0,x1,y]
"""
import os, sys
import numpy as np
from PIL import Image, ImageDraw, ImageFont

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import brawl_stage  # noqa: E402

SRC, OUT, TITLE = sys.argv[1:4]
SOL = [float(x) for x in sys.argv[4].split(',')] if len(sys.argv) > 4 else None
pac = SRC
if os.path.isdir(SRC):
    pac = next(os.path.join(r, f) for r, _, fs in os.walk(SRC) for f in fs if f.lower().endswith('.pac'))
TEX, models = brawl_stage.load(pac, verbose=True)
print('TEXTURES', len(TEX))
ARR = {n: np.asarray(im) for n, im in TEX.items()}
tris = []
for name, mn, objs in models:
    for o in objs:
        lo, hi = o['pos'].reshape(-1, 3).min(0), o['pos'].reshape(-1, 3).max(0)
        print('OBJET %-20s %-12s tex %-18s tri %6d  x[%7.0f %7.0f] y[%7.0f %7.0f] z[%7.0f %7.0f]'
              % (mn + '/' + o['name'], o['mat'], o['tex'], len(o['pos']), lo[0], hi[0], lo[1], hi[1], lo[2], hi[2]))
        T = ARR.get(o['tex'])
        for k, pts in enumerate(o['pos']):
            col = np.array([170, 170, 170.0])
            if T is not None and o['uv'] is not None:
                u, v = o['uv'][k].mean(0)
                c = T[int((v % 1.0) * (T.shape[0] - 1)), int((u % 1.0) * (T.shape[1] - 1))]
                if c[3] < 40:
                    continue
                col = c[:3].astype(float)
            n = np.cross(pts[1] - pts[0], pts[2] - pts[0])
            nl = np.linalg.norm(n)
            tris.append((pts, col, n / nl if nl > 1e-9 else np.array([0, 0, 1.0])))
print('triangles', len(tris))

W_, H_ = 900, 520
light = np.array([0.3, 0.5, 0.8])
light /= np.linalg.norm(light)


def view(x0, x1, y0, y1, zmax=None):
    im = Image.new('RGB', (W_, H_), (30, 32, 44))
    d = ImageDraw.Draw(im)
    sc = min(W_ / (x1 - x0), H_ / (y1 - y0))
    cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
    for pts, col, n in sorted(tris, key=lambda t: t[0][:, 2].mean()):
        if zmax is not None and pts[:, 2].min() > zmax:
            continue
        sh = 0.55 + 0.45 * abs(float(np.dot(n, light)))
        c = tuple(int(v) for v in np.clip(col * sh, 0, 255))
        d.polygon([(W_ / 2 + (p[0] - cx) * sc, H_ / 2 - (p[1] - cy) * sc) for p in pts], fill=c)
    gy = H_ / 2 - (0 - cy) * sc
    for gx in range(int(x0 // 50) * 50, int(x1) + 1, 50):
        X = W_ / 2 + (gx - cx) * sc
        d.line([(X, gy - 5), (X, gy + 5)], fill=(255, 90, 90), width=2)
    if SOL:
        a, b, y = SOL
        Y = H_ / 2 - (y - cy) * sc
        d.line([(W_ / 2 + (a - cx) * sc, Y), (W_ / 2 + (b - cx) * sc, Y)], fill=(80, 255, 120), width=4)
    return im, sc


if tris:
    allp = np.vstack([t[0] for t in tris])
    lo, hi = np.percentile(allp, 2, axis=0), np.percentile(allp, 98, axis=0)
    big, _ = view(lo[0], hi[0], lo[1], hi[1])
    near, _ = view(-200, 200, -90, 150, zmax=60)
    out = Image.new('RGB', (W_, H_ * 2 + 120), (255, 255, 255))
    out.paste(big, (0, 50))
    out.paste(near, (0, H_ + 110))
    dr = ImageDraw.Draw(out)
    try:
        f = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf', 24)
        f2 = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf', 17)
    except Exception:  # noqa: BLE001
        f = f2 = ImageFont.load_default()
    dr.text((10, 12), TITLE, fill=(20, 20, 30), font=f)
    dr.text((10, H_ + 62), 'zone de combat (traits rouges tous les 50, hauteur 0)' +
            (' ; vert = sol du stage de Terry' if SOL else ''), fill=(60, 60, 70), font=f2)
    out.save(OUT)
    print('apercu', OUT)
