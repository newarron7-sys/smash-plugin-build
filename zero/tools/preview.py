"""Planche de Zero converti : le modèle Ultimate relu depuis les fichiers écrits, au repos et dans des poses
tirées de ses animations Brawl (même squelette, donc ce que fera le jeu une fois les animations converties).

usage : python3 preview.py <paquet extrait> <sortie de build_model.py> <image>
"""
import json, math, os, sys
import numpy as np
import ssbh_data_py as S
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
for p in ('../../black-shadow/tools', '../../black-shadow/tools/anim', '.'):
    sys.path.insert(0, os.path.join(HERE, p))
from arc import arc_entries  # noqa: E402
from brres import brres_files  # noqa: E402
import brawl_anim  # noqa: E402
from lz import decompress  # noqa: E402

PKG, OUT, IMG = sys.argv[1:4]
MDL, PNG = os.path.join(OUT, 'model'), os.path.join(OUT, 'png')
info = json.load(open(os.path.join(OUT, 'model.json')))
mesh = S.mesh_data.read_mesh(os.path.join(MDL, 'model.numshb'))
modl = S.modl_data.read_modl(os.path.join(MDL, 'model.numdlb'))
matl = S.matl_data.read_matl(os.path.join(MDL, 'model.numatb'))
mat_of = {(e.mesh_object_name, e.mesh_object_subindex): e.material_label for e in modl.entries}
col_of, prm_of = {}, {}
for e in matl.entries:
    for t in e.textures:
        if t.param_id.name == 'Texture0' or (t.param_id.name == 'Texture5' and e.material_label not in col_of):
            col_of[e.material_label] = t.data
        if t.param_id.name == 'Texture6':
            prm_of[e.material_label] = t.data
TEXC = {}


def tex(name):
    if name not in TEXC:
        p = os.path.join(PNG, name + '.png')
        TEXC[name] = np.asarray(Image.open(p).convert('RGBA'), dtype=float) / 255 if os.path.exists(p) else np.full((2, 2, 4), 0.8)
    return TEXC[name]


# squelette Brawl (pour les poses) et correspondance des noms
tmp = '/tmp/zero_prev_model.pac'
open(tmp, 'wb').write(decompress(open(os.path.join(PKG, 'Costumes/PacFiles/0000/FitZero00.pac'), 'rb').read()))
BR = brawl_anim.read_bones(tmp)
sk = S.skel_data.read_skel(os.path.join(MDL, 'model.nusktb'))
UN = [b.name for b in sk.bones]
BIND = {}
for i, b in enumerate(sk.bones):
    loc = np.array(b.transform, dtype=float).T
    BIND[b.name] = (BIND[UN[b.parent_index]] @ loc) if b.parent_index is not None else loc
U2B = info['u2b']

md = decompress(open(os.path.join(PKG, 'PacFiles/FitZeroMotion.pac'), 'rb').read())
ANIMS = {}
for e in arc_entries(md):
    if e[4][:4] == b'bres':
        for folder, items in brres_files(e[4]).items():
            if 'Chr' in folder:
                for n, o in items:
                    if n not in ANIMS:
                        try:
                            ANIMS[n] = brawl_anim.read_chr0(e[4], o)
                        except Exception as ex:  # noqa: BLE001
                            print('anim illisible', n, ex)
print('ANIMATIONS', len(ANIMS))
json.dump({n: a['frames'] for n, a in ANIMS.items()}, open(os.path.join(OUT, 'anims.json'), 'w'), indent=0)


def pose_mats(anim_name, f):
    if anim_name is None:
        return {u: np.eye(4) for u in UN}
    W = brawl_anim.world_pose(BR, ANIMS[anim_name], f)
    top = W.get('TopN', np.eye(4))
    inv_top = np.linalg.inv(top)
    out = {}
    for u in UN:
        b = U2B.get(u)
        out[u] = (inv_top @ W[b] @ np.linalg.inv(BIND[u])) if b in W else np.eye(4)
    return out


