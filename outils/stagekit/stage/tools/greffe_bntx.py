"""Greffe les pixels d'un .bntx neuf (ultimate_tex_cli) dans un .bntx de référence qui marche en jeu : en-têtes et
nom interne de la référence gardés (ex. stage_2_YoshiCartboard), seules les données d'image (BRTD + 0x10) remplacées.
Les deux doivent avoir mêmes dimensions, format, mipmaps et taille d'image.
usage : python3 greffe_bntx.py <réf .bntx> <neuf .bntx> <sortie .bntx>"""
import struct, sys

REF, NEW, OUT = sys.argv[1:4]


def info(d):
    i = d.find(b'BRTI')
    j = d.find(b'BRTD')
    assert i >= 0 and j >= 0
    mips = struct.unpack_from('<H', d, i + 0x16)[0]
    fmt = struct.unpack_from('<I', d, i + 0x1C)[0]
    w, h = struct.unpack_from('<II', d, i + 0x24)
    bh = struct.unpack_from('<I', d, i + 0x34)[0]
    size = struct.unpack_from('<I', d, i + 0x50)[0]
    nom_o = d.find(b'stage_')
    return dict(mips=mips, fmt='%08X' % fmt, w=w, h=h, bh=bh, size=size, brtd=j), d[nom_o:nom_o + 40].split(b'\0')[0]


r, n = open(REF, 'rb').read(), open(NEW, 'rb').read()
ir, nr = info(r)
inn, _ = info(n)
print('réf', ir, nr)
print('neuf', inn)
for k in ('mips', 'fmt', 'w', 'h', 'bh', 'size'):
    assert ir[k] == inn[k], (k, ir[k], inn[k])
a, b = ir['brtd'] + 0x10, inn['brtd'] + 0x10
out = bytearray(r)
out[a:a + ir['size']] = n[b:b + inn['size']]
open(OUT, 'wb').write(bytes(out))
print('greffé ->', OUT, len(out), 'octets, nom interne', nr)
