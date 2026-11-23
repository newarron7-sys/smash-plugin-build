"""Diagnostic du rendu de son modèle : matériaux (shader, textures, paramètres), maillages (normales lisses ou à
facettes), textures (taille, format, luminosité, canaux PRM/NOR).

usage : python3 diag.py <dossier c00 du modèle> <dossier png des textures> <sortie>
"""
import json
import os
import sys

import numpy as np
import ssbh_data_py as S
from PIL import Image

C00, PNG, OUT = sys.argv[1:4]
os.makedirs(OUT, exist_ok=True)
mat = S.matl_data.read_matl(os.path.join(C00, 'model.numatb'))
rep = {'materiaux': [], 'maillages': [], 'textures': {}}
for e in mat.entries:
    m = {'label': e.material_label, 'shader': e.shader_label,
         'textures': {p.param_id.name: p.data for p in e.textures},
         'vecteurs': {p.param_id.name: list(p.data) for p in e.vectors},
         'flottants': {p.param_id.name: p.data for p in e.floats},
         'booleens': {p.param_id.name: p.data for p in e.booleans}}
    rep['materiaux'].append(m)
    print('MAT', m['label'], m['shader'], m['textures'])
    print('    vec', {k: [round(x, 3) for x in v] for k, v in m['vecteurs'].items()})
modl = S.modl_data.read_modl(os.path.join(C00, 'model.numdlb'))
mesh = S.mesh_data.read_mesh(os.path.join(C00, 'model.numshb'))
lab = {(x.mesh_object_name, x.mesh_object_subindex): x.material_label for x in modl.entries}
for o in mesh.objects:
    p = np.array(o.positions[0].data)[:, :3]
    n = np.array(o.normals[0].data)[:, :3] if o.normals else None
    # facettes : une même position avec plusieurs normales différentes
    key = {}
    if n is not None:
        for i, q in enumerate(np.round(p, 3)):
            key.setdefault(tuple(q), []).append(n[i])
    split = sum(1 for v in key.values() if len(v) > 1 and np.ptp(np.array(v), axis=0).max() > 0.2)
    info = {'nom': o.name, 'sub': o.subindex, 'materiau': lab.get((o.name, o.subindex)), 'sommets': len(p),
            'triangles': len(o.vertex_indices) // 3, 'positions_a_normales_multiples': split,
            'attributs': [a.name for a in (o.positions + o.normals + o.tangents + o.texture_coordinates + o.color_sets)]}
    rep['maillages'].append(info)
    print('MESH', info)
for f in sorted(os.listdir(PNG)):
    im = np.asarray(Image.open(os.path.join(PNG, f)).convert('RGBA'), dtype=float) / 255
    st = {'taille': list(im.shape[:2]), 'moy': np.round(im.reshape(-1, 4).mean(0), 3).tolist(),
          'max': np.round(np.percentile(im.reshape(-1, 4), 99, axis=0), 3).tolist(),
          'min': np.round(np.percentile(im.reshape(-1, 4), 1, axis=0), 3).tolist()}
    rep['textures'][f] = st
    print('TEX', f, st)
json.dump(rep, open(os.path.join(OUT, 'diag.json'), 'w'), indent=1, ensure_ascii=False)
