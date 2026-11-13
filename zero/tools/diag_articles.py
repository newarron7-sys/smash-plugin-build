"""Diagnostic : hitbox rangées hors des sous-actions de Zero (articles : tirs de buster), pour la v0.3.

Cherche dans le moveset P+Ex toutes les listes d'événements qui contiennent une hitbox (06000D00 / 06150F00),
retire celles des sous-actions du personnage, et affiche le reste (projectiles).
usage : python3 diag_articles.py <paquet extrait>
"""
import os, struct, sys
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, '../../black-shadow/tools'))
sys.path.insert(0, HERE)
import gen  # noqa: E402
from psa import load, subactions, header, events, sections  # noqa: E402

PKG = sys.argv[1]
m = load(os.path.join(PKG, 'PacFiles/FitZero.pac'))
base, subs = subactions(m)
_, h, ext = header(m)
known = set()
n = (h['subGFX'] - h['subMain']) // 4
for tbl in ('subMain', 'subGFX', 'subSFX', 'subOther'):
    for i in range(n):
        o = struct.unpack_from('>I', m, base + h[tbl] + 4 * i)[0]
        if o:
            known.add(o)
na = (h['exitActions'] - h['entryActions']) // 4 if h['exitActions'] > h['entryActions'] else 0
for tbl in ('entryActions', 'exitActions'):
    for i in range(max(na, 0)):
        try:
            o = struct.unpack_from('>I', m, base + h[tbl] + 4 * i)[0]
        except struct.error:
            break
        if o:
            known.add(o)
print('sous-actions/actions connues', len(known), '| sections', sections(m)[1])
print('externes', [x[1] for x in ext][:40])
CODES = (0x06000D00, 0x06150F00, 0x06001100)
size = len(m)
starts = {}
for o in range(base, size - 8, 4):
    code = struct.unpack_from('>I', m, o)[0]
    if code not in CODES:
        continue
    # remonte au début de la liste (événements de 8 octets, terminée par 0)
    s = o
    while s - 8 >= base:
        c = struct.unpack_from('>I', m, s - 8)[0]
        ns = c >> 24
        if c == 0 or ns > 0x20:
            break
        s -= 8
    starts.setdefault(s - base, []).append(o - base)
print('listes avec hitbox', len(starts), 'dont hors personnage', len([s for s in starts if s not in known]))
for s in sorted(starts):
    if s in known:
        continue
    print('=== liste a 0x%X' % s)
    f = 0.0
    for code, params in events(m, base, s, 80):
        if code == 0x00010100:
            f += gen.pv(*params[0])
            continue
        if code == 0x00020100:
            f = gen.pv(*params[0])
            continue
        vals = [round(gen.pv(t, v), 3) for t, v in params][:16]
        if code in CODES:
            p = params
            print('   %5.1f HIT dmg %s angle %s kbg %s fkb(wdsk) %s bkb %s size %s off %s %s %s hitlag %s sdi %s flags %08X'
                  % (f, vals[1], int(vals[2]) & 0xffff, p[3][1] & 0xffff, (p[3][1] >> 16) & 0xffff, p[4][1] & 0xffff,
                     vals[5], vals[6], vals[7], vals[8], vals[10], vals[11], p[12][1] & 0xffffffff))
        else:
            print('   %5.1f %08X %s' % (f, code, vals))


# v0.4 : toutes les hitbox hors des listes du personnage, une par une, avec les événements qui les entourent
print()
print('##### HITBOX HORS PERSONNAGE (adresse, contexte)')
mine = set()
for st in known:
    o = base + st
    for _ in range(400):
        c = struct.unpack_from('>Q', m, o)[0]
        if c == 0:
            break
        mine.add(o)
        o += 8
for o in range(base, size - 8, 4):
    code = struct.unpack_from('>I', m, o)[0]
    if code not in CODES or o in mine:
        continue
    ns, eid, npar, _ = struct.unpack_from('>BBBB', m, o)
    po = struct.unpack_from('>I', m, o + 4)[0]
    if po == 0 or base + po + 8 * npar > size:
        continue
    params = [struct.unpack_from('>Ii', m, base + po + 8 * i) for i in range(npar)]
    vals = [round(gen.pv(t, v), 3) for t, v in params]
    ctx = []
    for k in range(-6, 4):
        q = o + 8 * k
        if base <= q < size - 8:
            ctx.append('%08X' % struct.unpack_from('>I', m, q)[0])
    print('0x%X dmg %s angle %s kbg %s wdsk %s bkb %s size %s off %s %s %s hitlag %s sdi %s flags %08X | %s'
          % (o - base, vals[1], int(vals[2]) & 0xffff, params[3][1] & 0xffff, (params[3][1] >> 16) & 0xffff,
             params[4][1] & 0xffff, vals[5], vals[6], vals[7], vals[8], vals[10], vals[11], params[12][1] & 0xffffffff,
             ' '.join(ctx)))
