"""Images d'interface de Black Shadow (portraits, icône de stock) rendues à partir de son skin et de ses poses.

usage : python3 render_ui.py <dossier BlackShadow des assets> <dossier des textures .png> <dims.json> <sortie>
Écrit <sortie>/png/chara_X_ganon_00.png (puis le workflow les convertit en .bntx) et une planche de contrôle.
"""
import json
import math
import os
import sys

import numpy as np
import ssbh_data_py as S
from PIL import Image, ImageDraw, ImageFilter, ImageFont

ASSETS, TEXDIR, DIMS, OUT = sys.argv[1:5]
MDL = os.path.join(ASSETS, 'fighter/ganon/model/body/c00')
MOT = os.path.join(ASSETS, 'fighter/ganon/motion/body/c00')
os.makedirs(os.path.join(OUT, 'png'), exist_ok=True)

# tailles par défaut (remplacées par celles du mod de référence)
DEFAULT = {'chara_0': (128, 128), 'chara_1': (512, 512), 'chara_2': (64, 64), 'chara_3': (1024, 1024),
           'chara_4': (256, 256), 'chara_6': (512, 256), 'chara_7': (456, 300)}  # chara_7 : lu dans chara_7_allmight_00.bntx (mod chara7 du joueur)
dims = dict(DEFAULT)
try:
    for k, v in json.load(open(DIMS)).items():
        if 'width' in v:
            dims[k] = (v['width'], v['height'])
except (OSError, ValueError):
    pass
print('dimensions', dims)


def col(m):
    return np.array(m, dtype=float).T


