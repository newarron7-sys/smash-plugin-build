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


def adresses(w, h, bpp, bh):
    """adresse (octets) de chaque élément (h, w) dans une surface Tegra X1 en blocs linéaires (GOB 64 o x 8 lignes)"""
    import numpy as np
    wg = (w * bpp + 63) // 64
    x = (np.arange(w) * bpp)[None, :]
    y = np.arange(h)[:, None]
    gob = (y // (8 * bh)) * 512 * bh * wg + (x // 64) * 512 * bh + ((y % (8 * bh)) // 8) * 512
    xx, yy = x % 64, y % 8
    return gob + (xx // 32) * 256 + (yy // 2) * 64 + ((xx % 32) // 16) * 32 + (yy % 2) * 16 + (xx % 16)


def reswizzle(data, w, h, bpp, bh_src, bh_dst, size):
    import numpy as np
    src = np.frombuffer(data, np.uint8)
    out = np.zeros(size, np.uint8)
    As, Ad = adresses(w, h, bpp, bh_src).ravel(), adresses(w, h, bpp, bh_dst).ravel()
    k = np.arange(bpp)
    out[(Ad[:, None] + k).ravel()] = src[(As[:, None] + k).ravel()]
    return out.tobytes()


r, n = open(REF, 'rb').read(), open(NEW, 'rb').read()
ir, nr = info(r)
inn, _ = info(n)
print('réf', ir, nr)
print('neuf', inn)
for k in ('mips', 'fmt', 'w', 'h', 'size'):
    assert ir[k] == inn[k], (k, ir[k], inn[k])
a, b = ir['brtd'] + 0x10, inn['brtd'] + 0x10
data = n[b:b + inn['size']]
if ir['bh'] != inn['bh']:
    # hauteur de bloc différente : on remet les blocs dans la disposition de la référence (blocs de 4x4 px, 16 octets)
    assert ir['mips'] == 1, 'reswizzle seulement sans mipmaps'
    BPP = 16 if 0x1A <= (int(ir['fmt'], 16) >> 8) & 0xFF <= 0x20 else 4   # BC1..BC7 : blocs de 16 octets (BC1 = 8, non géré)
    bw, bhp = ((ir['w'] + 3) // 4, (ir['h'] + 3) // 4) if BPP == 16 else (ir['w'], ir['h'])
    data = reswizzle(data, bw, bhp, BPP, 1 << inn['bh'], 1 << ir['bh'], ir['size'])
    print('reswizzle : hauteur de bloc', 1 << inn['bh'], '->', 1 << ir['bh'])
out = bytearray(r)
out[a:a + ir['size']] = data
open(OUT, 'wb').write(bytes(out))
print('greffé ->', OUT, len(out), 'octets, nom interne', nr)
