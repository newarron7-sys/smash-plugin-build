"""Rendu visuel de Black Shadow façon Ultimate (règles de Smush Material Research, ScanMountGoat) :
- couleurs : textures doublées en taille, parties claires remontées vers ~70 % (les plus sombres gardent leur noir) ;
- NOR par matière : relief tiré de la peinture de la texture (muscles, plis), cavité en alpha, linéaire ;
- PRM par matière : R métal (vrai métal seulement), G rugosité, B occlusion douce, A reflet (~0,16) ;
- numatb : chaque matériau pointe sur ses propres NOR / PRM.
Puis une planche avant / après rendue avec un éclairage proche de celui du jeu.

usage : python3 ameliorer.py <dossier c00 du modèle> <dossier png des textures d'origine> <sortie>
"""
import colorsys
import math
import os
import sys

import numpy as np
import ssbh_data_py as S
from PIL import Image, ImageDraw, ImageFilter, ImageFont
from scipy import ndimage

C00, PNG, OUT = sys.argv[1:4]
NEW = os.path.join(OUT, 'png_new')
MDL_OUT = os.path.join(OUT, 'BlackShadow/fighter/ganon/model/body/c00')
os.makedirs(NEW, exist_ok=True)
os.makedirs(MDL_OUT, exist_ok=True)


def load(name):
    return np.asarray(Image.open(os.path.join(PNG, name + '.png')).convert('RGBA'), dtype=float) / 255.0


def save(arr, name):
    Image.fromarray((np.clip(arr, 0, 1) * 255 + 0.5).astype(np.uint8), 'RGBA').save(os.path.join(NEW, name + '.png'))


def upscale(arr, k=2):
    im = Image.fromarray((np.clip(arr, 0, 1) * 255 + 0.5).astype(np.uint8), 'RGBA')
    im = im.resize((im.width * k, im.height * k), Image.LANCZOS).filter(ImageFilter.UnsharpMask(radius=2, percent=60, threshold=2))
    return np.asarray(im, dtype=float) / 255.0


def lum(c):
    return c[..., 0] * 0.299 + c[..., 1] * 0.587 + c[..., 2] * 0.114


def hsv(c):
    mx, mn = c[..., :3].max(-1), c[..., :3].min(-1)
    sat = np.where(mx > 1e-4, (mx - mn) / np.maximum(mx, 1e-4), 0)
    r, g, b = c[..., 0], c[..., 1], c[..., 2]
    d = np.maximum(mx - mn, 1e-4)
    h = np.where(mx == r, ((g - b) / d) % 6, np.where(mx == g, (b - r) / d + 2, (r - g) / d + 4)) * 60
    return h, sat, mx


def brighten(c, target, gmax=1.7):
    """remonte les parties claires vers `target` (99e centile) sans toucher le noir profond"""
    p = np.percentile(lum(c), 99)
    g = float(np.clip(target / max(p, 1e-3), 1.0, gmax))
    out = c.copy()
    out[..., :3] = np.clip(c[..., :3] * g, 0, 1)
    return out, g


def nor_from(col, strength, cavity, blur=2.0):
    """relief à partir de la peinture (clair = bosse), cavité en alpha ; format NOR d'Ultimate"""
    h = ndimage.gaussian_filter(lum(col), blur)
    dx = ndimage.sobel(h, axis=1)
    dy = ndimage.sobel(h, axis=0)
    n = np.dstack([-dx * strength, dy * strength, np.ones_like(h)])
    n /= np.linalg.norm(n, axis=-1, keepdims=True)
    hp = h - ndimage.gaussian_filter(h, 6)
    cav = np.clip(1 + hp * cavity, 0.55, 1.0)
    return np.dstack([n[..., 0] * 0.5 + 0.5, n[..., 1] * 0.5 + 0.5, np.ones_like(h), cav])


def ao_from(col, lo):
    a = ndimage.gaussian_filter(lum(col), 8)
    a = (a - a.min()) / max(a.max() - a.min(), 1e-4)
    return lo + (1 - lo) * a


def prm(metal, rough, ao, spec):
    return np.dstack([metal, rough, ao, spec])


