"""Méthode de Zero : le modèle de REX (construit en exo) est rattaché DIRECTEMENT aux os du perso de base.

Chaque os H_Exo_X du maillage -> l'os du perso de base associé à X, ou celui de son plus proche parent associé
(cheveux, veste, visage -> tête, buste…). Le squelette redevient celui du perso de base tel quel, et le nuhlpb
reprend seulement ses contraintes à lui : plus aucune copie de rotation, ce que montre l'aperçu = ce que fait le jeu.
usage : python3 deexo.py <dossier model exo> <model.json> <dossier de sortie>
"""
import json, os, shutil, sys
import ssbh_data_py as S

SRC, INFO, OUT = sys.argv[1:4]
os.makedirs(OUT, exist_ok=True)
pair = json.load(open(INFO))['exo']['paires']
sk = S.skel_data.read_skel(os.path.join(SRC, 'model.nusktb'))
names = [b.name for b in sk.bones]
par = {b.name: (names[b.parent_index] if b.parent_index is not None else None) for b in sk.bones}


def cible(n):
    while n and n.startswith('H_Exo_'):
        b = n[6:]
        if b in pair:
            return pair[b]
        n = par[n]
    return n or 'Hip'


mesh = S.mesh_data.read_mesh(os.path.join(SRC, 'model.numshb'))
used = {}
for ob in mesh.objects:
    acc = {}
    for inf in ob.bone_influences:
        t = cible(inf.bone_name)
        used[t] = used.get(t, 0) + 1
        d = acc.setdefault(t, {})
        for w in inf.vertex_weights:
            d[w.vertex_index] = d.get(w.vertex_index, 0.0) + w.vertex_weight
    infl = []
    for t, d in acc.items():
        bi = S.mesh_data.BoneInfluence(t, [S.mesh_data.VertexWeight(i, float(w)) for i, w in sorted(d.items())])
        infl.append(bi)
    ob.bone_influences = infl
mesh.save(os.path.join(OUT, 'model.numshb'))
base = [b for b in sk.bones if not b.name.startswith('H_Exo_')]
nb = len(base)
assert all(not b.name.startswith('H_Exo_') for b in sk.bones[:nb])
sk.bones = sk.bones[:nb]
sk.save(os.path.join(OUT, 'model.nusktb'))
h = S.hlpb_data.read_hlpb(os.path.join(SRC, 'model.nuhlpb'))
h.orient_constraints = [c for c in h.orient_constraints if not c.target_bone_name.startswith('H_Exo_')]
h.save(os.path.join(OUT, 'model.nuhlpb'))
for f in os.listdir(SRC):
    if f not in ('model.numshb', 'model.nusktb', 'model.nuhlpb'):
        shutil.copy(os.path.join(SRC, f), os.path.join(OUT, f))
print('os du perso de base', nb, '; contraintes gardees', len(h.orient_constraints), '; os utilises', sorted(used))
