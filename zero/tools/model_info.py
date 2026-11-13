"""Zero (REX) : contenu du modèle Brawl (os, poids, objets, matériaux, textures) et des animations.

usage : python3 model_info.py <dossier du paquet extrait>
"""
import os, struct, sys
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, '../../black-shadow/tools'))
sys.path.insert(0, os.path.join(HERE, '../../black-shadow/tools/fs'))
sys.path.insert(0, os.path.join(HERE, '../../black-shadow/tools/anim'))
from arc import arc_entries  # noqa: E402
from brres import brres_files, dict_entries  # noqa: E402
import mdl0  # noqa: E402
import brawl_anim  # noqa: E402
from lz import decompress  # noqa: E402

pkg = sys.argv[1]
src = os.path.join(pkg, 'Costumes/PacFiles/0000/FitZero00.pac')
d = decompress(open(src, 'rb').read())
pac = '/tmp/FitZero00_dec.pac'
open(pac, 'wb').write(d)
for e in arc_entries(d):
    print('ARC type', e[0], 'idx', e[1], 'taille', e[2], 'debut', e[4][:4])
dd = [x for x in arc_entries(d) if x[0] == 2][0][4]
files = brres_files(dd)
for k, v in files.items():
    print('BRRES', k, len(v), [n for n, _ in v][:40])
n, o = files['3DModels(NW4R)'][0]
offs = struct.unpack_from('>11i', dd, o + 0x10)
names = ['defs', 'bones', 'pos', 'nrm', 'clr', 'uv', 'mats', 'shaders', 'objs', 'tex', 'pal']
for k, so in zip(names, offs):
    ents = dict_entries(dd, o + so) if so else []
    print('SECTION', k, len(ents), [x for x, _ in ents][:12])
bones = brawl_anim.read_bones(pac)
print('OS', len(bones))
for b in sorted(bones.values(), key=lambda b: b.index):
    print('  %3d %-22s parent %-18s t %s' % (b.index, b.name, b.parent, [round(x, 2) for x in b.t]))
m = mdl0.read_model(dd, o)
for nm, ob in m['objs']:
    bset = sorted(set(v.get('bone', ob['node']) for t in ob['tris'] for v in t))
    print('OBJET %-24s tris %5d attrs %s node %s matrices %s' % (nm, len(ob['tris']), list(ob['attrs']), ob['node'], bset[:20]))
for k, v in m['mats'].items():
    print('MATERIAU', k, v)
# poids : NodeMix
for k, so in zip(names, offs):
    if k != 'defs' or not so:
        continue
    for dn, do in dict_entries(dd, o + so):
        cnt = {}
        p = do
        for _ in range(100000):
            c = dd[p]
            if c == 1:
                break
            if c == 2:
                p += 5
            elif c == 3:
                nw = dd[p + 3]
                p += 4 + 6 * nw
            elif c == 4:
                p += 8
            elif c == 5:
                p += 5
            else:
                break
            cnt[c] = cnt.get(c, 0) + 1
        print('DEF', dn, cnt)
md = decompress(open(os.path.join(pkg, 'PacFiles/FitZeroMotion.pac'), 'rb').read())
mot = '/tmp/FitZeroMotion_dec.pac'
open(mot, 'wb').write(md)
print('MOTION arc', [(e[0], e[1], e[2], e[4][:4]) for e in arc_entries(md)][:10])
an = brawl_anim.read_anims(mot)
print('ANIMATIONS lues', len(an))
print(sorted(an)[:400])
