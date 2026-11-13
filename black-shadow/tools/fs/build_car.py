"""Construit la Black Bull au format Ultimate, à la place du Ganon du Final Smash (boss/ganonboss/model/body/c00).

usage : python3 build_car.py <FitBlackShadowFinal.pac extrait (dossier du mod)> <dossier c00 de référence (Egg Emperor)>
                             <sortie c00> <dossier png>
- géométrie, normales et UV de la voiture lues dans le MDL0 (mdl0.py), placées avec les matrices de leurs os ;
- toute la voiture suit l'os Trans (au sol) : elle fonce avec la bête sans se déformer ;
- squelette, os d'aide et matériaux repris du modèle de référence (qui marche en jeu), textures remplacées
  en gardant leurs noms (pas de config.json).
"""
import json
import os
import shutil
import sys

import numpy as np
import ssbh_data_py as S
from PIL import Image

sys.path.insert(0, os.path.dirname(__file__))
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))
from arc import arc_entries  # noqa: E402
from brres import brres_files  # noqa: E402
import mdl0  # noqa: E402
import tex0  # noqa: E402

FINAL, REF, OUT, PNG = sys.argv[1:5]
os.makedirs(OUT, exist_ok=True)
os.makedirs(PNG, exist_ok=True)

fin = open(FINAL, 'rb').read()
ents = arc_entries(fin)
dd = ents[2][4]
n, o = brres_files(dd)['3DModels(NW4R)'][0]
M = mdl0.read_model(dd, o)
bone_w = {b[1]: b[2] for b in M['bones']}

# ------------------------------------------------------------------ géométrie (deux groupes : carrosserie, vitre)
groups = {'body': {'p': [], 'n': [], 'uv': []}, 'glass': {'p': [], 'n': [], 'uv': []}}
for i, (on, ob) in enumerate(M['objs']):
    P = M['arrays']['pos'][ob['ids'][0]][0]
    N = M['arrays']['nrm'][ob['ids'][1]][0] if ob['ids'][1] >= 0 else None
    U = M['arrays']['uv'][ob['ids'][4]][0] if ob['ids'][4] >= 0 else None
    g = groups['glass' if M['obj_mat'].get(i) == 1 else 'body']
    for t in ob['tris']:
        for v in t:
            b = v.get('bone', ob['node'])
            w = bone_w.get(b, np.eye(4)) if b is not None and b >= 0 else np.eye(4)
            g['p'].append((w @ np.r_[P[v['pos']], 1])[:3])
            nn = (w[:3, :3] @ N[v['nrm']]) if N is not None and 'nrm' in v else np.array([0, 1, 0])
            g['n'].append(nn / (np.linalg.norm(nn) or 1))
            g['uv'].append(U[v['uv0']] if U is not None and 'uv0' in v else np.zeros(2))

allp = np.concatenate([np.array(g['p']) for g in groups.values() if g['p']])
lo, hi = allp.min(0), allp.max(0)
# au sol, centrée sur Trans (la voiture est 2x plus grande que la bête de Brawl : on la ramène à la taille du Ganon)
offset = np.array([-(lo[0] + hi[0]) / 2, -lo[1], -(lo[2] + hi[2]) / 2])
SCALE = 1.0
print('voiture : boite', np.round(lo, 2).tolist(), np.round(hi, 2).tolist(), 'decalage', np.round(offset, 2).tolist())

# ------------------------------------------------------------------ maillage
ref_mesh = S.mesh_data.read_mesh(os.path.join(REF, 'model.numshb'))
proto = ref_mesh.objects[0]
A = S.mesh_data


def attr(name, data):
    a = A.AttributeData(name)
    a.data = np.asarray(data, dtype=np.float32)
    return a


