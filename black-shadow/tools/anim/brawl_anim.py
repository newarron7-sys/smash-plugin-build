"""Lecture des animations Brawl (CHR0) et du squelette du modèle (MDL0) de Black Shadow.

Formats d'après BrawlLib (BrawlBox / BrawlCrate) :
- CHR0 : une entrée par os, avec un code de 32 bits qui dit quelles composantes existent (échelle, rotation,
  translation), lesquelles sont fixes (un flottant) et le format des images-clés des autres.
- formats d'images-clés : I4 / I6 / I12 (clés interpolées en Hermite), L1 / L2 / L4 (une valeur par image).
- matrice locale = T @ Rz @ Ry @ Rx @ S (rotations en degrés).
"""
import math
import struct
import sys

import numpy as np

sys.path.insert(0, __file__.rsplit('/', 2)[0])
from arc import arc_entries  # noqa: E402
from brres import brres_files, dict_entries  # noqa: E402


# ------------------------------------------------------------------ matrices

def srt_matrix(s, r, t):
    cx, sx = math.cos(math.radians(r[0])), math.sin(math.radians(r[0]))
    cy, sy = math.cos(math.radians(r[1])), math.sin(math.radians(r[1]))
    cz, sz = math.cos(math.radians(r[2])), math.sin(math.radians(r[2]))
    rx = np.array([[1, 0, 0], [0, cx, -sx], [0, sx, cx]])
    ry = np.array([[cy, 0, sy], [0, 1, 0], [-sy, 0, cy]])
    rz = np.array([[cz, -sz, 0], [sz, cz, 0], [0, 0, 1]])
    m = np.eye(4)
    m[:3, :3] = rz @ ry @ rx @ np.diag(s)
    m[:3, 3] = t
    return m


# ------------------------------------------------------------------ squelette (MDL0)

class Bone:
    def __init__(self, name, index, parent, s, r, t, world):
        self.name, self.index, self.parent = name, index, parent
        self.s, self.r, self.t = s, r, t
        self.world = world  # matrice monde de repos (4x4, colonne)


def read_bones(model_pac):
    """os du modèle Brawl : nom, parent, SRT de repos et matrice monde de repos (lue dans le fichier)."""
    d = open(model_pac, 'rb').read()
    dd = [x for x in arc_entries(d) if x[0] == 2][0][4]
    n, o = brres_files(dd)['3DModels(NW4R)'][0]
    offs = struct.unpack_from('>11i', dd, o + 0x10)
    bones = {}
    by_off = {}
    for name, bo in dict_entries(dd, o + offs[1]):
        idx = struct.unpack_from('>i', dd, bo + 0xC)[0]
        s = struct.unpack_from('>3f', dd, bo + 0x20)
        r = struct.unpack_from('>3f', dd, bo + 0x2C)
        t = struct.unpack_from('>3f', dd, bo + 0x38)
        par = struct.unpack_from('>i', dd, bo + 0x5C)[0]
        m = struct.unpack_from('>12f', dd, bo + 0x70)
        w = np.eye(4)
        w[:3, :] = np.array(m).reshape(3, 4)
        by_off[bo] = name
        bones[name] = Bone(name, idx, (bo + par) if par else None, s, r, t, w)
    for b in bones.values():
        b.parent = by_off.get(b.parent) if b.parent is not None else None
    return bones


# ------------------------------------------------------------------ images-clés

def hermite(keys, f):
    """keys = [(image, valeur, tangente)] ; interpolation de BrawlLib."""
    if f <= keys[0][0]:
        return keys[0][1]
    if f >= keys[-1][0]:
        return keys[-1][1]
    for i in range(len(keys) - 1):
        f0, v0, t0 = keys[i]
        f1, v1, t1 = keys[i + 1]
        if f0 <= f <= f1:
            span = f1 - f0
            off = f - f0
            if off == 0 or span == 0:
                return v0
            time = off / span
            inv = time - 1.0
            return v0 + off * inv * (inv * t0 + time * t1) + time * time * (3.0 - 2.0 * time) * (v1 - v0)
    return keys[-1][1]


class Track:
    def __init__(self, kind, data):
        self.kind, self.data = kind, data

    def at(self, f):
        if self.kind == 'fixed':
            return self.data
        if self.kind == 'keys':
            return hermite(self.data, f)
        vals = self.data  # une valeur par image
        i = int(math.floor(f))
        if i >= len(vals) - 1:
            return vals[-1]
        a = f - i
        return vals[i] * (1 - a) + vals[i + 1] * a


