"""Diagnostic du lancer bas et du coup pendant la prise.

usage : python3 throw_dbg.py <dossier du mod> <dossier BlackShadow des assets> <dossier BlackShadow des animations converties>
- événements bruts du PSA (code, image) de ThrowLw et CatchAttack, pour voir les changements de vitesse d'animation ;
- hauteur de l'os Throw (là où l'adversaire est tenu) image par image dans e01throwlw ;
- distance entre la hitbox du coup pendant la prise et l'os Throw.
"""
import os
import sys

import numpy as np
import ssbh_data_py as S

HERE = os.path.dirname(__file__)
sys.path.insert(0, os.path.join(HERE, '..'))
import gen  # noqa: E402
from psa import load, subactions, events  # noqa: E402

MOD, ASSETS, CONV = sys.argv[1:4]
sk = S.skel_data.read_skel(os.path.join(ASSETS, 'fighter/ganon/model/body/c00/model.nusktb'))
UN = [b.name for b in sk.bones]
PAR = [b.parent_index for b in sk.bones]
L0 = [np.array(b.transform, dtype=float).T for b in sk.bones]


def mat_from(sc, q, tr):
    x, y, z, w = q
    r = np.array([[1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
                  [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
                  [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)]])
    m = np.eye(4)
    m[:3, :3] = r @ np.diag(sc)
    m[:3, 3] = tr
    return m


def worlds(path):
    a = S.anim_data.read_anim(path)
    tracks = {}
    for g in a.groups:
        for nd in g.nodes:
            if nd.tracks and hasattr(nd.tracks[0].values[0], 'rotation'):
                tracks[nd.name] = nd.tracks[0].values
    out = []
    for k in range(int(a.final_frame_index) + 1):
        W = [None] * len(UN)

        def w(i):
            if W[i] is None:
                p = PAR[i]
                pw = w(p) if p is not None and p >= 0 else np.eye(4)
                v = tracks.get(UN[i])
                W[i] = pw @ (mat_from(v[min(k, len(v) - 1)].scale, v[min(k, len(v) - 1)].rotation,
                                      v[min(k, len(v) - 1)].translation) if v else L0[i])
            return W[i]
        for i in range(len(UN)):
            w(i)
        out.append(W)
    return out


m = load(os.path.join(MOD, 'FitBlackShadow.pac'))
base, subs = subactions(m)
byname = {}
for i, n, mo in subs:
    if n and mo and n not in byname:
        byname[n] = mo
bones = gen.bone_names(os.path.join(MOD, 'FitBlackShadow00.pac'))
lens = gen.anim_lengths(os.path.join(MOD, 'FitBlackShadowMotionEtc.pac'))
for sub in ('ThrowLw', 'CatchAttack'):
    print('==', sub, 'longueur anim', lens.get(sub))
    f = 0.0
    for code, params in events(m, base, byname[sub]):
        if code == 0x00010100:
            f += gen.pv(*params[0])
        elif code == 0x00020100:
            f = gen.pv(*params[0])
        print('EV %5.1f %08X %s' % (f, code, [round(gen.pv(t, x), 3) for t, x in params][:14]))

MOT = os.path.join(CONV, 'fighter/ganon/motion/body/c00')
ti = UN.index('Throw')
W = worlds(os.path.join(MOT, 'e01throwlw.nuanmb'))
print('== e01throwlw : os Throw (y, z) par image')
for k, Wk in enumerate(W):
    p = Wk[ti][:3, 3]
    print('THROW %3d y %6.2f z %6.2f' % (k, p[1], p[2]))

W = worlds(os.path.join(MOT, 'e00catchattack.nuanmb'))
tl = gen.timeline(m, base, byname['CatchAttack'])
lower = {u.lower(): u for u in UN}
for f, kind, p in tl:
    if kind != 'hit':
        continue
    v = [gen.pv(t, x) for t, x in p]
    raw0 = p[0][1] & 0xffffffff
    ub = gen.BONES.get(bones.get(raw0 >> 16, ''), 'top')
    bn = lower.get(ub)
    k = min(int(f), len(W) - 1)
    hp = (W[k][UN.index(bn)] @ np.array([v[6], v[7], v[8], 1.0]))[:3] if bn else np.array([v[6], v[7], v[8]])
    tp = W[k][ti][:3, 3]
    print('PUMMEL image %s os %s taille %.1f degats %.1f hitbox %s os Throw %s distance %.1f'
          % (f, bn, v[5], v[1], np.round(hp, 1).tolist(), np.round(tp, 1).tolist(), np.linalg.norm(hp - tp)))
