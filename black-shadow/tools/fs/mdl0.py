"""Lecture d'un modèle Brawl (MDL0, version 9) : os, tableaux de sommets / normales / UV, objets (listes d'affichage
GX) et matériaux -> textures. Sert à convertir la Black Bull (Final Smash de Black Shadow).

Formats d'après BrawlLib.
"""
import struct
import sys
import os

import numpy as np

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))
from brres import dict_entries  # noqa: E402

COMP_SIZE = {0: 1, 1: 1, 2: 2, 3: 2, 4: 4}
COMP_FMT = {0: 'B', 1: 'b', 2: 'H', 3: 'h', 4: 'f'}


def read_array(d, o, kind):
    """tableau de sommets (kind 'pos'), normales ('nrm') ou UV ('uv') -> numpy (n, 3 ou 2)"""
    data_off = struct.unpack_from('>i', d, o + 0x08)[0]
    if kind == 'pos':
        is3, typ = struct.unpack_from('>ii', d, o + 0x14)
        div, stride, n = struct.unpack_from('>BBH', d, o + 0x1C)
        ncomp = 3 if is3 else 2
    elif kind == 'nrm':
        ntype, typ = struct.unpack_from('>ii', d, o + 0x14)
        div, stride, n = struct.unpack_from('>BBH', d, o + 0x1C)
        ncomp = 3 if ntype == 0 else 9
    else:
        is2, typ = struct.unpack_from('>ii', d, o + 0x14)
        div, stride, n = struct.unpack_from('>BBH', d, o + 0x1C)
        ncomp = 2 if is2 else 1
    fmt = '>' + COMP_FMT[typ] * ncomp
    out = np.zeros((n, ncomp))
    for i in range(n):
        out[i] = struct.unpack_from(fmt, d, o + data_off + i * stride)
    if typ != 4:
        out /= float(1 << div)
    return out, {'type': typ, 'div': div, 'stride': stride, 'n': n, 'ncomp': ncomp}


def attr_types(lo, hi):
    """types d'attribut de la description de sommet (0 aucun, 1 direct, 2 index 8 bits, 3 index 16 bits)"""
    a = {'pmi': lo & 1}
    for t in range(8):
        a['tmi%d' % t] = (lo >> (1 + t)) & 1
    a['pos'] = (lo >> 9) & 3
    a['nrm'] = (lo >> 11) & 3
    a['clr0'] = (lo >> 13) & 3
    a['clr1'] = (lo >> 15) & 3
    for t in range(8):
        a['uv%d' % t] = (hi >> (2 * t)) & 3
    return a


ORDER = ['pmi'] + ['tmi%d' % t for t in range(8)] + ['pos', 'nrm', 'clr0', 'clr1'] + ['uv%d' % t for t in range(8)]


