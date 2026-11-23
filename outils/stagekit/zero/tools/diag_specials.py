"""Diagnostic : les spéciaux de Zero dans P+Ex (sous-actions, animations, événements), pour la v0.2.

usage : python3 diag_specials.py <paquet extrait>
"""
import os, sys
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, '../../black-shadow/tools'))
sys.path.insert(0, HERE)
import gen  # noqa: E402
from arc import arc_entries  # noqa: E402
from brres import brres_files, chr0_info  # noqa: E402
from psa import load, subactions  # noqa: E402
from lz import decompress  # noqa: E402

PKG = sys.argv[1]
tmp = '/tmp/diag_sp_model.pac'
open(tmp, 'wb').write(decompress(open(os.path.join(PKG, 'Costumes/PacFiles/0000/FitZero00.pac'), 'rb').read()))
bones = gen.bone_names(tmp)
gen.BONES = {n: n for n in bones.values()}
md = decompress(open(os.path.join(PKG, 'PacFiles/FitZeroMotion.pac'), 'rb').read())
lens = {}
for e in arc_entries(md):
    if e[4][:4] == b'bres':
        for folder, items in brres_files(e[4]).items():
            if 'Chr' in folder:
                for n, o in items:
                    lens.setdefault(n, chr0_info(e[4], o)[1])
print('ANIMATIONS SPECIAL', sorted((n, l) for n, l in lens.items() if 'Special' in n or 'Buster' in n or 'Final' in n))
m = load(os.path.join(PKG, 'PacFiles/FitZero.pac'))
base, subs = subactions(m)
for i, n, mo in subs:
    if not n or 'Special' not in n:
        continue
    tl = gen.timeline(m, base, mo) if mo else []
    print('=== %3d %-24s anim %s' % (i, n, lens.get(n)))
    for f, k, p in tl:
        if k == 'hit':
            print('   %5.1f HIT %s' % (f, gen.ev_line(f, p, bones)[:230]))
        else:
            print('   %5.1f %s %s' % (f, k, str(p)[:160] if p else ''))
from psa import events  # noqa: E402
print()
print('##### EVENEMENTS BRUTS DES SPECIAUX (code, paramètres)')
for i, n, mo in subs:
    if not n or 'Special' not in n or not mo:
        continue
    print('=== %s' % n)
    f = 0.0
    for code, params in events(m, base, mo):
        if code == 0x00010100:
            f += gen.pv(*params[0])
            continue
        if code == 0x00020100:
            f = gen.pv(*params[0])
            continue
        print('   %5.1f %08X %s' % (f, code, [round(gen.pv(t, v), 3) for t, v in params][:14]))
# tous les types d'événements bruts d'un spécial (pour voir articles / projectiles / variables)
if hasattr(gen, 'raw_events'):
    pass
