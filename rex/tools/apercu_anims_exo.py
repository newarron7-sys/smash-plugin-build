"""Planche de contrôle des animations « exo » : le modèle de Rock (EXO=1) posé avec les .nuanmb produits par
retarget_exo.py, évalués comme dans le jeu : pistes des os de Mario et des os H_Exo_, puis la rotation de chaque os
H_Exo_ associé est recopiée depuis son os de Mario (contraintes du nuhlpb). Vue de côté (comme en jeu, il regarde à
droite), Trans remis à zéro (le jeu s'en sert pour déplacer le perso, pas pour le dessiner).

usage : python3 apercu_anims_exo.py <dossier model> <model.json> <dossier des .nuanmb> <image> [fichiers...]
"""
import json, math, os, sys
import numpy as np
import ssbh_data_py as S
from PIL import Image, ImageDraw, ImageFont

MDL, INFO, MOT, OUT = sys.argv[1:5]
FILES = sys.argv[5:] or sorted(f for f in os.listdir(MOT) if f.endswith('.nuanmb'))
PAIR = json.load(open(INFO))['exo']['paires']
sk = S.skel_data.read_skel(os.path.join(MDL, 'model.nusktb'))
UB = sk.bones
UN = [b.name for b in UB]
IDX = {n: i for i, n in enumerate(UN)}
PAR = [b.parent_index if b.parent_index is not None and b.parent_index >= 0 else None for b in UB]
L0 = [np.array(b.transform, dtype=float).T for b in UB]
mesh = S.mesh_data.read_mesh(os.path.join(MDL, 'model.numshb'))
COL = {'Hair': (0.93, 0.80, 0.38), 'Zippers': (0.8, 0.8, 0.8), 'Jacket': (0.78, 0.17, 0.16), 'Pants': (0.22, 0.24, 0.32),
       'Shoes': (0.35, 0.25, 0.2), 'Gloves': (0.2, 0.2, 0.22), 'Belt': (0.3, 0.22, 0.16), 'Shirt': (0.85, 0.85, 0.82),
       'Eyes': (0.3, 0.5, 0.8)}


def worlds(L):
    W = [None] * len(UB)

    def w(i):
        if W[i] is None:
            W[i] = (w(PAR[i]) @ L[i]) if PAR[i] is not None else L[i]
        return W[i]
    for i in range(len(UB)):
        w(i)
    return W


W0 = worlds(L0)
INV0 = [np.linalg.inv(m) for m in W0]
EXO = [i for i, n in enumerate(UN) if n.startswith('H_Exo_')]


def mat(v):
    x, y, z, w = v.rotation
    r = np.array([[1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
                  [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
                  [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)]])
    m = np.eye(4)
    m[:3, :3] = r * np.array(v.scale)[None, :]
    m[:3, 3] = v.translation
    return m


def pose(an, f):
    L = [m.copy() for m in L0]
    for g in an.groups:
        if g.group_type.name != 'Transform':
            continue
        for nd in g.nodes:
            if nd.name in IDX and nd.tracks:
                vals = nd.tracks[0].values
                L[IDX[nd.name]] = mat(vals[min(f, len(vals) - 1)])
    L[IDX['Trans']] = np.eye(4)
    W = worlds(L)
    Wf = list(W)
    for i in EXO:                                         # parents avant enfants (ordre du fichier)
        m = Wf[PAR[i]] @ L[i]
        b = UN[i][6:]
        if b in PAIR and PAIR[b] in IDX:
            m = m.copy()
            m[:3, :3] = W[IDX[PAIR[b]]][:3, :3]
        Wf[i] = m
    return [Wf[i] @ INV0[i] for i in range(len(UB))]


def render(P, size=(230, 300), span=32.0):
    w, h = size[0] * 2, size[1] * 2
    im = Image.new('RGB', (w, h), (36, 33, 52))
    dr = ImageDraw.Draw(im)
    sc = w / span
    gy = h * 0.9
    dr.line([(0, gy), (w, gy)], fill=(80, 76, 100), width=2)
    Ld = np.array([0.45, 0.6, 0.65])
    Ld /= np.linalg.norm(Ld)
    tris = []
    for ob in mesh.objects:
        pos = np.array(ob.positions[0].data)[:, :3]
        acc = np.zeros_like(pos)
        for inf in ob.bone_influences:
            M = P[IDX[inf.bone_name]] if inf.bone_name in IDX else np.eye(4)
            ids = np.array([x.vertex_index for x in inf.vertex_weights])
            ws = np.array([x.vertex_weight for x in inf.vertex_weights])
            acc[ids] += ws[:, None] * (pos[ids] @ M[:3, :3].T + M[:3, 3])
        # vue de côté : écran x = avant du perso (z), profondeur = -x (on voit son côté droit, comme en jeu)
        p = np.stack([acc[:, 2], acc[:, 1], -acc[:, 0]], axis=1)
        t = np.array(ob.vertex_indices).reshape(-1, 3)
        a, b, c = p[t[:, 0]], p[t[:, 1]], p[t[:, 2]]
        n = np.cross(b - a, c - a)
        n /= np.maximum(np.linalg.norm(n, axis=1, keepdims=True), 1e-9)
        lam = np.abs(n @ Ld)
        base = next((v for k, v in COL.items() if k in ob.name), (0.93, 0.72, 0.58))
        depth = (a[:, 2] + b[:, 2] + c[:, 2]) / 3
        for k in range(len(t)):
            tris.append((depth[k], a[k], b[k], c[k], tuple(int(255 * min(1.0, x * (0.35 + 0.75 * lam[k]))) for x in base)))
    tris.sort(key=lambda x: x[0])
    for _, a, b, c, col in tris:
        dr.polygon([(w / 2 + q[0] * sc, gy - q[1] * sc) for q in (a, b, c)], fill=col)
    return im.resize(size, Image.LANCZOS)


try:
    FT = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf', 15)
    FS = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf', 12)
except OSError:
    FT = FS = ImageFont.load_default()
COLS = 6
CW, CH = 230, 300
out = Image.new('RGB', (180 + COLS * CW, len(FILES) * (CH + 6) + 10), (20, 18, 30))
d = ImageDraw.Draw(out)
for r, fn in enumerate(FILES):
    an = S.anim_data.read_anim(os.path.join(MOT, fn))
    nf = int(an.final_frame_index) + 1
    y0 = 6 + r * (CH + 6)
    d.text((8, y0 + 10), fn.replace('.nuanmb', ''), font=FT, fill=(150, 230, 170))
    d.text((8, y0 + 32), '%d images' % nf, font=FS, fill=(200, 200, 210))
    for j in range(COLS):
        f = int(round(j * (nf - 1) / (COLS - 1)))
        out.paste(render(pose(an, f)), (180 + j * CW, y0))
        d.text((180 + j * CW + 6, y0 + 4), 'image %d' % f, font=FS, fill=(220, 220, 230))
    print('apercu', fn, nf)
out.save(OUT)
print('planche', OUT)