HIDE = {'BusterOnM', 'EyeYellowM'}


def render(anim=None, f=0, yaw=-25, size=(420, 560), span=26.0, cy=11.0, pbr=True):
    S2 = 2
    w, h = size[0] * S2, size[1] * S2
    img = np.zeros((h, w, 3)) + np.array([0.12, 0.11, 0.17])
    zb = np.full((h, w), -1e9)
    R = np.array([[math.cos(math.radians(yaw)), 0, math.sin(math.radians(yaw))], [0, 1, 0],
                  [-math.sin(math.radians(yaw)), 0, math.cos(math.radians(yaw))]])
    L = np.array([-0.4, 0.6, 0.7])
    L /= np.linalg.norm(L)
    PM = pose_mats(anim, f)
    sc = w / span
    for ob in mesh.objects:
        if info.get('visibilite', {}).get(ob.name.split('_VIS')[0]) in HIDE:
            continue
        pos = np.array(ob.positions[0].data)[:, :3]
        nrm = np.array(ob.normals[0].data)[:, :3]
        uv = np.array(ob.texture_coordinates[0].data)[:, :2]
        nv = len(pos)
        acc = np.zeros((nv, 3))
        accn = np.zeros((nv, 3))
        for inf in ob.bone_influences:
            M = PM.get(inf.bone_name, np.eye(4))
            ids = np.array([x.vertex_index for x in inf.vertex_weights])
            ws = np.array([x.vertex_weight for x in inf.vertex_weights])[:, None]
            acc[ids] += ws * ((pos[ids] @ M[:3, :3].T) + M[:3, 3])
            accn[ids] += ws * (nrm[ids] @ M[:3, :3].T)
        p = acc @ R.T
        n = accn @ R.T
        n /= np.maximum(np.linalg.norm(n, axis=1, keepdims=True), 1e-9)
        mlab = mat_of.get((ob.name, ob.subindex))
        kind = info.get('types', {}).get(mlab, 'opaque')
        T = tex(col_of.get(mlab, ''))
        PR = tex(prm_of.get(mlab, '')) if pbr and mlab in prm_of else None
        V = np.array([0, 0, 1.0])
        Hh = (L + V) / np.linalg.norm(L + V)
        sx = p[:, 0] * sc + w / 2
        sy = h * 0.92 - (p[:, 1]) * sc
        idx = np.array(ob.vertex_indices).reshape(-1, 3)
        for a, b, c in idx:
            xs, ys = (sx[a], sx[b], sx[c]), (sy[a], sy[b], sy[c])
            den = (ys[1] - ys[2]) * (xs[0] - xs[2]) + (xs[2] - xs[1]) * (ys[0] - ys[2])
            if abs(den) < 1e-9:
                continue
            x0, x1 = int(max(0, min(xs))), int(min(w - 1, max(xs) + 1))
            y0, y1 = int(max(0, min(ys))), int(min(h - 1, max(ys) + 1))
            if x0 > x1 or y0 > y1:
                continue
            gx, gy = np.meshgrid(np.arange(x0, x1 + 1) + 0.5, np.arange(y0, y1 + 1) + 0.5)
            l0 = ((ys[1] - ys[2]) * (gx - xs[2]) + (xs[2] - xs[1]) * (gy - ys[2])) / den
            l1 = ((ys[2] - ys[0]) * (gx - xs[2]) + (xs[0] - xs[2]) * (gy - ys[2])) / den
            l2 = 1 - l0 - l1
            ins = (l0 >= 0) & (l1 >= 0) & (l2 >= 0)
            if not ins.any():
                continue
            z = l0 * p[a, 2] + l1 * p[b, 2] + l2 * p[c, 2]
            sub = zb[y0:y1 + 1, x0:x1 + 1]
            vis = ins & (z > sub)
            if not vis.any():
                continue
            u = l0 * uv[a, 0] + l1 * uv[b, 0] + l2 * uv[c, 0]
            v = l0 * uv[a, 1] + l1 * uv[b, 1] + l2 * uv[c, 1]
            th, tw = T.shape[:2]
            smp = T[(np.mod(v, 1) * (th - 1)).astype(int), (np.mod(u, 1) * (tw - 1)).astype(int)]
            col = smp[..., :3]
            if kind != 'opaque':
                vis &= smp[..., 3] > 0.3
                if not vis.any():
                    continue
            sub[vis] = z[vis]
            nn = l0[..., None] * n[a] + l1[..., None] * n[b] + l2[..., None] * n[c]
            nn /= np.maximum(np.linalg.norm(nn, axis=-1, keepdims=True), 1e-9)
            lam = np.abs(nn @ L)[..., None]
            if not pbr:
                out = np.clip(col * (0.35 + 0.8 * lam), 0, 1)
            elif kind in ('emit', 'lame'):
                out = np.clip(col * 1.15, 0, 1)
            else:
                alb = col ** 2.2
                pm = PR[(np.mod(v, 1) * (PR.shape[0] - 1)).astype(int), (np.mod(u, 1) * (PR.shape[1] - 1)).astype(int)] if PR is not None else np.zeros(col.shape[:-1] + (4,)) + [0, 0.6, 1, 0.3]
                met, rg, ao, sp = pm[..., 0:1], np.clip(pm[..., 1:2], 0.05, 1), pm[..., 2:3], pm[..., 3:4]
                ndh = np.clip(np.abs(nn @ Hh), 0, 1)[..., None]
                ndv = np.clip(np.abs(nn @ V), 0, 1)[..., None]
                a2 = rg ** 4
                D = a2 / (math.pi * (ndh * ndh * (a2 - 1) + 1) ** 2)
                F0 = sp * 0.2 * (1 - met) + alb * met
                spec_c = F0 * D * lam * 0.9
                dif = alb * (1 - met) * (lam * 1.6 + 0.45 * ao)
                env = (alb * met + 0.06 * sp) * (0.35 + 0.65 * (1 - rg)) * ao
                rim = 0.12 * (1 - ndv) ** 3
                out = np.clip(dif + spec_c + env + rim, 0, 1) ** (1 / 2.2)
            reg = img[y0:y1 + 1, x0:x1 + 1]
            reg[vis] = out[vis]
    im = Image.fromarray((img * 255).astype(np.uint8))
    return im.resize(size, Image.LANCZOS)


