"""Aperçu du squelette « exo » : le modèle construit avec EXO=1, posé avec quelques rotations des os de Mario
(bras levés, genoux pliés, buste tourné) ; les os H_Exo_ suivent comme dans le jeu (copie de rotation du nuhlpb).
Sert à vérifier avant le jeu que le maillage suit bien les os de Mario sans se déchirer.

usage : python3 apercu_exo.py <dossier model> <model.json> <sortie.png>
"""
import json, math, os, sys
import numpy as np
import ssbh_data_py as S
from PIL import Image, ImageDraw, ImageFont

MDL, INFO, OUT = sys.argv[1:4]
info = json.load(open(INFO))
PAIR = info.get('exo', {}).get('paires', {})          # os REX -> os de Mario
sk = S.skel_data.read_skel(os.path.join(MDL, 'model.nusktb'))
UB = sk.bones
UN = [b.name for b in UB]
PAR = [b.parent_index for b in UB]
L0 = [np.array(b.transform, dtype=float).T for b in UB]
mesh = S.mesh_data.read_mesh(os.path.join(MDL, 'model.numshb'))


def worlds(L):
    W = [None] * len(UB)

    def w(i):
        if W[i] is None:
            W[i] = (w(PAR[i]) @ L[i]) if PAR[i] is not None and PAR[i] >= 0 else L[i]
        return W[i]
    for i in range(len(UB)):
        w(i)
    return W


W0 = worlds(L0)
INV0 = [np.linalg.inv(m) for m in W0]


def rot(axis, deg):
    a = np.asarray(axis, float)
    a = a / np.linalg.norm(a)
    x, y, z = a
    c, s = math.cos(math.radians(deg)), math.sin(math.radians(deg))
    C = 1 - c
    R = np.eye(4)
    R[:3, :3] = [[c + x * x * C, x * y * C - z * s, x * z * C + y * s], [y * x * C + z * s, c + y * y * C, y * z * C - x * s],
                 [z * x * C - y * s, z * y * C + x * s, c + z * z * C]]
    return R


def pose(spec):
    L = [m.copy() for m in L0]
    for name, axis, deg in spec:
        if name in UN:
            i = UN.index(name)
            L[i] = L[i] @ rot(axis, deg)                 # rotation locale (repère de l'os)
    W = worlds(L)
    # os H_Exo_ associés : leur rotation monde = celle de l'os de Mario (orientations de repos identiques)
    Wf = [m.copy() for m in W]
    exo = [i for i, n in enumerate(UN) if n.startswith('H_Exo_')]
    for i in exo:                                         # parents avant enfants (ordre du fichier)
        n = UN[i][6:]
        p = PAR[i]
        wp = Wf[p]
        m = wp @ L[i]
        if n in PAIR and PAIR[n] in UN:
            m[:3, :3] = W[UN.index(PAIR[n])][:3, :3]
        Wf[i] = m
    return [Wf[i] @ INV0[i] for i in range(len(UB))]


