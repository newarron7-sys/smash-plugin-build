"""Armes de Master Chief (REX) : modèles d'effets de REX (fusil d'assaut, épée à énergie, sniper, fusil à pompe…)
extraits en maillages statiques (sommets, normales, UV, textures en PNG) pour être ajoutés au modèle Ultimate.

Dans REX, ses armes ne font pas partie du modèle : ce sont des effets (banque 0x1FB du FitChief.pac) posés sur l'os
de la main (RHaveN / LHaveN) avec un décalage, une rotation et une échelle (commande 11010A00 de ses scripts).

usage : python3 armes_rex.py FitChief.pac <sortie> [FitChiefFinal.pac]   -> <sortie>/armes.npz, <sortie>/armes.json, <sortie>/png/*.png
"""
import json, os, struct, sys
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
for p in ('../../black-shadow/tools', '../../black-shadow/tools/fs', '../../zero/tools', '.'):
    sys.path.insert(0, os.path.join(HERE, p))
from arc import arc_entries  # noqa: E402
from brres import brres_files  # noqa: E402
import mdl0  # noqa: E402
import tex0  # noqa: E402
from lz import decompress  # noqa: E402
from PIL import Image  # noqa: E402

SRC, OUT = sys.argv[1:3]
os.makedirs(os.path.join(OUT, 'png'), exist_ok=True)

# effet de REX (n° dans la banque 0x1FB) -> (nom, n° du modèle dans la banque, os, position, rotation (deg), échelle)
# (liste EFLS du FitChief.pac et commandes 11010A00 de ses scripts, réglage le plus courant pour chaque arme)
ARMES = {
    'rifle':   (0x11, 0, 'RHaveN', (0, 1, 2), (90, 0, 0), 2.0),     # fusil d'assaut (toujours en main)
    'sword':   (0x01, 1, 'RHaveN', (0, 0, -2), (90, 0, 0), 2.0),    # épée à énergie (poignée)
    'swordb':  (0x09, 9, 'RHaveN', (0, 0, -2), (90, 0, 0), 1.0),    # épée à énergie (lames lumineuses)
    'smg':     (0x02, 2, 'RHaveN', (0, 0, 0), (0, 90, 0), 1.0),     # mitraillette (up tilt)
    'laser':   (0x03, 3, 'ThrowN', (0, 0, 0), (0, 0, 0), 1.0),      # laser spartan (up smash)
    'sniper':  (0x04, 4, 'RHaveN', (0, 0, 0), (0, 0, 0), 1.0),      # fusil de précision (down smash)
    'shotgun': (0x05, 5, 'RHaveN', (0, 0.2, 0), (0, 0, 0), 1.2),    # fusil à pompe (forward smash)
    'battle':  (0x0B, 11, 'RHaveN', (0, 0.2, 0.45), (8, 1, 0), 1.8),  # fusil de combat (forward tilt)
    'plasmar': (0x0F, 15, 'RHaveN', (0, 0, 0), (0, 90, 0), 1.0),    # fusil plasma (up air)
    'magnum':  (0x12, 18, 'LHaveN', (0, 0, 0), (0, -1, 0), 1.7),    # magnum (B neutre)
    'plasmap': (0x14, 20, 'RHaveN', (0, 0, 0), (0, 0, 0), 1.7),     # pistolet plasma (down tilt)
    'rifledos': (0x11, 0, 'BustN', (0, 2, -3), (0, 90, 145), 2.0),  # fusil d'assaut rangé dans le dos (os 0xF)
    'smgl':    (0x02, 2, 'LHaveN', (0, 0, 0), (0, 90, 0), 1.0),     # 2e mitraillette (main gauche, os 0x1D)
    'plasmarl': (0x0F, 15, 'LHaveN', (0, 0, 0), (0, 90, 0), 1.0),   # 2e fusil plasma (main gauche)
}

