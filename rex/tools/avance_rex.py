"""Avancée (TransN) des animations de REX image par image -> table Rust pour le plugin (generated.rs).

Pour les coups au sol dont l'animation avance mais dont le mouvement de Snake ne déplace pas le perso (« move » faux
dans son motion_list : le modèle partait devant puis revenait d'un coup = téléportation) : l'animation est mise sur
place (retarget_exo.py) et le plugin fait avancer le perso de la même distance, image par image.
usage : python3 avance_rex.py FitChief00.pac FitChiefMotion.pac motion=Anim1+Anim2 ... >> generated.rs
"""
import sys, os
import numpy as np
HERE = os.path.dirname(os.path.abspath(__file__))
for p in ('../../black-shadow/tools', '../../black-shadow/tools/anim', '../../black-shadow/tools/fs', '../../zero/tools', '.'):
    sys.path.insert(0, os.path.join(HERE, p))
import brawl_anim as BA  # noqa: E402
from arc import arc_entries  # noqa: E402
from brres import brres_files  # noqa: E402
from gen_rex import plain  # noqa: E402

MODEL, MOTION = sys.argv[1:3]
import contextlib  # noqa: E402
with contextlib.redirect_stdout(sys.stderr):
    B = BA.read_bones(plain(MODEL))
    _MOT = plain(MOTION)
AN = {}


def walk(d):
    for e in arc_entries(d):
        if e[4][:4] == b'ARC\0':
            walk(e[4])
        elif e[4][:4] == b'bres':
            for fo, items in brres_files(e[4]).items():
                if 'Chr' in fo:
                    for n, o in items:
                        AN.setdefault(n, (e[4], o))


walk(open(_MOT, 'rb').read())
print('// avancée (TransN, axe avant) cumulée de ses animations de REX, image par image (rex/tools/avance_rex.py)')
print('pub static AVANCE: &[(&str, &[f32])] = &[')
for arg in sys.argv[3:]:
    mot, anims = arg.split('=')
    # chaque morceau repart de là où le précédent a mené le perso (Brawl déplace le perso de TransN quand
    # l'animation change : smash avant = début +9,5 puis coup +6,3)
    vals, base = [], 0.0
    for a in anims.split('+'):
        an = BA.read_chr0(*AN[a])
        n = int(an['frames'])
        z0 = None
        for f in range(n):
            W = BA.world_pose(B, an, float(f))
            z = float(W['TransN'][2, 3])
            if z0 is None:
                z0 = z
            vals.append(base + z - z0)
        base = vals[-1]
    print('    ("%s", &[%s]),' % (mot, ', '.join('%.3f' % v for v in vals)))
    print('avance', mot, len(vals), 'images', round(vals[-1], 2), file=sys.stderr)
print('];')
