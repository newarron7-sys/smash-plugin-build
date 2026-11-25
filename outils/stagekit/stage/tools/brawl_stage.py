"""Lecture d'un stage de Brawl (STGxxx.pac) : modèles du décor (triangles placés dans le monde, UV, matériau ->
texture) et textures. Partagé par l'aperçu et la conversion vers Ultimate. Usage perso.

- les .pac de stage sont des ARC dont les entrées peuvent être compressées (LZ77 0x10 / 0x11) et imbriquées ;
- la matrice « monde » rangée dans chaque os ne contient PAS celle de la racine (TopN a souvent une échelle 0,003 et
  un quart de tour : modèle exporté en Z vers le haut) : on recalcule le monde en enchaînant les SRT des parents ;
- références de textures des matériaux : nombre à 0x28 ou 0x2C selon les fichiers -> on garde celle qui donne des
  noms de textures existants.
"""
import os, struct, sys
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
for p in ('../../black-shadow/tools', '../../black-shadow/tools/fs', '../../black-shadow/tools/anim', '../../zero/tools'):
    sys.path.insert(0, os.path.join(HERE, p))
from arc import arc_entries, TYPES  # noqa: E402
from brres import brres_files, dict_entries  # noqa: E402
import mdl0  # noqa: E402
import tex0  # noqa: E402
from brawl_anim import srt_matrix, read_chr0  # noqa: E402
from lz import decompress  # noqa: E402


def _unpack(d):
    if d[:1] in (b'\x10', b'\x11') and len(d) > 16:
        try:
            return decompress(d)
        except Exception:  # noqa: BLE001
            return d
    return d


