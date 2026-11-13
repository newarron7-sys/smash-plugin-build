"""Marche et course : le jeu fige quand il court (pas de plantage, l'image se bloque).

Dans Ultimate, la vitesse de lecture des boucles de marche et de course = vitesse du perso / avancée de l'os Trans
par image. Les animations de Brawl tournent sur place (Trans immobile) : avancée 0, vitesse infinie, le jeu boucle
sans fin. On fait avancer Trans en ligne droite à la vitesse du pied posé au sol (les pieds ne glissent pas).

usage : python3 fix_move.py <squelette .nusktb> <dossier des .nuanmb>   (modifie les fichiers sur place)
Affiche aussi l'avancée de Trans de toutes les animations (diagnostic).
"""
import os
import sys

import numpy as np
import ssbh_data_py as S

SKEL, D = sys.argv[1:3]
# boucles qui avancent (axe z = devant) : vitesse par défaut si le pied ne donne rien de propre
LOOPS = {'a01walkslow': 0.25, 'a01walkmiddle': 0.5, 'a01walkfast': 0.77, 'a02run': 1.34}

sk = S.skel_data.read_skel(SKEL)
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


def trans_node(a):
    for g in a.groups:
        for nd in g.nodes:
            if nd.name == 'Trans' and nd.tracks and hasattr(nd.tracks[0].values[0], 'translation'):
                return nd
    return None


def foot_speed(a):
    """vitesse (unités par image) à laquelle le pied posé part en arrière = vitesse de marche naturelle"""
    tracks = {}
    for g in a.groups:
        for nd in g.nodes:
            if nd.tracks and hasattr(nd.tracks[0].values[0], 'rotation'):
                tracks[nd.name] = nd.tracks[0].values
    n = int(a.final_frame_index) + 1
    feet = [u for u in ('FootL', 'FootR', 'ToeL', 'ToeR') if u in UN]
    pos = {u: [] for u in feet}
    for k in range(n):
        W = [None] * len(UN)

        def w(i):
            if W[i] is None:
                p = PAR[i]
                pw = w(p) if p is not None and p >= 0 else np.eye(4)
                v = tracks.get(UN[i])
                if v:
                    t = v[min(k, len(v) - 1)]
                    W[i] = pw @ mat_from(t.scale, t.rotation, t.translation)
                else:
                    W[i] = pw @ L0[i]
            return W[i]
        for u in feet:
            pos[u].append(w(UN.index(u))[:3, 3].copy())
    sides = [('FootL', 'ToeL'), ('FootR', 'ToeR')]
    sp = []
    for k in range(n - 1):
        best = None
        for fo, to in sides:
            u = to if to in pos else fo
            if u not in pos:
                continue
            y = pos[u][k][1]
            if best is None or y < best[0]:
                best = (y, u)
        if best is None:
            continue
        u = best[1]
        dz = pos[u][k + 1][2] - pos[u][k][2]
        sp.append(-dz)
    sp = [s for s in sp if s > 0.02]
    return float(np.median(sp)) if len(sp) >= 3 else None


fixed = 0
for f in sorted(os.listdir(D)):
    if not f.endswith('.nuanmb'):
        continue
    p = os.path.join(D, f)
    a = S.anim_data.read_anim(p)
    nd = trans_node(a)
    n = int(a.final_frame_index) + 1
    if nd is None:
        print('%-28s images %4d  Trans ABSENT' % (f, n))
        continue
    tr = np.array([v.translation for v in nd.tracks[0].values], dtype=float)
    d = tr[-1] - tr[0]
    rg = tr.max(0) - tr.min(0)
    print('%-28s images %4d  Trans debut->fin x %6.1f y %6.1f z %6.1f   plage x %5.1f y %5.1f z %5.1f'
          % (f, n, d[0], d[1], d[2], rg[0], rg[1], rg[2]))
    name = f[:-len('.nuanmb')]
    if name not in LOOPS:
        continue
    if abs(d[2]) > 1.0:
        print('   -> avance deja, pas touche')
        continue
    fs = foot_speed(a)
    v = fs if fs is not None and 0.1 < fs < 4.0 else LOOPS[name]
    vals = list(nd.tracks[0].values)
    if len(vals) < n:
        vals = vals + [vals[-1]] * (n - len(vals))
    z0 = vals[0].translation[2]
    out = []
    for k, t in enumerate(vals[:n]):
        tt = list(t.translation)
        tt[2] = float(z0 + v * k)
        out.append(S.anim_data.Transform(list(t.scale), list(t.rotation), tt))
    nd.tracks[0].values = out
    a.save(p)
    fixed += 1
    print('   -> CORRIGE : Trans avance de %.3f par image (pied : %s), soit %.1f par boucle'
          % (v, 'none' if fs is None else '%.3f' % fs, v * (n - 1)))
print('boucles corrigees :', fixed)