# ------------------------------------------------------------------ textures
OUTTEX = {}
# v0.19 (c121) : en jeu la lumière est bien plus forte que dans l'aperçu, le modèle sortait délavé et la cape blanche :
# couleurs presque d'origine (+15 % au plus), cape inchangée
body = upscale(brighten(load('bs_sdw_body_col'), 0.5, 1.15)[0])
h, s, v = hsv(body)
gold = (h > 35) & (h < 65) & (s > 0.55) & (v > 0.42)
skin = (h > 15) & (h < 50) & (s > 0.22) & ~gold
metal = ndimage.gaussian_filter(gold.astype(float), 0.8)
rough = np.where(gold, 0.32, np.where(skin, 0.62, 0.46))          # costume satiné, peau plus mate, or poli
spec = np.where(skin, 0.16, 0.2)
OUTTEX['bs_sdw_body_col'] = body
OUTTEX['bs_body_nor'] = nor_from(body, 1.6, 3.0)
OUTTEX['bs_body_prm'] = prm(metal, ndimage.gaussian_filter(rough, 0.8), ao_from(body, 0.8), spec)

horn = upscale(load('bs_sdw_horn_col'))
horn[..., :3] = horn[..., :3] * 0.88                                 # pas de blanc pur
h, s, v = hsv(horn)
silver = (s < 0.18) & (v > 0.62)
goldh = (h > 30) & (h < 65) & (s > 0.45) & (v > 0.45)
leather = (h > 10) & (h < 45) & (s > 0.3) & (v < 0.5) & ~goldh
metal = ndimage.gaussian_filter((silver | goldh).astype(float), 0.8)
rough = np.where(silver | goldh, 0.28, np.where(leather, 0.62, 0.3))  # clous métal, cuir, corne polie
spec = np.where(leather, 0.14, 0.45)
OUTTEX['bs_sdw_horn_col'] = horn
OUTTEX['bs_horn_nor'] = nor_from(horn, 1.8, 2.5)
OUTTEX['bs_horn_prm'] = prm(metal, ndimage.gaussian_filter(rough, 0.8), ao_from(horn, 0.85), spec)


for k, a in OUTTEX.items():
    save(a, k)
    print('TEXTURE', k, a.shape[1], 'x', a.shape[0])

# ------------------------------------------------------------------ matériaux
MATS = {'BS_body': ('bs_body_nor', 'bs_body_prm'), 'BS_horn': ('bs_horn_nor', 'bs_horn_prm')}
matl = S.matl_data.read_matl(os.path.join(C00, 'model.numatb'))
for e in matl.entries:
    if e.material_label not in MATS:
        continue
    n, p = MATS[e.material_label]
    for t in e.textures:
        if t.param_id.name == 'Texture4':
            t.data = n
        elif t.param_id.name == 'Texture6':
            p_ = p
            t.data = p_
    print('MATERIAU', e.material_label, '->', n, p)
matl.save(os.path.join(MDL_OUT, 'model.numatb'))

# ------------------------------------------------------------------ planche avant / après (éclairage proche du jeu)
sk = S.skel_data.read_skel(os.path.join(C00, 'model.nusktb'))
UN = [b.name for b in sk.bones]
PAR = [b.parent_index for b in sk.bones]
L0 = [np.array(b.transform, dtype=float).T for b in sk.bones]
W0 = [None] * len(UN)


def w0(i):
    if W0[i] is None:
        p = PAR[i]
        W0[i] = (w0(p) if p is not None and p >= 0 else np.eye(4)) @ L0[i]
    return W0[i]


for i in range(len(UN)):
    w0(i)
mesh = S.mesh_data.read_mesh(os.path.join(C00, 'model.numshb'))
modl = S.modl_data.read_modl(os.path.join(C00, 'model.numdlb'))
mat_of = {(e.mesh_object_name, e.mesh_object_subindex): e.material_label for e in modl.entries}
COLS = {'BS_body': 'bs_sdw_body_col', 'BS_horn': 'bs_sdw_horn_col', 'BS_mant_in': 'bs_sdw_mant_in_col',
        'BS_mant_out': 'bs_sdw_mant_out_col', 'BS_eye': 'bs_sdw_eye_col'}
FLATN = load('bs_nor')
FLATP = load('bs_prm')