def mid(n, frac):
    a = ANIMS.get(n)
    return (n, a['frames'] * frac) if a else (None, 0)


views = [('AVANT (couleurs seules)', None, 0, -20, False), ('APRÈS : matériaux Ultimate', None, 0, -20, True),
         ('APRÈS (dos)', None, 0, 160, True)]
for title, n, frac in (('Attente', 'Wait1', 0.0), ('Smash côté', 'AttackS4S', 0.45), ('Victoire', 'Win1', 0.95)):
    an, f = mid(n, frac)
    if an:
        views.append((title, an, f, -30, True))
try:
    fb = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf', 22)
except OSError:
    fb = ImageFont.load_default()
cols = 3
rows = (len(views) + cols - 1) // cols
sheet = Image.new('RGB', (cols * 430 + 10, rows * 600 + 60), (20, 18, 30))
d = ImageDraw.Draw(sheet)
d.text((12, 14), 'Zero (REX) pour Ultimate : rendu avant / après (aperçu hors du jeu)', font=fb, fill=(240, 235, 225))
for k, (title, an, f, yaw, pbr) in enumerate(views):
    im = render(an, f, yaw, pbr=pbr)
    x, y = 10 + (k % cols) * 430, 60 + (k // cols) * 600
    sheet.paste(im, (x, y + 30))
    d.text((x + 4, y), title, font=fb, fill=(130, 220, 150))
sheet.save(IMG)
print('planche ok', len(views), 'vues')