def render(P, size=(360, 420), span=26.0, yaw=0):
    w, h = size[0] * 2, size[1] * 2
    img = np.zeros((h, w, 3)) + np.array([0.14, 0.13, 0.2])
    zb = np.full((h, w), -1e9)
    c, s_ = math.cos(math.radians(yaw)), math.sin(math.radians(yaw))
    R = np.array([[c, 0, s_], [0, 1, 0], [-s_, 0, c]])
    Ld = np.array([-0.3, 0.7, 0.65])
    Ld /= np.linalg.norm(Ld)
    sc = w / span
    gy = int(h * 0.92)
    for ob in mesh.objects:
        pos = np.array(ob.positions[0].data)[:, :3]
        nrm = np.array(ob.normals[0].data)[:, :3]
        acc = np.zeros_like(pos)
        accn = np.zeros_like(pos)
        for inf in ob.bone_influences:
            M = P[UN.index(inf.bone_name)] if inf.bone_name in UN else np.eye(4)
            ids = np.array([x.vertex_index for x in inf.vertex_weights])
            ws = np.array([x.vertex_weight for x in inf.vertex_weights])
            acc[ids] += ws[:, None] * (pos[ids] @ M[:3, :3].T + M[:3, 3])
            accn[ids] += ws[:, None] * (nrm[ids] @ M[:3, :3].T)
        p = acc @ R.T
        n = accn @ R.T
        n /= np.maximum(np.linalg.norm(n, axis=1, keepdims=True), 1e-9)
        sx = p[:, 0] * sc + w / 2
        sy = gy - p[:, 1] * sc
        base = np.array([0.85, 0.32, 0.3]) if 'Hair' not in ob.name else np.array([0.95, 0.85, 0.35])
        for a, b, cc in np.array(ob.vertex_indices).reshape(-1, 3):
            xs, ys = (sx[a], sx[b], sx[cc]), (sy[a], sy[b], sy[cc])
            den = (ys[1] - ys[2]) * (xs[0] - xs[2]) + (xs[2] - xs[1]) * (ys[0] - ys[2])
            if abs(den) < 1e-9:
                continue
            x0, x1 = int(max(0, min(xs))), int(min(w - 1, max(xs) + 1))
            y0, y1 = int(max(0, min(ys))), int(min(h - 1, max(ys) + 1))
            if x0 > x1 or y0 > y1:
                continue
            gx, gyy = np.meshgrid(np.arange(x0, x1 + 1) + 0.5, np.arange(y0, y1 + 1) + 0.5)
            l0 = ((ys[1] - ys[2]) * (gx - xs[2]) + (xs[2] - xs[1]) * (gyy - ys[2])) / den
            l1 = ((ys[2] - ys[0]) * (gx - xs[2]) + (xs[0] - xs[2]) * (gyy - ys[2])) / den
            l2 = 1 - l0 - l1
            ins = (l0 >= 0) & (l1 >= 0) & (l2 >= 0)
            if not ins.any():
                continue
            z = l0 * p[a, 2] + l1 * p[b, 2] + l2 * p[cc, 2]
            sub = zb[y0:y1 + 1, x0:x1 + 1]
            vis = ins & (z > sub)
            if not vis.any():
                continue
            sub[vis] = z[vis]
            nn = l0[..., None] * n[a] + l1[..., None] * n[b] + l2[..., None] * n[cc]
            nn /= np.maximum(np.linalg.norm(nn, axis=-1, keepdims=True), 1e-9)
            lam = np.abs(nn @ Ld)[..., None]
            reg = img[y0:y1 + 1, x0:x1 + 1]
            reg[vis] = np.clip(base * (0.35 + 0.75 * lam), 0, 1)[vis]
    return Image.fromarray((img * 255).astype(np.uint8)).resize(size, Image.LANCZOS)


POSES = [
    ('repos', []),
    ('bras leves', [('ShoulderL', [0, 0, 1], 60), ('ShoulderR', [0, 0, 1], 60)]),
    ('genoux plies', [('LegL', [0, 0, 1], 45), ('KneeL', [0, 0, 1], -70), ('LegR', [0, 0, 1], 45), ('KneeR', [0, 0, 1], -70)]),
    ('buste tourne', [('Waist', [0, 1, 0], 35), ('Head', [0, 1, 0], -30), ('ArmL', [0, 0, 1], 60)]),
]
out = Image.new('RGB', (len(POSES) * 360, 2 * 420 + 40), (20, 18, 30))
d = ImageDraw.Draw(out)
try:
    f = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf', 18)
except OSError:
    f = ImageFont.load_default()
for j, (lab, spec) in enumerate(POSES):
    P = pose(spec)
    out.paste(render(P, yaw=0), (j * 360, 40))
    out.paste(render(P, yaw=90), (j * 360, 40 + 420))
    d.text((j * 360 + 8, 10), lab, font=f, fill=(150, 230, 170))
out.save(OUT)
print('apercu exo', OUT, 'paires', len(PAIR))
