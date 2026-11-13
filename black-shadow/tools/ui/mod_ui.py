"""Extrait les images d'interface contenues dans l'archive du mod Black Shadow (P+) : portraits de sélection,
icônes de stock, portraits de combat… (textures TEX0 des fichiers BRRES, éventuellement dans des ARC ou
compressées en LZ77). Écrit un .png par texture et une planche.

usage : python3 mod_ui.py <sortie>
"""
import io
import json
import os
import struct
import sys
import urllib.request
import zipfile

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))
from brres import brres_files  # noqa: E402

from PIL import Image, ImageDraw  # noqa: E402

OUT = sys.argv[1]
os.makedirs(os.path.join(OUT, 'textures'), exist_ok=True)
UA = {'User-Agent': 'Mozilla/5.0'}


def get(u):
    return urllib.request.urlopen(urllib.request.Request(u, headers=UA), timeout=300).read()


p = json.loads(get('https://gamebanana.com/apiv11/Mod/591515/ProfilePage'))
z = zipfile.ZipFile(io.BytesIO(get(p['_aFiles'][0]['_sDownloadUrl'])))
names = [n for n in z.namelist() if not n.endswith('/') and '__MACOSX' not in n]
open(os.path.join(OUT, 'liste_archive.txt'), 'w').write('\n'.join('%9d  %s' % (z.getinfo(n).file_size, n) for n in names))
print(len(names), 'fichiers dans l\'archive')
for n in names:
    low = n.lower()
    if not low.endswith(('.pac', '.pcs', '.brres', '.png', '.jpg', '.tpl')) or 'fitblackshadow' in low and 'motion' in low:
        print('  ', n)


# ------------------------------------------------------------------ décompression LZ77 (Nintendo 0x10 / 0x11)
def lz77(d):
    t = d[0]
    size = d[1] | d[2] << 8 | d[3] << 16
    o = 4
    if size == 0:
        size = struct.unpack_from('<I', d, 4)[0]
        o = 8
    out = bytearray()
    while len(out) < size and o < len(d):
        flags = d[o]
        o += 1
        for bit in range(8):
            if len(out) >= size or o >= len(d):
                break
            if flags & (0x80 >> bit):
                if t == 0x10:
                    b1, b2 = d[o], d[o + 1]
                    o += 2
                    ln = (b1 >> 4) + 3
                    disp = ((b1 & 0xF) << 8 | b2) + 1
                else:
                    b1 = d[o]
                    ind = b1 >> 4
                    if ind == 0:
                        ln = (((b1 & 0xF) << 4) | (d[o + 1] >> 4)) + 0x11
                        disp = ((d[o + 1] & 0xF) << 8 | d[o + 2]) + 1
                        o += 3
                    elif ind == 1:
                        ln = (((b1 & 0xF) << 12) | (d[o + 1] << 4) | (d[o + 2] >> 4)) + 0x111
                        disp = ((d[o + 2] & 0xF) << 8 | d[o + 3]) + 1
                        o += 4
                    else:
                        ln = ind + 1
                        disp = ((b1 & 0xF) << 8 | d[o + 1]) + 1
                        o += 2
                for _ in range(ln):
                    out.append(out[-disp])
            else:
                out.append(d[o])
                o += 1
    return bytes(out)


