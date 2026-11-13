"""Diagnostic : os du sabre de Zero (Brawl) et de l'épée de Roy (Ultimate), pour placer ses hitbox sur l'épée de Roy.

usage : python3 diag_sword.py <paquet extrait> <dossier modèle Roy> <generated.rs>
"""
import os, re, sys
import numpy as np
import ssbh_data_py as S
HERE = os.path.dirname(os.path.abspath(__file__))
for p in ('../../black-shadow/tools', '../../black-shadow/tools/anim', '.'):
    sys.path.insert(0, os.path.join(HERE, p))
from arc import arc_entries  # noqa: E402
from brres import brres_files  # noqa: E402
import brawl_anim  # noqa: E402
from lz import decompress  # noqa: E402

PKG, ROY, GEN = sys.argv[1:4]
tmp = '/tmp/diag_sword.pac'
open(tmp, 'wb').write(decompress(open(os.path.join(PKG, 'Costumes/PacFiles/0000/FitZero00.pac'), 'rb').read()))
BR = brawl_anim.read_bones(tmp)
md = decompress(open(os.path.join(PKG, 'PacFiles/FitZeroMotion.pac'), 'rb').read())
ANIMS = {}
for e in arc_entries(md):
    if e[4][:4] == b'bres':
        for folder, items in brres_files(e[4]).items():
            if 'Chr' in folder:
                for n, o in items:
                    ANIMS.setdefault(n, (e[4], o))
names = {b: BR[b] for b in BR}
print('== ZERO (Brawl), os du bras droit et du sabre')
chain = [b for b in BR if re.search(r'sword|have|hand|arm', b, re.I)]
for b in chain:
    bb = BR[b]
    print('%-16s parent %-14s t %s r %s monde %s' % (b, bb.parent, np.round(bb.t, 3), np.round(bb.r, 1), np.round(bb.world[:3, 3], 3)))
for an, f in (('AttackS4S', 8), ('AttackS3S', 8), ('Attack11', 4)):
    d, o = ANIMS[an]
    A = brawl_anim.read_chr0(d, o)
    W = brawl_anim.world_pose(BR, A, f)
    print('-- pose', an, f)
    for b in chain:
        if 'Sword' in b or 'Have' in b:
            m = W[b]
            print('   %-16s pos %s  axes x %s y %s z %s' % (b, np.round(m[:3, 3], 2), np.round(m[:3, 0], 2), np.round(m[:3, 1], 2), np.round(m[:3, 2], 2)))
    # longueur du sabre : distance entre la poignée et le bout des os de lame
    if 'SwordN' in W:
        h = W['SwordN'][:3, 3]
        for b in chain:
            if 'Blade' in b:
                print('   distance SwordN -> %s : %.2f' % (b, np.linalg.norm(W[b][:3, 3] - h)))
print('== ROY (Ultimate)')
sk = S.skel_data.read_skel(os.path.join(ROY, 'model.nusktb'))
W = {}


def wm(i):
    b = sk.bones[i]
    if b.name not in W:
        loc = np.array(b.transform, dtype=float).T
        W[b.name] = (wm(b.parent_index) @ loc) if b.parent_index is not None else loc
    return W[b.name]


for i in range(len(sk.bones)):
    wm(i)
for b in sk.bones:
    if re.search(r'sword|have|hand|arm|katana|blade', b.name, re.I):
        loc = np.array(b.transform, dtype=float).T
        par = sk.bones[b.parent_index].name if b.parent_index is not None else '-'
        print('%-16s parent %-14s t %s monde %s axes x %s z %s' % (b.name, par, np.round(loc[:3, 3], 3), np.round(W[b.name][:3, 3], 2),
                                                               np.round(W[b.name][:3, 0], 2), np.round(W[b.name][:3, 2], 2)))
print('== hitbox du sabre (generated.rs)')
for l in open(GEN):
    if 'sword' in l:
        m = re.search(r'f: ([\d.]+).*bone: "([^"]+)".*size: ([\d.]+), x: ([-\d.]+), y: ([-\d.]+), z: ([-\d.]+)', l)
        if m:
            print('  ', m.groups())
    elif 'script:' in l:
        print(l.strip()[:60])
