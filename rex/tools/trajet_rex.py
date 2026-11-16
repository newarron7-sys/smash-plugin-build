"""Trajet (TransN : avant z, hauteur y) des animations de REX, image par image -> table Rust pour le plugin.

Comme avance_rex.py, mais avec la hauteur : spéciaux qui montent ou plongent (Knuckle Joe : Rising Break, bas B).
usage : python3 trajet_rex.py FitXxx00.pac FitXxxMotionEtc.pac nom=Anim1+Anim2 ... >> generated.rs
"""
import sys, os, contextlib
HERE = os.path.dirname(os.path.abspath(__file__))
for p in ('../../black-shadow/tools', '../../black-shadow/tools/anim', '../../black-shadow/tools/fs', '../../zero/tools', '.'):
    sys.path.insert(0, os.path.join(HERE, p))
import brawl_anim as BA  # noqa: E402
from arc import arc_entries  # noqa: E402
from brres import brres_files  # noqa: E402
from gen_rex import plain  # noqa: E402

MODEL, MOTION = sys.argv[1:3]
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
print('// trajet (TransN : avant, hauteur) cumulé de ses animations de REX, image par image (rex/tools/trajet_rex.py)')
print('pub static TRAJET: &[(&str, &[(f32, f32)])] = &[')
for arg in sys.argv[3:]:
    nom, anims = arg.split('=')
    vals, bz, by = [], 0.0, 0.0
    for a in anims.split('+'):
        an = BA.read_chr0(*AN[a])
        n = int(an['frames'])
        z0 = y0 = None
        for f in range(n):
            W = BA.world_pose(B, an, float(f))
            z, y = float(W['TransN'][2, 3]), float(W['TransN'][1, 3])
            if z0 is None:
                z0, y0 = z, y
            vals.append((bz + z - z0, by + y - y0))
        bz, by = vals[-1]
    print('    ("%s", &[%s]),' % (nom, ', '.join('(%.3f, %.3f)' % v for v in vals)))
    print('trajet', nom, len(vals), 'images, fin', [round(x, 2) for x in vals[-1]], file=sys.stderr)
print('];')