# ------------------------------------------------------------------ couleurs GameCube / Wii
def rgb565(v):
    return ((v >> 11 & 31) * 255 // 31, (v >> 5 & 63) * 255 // 63, (v & 31) * 255 // 31, 255)


def rgb5a3(v):
    if v & 0x8000:
        return ((v >> 10 & 31) * 255 // 31, (v >> 5 & 31) * 255 // 31, (v & 31) * 255 // 31, 255)
    return ((v >> 8 & 15) * 17, (v >> 4 & 15) * 17, (v & 15) * 17, (v >> 12 & 7) * 255 // 7)


def ia8(v):
    return (v & 255, v & 255, v & 255, v >> 8)


def decode(d, o, w, h, fmt, pal=None):
    img = Image.new('RGBA', (w, h))
    px = img.load()
    bw, bh = {0: (8, 8), 1: (8, 4), 2: (8, 4), 3: (4, 4), 4: (4, 4), 5: (4, 4), 6: (4, 4), 8: (8, 8), 9: (8, 4),
              10: (4, 4), 14: (8, 8)}[fmt]
    for by in range(0, h, bh):
        for bx in range(0, w, bw):
            if fmt == 14:  # CMPR : 4 blocs DXT1
                for sb in range(4):
                    c0, c1, idx = struct.unpack_from('>HHI', d, o)
                    o += 8
                    a, b = rgb565(c0), rgb565(c1)
                    if c0 > c1:
                        cols = [a, b, tuple((2 * a[i] + b[i]) // 3 for i in range(3)) + (255,),
                                tuple((a[i] + 2 * b[i]) // 3 for i in range(3)) + (255,)]
                    else:
                        cols = [a, b, tuple((a[i] + b[i]) // 2 for i in range(3)) + (255,), (0, 0, 0, 0)]
                    sx, sy = bx + (sb % 2) * 4, by + (sb // 2) * 4
                    for j in range(16):
                        x, y = sx + j % 4, sy + j // 4
                        if x < w and y < h:
                            px[x, y] = cols[(idx >> (30 - 2 * j)) & 3]
                continue
            if fmt == 6:  # RGBA8 : AR puis GB
                ar = d[o:o + 32]
                gb = d[o + 32:o + 64]
                o += 64
                for j in range(16):
                    x, y = bx + j % 4, by + j // 4
                    if x < w and y < h:
                        px[x, y] = (ar[2 * j + 1], gb[2 * j], gb[2 * j + 1], ar[2 * j])
                continue
            for yy in range(bh):
                for xx in range(bw):
                    x, y = bx + xx, by + yy
                    if fmt in (0, 8):
                        v = d[o + (yy * bw + xx) // 2]
                        v = v >> 4 if xx % 2 == 0 else v & 15
                        c = (v * 17,) * 3 + (255,) if fmt == 0 else (pal[v] if pal and v < len(pal) else (0, 0, 0, 0))
                    elif fmt in (1, 2, 9):
                        v = d[o + yy * bw + xx]
                        if fmt == 1:
                            c = (v, v, v, 255)
                        elif fmt == 2:
                            c = ((v & 15) * 17,) * 3 + ((v >> 4) * 17,)
                        else:
                            c = pal[v] if pal and v < len(pal) else (0, 0, 0, 0)
                    else:
                        v = struct.unpack_from('>H', d, o + 2 * (yy * bw + xx))[0]
                        if fmt == 3:
                            c = ia8(v)
                        elif fmt == 4:
                            c = rgb565(v)
                        elif fmt == 5:
                            c = rgb5a3(v)
                        else:
                            c = pal[v & 0x3FFF] if pal else (0, 0, 0, 0)
                    if x < w and y < h:
                        px[x, y] = c
            o += {0: 32, 1: 32, 2: 32, 3: 32, 4: 32, 5: 32, 8: 32, 9: 32, 10: 32}[fmt]
    return img


def palette(d, o):
    data_off = struct.unpack_from('>i', d, o + 0x10)[0]
    fmt = struct.unpack_from('>I', d, o + 0x18)[0]
    n = struct.unpack_from('>H', d, o + 0x1C)[0]
    f = {0: ia8, 1: rgb565, 2: rgb5a3}[fmt]
    return [f(struct.unpack_from('>H', d, o + data_off + 2 * i)[0]) for i in range(n)]


found = []


def scan_brres(dd, where):
    files = brres_files(dd)
    pals = {n: o for n, o in files.get('Palettes(NW4R)', [])}
    for n, o in files.get('Textures(NW4R)', []):
        try:
            data_off = struct.unpack_from('>i', dd, o + 0x10)[0]
            w, h = struct.unpack_from('>HH', dd, o + 0x1C)
            fmt = struct.unpack_from('>I', dd, o + 0x20)[0]
            pal = palette(dd, pals[n]) if fmt in (8, 9, 10) and n in pals else None
            img = decode(dd, o + data_off, w, h, fmt, pal)
            fn = ('%s__%s.png' % (where.replace('/', '_'), n)).replace(' ', '_')
            img.save(os.path.join(OUT, 'textures', fn))
            found.append((fn, w, h, fmt))
        except Exception as e:  # noqa: BLE001
            print('texture illisible', where, n, e)


def scan(d, where, depth=0):
    if depth > 4 or len(d) < 8:
        return
    if d[0] in (0x10, 0x11) and d[4:8] != b'bres':
        try:
            d = lz77(d)
        except Exception:  # noqa: BLE001
            return
    if d[:4] == b'bres':
        scan_brres(d, where)
    elif d[:4] == b'ARC\0':
        from arc import arc_entries
        for i, (typ, idx, size, red, data) in enumerate(arc_entries(d)):
            scan(data, '%s#%d' % (where, i), depth + 1)


for n in names:
    low = n.lower()
    if 'motionetc' in low:
        continue
    if low.endswith(('.pac', '.pcs', '.brres')):
        scan(z.read(n), n)
    elif low.endswith(('.png', '.jpg', '.jpeg')):
        fn = n.replace('/', '_').replace(' ', '_')
        open(os.path.join(OUT, 'textures', fn), 'wb').write(z.read(n))
        im = Image.open(os.path.join(OUT, 'textures', fn))
        found.append((fn, im.size[0], im.size[1], 'img'))

# on écarte les textures du modèle et des effets : on garde celles qui ressemblent à des images d'interface
ui = [f for f in found if not ('FitBlackShadow0' in f[0] or 'FitBlackShadow.pac' in f[0] or 'Final' in f[0])]
print(len(found), 'textures,', len(ui), 'candidates')
for f in ui:
    print('  ', f)
json.dump(found, open(os.path.join(OUT, 'textures.json'), 'w'), indent=1)

# planche
cells = ui[:60]
cw = 220
sheet = Image.new('RGB', (cw * 5 + 10, 10 + ((len(cells) + 4) // 5) * (cw + 30)), (30, 28, 40))
dr = ImageDraw.Draw(sheet)
for i, (fn, w, h, fmt) in enumerate(cells):
    im = Image.open(os.path.join(OUT, 'textures', fn)).convert('RGBA')
    im.thumbnail((cw - 10, cw - 10))
    bg = Image.new('RGBA', im.size, (70, 66, 90, 255))
    bg.alpha_composite(im)
    x, y = 5 + (i % 5) * cw, 5 + (i // 5) * (cw + 30)
    sheet.paste(bg.convert('RGB'), (x, y))
    dr.text((x, y + cw - 5), ('%dx%d %s' % (w, h, fn))[:34], fill=(230, 220, 200))
sheet.save(os.path.join(OUT, 'planche_ui_du_mod.png'))
