"""v0.28 : optimisation du modèle : soude les sommets identiques du numshb (le modèle converti de REX avait 3 sommets
par triangle : 54 522 sommets pour 18 174 triangles). Même rendu (positions, normales, tangentes, UV, couleurs et poids
identiques), fichier bien plus léger et moins de travail pour la console.

usage : python3 weld_mesh.py <model.numshb> [sortie.numshb]
"""
import sys
import numpy as np
import ssbh_data_py as S

src = sys.argv[1]
dst = sys.argv[2] if len(sys.argv) > 2 else src
m = S.mesh_data.read_mesh(src)
GROUPS = ['positions', 'normals', 'binormals', 'tangents', 'texture_coordinates', 'color_sets']
tot0 = tot1 = 0
for o in m.objects:
    n = len(o.positions[0].data)
    cols = []
    for g in GROUPS:
        for a in getattr(o, g):
            cols.append(np.round(np.asarray(a.data, dtype=np.float64).reshape(n, -1), 5))
    # poids : un vecteur (os -> poids) par sommet
    names = [inf.bone_name for inf in o.bone_influences]
    wmat = np.zeros((n, max(1, len(names))))
    for j, inf in enumerate(o.bone_influences):
        for w in inf.vertex_weights:
            wmat[w.vertex_index, j] = w.vertex_weight
    cols.append(np.round(wmat, 5))
    key = np.concatenate(cols, axis=1)
    _, first, inv = np.unique(key, axis=0, return_index=True, return_inverse=True)
    inv = inv.reshape(-1)
    # ordre stable : les sommets gardés dans leur ordre d'apparition
    order = np.argsort(first)
    remap = np.empty_like(order)
    remap[order] = np.arange(len(order))
    keep = first[order]
    newidx = remap[inv]
    for g in GROUPS:
        for a in getattr(o, g):
            a.data = np.asarray(a.data)[keep]
    o.vertex_indices = newidx[np.asarray(o.vertex_indices)].astype(np.uint32)
    infl = []
    for j, inf in enumerate(o.bone_influences):
        ws = [S.mesh_data.VertexWeight(int(v), float(wmat[keep[v], j])) for v in range(len(keep)) if wmat[keep[v], j] > 0]
        if ws:
            infl.append(S.mesh_data.BoneInfluence(inf.bone_name, ws))
    o.bone_influences = infl
    tot0 += n
    tot1 += len(keep)
    print('%-14s %6d -> %6d sommets' % (o.name, n, len(keep)))
m.save(dst)
print('total', tot0, '->', tot1)