def texset(mat, after):
    c = COLS.get(mat, 'bs_sdw_body_col')
    if not after or mat not in MATS:
        return load(c), FLATN, FLATP
    return OUTTEX[c], OUTTEX[MATS[mat][0]], OUTTEX[MATS[mat][1]]


def sample(t, u, v):
    th, tw = t.shape[:2]
    return t[(np.mod(v, 1.0) * (th - 1)).astype(int), (np.mod(u, 1.0) * (tw - 1)).astype(int)]


def render(after, size=(560, 760), yaw=-25, pitch=4, center=(0, 12.5, 0), span=24.0, ss=2):
    w, hgt = size[0] * ss, size[1] * ss
    cy, sy = math.cos(math.radians(yaw)), math.sin(math.radians(yaw))
    cp, sp = math.cos(math.radians(pitch)), math.sin(math.radians(pitch))
    R = np.array([[1, 0, 0], [0, cp, -sp], [0, sp, cp]]) @ np.array([[cy, 0, sy], [0, 1, 0], [-sy, 0, cy]])
    zb = np.full((hgt, w), -1e9)
    img = np.zeros((hgt, w, 3)) + np.array([0.11, 0.1, 0.16])
    L = np.array([-0.45, 0.55, 0.7])
    L /= np.linalg.norm(L)
    V = np.array([0, 0, 1.0])
    Hh = (L + V) / np.linalg.norm(L + V)
    sc = w / span
    c = R @ np.asarray(center, dtype=float)
    for o in mesh.objects:
        if 'shadow' in o.name.lower():
            continue
        mat = mat_of.get((o.name, o.subindex), 'BS_body')
        col, nor, pr = texset(mat, after)
        pos = np.array(o.positions[0].data)[:, :3]
        nrm = np.array(o.normals[0].data)[:, :3]
        tan = np.array(o.tangents[0].data) if o.tangents else np.c_[np.tile([1.0, 0, 0], (len(pos), 1)), np.ones(len(pos))]
        uv = np.array(o.texture_coordinates[0].data)[:, :2]
        idx = np.array(o.vertex_indices, dtype=int).reshape(-1, 3)
        # pose de repos (liaison) : sommets déjà en place
        p = pos @ R.T
        n = nrm @ R.T
        t = tan[:, :3] @ R.T
        sx = (p[:, 0] - c[0]) * sc + w / 2
        syy = hgt / 2 - (p[:, 1] - c[1]) * sc
        for a, b, cc in idx:
            xs, ys = (sx[a], sx[b], sx[cc]), (syy[a], syy[b], syy[cc])
            minx, maxx = int(max(0, math.floor(min(xs)))), int(min(w - 1, math.ceil(max(xs))))
            miny, maxy = int(max(0, math.floor(min(ys)))), int(min(hgt - 1, math.ceil(max(ys))))
            if minx > maxx or miny > maxy:
                continue
            den = (ys[1] - ys[2]) * (xs[0] - xs[2]) + (xs[2] - xs[1]) * (ys[0] - ys[2])
            if abs(den) < 1e-12:
                continue
            gx, gy = np.meshgrid(np.arange(minx, maxx + 1) + 0.5, np.arange(miny, maxy + 1) + 0.5)
            l0 = ((ys[1] - ys[2]) * (gx - xs[2]) + (xs[2] - xs[1]) * (gy - ys[2])) / den
            l1 = ((ys[2] - ys[0]) * (gx - xs[2]) + (xs[0] - xs[2]) * (gy - ys[2])) / den
            l2 = 1 - l0 - l1
            ins = (l0 >= 0) & (l1 >= 0) & (l2 >= 0)
            if not ins.any():
                continue
            z = l0 * p[a, 2] + l1 * p[b, 2] + l2 * p[cc, 2]
            sub = zb[miny:maxy + 1, minx:maxx + 1]
            vis = ins & (z > sub)
            if not vis.any():
                continue
            sub[vis] = z[vis]
            nn = l0[..., None] * n[a] + l1[..., None] * n[b] + l2[..., None] * n[cc]
            nn /= np.maximum(np.linalg.norm(nn, axis=-1, keepdims=True), 1e-9)
            # face arrière (intérieur de la cape) : on la voit de l'autre côté, normale retournée
            back = (nn @ V) < 0
            nn[back] = -nn[back]
            tt = l0[..., None] * t[a] + l1[..., None] * t[b] + l2[..., None] * t[cc]
            tt -= nn * (tt * nn).sum(-1, keepdims=True)
            tt /= np.maximum(np.linalg.norm(tt, axis=-1, keepdims=True), 1e-9)
            bb = np.cross(nn, tt) * (tan[a, 3] if tan.shape[1] > 3 else 1.0)
            u = l0 * uv[a, 0] + l1 * uv[b, 0] + l2 * uv[cc, 0]
            v = l0 * uv[a, 1] + l1 * uv[b, 1] + l2 * uv[cc, 1]
            alb = sample(col, u, v)[..., :3] ** 2.2
            nm = sample(nor, u, v)
            pm = sample(pr, u, v)
            tx, ty = nm[..., 0] * 2 - 1, nm[..., 1] * 2 - 1
            tz = np.sqrt(np.clip(1 - tx * tx - ty * ty, 0, 1))
            N = tt * tx[..., None] + bb * ty[..., None] + nn * tz[..., None]
            N /= np.maximum(np.linalg.norm(N, axis=-1, keepdims=True), 1e-9)
            met, rgh, ao, spc, cav = pm[..., 0:1], np.clip(pm[..., 1:2], 0.05, 1), pm[..., 2:3], pm[..., 3:4], nm[..., 3:4]
            ndl = np.clip(N @ L, 0, 1)[..., None]
            ndh = np.clip(N @ Hh, 0, 1)[..., None]
            ndv = np.clip(N @ V, 0, 1)[..., None]
            a2 = rgh ** 4
            D = a2 / (math.pi * (ndh * ndh * (a2 - 1) + 1) ** 2)
            F0 = spc * 0.2 * (1 - met) + alb * met
            spec_c = F0 * D * ndl * 0.9 * cav
            dif = alb * (1 - met) * (ndl * 1.6 + 0.45 * ao)
            env = alb * met * (0.35 + 0.65 * (1 - rgh)) * ao
            rim = 0.18 * (1 - ndv) ** 3 * cav
            out = np.clip(dif + spec_c + env + rim, 0, 1) ** (1 / 2.2)
            region = img[miny:maxy + 1, minx:maxx + 1]
            region[vis] = out[vis]
    im = Image.fromarray((img * 255).astype(np.uint8), 'RGB')
    return im.resize(size, Image.LANCZOS)