def walk(d, where, out, verbose=False):
    d = _unpack(d)
    if d[:4] != b'ARC\0':
        return
    for typ, idx, size, red, data in arc_entries(d):
        data = _unpack(data)
        name = '%s/%s%d' % (where, TYPES.get(typ, typ), idx)
        if verbose:
            print('  %-50s %6d Ko  %s' % (name, len(data) // 1024, data[:4]))
        if data[:4] == b'bres':
            out.append((name, data))
        elif data[:4] == b'ARC\0':
            walk(data, name, out, verbose)


def chr0_f0(dd, bf):
    """os -> (échelle, rotation, translation) à la première image des CHR0 du même .brres (placement des objets que
    le stage pose par animation, ex. portes de BSL Station agrandies x3 au fond) ; activé par BRAWL_CHR0=1"""
    out = {}
    if os.environ.get('BRAWL_CHR0') != '1':
        return out
    for an, ao in bf.get('AnmChr(NW4R)', []):
        try:
            c = read_chr0(dd, ao)
        except Exception as e:  # noqa: BLE001
            print('CHR0 illisible', an, e)
            continue
        for bn, ba in c['bones'].items():
            out[bn] = ba
    return out


def bone_worlds(dd, mo, anim=None):
    """nom -> (n° de matrice, monde 4x4) en enchaînant les SRT (parent = décalage relatif à 0x5C)"""
    offs = struct.unpack_from('>11i', dd, mo + 0x10)
    if not offs[1]:
        return {}
    ents = dict_entries(dd, mo + offs[1])
    at = {bo: n for n, bo in ents}
    info = {}
    for n, bo in ents:
        s = struct.unpack_from('>3f', dd, bo + 0x20)
        r = struct.unpack_from('>3f', dd, bo + 0x2C)
        t = struct.unpack_from('>3f', dd, bo + 0x38)
        po = struct.unpack_from('>i', dd, bo + 0x5C)[0]
        nid = struct.unpack_from('>i', dd, bo + 0x10)[0]
        if anim and n in anim:
            class _B:
                pass
            b_ = _B()
            b_.s, b_.r, b_.t = s, r, t
            s2, r2, t2 = anim[n].srt(0, b_)
            if (s2, r2, t2) != (s, r, t):
                print('CHR0 image 0 :', n, 'S', tuple(round(x, 2) for x in s2), 'T', tuple(round(x, 1) for x in t2))
            s, r, t = s2, r2, t2
        info[n] = (nid, srt_matrix(s, r, t), at.get(bo + po) if po else None)
    W = {}

    def world(n):
        if n not in W:
            nid, m, par = info[n]
            W[n] = (world(par) @ m) if par and par in info and par != n else m
        return W[n]
    return {n: (info[n][0], world(n)) for n in info}


def mat_textures(dd, mo, texnames):
    """matériau -> [noms de textures] (disposition 0x28 ou 0x2C, choisie d'après les textures existantes)"""
    offs = struct.unpack_from('>11i', dd, mo + 0x10)
    out = {}
    if not offs[6]:
        return out
    for n, m0 in dict_entries(dd, mo + offs[6]):
        best = []
        for off in (0x28, 0x2C):
            nt, to = struct.unpack_from('>ii', dd, m0 + off)
            if not (0 < nt <= 8 and 0 < to < 0x10000):
                continue
            names = []
            for i in range(nt):
                e = m0 + to + 0x34 * i
                so = struct.unpack_from('>i', dd, e)[0]
                try:
                    names.append(dd[e + so:dd.index(b'\0', e + so)].decode('latin1') if 0 < e + so < len(dd) else '')
                except Exception:  # noqa: BLE001
                    names.append('')
            if any(x in texnames for x in names):
                best = names
                break
        out[n] = best
    return out


def mat_additif(dd, mo):
    """matériau -> True si le mélange GX est additif (registre BP 0x41 : source SRCALPHA/ONE, destination ONE ou
    DSTALPHA, la mémoire d'image de la Wii n'ayant pas d'alpha)"""
    out = {}
    try:
        offs = struct.unpack_from('>11i', dd, mo + 0x10)
        if not offs[6]:
            return out
        for n, m0 in dict_entries(dd, mo + offs[6]):
            try:
                size = struct.unpack_from('>i', dd, m0)[0]
                blob = dd[m0:m0 + max(0, min(size, 0x2000))]
                i = blob.find(b'\x61\x41')
                if i < 0:
                    continue
                v = int.from_bytes(blob[i + 2:i + 5], 'big')
                src, dst = (v >> 8) & 7, (v >> 5) & 7
                out[n] = bool(v & 1) and src in (1, 4) and dst in (1, 6)
            except Exception:  # noqa: BLE001
                continue
    except Exception:  # noqa: BLE001
        pass
    return out


def load(pac, verbose=False):
    """-> (textures {nom: image RGBA PIL}, modèles [(chemin, nom, objets)]) ; objet = dict(name, mat, tex,
    pos (n,3,3) monde, uv (n,3,2) ou None, nrm (n,3,3) monde ou None)"""
    br = []
    walk(open(pac, 'rb').read(), os.path.basename(pac), br, verbose)
    TEX = {}
    for name, d in br:
        try:
            bf = brres_files(d)
        except Exception:  # noqa: BLE001
            continue
        pals = {}
        for n, o in bf.get('Palettes(NW4R)', []):
            try:
                pals[n] = tex0.palette(d, o)
            except Exception:  # noqa: BLE001
                pass
        for n, o in bf.get('Textures(NW4R)', []):
            try:
                TEX[n] = tex0.read_tex0(d, o, pals.get(n)).convert('RGBA')
            except Exception as e:  # noqa: BLE001
                print('texture illisible', n, e)
    models = []
    for name, d in br:
        try:
            bf = brres_files(d)
        except Exception:  # noqa: BLE001
            continue
        for mn, mo in bf.get('3DModels(NW4R)', []):
            try:
                M = mdl0.read_model(d, mo)
            except Exception as e:  # noqa: BLE001
                print('MODELE illisible', name, mn, e)
                continue
            BW = bone_worlds(d, mo, chr0_f0(d, bf))
            byid = {nid: w for nid, w in BW.values()}
            MT = mat_textures(d, mo, TEX)
            MA = mat_additif(d, mo)
            objs = []
            for oi, (on, ob) in enumerate(M['objs']):
                if not ob['tris'] or not (0 <= ob['ids'][0] < len(M['arrays']['pos'])):
                    continue
                P = M['arrays']['pos'][ob['ids'][0]][0]
                if P.shape[1] == 2:
                    P = np.c_[P, np.zeros(len(P))]
                N = M['arrays']['nrm'][ob['ids'][1]][0] if 0 <= ob['ids'][1] < len(M['arrays']['nrm']) else None
                U = M['arrays']['uv'][ob['ids'][4]][0] if 0 <= ob['ids'][4] < len(M['arrays']['uv']) else None
                U1 = M['arrays']['uv'][ob['ids'][5]][0] if 0 <= ob['ids'][5] < len(M['arrays']['uv']) else None
                mi = M['obj_mat'].get(oi)
                mname = M['mat_names'][mi] if mi is not None and mi < len(M['mat_names']) else ''
                pos, uv, nrm, uv1 = [], [], [], []
                for t in ob['tris']:
                    tp, tu, tn, tu1 = [], [], [], []
                    for v in t:
                        w = byid.get(v.get('bone', ob['node']), np.eye(4))
                        tp.append((w @ np.r_[P[v['pos']], 1.0])[:3])
                        tu.append(U[v['uv0']] if U is not None and 'uv0' in v else (0.0, 0.0))
                        tu1.append(U1[v['uv1']] if U1 is not None and 'uv1' in v else (0.0, 0.0))
                        if N is not None and 'nrm' in v:
                            nn = w[:3, :3] @ N[v['nrm']][:3]
                            tn.append(nn / (np.linalg.norm(nn) or 1.0))
                        else:
                            tn.append((0.0, 1.0, 0.0))
                    pos.append(tp)
                    uv.append(tu)
                    nrm.append(tn)
                    uv1.append(tu1)
                texs = MT.get(mname, [])
                objs.append({'name': on, 'mat': mname, 'tex': texs[0] if texs else None, 'pos': np.array(pos),
                             'uv': np.array(uv) if U is not None else None, 'nrm': np.array(nrm),
                             'uv1': np.array(uv1) if U1 is not None else None, 'texs': list(texs),
                             'add': MA.get(mname, False)})
            models.append((name, mn, objs))
    return TEX, models