def read_keys(d, o, fmt, nframes):
    # codes de BrawlLib : 1 = I4, 2 = I6, 3 = I12, 4 = L1, 5 = L2, 6 = L4 (0 = aucun)
    fmt -= 1
    if fmt == 0:  # I4
        n, _, _, step, base = struct.unpack_from('>HHfff', d, o)
        keys = []
        for i in range(n):
            v = struct.unpack_from('>I', d, o + 16 + 4 * i)[0]
            fr = v >> 24
            st = (v >> 12) & 0xFFF
            tan = v & 0xFFF
            if tan & 0x800:
                tan -= 0x1000
            keys.append((float(fr), base + step * st, tan / 32.0))
        return Track('keys', keys)
    if fmt == 1:  # I6
        n, _, _, step, base = struct.unpack_from('>HHfff', d, o)
        keys = []
        for i in range(n):
            fr, st, tan = struct.unpack_from('>HHh', d, o + 16 + 6 * i)
            keys.append((fr / 32.0, base + step * st, tan / 256.0))
        return Track('keys', keys)
    if fmt == 2:  # I12
        n, _, _ = struct.unpack_from('>HHf', d, o)
        keys = [struct.unpack_from('>fff', d, o + 8 + 12 * i) for i in range(n)]
        return Track('keys', [tuple(float(x) for x in k) for k in keys])
    if fmt == 3:  # L1
        step, base = struct.unpack_from('>ff', d, o)
        return Track('frames', [base + step * d[o + 8 + i] for i in range(nframes)])
    if fmt == 4:  # L2
        step, base = struct.unpack_from('>ff', d, o)
        return Track('frames', [base + step * struct.unpack_from('>H', d, o + 8 + 2 * i)[0] for i in range(nframes)])
    if fmt == 5:  # L4
        return Track('frames', [struct.unpack_from('>f', d, o + 4 * i)[0] for i in range(nframes)])
    raise ValueError('format d\'images-clés inconnu %d' % fmt)


class BoneAnim:
    """animation d'un os : 3 composantes (échelle, rotation, translation) de 3 pistes chacune, ou None"""

    def __init__(self, code, scale, rot, trans):
        self.code, self.scale, self.rot, self.trans = code, scale, rot, trans

    def srt(self, f, bone):
        c = self.code
        if c & (1 << 1):  # identité
            return (1.0, 1.0, 1.0), (0.0, 0.0, 0.0), (0.0, 0.0, 0.0)
        s = tuple(t.at(f) for t in self.scale) if self.scale else (bone.s if c & (1 << 7) else (1.0, 1.0, 1.0))
        r = tuple(t.at(f) for t in self.rot) if self.rot else (bone.r if c & (1 << 8) else (0.0, 0.0, 0.0))
        tr = tuple(t.at(f) for t in self.trans) if self.trans else (bone.t if c & (1 << 9) else (0.0, 0.0, 0.0))
        return s, r, tr


def read_chr0(dd, o):
    tag, size, ver = struct.unpack_from('>4sII', dd, o)
    assert tag == b'CHR0', tag
    data_off = struct.unpack_from('>i', dd, o + 0x10)[0]
    if ver == 5:
        nf, ne, loop = struct.unpack_from('>HHI', dd, o + 0x20)
    else:
        nf, ne, loop = struct.unpack_from('>HHI', dd, o + 0x1C)
    anims = {}
    stats = {}
    for name, eo in dict_entries(dd, o + data_off):
        code = struct.unpack_from('>I', dd, eo + 4)[0]
        p = eo + 8

        def comp(has_bit, iso_bit, fixed_bits, fmt):
            nonlocal p
            if not code & (1 << has_bit):
                return None
            axes = 1 if code & (1 << iso_bit) else 3
            tracks = []
            for a in range(axes):
                if code & (1 << fixed_bits[a]):
                    tracks.append(Track('fixed', struct.unpack_from('>f', dd, p)[0]))
                else:
                    ko = struct.unpack_from('>i', dd, p)[0]
                    tracks.append(read_keys(dd, eo + ko, fmt, nf))
                    stats[fmt] = stats.get(fmt, 0) + 1
                p += 4
            if axes == 1:
                tracks = tracks * 3
                stats['iso'] = stats.get('iso', 0) + 1
            return tracks

        sc = comp(22, 4, (13, 14, 15), (code >> 25) & 3)
        rt = comp(23, 5, (16, 17, 18), (code >> 27) & 7)
        tr = comp(24, 6, (19, 20, 21), (code >> 30) & 3)
        anims[name] = BoneAnim(code, sc, rt, tr)
    return {'frames': nf, 'loop': bool(loop), 'bones': anims, 'stats': stats}


def read_anims(motion_pac):
    """toutes les animations CHR0 du fichier MotionEtc : nom -> données"""
    d = open(motion_pac, 'rb').read()
    res = {}
    for typ, idx, size, red, data in arc_entries(d):
        if data[:4] != b'ARC\0':
            continue
        for t2, i2, s2, r2, dd in arc_entries(data):
            if dd[:4] != b'bres':
                continue
            for folder, items in brres_files(dd).items():
                if 'Chr' in folder:
                    for n, o in items:
                        if n not in res:
                            res[n] = read_chr0(dd, o)
    return res


# ------------------------------------------------------------------ pose

def world_pose(bones, anim, f):
    """matrices monde de chaque os à l'image f (anim None = pose de repos recalculée depuis les SRT)"""
    out = {}

    def w(name):
        if name in out:
            return out[name]
        b = bones[name]
        ba = anim['bones'].get(name) if anim else None
        if ba is not None:
            s, r, t = ba.srt(f, b)
        else:
            s, r, t = b.s, b.r, b.t
        loc = srt_matrix(s, r, t)
        m = (w(b.parent) @ loc) if b.parent else loc
        out[name] = m
        return m

    for n in bones:
        w(n)
    return out