raw = decompress(open(SRC, 'rb').read())
inner = [d for t, i, s, r, d in arc_entries(raw) if d[:4] == b'ARC\x00'][0]
models, texs = {}, {}
for t, i, s, r, d in arc_entries(inner):
    if d[:4] != b'bres':
        continue
    f = brres_files(d)
    if t == 2 and '3DModels(NW4R)' in f:
        models[i] = (d, f['3DModels(NW4R)'][0])
    for n, o in f.get('Textures(NW4R)', []):
        texs[n] = (d, o, dict(f.get('Palettes(NW4R)', [])))
# v27 : lance-roquettes du Final Smash (FitChiefFinal.pac : modèles 0 = roquette, 1 = lance-roquettes ; effets 0xE0000x)
FINAL = sys.argv[3] if len(sys.argv) > 3 else None
if FINAL:
    rawf = decompress(open(FINAL, 'rb').read())
    innerf = [d for t, i, s, r, d in arc_entries(rawf) if d[:4] == b'ARC\x00'][0]
    for t, i, s_, r, d in arc_entries(innerf):
        if d[:4] != b'bres':
            continue
        f = brres_files(d)
        if t == 2 and '3DModels(NW4R)' in f:
            models[100 + i] = (d, f['3DModels(NW4R)'][0])
        for n, o in f.get('Textures(NW4R)', []):
            texs.setdefault(n, (d, o, dict(f.get('Palettes(NW4R)', []))))
    ARMES['lanceur'] = (0xE00007, 101, 'RHaveN', (0, 0, 0), (0, 0, 90), 1.0)   # 11010A00 du Final Smash
print('modeles d\'effets', sorted(models), 'textures', len(texs))


def rot(rx, ry, rz):
    """rotation Brawl (degrés, ordre X puis Y puis Z)"""
    a, b, c = np.radians([rx, ry, rz])
    X = np.array([[1, 0, 0], [0, np.cos(a), -np.sin(a)], [0, np.sin(a), np.cos(a)]])
    Y = np.array([[np.cos(b), 0, np.sin(b)], [0, 1, 0], [-np.sin(b), 0, np.cos(b)]])
    Z = np.array([[np.cos(c), -np.sin(c), 0], [np.sin(c), np.cos(c), 0], [0, 0, 1]])
    return Z @ Y @ X


def mat_textures(dd, mo, mname):
    """textures d'un matériau (comme build_model_rex : on cherche les noms de textures connues dans sa structure)"""
    from brres import dict_entries
    so = struct.unpack_from('>i', dd, mo + 0x10 + 4 * 6)[0]
    for n, moff in dict_entries(dd, mo + so):
        if n != mname:
            continue
        size = struct.unpack_from('>i', dd, moff)[0]
        found = []
        for e in range(moff, moff + max(0, min(size, 0x1000)) - 4, 4):
            o2 = struct.unpack_from('>i', dd, e)[0]
            if 0 < o2 < 0x200000 and e + o2 < len(dd):
                try:
                    t = dd[e + o2:dd.index(b'\0', e + o2)].decode('latin1')
                except ValueError:
                    continue
                if t in texs and t not in found:
                    found.append(t)
        return found
    return []


def srt_world(dd, mo):
    """matrices des os calculées depuis leur échelle / rotation / translation (ce que fait le jeu en dessinant
    l'effet) : la matrice « monde » rangée dans le MDL0 n'est pas à jour pour certains modèles (sniper, fusil à
    pompe : tournée de 90°, épée : poignée décalée de 3,85)"""
    from brres import dict_entries
    so = struct.unpack_from('>11i', dd, mo + 0x10)[1]
    loc, par, nid_of = {}, {}, {}
    for n, bo in dict_entries(dd, mo + so):
        sc = struct.unpack_from('>3f', dd, bo + 0x20)
        rr = struct.unpack_from('>3f', dd, bo + 0x2C)
        tr = struct.unpack_from('>3f', dd, bo + 0x38)
        m = np.eye(4)
        m[:3, :3] = rot(*rr) @ np.diag(sc)
        m[:3, 3] = tr
        loc[bo] = m
        p = struct.unpack_from('>i', dd, bo + 0x5C)[0]
        par[bo] = bo + p if p else None
        nid_of[bo] = struct.unpack_from('>i', dd, bo + 0x10)[0]

    def world(bo):
        return loc[bo] if par[bo] is None or par[bo] not in loc else world(par[bo]) @ loc[bo]
    return {nid_of[bo]: world(bo) for bo in loc}


