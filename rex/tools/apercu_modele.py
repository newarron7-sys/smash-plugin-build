"""Aperçu d'un modèle Ultimate converti (numshb + numdlb + numatb + textures png) : vue de face et de profil, chaque
triangle coloré par sa texture (au centre de ses UV), ombrage simple. Sert de planche avant de livrer.

usage : python3 apercu_modele.py <dossier du modèle (model.*)> <dossier png des textures> <sortie.png> <titre>
"""
import os, sys
import numpy as np
import ssbh_data_py as S
from PIL import Image, ImageDraw, ImageFont

MDL, PNG, OUT, TITLE = sys.argv[1:5]
mesh = S.mesh_data.read_mesh(os.path.join(MDL, 'model.numshb'))
modl = S.modl_data.read_modl(os.path.join(MDL, 'model.numdlb'))
matl = S.matl_data.read_matl(os.path.join(MDL, 'model.numatb'))
mat_tex = {}
for e in matl.entries:
    for t in e.textures:
        if t.param_id.name == 'Texture0':
            mat_tex[e.material_label] = t.data
obj_mat = {(e.mesh_object_name, e.mesh_object_subindex): e.material_label for e in modl.entries}
TEX = {}


def tex(name):
    if name not in TEX:
        p = os.path.join(PNG, name + '.png')
        TEX[name] = np.asarray(Image.open(p).convert('RGBA')) if os.path.exists(p) else None
    return TEX[name]


tris = []                                   # (positions 3x3, couleur rgb, normale)
for o in mesh.objects:
    P = np.array(o.positions[0].data)[:, :3]
    UV = np.array(o.texture_coordinates[0].data)[:, :2] if o.texture_coordinates else np.zeros((len(P), 2))
    T = tex(mat_tex.get(obj_mat.get((o.name, o.subindex), ''), ''))
    idx = np.array(o.vertex_indices).reshape(-1, 3)
    for a, b, c in idx:
        tri = P[[a, b, c]]
        col = np.array([180, 180, 180])
        if T is not None:
            u, v = UV[[a, b, c]].mean(0)
            x = int((u % 1.0) * (T.shape[1] - 1))
            y = int((v % 1.0) * (T.shape[0] - 1))
            col = T[y, x, :3].astype(float)
        n = np.cross(tri[1] - tri[0], tri[2] - tri[0])
        nl = np.linalg.norm(n)
        tris.append((tri, col, n / nl if nl > 1e-9 else np.array([0, 0, 1.0])))
print('triangles', len(tris), 'objets', len(mesh.objects))

W, H = 520, 620
light = np.array([0.3, 0.5, 0.8])
light /= np.linalg.norm(light)


def view(axis):
    """axis 'face' : x/y, profondeur z ; 'profil' : z/y, profondeur x"""
    im = Image.new('RGB', (W, H), (238, 240, 246))
    d = ImageDraw.Draw(im)
    allp = np.vstack([t[0] for t in tris])
    hx = (0, 1) if axis == 'face' else (2, 1)
    dep = 2 if axis == 'face' else 0
    lo, hi = allp[:, list(hx)].min(0), allp[:, list(hx)].max(0)
    sc = 0.88 * min(W / max(hi[0] - lo[0], 1e-3), H / max(hi[1] - lo[1], 1e-3))
    cx, cy = (lo + hi) / 2
    order = sorted(range(len(tris)), key=lambda i: tris[i][0][:, dep].mean())
    for i in order:
        tri, col, n = tris[i]
        nn = n if axis == 'face' else np.array([n[2], n[1], n[0]])
        sh = 0.45 + 0.55 * abs(float(np.dot(nn, light)))
        c = tuple(int(v) for v in np.clip(col * sh, 0, 255))
        pts = [(W / 2 + (p[hx[0]] - cx) * sc, H / 2 - (p[hx[1]] - cy) * sc) for p in tri]
        d.polygon(pts, fill=c)
    return im


face, prof = view('face'), view('profil')
out = Image.new('RGB', (W * 2 + 30, H + 70), (255, 255, 255))
out.paste(face, (10, 60))
out.paste(prof, (W + 20, 60))
dr = ImageDraw.Draw(out)
try:
    f = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf', 26)
    f2 = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf', 18)
except Exception:  # noqa: BLE001
    f = f2 = ImageFont.load_default()
dr.text((14, 14), TITLE, fill=(20, 20, 30), font=f)
dr.text((14, H + 62 - 6), 'face', fill=(90, 90, 100), font=f2)
dr.text((W + 24, H + 62 - 6), 'profil', fill=(90, 90, 100), font=f2)
out.save(OUT)
print('apercu', OUT)
