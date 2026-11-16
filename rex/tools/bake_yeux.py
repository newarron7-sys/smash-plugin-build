"""Knuckle Joe (REX) : peint les yeux (objet EyesDefault, texture à trous) directement dans la texture du visage.

Les yeux de REX sont un masque posé devant le visage, dessiné avec transparence. En jeu la transparence n'était pas
prise (masque noir), puis le masque opaque peint sur la couleur de peau ressortait en zone plus foncée (photo du
16/11). Ici : chaque pixel de la texture du corps couvert par le visage (objet Head) est replacé en 3D, projeté sur
le masque des yeux (vers l'avant), et la couleur des yeux y est mélangée selon leur transparence. Le masque est
ensuite retiré du modèle (DROP_OBJ) : les yeux font partie du visage, même lumière, même matière.
usage : python3 bake_yeux.py <FitXxx00.pac> <objet visage> <objet yeux> <sortie.png> [texture du visage]
"""
import os, sys, struct, contextlib
import numpy as np
from PIL import Image
HERE = os.path.dirname(os.path.abspath(__file__))
for p in ('../../black-shadow/tools', '../../black-shadow/tools/fs', '../../zero/tools', '.'):
    sys.path.insert(0, os.path.join(HERE, p))
from arc import arc_entries
from brres import brres_files
from lz import decompress
import mdl0, tex0

PAC, VIS, YEUX, OUT = sys.argv[1:5]
raw = decompress(open(PAC, 'rb').read())
dd = [x for x in arc_entries(raw) if x[0] == 2][0][4]
n, o = brres_files(dd)['3DModels(NW4R)'][0]
M = mdl0.read_model(dd, o)
TEX = {}
for ent in arc_entries(raw):
    if ent[4][:4] != b'bres':
        continue
    bf = brres_files(ent[4])
    pals = {k: tex0.palette(ent[4], v) for k, v in bf.get('Palettes(NW4R)', [])}
    for k, v in bf.get('Textures(NW4R)', []):
        TEX.setdefault(k, (ent[4], v, pals.get(k)))
objs = {on: ob for on, ob in M['objs']}


def tris(on):
    ob = objs[on]
    P = M['arrays']['pos'][ob['ids'][0]][0]
    U = M['arrays']['uv'][ob['ids'][4]][0]
    return [(np.array([P[v['pos']][:3] for v in t], float), np.array([U[v['uv0']][:2] for v in t], float)) for t in ob['tris']]


face = tris(VIS)
eyes = tris(YEUX)
tex_face = sys.argv[5] if len(sys.argv) > 5 else '0_model0'
T = np.asarray(tex0.read_tex0(TEX[tex_face][0], TEX[tex_face][1], TEX[tex_face][2]).convert('RGBA'), float)
te = [k for k in TEX if 'eyesdefault' in k.lower()][0]
E = np.asarray(tex0.read_tex0(*TEX[te]).convert('RGBA'), float)
H, W = T.shape[:2]
he, we = E.shape[:2]
out = T.copy()
# masque des yeux : triangles en 2D (x, y) vus de face, avec leur profondeur z et leurs UV
lo = np.min([p.min(0) for p, _ in eyes], 0)
hi = np.max([p.max(0) for p, _ in eyes], 0)
print('masque des yeux : de', lo.round(2), 'a', hi.round(2))


def bary(p, a, b, c):
    v0, v1, v2 = b - a, c - a, p - a
    d00, d01, d11 = v0 @ v0, v0 @ v1, v1 @ v1
    d20, d21 = v2 @ v0, v2 @ v1
    den = d00 * d11 - d01 * d01
    if abs(den) < 1e-12:
        return None
    v = (d11 * d20 - d01 * d21) / den
    w = (d00 * d21 - d01 * d20) / den
    return np.array([1 - v - w, v, w])


def yeux_uv(q):
    """point 3D du visage -> UV dans la texture des yeux (projection vers l'avant), ou None"""
    best = None
    for p, uv in eyes:
        b = bary(q[:2], p[0, :2], p[1, :2], p[2, :2])
        if b is None or b.min() < -1e-4:
            continue
        z = b @ p[:, 2]
        if z >= q[2] - 0.3 and (best is None or z < best[0]):
            best = (z, b @ uv)
    return None if best is None else best[1]


peints = 0
for p, uv in face:
    if p[:, 0].max() < lo[0] or p[:, 0].min() > hi[0] or p[:, 1].max() < lo[1] or p[:, 1].min() > hi[1]:
        continue
    # pixels de la texture couverts par ce triangle (UV -> pixels ; v vers le bas comme Brawl)
    px = np.c_[(uv[:, 0] % 1.0) * (W - 1), (uv[:, 1] % 1.0) * (H - 1)]
    if np.ptp(uv[:, 0]) > 0.5 or np.ptp(uv[:, 1]) > 0.5:
        continue                                    # triangle à cheval sur un bord de texture : ignoré
    x0, y0 = np.floor(px.min(0)).astype(int)
    x1, y1 = np.ceil(px.max(0)).astype(int)
    for y in range(max(y0, 0), min(y1, H - 1) + 1):
        for x in range(max(x0, 0), min(x1, W - 1) + 1):
            b = bary(np.array([x, y], float), px[0], px[1], px[2])
            if b is None or b.min() < -0.02:
                continue
            q = b @ p
            e = yeux_uv(q)
            if e is None:
                continue
            u, v = np.clip(e[0], 0, 1), np.clip(e[1], 0, 1)
            c = E[int(v * (he - 1)), int(u * (we - 1))]
            a = c[3] / 255.0
            out[y, x, :3] = out[y, x, :3] * (1 - a) + c[:3] * a
            peints += 1
Image.fromarray(out.clip(0, 255).astype(np.uint8), 'RGBA').save(OUT)
print('pixels du visage peints :', peints, '->', OUT)