objects = []
for sub, (key, label) in enumerate((('body', 'def_ganond_002'), ('glass', 'alpha_ganond_002'))):
    g = groups[key]
    if not g['p']:
        continue
    p = (np.array(g['p']) + offset) * SCALE
    nrm = np.array(g['n'])
    uv = np.array(g['uv'])
    nv = len(p)
    o = A.MeshObjectData('BlackBull', sub)
    o.positions = [attr('Position0', p)]
    o.normals = [attr('Normal0', np.c_[nrm, np.zeros(nv)])]
    # tangente : perpendiculaire à la normale
    up = np.where(np.abs(nrm[:, 1:2]) > 0.9, np.array([[1, 0, 0]]), np.array([[0, 1, 0]]))
    tg = np.cross(up, nrm)
    tg /= np.maximum(np.linalg.norm(tg, axis=1, keepdims=True), 1e-6)
    o.tangents = [attr('Tangent0', np.c_[tg, np.ones(nv)])]
    o.texture_coordinates = [attr('map1', uv)]
    o.color_sets = [attr('colorSet1', np.full((nv, 4), 0.5))]
    o.vertex_indices = np.arange(nv, dtype=np.uint32)
    inf = A.BoneInfluence('Trans', [A.VertexWeight(int(i), 1.0) for i in range(nv)])
    o.bone_influences = [inf]
    objects.append((o, label))
    print('objet', key, nv // 3, 'triangles')

mesh = ref_mesh
mesh.objects = [o for o, _ in objects]
mesh.save(os.path.join(OUT, 'model.numshb'))

# ------------------------------------------------------------------ modèle, matériaux, squelette, aides
modl = S.modl_data.read_modl(os.path.join(REF, 'model.numdlb'))
modl.entries = [S.modl_data.ModlEntryData(o.name, o.subindex, label) for o, label in objects]
modl.save(os.path.join(OUT, 'model.numdlb'))
for f in ('model.numatb', 'model.nusktb', 'model.nuhlpb'):
    if os.path.exists(os.path.join(REF, f)):
        shutil.copy(os.path.join(REF, f), os.path.join(OUT, f))
try:
    mx = S.meshex_data.MeshExData.from_mesh_objects(mesh.objects)
    mx.save(os.path.join(OUT, 'model.numshexb'))
except Exception as e:  # noqa: BLE001
    print('numshexb non genere :', e)

# ------------------------------------------------------------------ textures (png, converties en nutexb ensuite)
TEX = {}
for tn, to in brres_files(ents[1][4]).get('Textures(NW4R)', []):
    TEX[tn] = tex0.read_tex0(ents[1][4], to).convert('RGBA')
TEX['airframe'].save(os.path.join(PNG, 'def_ganond_002_col.png'))
TEX['glas_sp'].resize((128, 128)).save(os.path.join(PNG, 'alpha_ganond_002_col.png'))
TEX['airframe'].save(os.path.join(PNG, 'def_ganond_001_col.png'))
flat_n = Image.new('RGBA', (64, 64), (128, 128, 255, 255))
prm_body = Image.new('RGBA', (64, 64), (60, 110, 255, 128))   # un peu métallique, assez lisse
prm_glass = Image.new('RGBA', (64, 64), (0, 40, 255, 160))    # vitre : lisse, brillante
for base in ('def_ganond_001', 'def_ganond_002', 'alpha_ganond_002'):
    flat_n.save(os.path.join(PNG, base + '_nor.png'))
prm_body.save(os.path.join(PNG, 'def_ganond_001_prm.png'))
prm_body.save(os.path.join(PNG, 'def_ganond_002_prm.png'))
prm_glass.save(os.path.join(PNG, 'alpha_ganond_002_prm.png'))
json.dump({'offset': offset.tolist(), 'objets': [o.name for o, _ in objects]}, open(os.path.join(OUT, '..', 'black_bull.json'), 'w'))
# aperçu du résultat (relu depuis le fichier écrit)
from PIL import ImageDraw  # noqa: E402
chk = S.mesh_data.read_mesh(os.path.join(OUT, 'model.numshb'))
cols = {0: np.asarray(TEX['airframe'], dtype=float) / 255, 1: np.asarray(TEX['glas_sp'], dtype=float) / 255}
img = Image.new('RGB', (900, 420), (40, 38, 55))
d = ImageDraw.Draw(img)
L = []
for k, ob in enumerate(chk.objects):
    pp = np.array(ob.positions[0].data)[:, :3]
    uu = np.array(ob.texture_coordinates[0].data)
    T = cols.get(ob.subindex, cols[0])
    for vi in range(0, len(ob.vertex_indices), 3):
        tri = [ob.vertex_indices[vi + j] for j in range(3)]
        q = pp[tri]
        # vue de trois quarts
        yaw = np.radians(35)
        x2 = np.cos(yaw) * q[:, 0] + np.sin(yaw) * q[:, 2]
        z2 = -np.sin(yaw) * q[:, 0] + np.cos(yaw) * q[:, 2]
        y2 = q[:, 1] * 0.95 - z2 * 0.3
        nrm = np.cross(q[1] - q[0], q[2] - q[0])
        lam = abs(nrm[1] * 0.6 + nrm[2] * 0.5 + nrm[0] * 0.3) / (np.linalg.norm(nrm) or 1)
        uv = uu[tri].mean(0)
        h, w = T.shape[:2]
        c = T[int(np.mod(uv[1], 1) * (h - 1)), int(np.mod(uv[0], 1) * (w - 1)), :3]
        L.append((z2.mean(), [(450 + x * 4.5, 330 - y * 4.5) for x, y in zip(x2, y2)], tuple(int(255 * min(1, v * (0.5 + 0.9 * lam))) for v in c)))
L.sort(key=lambda t: t[0])
for _, pts, c in L:
    d.polygon(pts, fill=c)
d.line([(40, 330), (860, 330)], fill=(120, 110, 150), width=2)
img.save(os.path.join(OUT, '..', 'black_bull_ultimate.png'))
print('voiture construite')