try:
    fb = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf', 26)
    f = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf', 18)
except OSError:
    fb = f = ImageFont.load_default()
before, after = render(False), render(True)
bb_, ba_ = render(False, yaw=160), render(True, yaw=160)
zb_, za_ = render(False, size=(560, 420), center=(0, 21.5, 0), span=11), render(True, size=(560, 420), center=(0, 21.5, 0), span=11)
sheet = Image.new('RGB', (1160, 70 + 760 + 50 + 420 + 50 + 760 + 120), (22, 20, 34))
d = ImageDraw.Draw(sheet)
d.text((14, 18), 'Black Shadow : rendu avant / après (aperçu, éclairage proche du jeu)', font=fb, fill=(240, 235, 225))
sheet.paste(before, (14, 70))
sheet.paste(after, (586, 70))
d.text((24, 80), 'AVANT', font=fb, fill=(240, 120, 120))
d.text((596, 80), 'APRÈS', font=fb, fill=(120, 230, 140))
sheet.paste(zb_, (14, 880))
sheet.paste(za_, (586, 880))
sheet.paste(bb_, (14, 1350))
sheet.paste(ba_, (586, 1350))
d.text((24, 1360), 'DOS', font=fb, fill=(240, 120, 120))
d.text((596, 1360), 'DOS', font=fb, fill=(120, 230, 140))
notes = ['Relief et brillant propres à chaque matière : costume satiné, cape mate, cornes polies, clous en métal.',
         'Couleurs remontées (sans perdre son noir), textures 2x plus grandes.',
         'Aperçu calculé hors du jeu : en jeu, la lumière du stage change un peu le résultat.']
for i, t in enumerate(notes):
    d.text((14, 2120 + i * 28), t, font=f, fill=(240, 200, 130))
sheet.save(os.path.join(OUT, 'avant_apres.png'))
print('planche ok')