arrays, info = {}, {}
used_tex = set()
for nom, (eid, mi, os_, pos, rr, sc) in ARMES.items():
    if mi not in models:
        print('ABSENT', nom, mi)
        continue
    dd, (mn, mo) = models[mi][0], models[mi][1]
    M = mdl0.read_model(dd, mo)
    bw = srt_world(dd, mo)
    A = np.eye(4)
    A[:3, :3] = rot(*rr) * sc
    A[:3, 3] = pos
    parts = []
    for oi, (on, ob) in enumerate(M['objs']):
        P = M['arrays']['pos'][ob['ids'][0]][0]
        N = M['arrays']['nrm'][ob['ids'][1]][0] if ob['ids'][1] >= 0 else None
        U = M['arrays']['uv'][ob['ids'][4]][0] if ob['ids'][4] >= 0 else None
        W = bw.get(ob['node'], np.eye(4)) if ob['node'] >= 0 else np.eye(4)
        p_, n_, u_ = [], [], []
        for t in ob['tris']:
            for v in (t[0], t[2], t[1]):
                pp = W @ np.r_[P[v['pos']], 1.0]
                nn = W[:3, :3] @ (N[v['nrm']][:3] if N is not None and 'nrm' in v else np.array([0, 1.0, 0]))
                p_.append((A @ pp)[:3])
                nn = A[:3, :3] @ nn
                n_.append(nn / (np.linalg.norm(nn) or 1))
                u_.append(U[v['uv0']] if U is not None and 'uv0' in v else np.zeros(2))
        if not p_:
            continue
        mat = M['mat_names'][M['obj_mat'].get(oi, 0)] if M['mat_names'] else ''
        tx = mat_textures(dd, mo, mat)
        key = '%s_%d' % (nom, oi)
        arrays[key + '_p'] = np.array(p_, np.float32)
        arrays[key + '_n'] = np.array(n_, np.float32)
        arrays[key + '_u'] = np.array(u_, np.float32)
        parts.append({'cle': key, 'objet': on, 'materiau': mat, 'textures': tx, 'triangles': len(p_) // 3})
        used_tex.update(tx[:1])
    info[nom] = {'effet': eid, 'modele': mn, 'os': os_, 'parties': parts,
                 'taille': (np.ptp(np.concatenate([arrays[p['cle'] + '_p'] for p in parts]), 0).round(2).tolist()
                            if parts else None)}
    print('ARME %-8s %-24s os %-6s %d parties %s' % (nom, mn, os_, len(parts), info[nom]['taille']))
    for p in parts:
        print('     ', p)

for n in sorted(used_tex):
    d, o, pals = texs[n]
    try:
        pal = tex0.palette(d, pals[n]) if n in pals else None      # textures à palette (lame de l'épée : C4)
        im = tex0.read_tex0(d, o, pal)
        if not isinstance(im, Image.Image):
            im = Image.fromarray(np.asarray(im))
        im.save(os.path.join(OUT, 'png', n + '.png'))
    except Exception as e:  # noqa: BLE001
        print('texture echec', n, e)
np.savez_compressed(os.path.join(OUT, 'armes.npz'), **arrays)
json.dump(info, open(os.path.join(OUT, 'armes.json'), 'w'), indent=1)
print('textures', sorted(used_tex))
