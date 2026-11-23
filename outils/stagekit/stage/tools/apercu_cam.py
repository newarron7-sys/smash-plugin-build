"""Aperçu « caméra de jeu » d'un stage de Brawl : vue en perspective depuis l'avant, comme en combat, avec le sol du
stage d'Ultimate qu'il remplace (ligne verte) et deux silhouettes de la taille d'un perso pour l'échelle.

usage : python3 apercu_cam.py <.pac> <sortie.png> <titre> <sol x0,x1> [modèles à ignorer, séparés par des virgules]
"""
import os, sys, math
import numpy as np
from PIL import Image, ImageDraw, ImageFont

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import brawl_stage  # noqa: E402

PAC, OUT, TITLE, SOL = sys.argv[1:5]
SKIP = set(sys.argv[5].split(',')) if len(sys.argv) > 5 else set()
X0, X1 = (float(v) for v in SOL.split(','))
TEX, models = brawl_stage.load(PAC)
ARR = {n: np.asarray(im) for n, im in TEX.items()}

P, C = [], []
for _, mn, objs in models:
    if mn in SKIP:
        continue
    for o in objs:
        T = ARR.get(o['tex'])
        for k, pts in enumerate(o['pos']):
            col = (170, 170, 170)
            if T is not None and o['uv'] is not None:
                u, v = o['uv'][k].mean(0)
                c = T[int((v % 1.0) * (T.shape[0] - 1)), int((u % 1.0) * (T.shape[1] - 1))]
                if c[3] < 40:
                    continue
                col = tuple(int(x) for x in c[:3])
            P.append(pts)
            C.append(col)
P = np.array(P)
C = np.array(C, dtype=float)


def render(eye, target, fov, W=960, H=540, extra=()):
    eye, target = np.array(eye, float), np.array(target, float)
    f = target - eye
    f /= np.linalg.norm(f)
    r = np.cross(f, [0, 1, 0])
    r /= np.linalg.norm(r)
    u = np.cross(r, f)
    rel = P - eye
    x, y, z = rel @ r, rel @ u, rel @ f
    keep = (z > 1.0).all(1)
    foc = (H / 2) / math.tan(math.radians(fov) / 2)
    sx = W / 2 + foc * x / np.maximum(z, 1e-3)
    sy = H / 2 - foc * y / np.maximum(z, 1e-3)
    # éclairage simple par la normale du triangle
    n = np.cross(P[:, 1] - P[:, 0], P[:, 2] - P[:, 0])
    n /= np.maximum(np.linalg.norm(n, axis=1, keepdims=True), 1e-9)
    L = np.array([0.3, 0.7, 0.6])
    L /= np.linalg.norm(L)
    sh = 0.55 + 0.45 * np.abs(n @ L)
    im = Image.new('RGB', (W, H), (25, 27, 38))
    d = ImageDraw.Draw(im)
    order = np.argsort(-z.mean(1))
    for i in order:
        if not keep[i]:
            continue
        xs, ys = sx[i], sy[i]
        if xs.max() < -50 or xs.min() > W + 50 or ys.max() < -50 or ys.min() > H + 50:
            continue
        d.polygon(list(zip(xs, ys)), fill=tuple(int(v) for v in np.clip(C[i] * sh[i], 0, 255)))

    def proj(p):
        q = np.array(p, float) - eye
        zz = q @ f
        return (W / 2 + foc * (q @ r) / zz, H / 2 - foc * (q @ u) / zz)
    d.line([proj((X0, 0, 0)), proj((X1, 0, 0))], fill=(70, 255, 110), width=4)
    for xx in (X0, X1):
        d.line([proj((xx, 0, 0)), proj((xx, 40, 0))], fill=(70, 255, 110), width=2)
    for xx in (-35, 35):            # silhouettes : 15 de haut, 6 de large (taille d'un perso moyen)
        a, b = proj((xx - 3, 0, 0)), proj((xx + 3, 15, 0))
        d.rectangle([a[0], b[1], b[0], a[1]], outline=(255, 255, 255), width=2)
    return im


views = [render((0, 38, 200), (0, 22, 0), 45), render((0, 70, 420), (0, 30, 0), 45)]
out = Image.new('RGB', (960, 540 * 2 + 140), (255, 255, 255))
out.paste(views[0], (0, 60))
out.paste(views[1], (0, 540 + 120))
dr = ImageDraw.Draw(out)
try:
    f1 = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf', 26)
    f2 = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf', 18)
except Exception:  # noqa: BLE001
    f1 = f2 = ImageFont.load_default()
dr.text((12, 14), TITLE, fill=(20, 20, 30), font=f1)
dr.text((12, 540 + 72), 'caméra éloignée (zoom arrière) ; vert = sol et murs du stage de Terry, blanc = taille d\'un perso',
        fill=(60, 60, 70), font=f2)
out.save(OUT)
print('apercu', OUT, len(P), 'triangles')
