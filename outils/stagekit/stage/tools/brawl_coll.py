"""Collisions d'un stage de Brawl (STGxxx.pac) : cherche les données de collision (en-tête : nb de points, décalage,
nb de plans, décalage, nb d'objets, décalage ; points = 2 flottants ; plans de 0x10 octets : point1, point2, liens,
-1, type, drapeaux) et les affiche. usage : python3 brawl_coll.py <.pac>"""
import os, struct, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import brawl_stage  # noqa: E402
from arc import arc_entries, TYPES  # noqa: E402


def blobs(d, where, out):
    d = brawl_stage._unpack(d)
    if d[:4] != b'ARC\0':
        return
    for typ, idx, size, red, data in arc_entries(d):
        data = brawl_stage._unpack(data)
        name = '%s/%s%d' % (where, TYPES.get(typ, typ), idx)
        if data[:4] == b'ARC\0':
            blobs(data, name, out)
        else:
            out.append((name, data))


def essai(d):
    if len(d) < 0x28:
        return None
    npt, npl, nob, unk, opt, opl, oob = struct.unpack_from('>4h3i', d, 0)
    if not (2 <= npt and 0 < npl and opt == 0x28 and opl == opt + 8 * npt and oob == opl + 16 * npl and oob <= len(d)):
        return None
    pts = [struct.unpack_from('>2f', d, opt + 8 * i) for i in range(npt)]
    pls = []
    for i in range(npl):
        p1, p2, l1, l2, mg, ty, fl = struct.unpack_from('>HHhhiHH', d, opl + 16 * i)
        pls.append((p1, p2, ty, fl))
    obs = []
    for i in range(nob):
        o = oob + 0x6C * i
        if o + 0x6C > len(d):
            break
        obs.append(d[o:o + 0x6C])
    return pts, pls, obs, oob


out = []
blobs(open(sys.argv[1], 'rb').read(), os.path.basename(sys.argv[1]), out)
for name, d in out:
    r = essai(d)
    if not r:
        continue
    pts, pls, obs, oob = r
    print('== COLLISION %s : %d points, %d plans, %d objets' % (name, len(pts), len(pls), len(obs)))
    for o in obs:
        print('  objet', struct.unpack_from('>2h', o, 0), struct.unpack_from('>6f', o, 8), o[0x24:0x44].split(b'\0')[0], o[0x44:0x64].split(b'\0')[0], struct.unpack_from('>2h', o, 0x64))
    for i, (p1, p2, ty, fl) in enumerate(pls):
        a, b = pts[p1], pts[p2]
        genre = ' '.join(n for bit, n in ((1, 'SOL'), (2, 'PLAFOND'), (4, 'MUR_G'), (8, 'MUR_D')) if ty & bit) or hex(ty)
        print('  plan %3d (%7.2f %7.2f) -> (%7.2f %7.2f)  type %04x %-12s drapeaux %04x' % (i, a[0], a[1], b[0], b[1], ty, genre, fl))
