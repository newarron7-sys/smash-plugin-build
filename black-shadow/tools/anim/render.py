"""Planche « style schema_v06 » : le skin de Black Shadow posé avec ses animations converties, vu de côté."""
import os

import numpy as np
import ssbh_data_py as S
from PIL import Image, ImageDraw, ImageFont

# lignes de la planche : (titre, sous-titre, animation Brawl, [(image, légende)])
ROWS = [
    ('Chope', 'attrape devant lui', 'Catch', [(0, 'début'), (6, 'attrape (image 6)'), (14, 'ramène')]),
    ('Coup pendant', 'la prise', 'CatchAttack', [(0, 'tient'), (3, 'frappe 3 %'), (12, 'reprend')]),
    ('Lancer avant', '7 % + 4 % à 45°', 'ThrowF', [(8, 'tient'), (16, 'coup 7 %'), (18, 'lâche à 45°'), (34, 'fin')]),
    ('Lancer arrière', '2 % + 12 % à 45°', 'ThrowB', [(15, 'se tourne'), (41, 'coup 2 %'), (42, 'lâche derrière'), (60, 'fin')]),
    ('Lancer haut', '10 % à 90°', 'ThrowHi', [(10, 'prépare'), (20, 'soulève'), (26, 'lâche vers le haut'), (42, 'fin')]),
    ('Lancer bas', '7 %, enterre', 'ThrowLw', [(10, 'soulève'), (20, 'abat'), (24, 'écrase (enterre)'), (42, 'fin')]),
]
CELL = 280
LABEL_W = 230
BG = (24, 22, 36)
CELL_BG = (38, 35, 58)
TXT = (235, 225, 200)


def font(size, bold=True):
    p = '/usr/share/fonts/truetype/dejavu/DejaVuSans%s.ttf' % ('-Bold' if bold else '')
    try:
        return ImageFont.truetype(p, size)
    except OSError:
        return ImageFont.load_default()


def load_mesh(assets, un):
    m = S.mesh_data.read_mesh(os.path.join(assets, 'fighter/ganon/model/body/c00/model.numshb'))
    objs = []
    for o in m.objects:
        name = o.name.lower()
        if 'shadow' in name or 'lod' in name:
            continue
        pos = np.array(o.positions[0].data, dtype=float)[:, :3]
        idx = np.array(o.vertex_indices, dtype=int).reshape(-1, 3)
        nv = len(pos)
        wmat = np.zeros((nv, len(un)))
        for inf in o.bone_influences:
            if inf.bone_name not in un:
                continue
            bi = un.index(inf.bone_name)
            for vw in inf.vertex_weights:
                wmat[vw.vertex_index, bi] += vw.vertex_weight
        parent = un.index(o.parent_bone_name) if (not o.bone_influences and o.parent_bone_name in un) else None
        gold = any(k in name for k in ('gold', 'hand', 'belt', 'ring', 'brace', 'metal', 'horn'))
        objs.append((o.name, pos, idx, wmat, parent, gold))
    return objs


def draw_pose(objs, W, W0inv, throw_i, trans_i, ring=True):
    img = Image.new('RGB', (CELL, CELL), CELL_BG)
    d = ImageDraw.Draw(img)
    skin = np.stack([W[i] @ W0inv[i] for i in range(len(W))])  # (B, 4, 4)
    cx = W[trans_i][2, 3]
    scale = CELL / 34.0
    tris = []
    light = np.array([0.45, 0.55, 0.7])
    light /= np.linalg.norm(light)
    for name, pos, idx, wmat, parent, gold in objs:
        hp = np.c_[pos, np.ones(len(pos))]
        if parent is not None:
            vw = (W[parent] @ hp.T).T[:, :3]
        else:
            ws = wmat.sum(1, keepdims=True)
            ws[ws == 0] = 1
            m = np.einsum('vb,bij->vij', wmat / ws, skin)
            vw = np.einsum('vij,vj->vi', m, hp)[:, :3]
        a, b, c = vw[idx[:, 0]], vw[idx[:, 1]], vw[idx[:, 2]]
        n = np.cross(b - a, c - a)
        nl = np.linalg.norm(n, axis=1)
        nl[nl == 0] = 1
        lam = np.abs((n / nl[:, None]) @ light)
        z = -(a[:, 0] + b[:, 0] + c[:, 0]) / 3  # caméra du côté -X
        base = np.array([200, 160, 70]) if gold else np.array([78, 66, 104])
        for t in range(len(idx)):
            col = tuple(int(v) for v in base * (0.3 + 0.7 * lam[t]))
            pts = [((p[2] - cx) * scale + CELL / 2, CELL - 18 - p[1] * scale) for p in (a[t], b[t], c[t])]
            tris.append((z[t], pts, col))
    tris.sort(key=lambda x: x[0])
    for _, pts, colr in tris:
        d.polygon(pts, fill=colr)
    # là où l'adversaire est tenu (os Throw)
    tp = W[throw_i][:3, 3]
    x, y = (tp[2] - cx) * scale + CELL / 2, CELL - 18 - tp[1] * scale
    if ring:
        d.ellipse([x - 7, y - 7, x + 7, y + 7], outline=(230, 60, 170), width=3)
    d.line([(10, CELL - 18), (CELL - 10, CELL - 18)], fill=(90, 85, 120), width=2)
    return img


def sheet(assets, sk, un, W0, poses, out):
    objs = load_mesh(assets, un)
    W0inv = [np.linalg.inv(w) for w in W0]
    throw_i = un.index('Throw')
    trans_i = un.index('Trans')
    rows = [r for r in ROWS if r[2] in poses]
    ncol = max(len(r[3]) for r in rows)
    width = LABEL_W + ncol * (CELL + 8) + 8
    height = 50 + len(rows) * (CELL + 30 + 10) + 50
    img = Image.new('RGB', (width, height), BG)
    d = ImageDraw.Draw(img)
    d.text((10, 12), 'Prise et lancers de Black Shadow (ses animations du mod, avec son skin)', font=font(20), fill=TXT)
    y = 50
    for title, sub, nm, frames in rows:
        W = poses[nm]
        d.text((10, y + CELL // 2 - 10), title, font=font(18), fill=TXT)
        d.text((10, y + CELL // 2 + 14), sub, font=font(15, False), fill=TXT)
        for j, (f, lab) in enumerate(frames):
            f = min(f, len(W) - 1)
            x = LABEL_W + j * (CELL + 8)
            d.text((x + 4, y), lab, font=font(15), fill=TXT)
            img.paste(draw_pose(objs, W[f], W0inv, throw_i, trans_i), (x, y + 24))
        y += CELL + 40
    d.text((10, height - 34), 'Rond rose : là où il tient l\'adversaire. Images = celles du mod Project+.', font=font(14, False), fill=TXT)
    img.save(out)
