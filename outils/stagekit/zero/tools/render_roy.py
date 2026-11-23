"""Planche de contrôle : le skin de Zero (_Silent_, squelette de Roy) posé avec les animations converties.

usage : python3 render_roy.py <dossier du modèle c00> <model.nusktb de Roy> <dossier des .nuanmb> <image>
Rendu logiciel simple (sans textures, éclairage doux), vue de côté : sert à vérifier les poses (main du sabre,
taille, pieds au sol) avant de tester en jeu.
"""
import math, os, sys
import numpy as np
import ssbh_data_py as S
from PIL import Image, ImageDraw, ImageFont

MDL, SKEL, MOT, IMG = sys.argv[1:5]
mesh = S.mesh_data.read_mesh(os.path.join(MDL, 'model.numshb'))
sk = S.skel_data.read_skel(SKEL)
UB = sk.bones
UN = [b.name for b in UB]
PAR = [b.parent_index for b in UB]
L0 = [np.array(b.transform, dtype=float).T for b in UB]


def worlds(locs):
    W = [None] * len(UB)

    def w(i):
        if W[i] is None:
            W[i] = (w(PAR[i]) @ locs[i]) if PAR[i] is not None and PAR[i] >= 0 else locs[i]
        return W[i]
    for i in range(len(UB)):
        w(i)
    return W


WB = worlds(L0)
IWB = {UN[i]: np.linalg.inv(WB[i]) for i in range(len(UB))}


