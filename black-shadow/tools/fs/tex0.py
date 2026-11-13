"""Décodage des textures TEX0 (et palettes PLT0) de Brawl en images PIL. Repris de ui/mod_ui.py."""
import struct

from PIL import Image


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




def read_tex0(dd, o, pal=None):
    data_off = struct.unpack_from('>i', dd, o + 0x10)[0]
    w, h = struct.unpack_from('>HH', dd, o + 0x1C)
    fmt = struct.unpack_from('>I', dd, o + 0x20)[0]
    return decode(dd, o + data_off, w, h, fmt, pal)
