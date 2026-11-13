"""Lit la Black Bull dans FitBlackShadowFinal.pac, affiche sa structure et la dessine (contrôle de la lecture).

usage : python3 inspect_car.py <sortie>
"""
import io
import json
import math
import os
import sys
import urllib.request
import zipfile

import numpy as np
from PIL import Image, ImageDraw

sys.path.insert(0, os.path.dirname(__file__))
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))
from arc import arc_entries  # noqa: E402
from brres import brres_files  # noqa: E402
import mdl0  # noqa: E402
import tex0  # noqa: E402

OUT = sys.argv[1]
os.makedirs(OUT, exist_ok=True)
UA = {'User-Agent': 'Mozilla/5.0'}
get = lambda u: urllib.request.urlopen(urllib.request.Request(u, headers=UA), timeout=300).read()  # noqa: E731
p = json.loads(get('https://gamebanana.com/apiv11/Mod/591515/ProfilePage'))
z = zipfile.ZipFile(io.BytesIO(get(p['_aFiles'][0]['_sDownloadUrl'])))
fin = z.read(next(n for n in z.namelist() if n.endswith('FitBlackShadowFinal.pac') and '__MACOSX' not in n))
ents = arc_entries(fin)

# textures (entrée 1) et modèle (entrée 2)
TEX = {}
for n, o in brres_files(ents[1][4]).get('Textures(NW4R)', []):
    im = tex0.read_tex0(ents[1][4], o)
    im.save(os.path.join(OUT, 'tex_%s.png' % n))
    TEX[n] = np.asarray(im.convert('RGBA'), dtype=float) / 255
    print('texture', n, im.size)
dd = ents[2][4]
n, o = brres_files(dd)['3DModels(NW4R)'][0]
M = mdl0.read_model(dd, o)
print('modele', n, 'version', M['version'])
for b in M['bones']:
    print('os', b[0], 'noeud', b[1], 'echelle', np.round(b[3], 3).tolist(), 'monde', np.round(b[2][:3, :], 3).tolist())
for k, arrs in M['arrays'].items():
    for a, info in arrs:
        print('tableau', k, info, 'min', np.round(a.min(0), 2).tolist(), 'max', np.round(a.max(0), 2).tolist())
print('materiaux', M['mats'], 'objet->materiau', M['obj_mat'])
for i, (on, ob) in enumerate(M['objs']):
    print('objet', i, on, 'os', ob['node'], 'ids', ob['ids'][:4], 'attrs', ob['attrs'], 'triangles', len(ob['tris']), 'nv', ob['nv'])

# dessin : 3 vues, couleur de la texture du matériau
bone_w = {b[1]: b[2] for b in M['bones']}


def project(v, yaw, pitch):
    cy, sy = math.cos(math.radians(yaw)), math.sin(math.radians(yaw))
    cp, sp = math.cos(math.radians(pitch)), math.sin(math.radians(pitch))
    x, y, zz = v[..., 0], v[..., 1], v[..., 2]
    x2 = cy * x + sy * zz
    z2 = -sy * x + cy * zz
    y2 = cp * y - sp * z2
    z3 = sp * y + cp * z2
    return np.stack([x2, y2, z3], -1)


def build(use_bone):
  tris = []
  for i, (on, ob) in enumerate(M['objs']):
    P = M['arrays']['pos'][ob['ids'][0]][0] if ob['ids'][0] >= 0 else None
    U = M['arrays']['uv'][ob['ids'][4]][0] if ob['ids'][4] >= 0 and M['arrays']['uv'] else None
    mi = M['obj_mat'].get(i)
    texn = (M['mats'].get(M['mat_names'][mi], [''])[0] if mi is not None and mi < len(M['mat_names']) else '')
    if texn not in TEX:  # références de matériau illisibles : le 2e matériau est la vitre, le reste la carrosserie
        texn = 'glas_sp' if mi == 1 else 'airframe'
    T = TEX.get(texn)
    for t in ob['tris']:
        pts = []
        for v in t:
            q = P[v['pos']]
            b = v.get('bone', ob['node'])
            if use_bone and b is not None and b >= 0:
                q = (bone_w.get(b, np.eye(4)) @ np.r_[q, 1])[:3]
            pts.append(q)
        col = np.array([0.5, 0.5, 0.55])
        if T is not None and U is not None and 'uv0' in t[0]:
            uv = np.mean([U[v['uv0']] for v in t], 0)
            h, w = T.shape[:2]
            col = T[int(np.mod(uv[1], 1) * (h - 1)), int(np.mod(uv[0], 1) * (w - 1)), :3]
        tris.append((np.array(pts), col))
  return tris


def sheet_of(tris, out):
    allp = np.concatenate([t[0] for t in tris]) if tris else np.zeros((1, 3))
    c = (allp.max(0) + allp.min(0)) / 2
    span = float((allp.max(0) - allp.min(0)).max()) or 1
    print(out, 'boite', np.round(allp.min(0), 2).tolist(), np.round(allp.max(0), 2).tolist())
    S = 420
    sheet = Image.new('RGB', (S * 3, S), (40, 38, 55))
    for k, (yaw, pitch) in enumerate(((0, 0), (90, 0), (35, 20))):
        img = Image.new('RGB', (S, S), (40, 38, 55))
        d = ImageDraw.Draw(img)
        L = []
        for pts, col in tris:
            q = project(pts - c, yaw, pitch)
            n = np.cross(q[1] - q[0], q[2] - q[0])
            nl = np.linalg.norm(n) or 1
            lam = abs(n[2] / nl)
            L.append((q[:, 2].mean(), [(x * S * 0.85 / span + S / 2, S / 2 - y * S * 0.85 / span) for x, y, _ in q],
                      tuple(int(255 * min(1, v * (0.45 + 0.9 * lam))) for v in col)))
        L.sort(key=lambda x: x[0])
        for _, pts, colr in L:
            d.polygon(pts, fill=colr)
        sheet.paste(img, (k * S, 0))
    sheet.save(os.path.join(OUT, out))


sheet_of(build(False), 'apercu_sans_os.png')
sheet_of(build(True), 'apercu_avec_os.png')
print('apercu ok')