def read_object(d, o):
    lo, hi = struct.unpack_from('>II', d, o + 0x0C)
    node = struct.unpack_from('>i', d, o + 0x08)[0]
    dl_size, _, dl_off = struct.unpack_from('>iii', d, o + 0x24)
    nv, nf = struct.unpack_from('>ii', d, o + 0x40)
    ids = struct.unpack_from('>12h', d, o + 0x48)
    at = attr_types(lo, hi)
    p = o + 0x24 + dl_off
    end = p + dl_size
    tris = []  # liste de sommets [(pos, nrm, uv0, pmi), ...] par triangle
    mtx_slots = {}
    while p < end:
        cmd = d[p]
        p += 1
        if cmd == 0x00:
            continue
        if cmd == 0x08:
            p += 5
            continue
        if cmd == 0x10:
            cnt = struct.unpack_from('>H', d, p)[0] + 1
            p += 4 + 4 * cnt
            continue
        if cmd in (0x20, 0x28, 0x30, 0x38):
            val, addr = struct.unpack_from('>HH', d, p)
            if cmd == 0x20:
                mtx_slots[(addr & 0xFFF) // 12] = val
            p += 4
            continue
        prim = cmd & 0xF8
        if prim not in (0x80, 0x90, 0x98, 0xA0, 0xA8, 0xB0, 0xB8):
            break
        count = struct.unpack_from('>H', d, p)[0]
        p += 2
        verts = []
        for _ in range(count):
            v = {}
            for key in ORDER:
                t = at[key]
                if t == 0:
                    continue
                if key == 'pmi' or key.startswith('tmi'):
                    v[key] = d[p]
                    p += 1
                elif t == 2:
                    v[key] = d[p]
                    p += 1
                elif t == 3:
                    v[key] = struct.unpack_from('>H', d, p)[0]
                    p += 2
                else:
                    raise ValueError('attribut direct non géré : %s' % key)
            if 'pmi' in v:
                v['bone'] = mtx_slots.get(v['pmi'] // 3, node)
            verts.append(v)
        if prim == 0x90:
            tris += [verts[i:i + 3] for i in range(0, count - 2, 3)]
        elif prim == 0x98:
            for i in range(count - 2):
                tris.append([verts[i], verts[i + 1], verts[i + 2]] if i % 2 == 0 else [verts[i + 1], verts[i], verts[i + 2]])
        elif prim == 0xA0:
            for i in range(1, count - 1):
                tris.append([verts[0], verts[i], verts[i + 1]])
        elif prim == 0x80:
            for i in range(0, count - 3, 4):
                q = verts[i:i + 4]
                tris += [[q[0], q[1], q[2]], [q[0], q[2], q[3]]]
    return {'node': node, 'ids': ids, 'attrs': {k: v for k, v in at.items() if v}, 'tris': tris, 'nv': nv, 'nf': nf}


def read_model(dd, o):
    """modèle MDL0 dans un BRRES `dd` à l'offset `o`"""
    ver = struct.unpack_from('>I', dd, o + 8)[0]
    offs = struct.unpack_from('>11i', dd, o + 0x10)
    secs = {}
    names = ['defs', 'bones', 'pos', 'nrm', 'clr', 'uv', 'mats', 'shaders', 'objs', 'tex', 'pal']
    for k, so in zip(names, offs):
        secs[k] = dict_entries(dd, o + so) if so else []
    bones = []
    for n, bo in secs['bones']:
        m = struct.unpack_from('>12f', dd, bo + 0x70)
        w = np.eye(4)
        w[:3, :] = np.array(m).reshape(3, 4)
        sc = struct.unpack_from('>3f', dd, bo + 0x20)
        nid = struct.unpack_from('>i', dd, bo + 0x10)[0]
        bones.append((n, nid, w, sc))
    arrays = {'pos': [read_array(dd, x, 'pos') for _, x in secs['pos']],
              'nrm': [read_array(dd, x, 'nrm') for _, x in secs['nrm']],
              'uv': [read_array(dd, x, 'uv') for _, x in secs['uv']]}
    objs = [(n, read_object(dd, x)) for n, x in secs['objs']]
    # textures de chaque matériau : sous-sections "texture references" (nom du TEX0)
    mats = {}
    for n, mo in secs['mats']:
        try:
            # en-tête de matériau : nombre de textures à 0x28, références de textures à 0x2C
            ntex = struct.unpack_from('>i', dd, mo + 0x28)[0]
            toff = struct.unpack_from('>i', dd, mo + 0x2C)[0]
            texs = []
            for i in range(min(ntex, 8)):
                e = mo + toff + 0x34 * i
                so = struct.unpack_from('>i', dd, e)[0]
                texs.append(dd[e + so:dd.index(b'\0', e + so)].decode('latin1') if so else '')
            mats[n] = texs
        except Exception as ex:  # noqa: BLE001
            mats[n] = ['?%s' % ex]
    # objet -> matériau : section définitions "DrawOpa"/"DrawXlu"
    obj_mat = {}
    for n, defo in secs['defs']:
        if not n.startswith('Draw'):
            continue
        p = defo
        while True:
            cmd = dd[p]
            if cmd == 0x04:  # DrawPolygon : matériau, objet, os, priorité
                mi, oi, bi = struct.unpack_from('>HHH', dd, p + 1)
                obj_mat[oi] = mi
                p += 8
            else:
                break
    return {'version': ver, 'bones': bones, 'arrays': arrays, 'objs': objs, 'mats': mats, 'obj_mat': obj_mat,
            'mat_names': [n for n, _ in secs['mats']]}