def mat_from(sc, q, tr):
    x, y, z, w = q
    r = np.array([[1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
                  [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
                  [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)]])
    m = np.eye(4)
    m[:3, :3] = r @ np.diag(sc)
    m[:3, 3] = tr
    return m


sk = S.skel_data.read_skel(os.path.join(MDL, 'model.nusktb'))
UB = sk.bones
UN = [b.name for b in UB]
PAR = [b.parent_index for b in UB]
L0 = [col(b.transform) for b in UB]
W0 = [col(sk.calculate_world_transform(b)) for b in UB]
W0inv = [np.linalg.inv(w) for w in W0]


def pose(anim_file, frame):
    a = S.anim_data.read_anim(os.path.join(MOT, anim_file))
    tracks = {}
    for g in a.groups:
        for nd in g.nodes:
            if nd.tracks and hasattr(nd.tracks[0].values[0], 'rotation'):
                tracks[nd.name] = nd.tracks[0].values
    W = [None] * len(UB)

    def w(i):
        if W[i] is not None:
            return W[i]
        p = PAR[i]
        pw = w(p) if p is not None and p >= 0 else np.eye(4)
        if UN[i] in tracks:
            v = tracks[UN[i]][min(frame, len(tracks[UN[i]]) - 1)]
            loc = mat_from(v.scale, v.rotation, v.translation)
        else:
            loc = L0[i]
        W[i] = pw @ loc
        return W[i]

    for i in range(len(UB)):
        w(i)
    # on recentre sur place (sans le déplacement de l'animation)
    t = W[UN.index('Trans')][:3, 3].copy()
    t[1] = 0
    for i in range(len(W)):
        W[i] = W[i].copy()
        W[i][:3, 3] -= t
    return W


# ------------------------------------------------------------------ modèle, matériaux, textures
mesh = S.mesh_data.read_mesh(os.path.join(MDL, 'model.numshb'))
modl = S.modl_data.read_modl(os.path.join(MDL, 'model.numdlb'))
matl = S.matl_data.read_matl(os.path.join(MDL, 'model.numatb'))
mat_of = {(e.mesh_object_name, e.mesh_object_subindex): e.material_label for e in modl.entries}
tex_of = {}
for e in matl.entries:
    for t in e.textures:
        # la texture de couleur (…_col) du matériau
        if str(t.data).lower().endswith('_col') and e.material_label not in tex_of:
            tex_of[e.material_label] = t.data
# secours par nom d'objet si le matériau ne dit rien
BY_NAME = {'capein': 'bs_sdw_mant_in_col', 'capeout': 'bs_sdw_mant_out_col', 'horn': 'bs_sdw_horn_col',
           'eye': 'bs_sdw_eye_col'}
TEX = {}
for f in os.listdir(TEXDIR):
    if f.endswith('.png'):
        TEX[f[:-4]] = np.asarray(Image.open(os.path.join(TEXDIR, f)).convert('RGBA'), dtype=float) / 255.0
print('textures', sorted(TEX), 'materiaux', tex_of)

OBJS = []
for o in mesh.objects:
    name = o.name.lower()
    if 'shadow' in name or '_lod' in name or 'eyel' in name and 'blink' in name:
        continue
    pos = np.array(o.positions[0].data, dtype=float)[:, :3]
    nrm = np.array(o.normals[0].data, dtype=float)[:, :3] if o.normals else np.zeros_like(pos)
    uv = np.array(o.texture_coordinates[0].data, dtype=float)[:, :2] if o.texture_coordinates else np.zeros((len(pos), 2))
    idx = np.array(o.vertex_indices, dtype=int).reshape(-1, 3)
    wm = np.zeros((len(pos), len(UN)))
    for inf in o.bone_influences:
        if inf.bone_name in UN:
            bi = UN.index(inf.bone_name)
            for vw in inf.vertex_weights:
                wm[vw.vertex_index, bi] += vw.vertex_weight
    parent = UN.index(o.parent_bone_name) if (not o.bone_influences and o.parent_bone_name in UN) else None
    tname = tex_of.get(mat_of.get((o.name, o.subindex), ''), '')
    if tname not in TEX:
        tname = next((v for k, v in BY_NAME.items() if k in name), 'bs_sdw_body_col')
    tex = TEX.get(tname)
    print('objet', o.name, '->', tname)
    OBJS.append((o.name, pos, nrm, uv, idx, wm, parent, tex))
print('objets', [o[0] for o in OBJS])


def skinned(W):
    skin = np.stack([W[i] @ W0inv[i] for i in range(len(W))])
    out = []
    for name, pos, nrm, uv, idx, wm, parent, tex in OBJS:
        hp = np.c_[pos, np.ones(len(pos))]
        if parent is not None:
            m = np.repeat(W[parent][None], len(pos), 0)
        else:
            s = wm.sum(1, keepdims=True)
            s[s == 0] = 1
            m = np.einsum('vb,bij->vij', wm / s, skin)
        vw = np.einsum('vij,vj->vi', m, hp)[:, :3]
        nw = np.einsum('vij,vj->vi', m[:, :3, :3], nrm)
        nw /= np.maximum(np.linalg.norm(nw, axis=1, keepdims=True), 1e-9)
        out.append((vw, nw, uv, idx, tex))
    return out


def camera(yaw, pitch):
    cy, sy = math.cos(math.radians(yaw)), math.sin(math.radians(yaw))
    cp, sp = math.cos(math.radians(pitch)), math.sin(math.radians(pitch))
    ry = np.array([[cy, 0, sy], [0, 1, 0], [-sy, 0, cy]])
    rx = np.array([[1, 0, 0], [0, cp, -sp], [0, sp, cp]])
    return rx @ ry


def render(W, size, center, span, yaw=150, pitch=6, ss=2, off=(0.0, 0.0)):
    """off : décalage du sujet dans l'image, en fraction de largeur / hauteur (+ = droite / bas)"""
    """rendu orthographique : `center` (monde) au centre, `span` unités sur la largeur"""
    w, h = size[0] * ss, size[1] * ss
    R = camera(yaw, pitch)
    zb = np.full((h, w), -1e9)
    img = np.zeros((h, w, 4))
    light = np.array([-0.4, 0.6, 0.7])
    light /= np.linalg.norm(light)
    sc = w / span
    c = R @ np.asarray(center, dtype=float)
    for vw, nw, uv, idx, tex in skinned(W):
        p = vw @ R.T
        n = nw @ R.T
        sx = (p[:, 0] - c[0]) * sc + w * (0.5 + off[0])
        sy = h * (0.5 + off[1]) - (p[:, 1] - c[1]) * sc
        sz = p[:, 2]
        for a, b, cc in idx:
            x0, x1, x2 = sx[a], sx[b], sx[cc]
            y0, y1, y2 = sy[a], sy[b], sy[cc]
            minx, maxx = int(max(0, math.floor(min(x0, x1, x2)))), int(min(w - 1, math.ceil(max(x0, x1, x2))))
            miny, maxy = int(max(0, math.floor(min(y0, y1, y2)))), int(min(h - 1, math.ceil(max(y0, y1, y2))))
            if minx > maxx or miny > maxy:
                continue
            den = (y1 - y2) * (x0 - x2) + (x2 - x1) * (y0 - y2)
            if abs(den) < 1e-12:
                continue
            gx, gy = np.meshgrid(np.arange(minx, maxx + 1) + 0.5, np.arange(miny, maxy + 1) + 0.5)
            l0 = ((y1 - y2) * (gx - x2) + (x2 - x1) * (gy - y2)) / den
            l1 = ((y2 - y0) * (gx - x2) + (x0 - x2) * (gy - y2)) / den
            l2 = 1 - l0 - l1
            ins = (l0 >= 0) & (l1 >= 0) & (l2 >= 0)
            if not ins.any():
                continue
            z = l0 * sz[a] + l1 * sz[b] + l2 * sz[cc]
            sub = zb[miny:maxy + 1, minx:maxx + 1]
            vis = ins & (z > sub)
            if not vis.any():
                continue
            sub[vis] = z[vis]
            nn = l0[..., None] * n[a] + l1[..., None] * n[b] + l2[..., None] * n[cc]
            nn /= np.maximum(np.linalg.norm(nn, axis=-1, keepdims=True), 1e-9)
            dif = np.abs(nn @ light)
            rim = np.clip(1 - np.abs(nn[..., 2]), 0, 1) ** 3
            if tex is not None:
                u = l0 * uv[a, 0] + l1 * uv[b, 0] + l2 * uv[cc, 0]
                v = l0 * uv[a, 1] + l1 * uv[b, 1] + l2 * uv[cc, 1]
                th, tw = tex.shape[:2]
                tu = (np.mod(u, 1.0) * (tw - 1)).astype(int)
                tv = (np.mod(v, 1.0) * (th - 1)).astype(int)
                base = tex[tv, tu, :3]
            else:
                base = np.broadcast_to(np.array([0.3, 0.26, 0.4]), nn.shape)
            # éclairage de portrait : ses textures sont très sombres, on éclaire franchement comme les rendus officiels
            shade = base * (0.75 + 1.25 * dif[..., None]) + 0.3 * rim[..., None] * np.array([0.75, 0.6, 1.0])
            shade = np.clip(shade, 0, 1) ** 0.8
            region = img[miny:maxy + 1, minx:maxx + 1]
            region[vis, :3] = np.clip(shade[vis], 0, 1)
            region[vis, 3] = 1.0
    im = Image.fromarray((img * 255).astype(np.uint8), 'RGBA')
    return im.resize(size, Image.LANCZOS)


def outline(im, px, color=(0, 0, 0, 255)):
    a = im.split()[3]
    big = a.filter(ImageFilter.MaxFilter(px * 2 + 1))
    bg = Image.new('RGBA', im.size, color)
    out = Image.new('RGBA', im.size, (0, 0, 0, 0))
    out.paste(bg, mask=big)
    out.alpha_composite(im)
    return out


# ------------------------------------------------------------------ images
WAIT = pose('a00wait1.nuanmb', 0)
POWER = WAIT
head = W0[UN.index('Head')][:3, 3]
hp = WAIT[UN.index('Head')][:3, 3] + np.array([0, 2.0, 0])

imgs = {}
# record / sélection (visage carré)
imgs['chara_0'] = render(WAIT, dims['chara_0'], hp, 15, off=(0.0, 0.12))
# sélection (ring) : corps entier en bas à droite
imgs['chara_1'] = render(POWER, dims['chara_1'], np.array([0, 15, 0]), 30, off=(0.18, 0.1))
# icône de stock : la tête avec un contour noir
imgs['chara_2'] = outline(render(WAIT, dims['chara_2'], hp, 15, yaw=155, pitch=5, ss=4, off=(0.0, 0.1)), 2)
# VS / résultats : corps entier en bas à gauche
imgs['chara_3'] = render(POWER, dims['chara_3'], np.array([0, 13, 0]), 40, off=(-0.18, 0.03))
# portrait de combat (visage)
imgs['chara_4'] = render(WAIT, dims['chara_4'], hp + np.array([0, -1.0, 0]), 17, off=(0.0, 0.12))
# Final Smash : bandeau sur les yeux
imgs['chara_6'] = render(WAIT, dims['chara_6'], hp + np.array([0, -1.2, 0]), 9)
# icône de la grille de sélection
# icône de la grille : il regarde droit devant
# icône de la grille : visage de face. Dans sa pose d'attente il tourne la tête, donc la caméra se place
# face à son visage (direction avant de l'os Head ; au repos, l'avant est vu avec yaw = 180)
hi = UN.index('Head')
fwd = WAIT[hi][:3, :3] @ np.linalg.inv(W0[hi][:3, :3]) @ np.array([0.0, 0.0, 1.0])
face_yaw = math.degrees(math.atan2(fwd[0], -fwd[2])) + 180.0  # vérifié en rendu : sans +180 on voit sa nuque
print('chara_7 : yaw du visage', round(face_yaw, 1))
imgs['chara_7'] = render(WAIT, dims['chara_7'], hp + np.array([0, -0.6, 0]), 13.5, yaw=face_yaw, pitch=2, off=(0.0, 0.04))

for k, im in imgs.items():
    im.save(os.path.join(OUT, 'png', '%s_ganon_00.png' % k))
    print('image', k, im.size)

# planche de contrôle
f = None
try:
    f = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf', 18)
except OSError:
    f = ImageFont.load_default()
NAMES = {'chara_0': 'record', 'chara_1': 'sélection', 'chara_2': 'icône de stock', 'chara_3': 'VS / résultats',
         'chara_4': 'portrait en combat', 'chara_6': 'Final Smash', 'chara_7': 'icône de la grille'}
cell = 300
sheet = Image.new('RGB', (cell * 4 + 50, 60 + 2 * (cell + 40)), (24, 22, 36))
d = ImageDraw.Draw(sheet)
d.text((10, 15), 'Interface de Black Shadow (rendu de son skin)', font=f, fill=(235, 225, 200))
for j, (k, im) in enumerate(imgs.items()):
    x, y = 10 + (j % 4) * (cell + 10), 60 + (j // 4) * (cell + 40)
    d.text((x, y), '%s (%s, %dx%d)' % (NAMES[k], k, im.size[0], im.size[1]), font=ImageFont.truetype(
        '/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf', 13) if f else None, fill=(235, 225, 200))
    t = im.copy()
    t.thumbnail((cell, cell - 24))
    bg = Image.new('RGBA', t.size, (60, 56, 84, 255))
    bg.alpha_composite(t)
    sheet.paste(bg.convert('RGB'), (x, y + 22))
sheet.save(os.path.join(OUT, 'planche_interface.png'))
print('planche ok')
