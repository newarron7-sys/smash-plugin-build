"""Côté B au sol : il saute plus haut (demandé). Hauteur du saut de l'os Trans de d01specials multipliée par K.

usage : python3 side_b_jump.py <animation d'entrée> <animation de sortie> [K]
"""
import sys

import ssbh_data_py as S

src, dst = sys.argv[1:3]
K = float(sys.argv[3]) if len(sys.argv) > 3 else 1.5
a = S.anim_data.read_anim(src)
done = False
# l'os qui porte le saut : Trans ne bouge pas dans cette animation, c'est la racine du corps (Rot ou Hip)
nodes = {nd.name: nd for g in a.groups for nd in g.nodes
         if nd.tracks and hasattr(nd.tracks[0].values[0], 'translation')}
pick = None
for name in ('Trans', 'Rot', 'Hip'):
    nd = nodes.get(name)
    if nd:
        ys = [v.translation[1] for v in nd.tracks[0].values]
        print(name, 'variation y %.2f' % (max(ys) - min(ys)))
        if pick is None and max(ys) - min(ys) > 0.5:
            pick = name
for g in a.groups:
    for nd in g.nodes:
        if nd.name != pick:
            continue
        vals = nd.tracks[0].values
        y0 = vals[0].translation[1]
        top = 0.0
        for v in vals:
            t = list(v.translation)
            t[1] = y0 + (t[1] - y0) * K
            top = max(top, t[1] - y0)
            v.translation = t
        nd.tracks[0].values = vals
        done = True
        print('%s : saut x%.2f, hauteur max %.1f' % (pick, K, top))
assert done, 'aucun os ne monte'
a.save(dst)
