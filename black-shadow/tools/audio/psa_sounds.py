"""Liste les sons joués par Black Shadow dans son mod (scripts SFX de chaque sous-action + table de sons « Misc »).

usage : python3 psa_sounds.py <dossier du mod>   ->  lignes SND <sous-action> <image> <code> <id>
"""
import os
import struct
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))
from psa import load, header, events, cstr  # noqa: E402

MOD = sys.argv[1]
# v2 : un fichier .pac donné directement (REX : FitRock.pac) ou le dossier du mod de Black Shadow
m = load(MOD if MOD.lower().endswith('.pac') else os.path.join(MOD, 'FitBlackShadow.pac'))
base, h, ext = header(m)
n = (h['subGFX'] - h['subMain']) // 4
seen = {}


def walk(off, depth=0, f0=0.0):
    out = []
    f = f0
    if depth > 4 or not off:
        return out
    for code, params in events(m, base, off):
        if code == 0x00010100:
            f += params[0][1] / 60000.0 if params[0][0] == 1 else params[0][1]
        elif code == 0x00020100:
            f = params[0][1] / 60000.0 if params[0][0] == 1 else params[0][1]
        ns = code >> 24
        if ns in (0x0A, 0x0C) and params:
            out.append((f, code, [p[1] for p in params]))
        elif ns == 0x0A or code in (0x0C0B0000, 0x0C190000, 0x0C1D0000, 0x0C050000):
            out.append((f, code, []))
        if code in (0x00070100, 0x00090100) and params:
            out += walk(params[0][1], depth + 1, f)
    return out


for i in range(n):
    it, fl, _, no = struct.unpack_from('>BBHI', m, base + h['subactionFlags'] + 8 * i)
    name = cstr(m, base, no) if no else 'sub%d' % i
    so = struct.unpack_from('>I', m, base + h['subSFX'] + 4 * i)[0]
    for f, code, p in walk(so):
        print('SND %-20s %6.1f %08X %s' % (name, f, code, ' '.join('0x%X' % x for x in p)))
