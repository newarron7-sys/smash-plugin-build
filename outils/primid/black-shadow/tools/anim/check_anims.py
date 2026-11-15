"""Vérifie les animations converties : valeurs invalides (NaN, infini), échelles nulles, déplacements énormes,
nombre d'images. Ce qui peut figer le jeu quand il se déplace.

usage : python3 check_anims.py <dossier des .nuanmb>
"""
import math
import os
import sys

import ssbh_data_py as S

D = sys.argv[1]
bad = 0
for f in sorted(os.listdir(D)):
    if not f.endswith('.nuanmb'):
        continue
    a = S.anim_data.read_anim(os.path.join(D, f))
    probs = []
    tr = {}
    for g in a.groups:
        for nd in g.nodes:
            for t in nd.tracks:
                for k, v in enumerate(t.values):
                    vals = []
                    if hasattr(v, 'translation'):
                        vals = list(v.translation) + list(v.rotation) + list(v.scale)
                        if any(abs(x) < 1e-6 for x in v.scale):
                            probs.append('echelle nulle %s' % nd.name)
                        if nd.name in ('Trans', 'Rot', 'Hip'):
                            tr.setdefault(nd.name, []).append(v.translation)
                    elif isinstance(v, (int, float)):
                        vals = [v]
                    if any(isinstance(x, float) and (math.isnan(x) or math.isinf(x)) for x in vals):
                        probs.append('NaN/inf %s image %d' % (nd.name, k))
                        break
    for name, ts in tr.items():
        for i in range(3):
            r = max(t[i] for t in ts) - min(t[i] for t in ts)
            if r > 60:
                probs.append('%s deplacement axe %d = %.1f' % (name, i, r))
    probs = sorted(set(probs))[:6]
    if probs:
        bad += 1
    print('%-28s images %4d %s' % (f, int(a.final_frame_index) + 1, ('PROBLEME ' + '; '.join(probs)) if probs else 'ok'))
print('animations a probleme :', bad)