def mat(v):
    x, y, z, w = v.rotation
    r = np.array([[1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
                  [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
                  [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)]])
    m = np.eye(4)
    m[:3, :3] = r @ np.diag(v.scale)
    m[:3, 3] = v.translation
    return m


def pose(fn, f):
    a = S.anim_data.read_anim(os.path.join(MOT, fn))
    tracks = {nd.name: nd.tracks[0].values for g in a.groups for nd in g.nodes}
    locs = [mat(tracks[u][min(int(f), len(tracks[u]) - 1)]) if u in tracks else L0[i] for i, u in enumerate(UN)]
    W = worlds(locs)
    return {UN[i]: W[i] @ IWB[UN[i]] for i in range(len(UB))}, int(a.final_frame_index) + 1, W


# on ne dessine pas les yeux / expressions de rechange
SKIP = ('Eye', 'eye', 'Mouth', 'mouth', 'Face', 'face', 'Blink', 'Half', 'Ouch', 'Talk', 'Capture', 'Ottotto', 'Heavy',
        'Fura', 'Attack', 'Down', 'Hurt', 'Close')


def render(P, W, yaw=90, size=(300, 340), span=34.0, center_y=None):
    S2 = 2
    w, h = size[0] * S2, size[1] * S2
    img = np.zeros((h, w, 3)) + np.array([0.14, 0.13, 0.2])
    zb = np.full((h, w), -1e9)
    c, s_ = math.cos(math.radians(yaw)), math.sin(math.radians(yaw))
    R = np.array([[c, 0, s_], [0, 1, 0], [-s_, 0, c]])
    L = np.array([-0.3, 0.7, 0.65])
    L /= np.linalg.norm(L)
    sc = w / span
    cx = W[UN.index('Hip')][0, 3] if 'Hip' in UN else 0
    # sol
    gy = int(h * 0.88)
    img[gy:, :] = [0.22, 0.21, 0.3]
    for ob in mesh.objects:
        if any(k in ob.name for k in SKIP) and 'Body' not in ob.name:
            continue
        pos = np.array(ob.positions[0].data)[:, :3]
        nrm = np.array(ob.normals[0].data)[:, :3]
        acc = np.zeros_like(pos)
        accn = np.zeros_like(pos)
        tot = np.zeros(len(pos))
        for inf in ob.bone_influences:
            M = P.get(inf.bone_name)
            if M is None:
                continue
            ids = np.array([x.vertex_index for x in inf.vertex_weights])
            ws = np.array([x.vertex_weight for x in inf.vertex_weights])
            acc[ids] += ws[:, None] * (pos[ids] @ M[:3, :3].T + M[:3, 3])
            accn[ids] += ws[:, None] * (nrm[ids] @ M[:3, :3].T)
            tot[ids] += ws
        if not ob.bone_influences:
            # maillage attaché à un seul os (parent_bone_name)
            M = P.get(ob.parent_bone_name, np.eye(4))
            acc = pos @ M[:3, :3].T + M[:3, 3]
            accn = nrm @ M[:3, :3].T
        else:
            acc[tot < 1e-6] = pos[tot < 1e-6]
        p = (acc - [cx, 0, 0]) @ R.T
        n = accn @ R.T
        n /= np.maximum(np.linalg.norm(n, axis=1, keepdims=True), 1e-9)
        base = np.array([0.85, 0.3, 0.3]) if 'Sword' not in ob.name and 'sword' not in ob.name else np.array([0.4, 1.0, 0.5])
        if any(k in ob.name.lower() for k in ('hair',)):
            base = np.array([0.95, 0.85, 0.35])
        sx = p[:, 0] * sc + w / 2
        sy = gy - p[:, 1] * sc if center_y is None else h / 2 - (p[:, 1] - center_y) * sc
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
            lam = np.abs(nn @ L)[..., None]
            reg = img[y0:y1 + 1, x0:x1 + 1]
            reg[vis] = np.clip(base * (0.35 + 0.75 * lam), 0, 1)[vis]
    im = Image.fromarray((img * 255).astype(np.uint8))
    # l'épée de Roy (le sabre du skin) est un modèle à part : on la dessine en trait vert le long de l'os Sword1
    if 'Sword1' in UN:
        Ws = W[UN.index('Sword1')]
        a3 = Ws[:3, 3]
        b3 = a3 + Ws[:3, 2] / max(np.linalg.norm(Ws[:3, 2]), 1e-6) * 13.0
        pa = (a3 - [cx, 0, 0]) @ R.T
        pb = (b3 - [cx, 0, 0]) @ R.T
        dr = ImageDraw.Draw(im)
        dr.line([(pa[0] * sc + w / 2, gy - pa[1] * sc), (pb[0] * sc + w / 2, gy - pb[1] * sc)], fill=(90, 255, 120), width=7)
    return im.resize(size, Image.LANCZOS)


files = sorted(f for f in os.listdir(MOT) if f.endswith('.nuanmb'))
# v0.24 : gros plan (haut du corps, de face et de côté) d'une seule animation : ZOOM_FILE=a00wait1.nuanmb
if os.environ.get('ZOOM_FILE'):
    fn = os.environ['ZOOM_FILE']
    _, n, W0_ = pose(fn, 0)
    cyb = W0_[UN.index('Bust')][1, 3] if 'Bust' in UN else 12.0
    out = Image.new('RGB', (4 * 420, 2 * 460 + 40), (20, 18, 30))
    for j, fr in enumerate((0.0, 0.33, 0.66, 0.99)):
        P, _, W = pose(fn, fr * (n - 1))
        for k_, yaw in enumerate((0, 90)):
            out.paste(render(P, W, yaw=yaw, size=(420, 460), span=16.0, center_y=cyb), (j * 420, 40 + k_ * 460))
    try:
        _f = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf', 22)
    except OSError:
        _f = ImageFont.load_default()
    ImageDraw.Draw(out).text((10, 8), '%s : haut du corps (face, puis côté), 4 moments' % fn, font=_f, fill=(150, 230, 170))
    out.save(IMG)
    print('gros plan ok', fn)
    sys.exit(0)
try:
    fb = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf', 16)
except OSError:
    fb = ImageFont.load_default()
FR = (0.2, 0.3, 0.45)
cols = len(FR)
per_row = 2
rows = (len(files) + per_row - 1) // per_row
CW, CH = 300, 340
sheet = Image.new('RGB', (per_row * (cols * CW + 12), rows * (CH + 26) + 10), (20, 18, 30))
d = ImageDraw.Draw(sheet)
for k, fn in enumerate(files):
    r, cpos = divmod(k, per_row)
    _, n, _ = pose(fn, 0)
    for j, fr in enumerate(FR):
        f = fr * (n - 1)
        P, _, W = pose(fn, f)
        im = render(P, W)
        x = cpos * (cols * CW + 12) + j * CW
        y = r * (CH + 26) + 26
        sheet.paste(im, (x, y))
    d.text((cpos * (cols * CW + 12) + 6, r * (CH + 26) + 4), '%s  (%d images)' % (fn[:-7], n), font=fb, fill=(150, 230, 170))
sheet.save(IMG)
print('planche ok', len(files))
