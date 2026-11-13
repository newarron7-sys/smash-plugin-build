"""Passage des décalages de hitbox de REX (repère de l'os de Brawl) au repère de l'os d'Ultimate associé.

Les os des membres de Brawl sont orientés le long de Y (jambe droite +Y, gauche -Y), ceux d'Ultimate autrement :
un décalage « y: 8 » sur le genou partait de côté au lieu de suivre le tibia (bas tilt qui ne touchait pas).
retarget_exo.py donne à l'os d'Ultimate u la rotation monde de E_n = BW_n . KEY_n (KEY_n = NW_n^-1 . EXW_n) :
un vecteur v dans le repère de l'os de Brawl n vaut donc R_KEY^T . v dans celui de u.

usage : python3 hit_keys.py <sortie de build_model_rex.py (model.json, model/model.nusktb)> <keys.json>
"""
import json, os, sys
import numpy as np
import ssbh_data_py as S

MOUT, OUT = sys.argv[1:3]
info = json.load(open(os.path.join(MOUT, 'model.json')))
PAIR = info['exo']['paires']
NW = {n: np.array(m, dtype=float) for n, m in info['exo']['nw'].items()}
sk = S.skel_data.read_skel(os.path.join(MOUT, 'model', 'model.nusktb'))
UN = [b.name for b in sk.bones]
IDX = {n: i for i, n in enumerate(UN)}
PAR = [b.parent_index if b.parent_index is not None and b.parent_index >= 0 else None for b in sk.bones]
L0 = [np.array(b.transform, dtype=float).T for b in sk.bones]
W = [None] * len(UN)


def w(i):
    if W[i] is None:
        W[i] = (w(PAR[i]) @ L0[i]) if PAR[i] is not None else L0[i]
    return W[i]


def orth(R):
    U, _, Vt = np.linalg.svd(R)
    m = U @ Vt
    if np.linalg.det(m) < 0:
        U[:, -1] *= -1
        m = U @ Vt
    return m


keys = {}
for n, u in PAIR.items():
    if n not in NW or ('H_Exo_' + n) not in IDX:
        continue
    key = np.linalg.inv(NW[n]) @ w(IDX['H_Exo_' + n])
    keys[n] = {'u': u.lower(), 'R': orth(key[:3, :3]).T.tolist()}
json.dump(keys, open(OUT, 'w'), indent=1)
print('reperes de hitbox :', len(keys), 'os ;', ', '.join('%s->%s' % (n, k['u']) for n, k in sorted(keys.items())))
