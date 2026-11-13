"""Objets de Master Chief dans REX (Fighter/chief/Item/ItmChiefNNBrres00.pac) -> maillages statiques + textures PNG,
pour remplacer les modèles des objets d'Ultimate qu'utilise le plugin (comme REX remplace ceux de Brawl) :
  ItmChief06 / 12 « Plasma Grenade » (remplace ItmChewing = bombe collante) -> item/chewing
  ItmChief54 « Fusion Coil » (textures fusion_coil) -> item/powderbox (caisse explosive)
  ItmChief57 « Gravitylift » (remplace le ressort de Brawl, absent d'Ultimate)
  ItmChief02 « FragGrenade » (remplace la grenade de Snake de Brawl)
usage : python3 objets_rex.py <dossier Item de REX> <sortie>  -> <sortie>/objets.npz, objets.json, png/
"""
import json, os, struct, sys
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
for p in ('../../black-shadow/tools', '../../black-shadow/tools/anim', '../../black-shadow/tools/fs', '../../zero/tools', '.'):
    sys.path.insert(0, os.path.join(HERE, p))
from arc import arc_entries  # noqa: E402
from brres import brres_files  # noqa: E402
import mdl0  # noqa: E402
import tex0  # noqa: E402
from lz import decompress  # noqa: E402
from PIL import Image  # noqa: E402

texs = {}


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


SRC, OUT = sys.argv[1:3]
os.makedirs(os.path.join(OUT, 'png'), exist_ok=True)
OBJ = {'grenade': 'ItmChief06', 'coil': 'ItmChief54', 'lift': 'ItmChief57', 'frag': 'ItmChief02'}
arrays, info = {}, {}
for nom, f in OBJ.items():
    raw = open(os.path.join(SRC, f + 'Brres00.pac'), 'rb').read()
    try:
        raw = decompress(raw)
    except Exception:  # noqa: BLE001
        pass
    models = []
    texs.clear()
    for t, i, s, r, d in arc_entries(raw):
        if d[:4] != b'bres':
            continue
        fl = brres_files(d)
        for n, o in fl.get('3DModels(NW4R)', []):
            models.append((d, n, o))
        for n, o in fl.get('Textures(NW4R)', []):
            texs[n] = (d, o, dict(fl.get('Palettes(NW4R)', [])))
    dd, mn, mo = models[0]
    M = mdl0.read_model(dd, mo)
    bw = srt_world(dd, mo)
    parts = []
    pose = None
    if nom == 'lift':
        # Gravity Lift : 2 objets ; le 0 = la capsule fermée (en main / lancée), le 1 = l'appareil déployé (pattes,
        # pétales) posé au sol, animé par ItmSpringRide (os à liaison rigide). Pose déployée = ItmSpringRide image 0 ;
        # ses sommets sont dans le repère de leur os (pas dans celui du modèle : la pose de repos a l'os « gravlift » à
        # l'échelle 0) : position = matrice monde de l'os à cette image × sommet. Résultat vérifié au rendu : étoile à
        # 3 pattes et 3 pétales à plat dans XZ, moyeu levé en Y (comme sur la vidéo de REX)
        import brawl_anim as BA
        tmp = os.path.join(OUT, '_lift.pac')
        open(tmp, 'wb').write(raw)
        BB = BA.read_bones(tmp)
        an = None
        for t, i, s, r, d in arc_entries(raw):
            if d[:4] == b'bres':
                for n, o in brres_files(d).get('AnmChr(NW4R)', []):
                    if n == 'ItmSpringRide':
                        an = BA.read_chr0(d, o)
        WA = BA.world_pose(BB, an, 0.0)
        so = struct.unpack_from('>11i', dd, mo + 0x10)[1]
        INV = {}
        from brres import dict_entries
        for bn, bo in dict_entries(dd, mo + so):
            m = np.eye(4)
            m[:3, :] = np.array(struct.unpack_from('>12f', dd, bo + 0xA0)).reshape(3, 4)
            INV[bn] = m
        nid2 = {nid: bn for bn, nid, w, s_ in M['bones']}
        pose = (WA, INV, nid2, None)
    for oi, (on, ob) in enumerate(M['objs']):
        if nom == 'lift' and oi != 1:
            continue
        P = M['arrays']['pos'][ob['ids'][0]][0]
        N = M['arrays']['nrm'][ob['ids'][1]][0] if ob['ids'][1] >= 0 else None
        U = M['arrays']['uv'][ob['ids'][4]][0] if ob['ids'][4] >= 0 else None
        W = bw.get(ob['node'], np.eye(4)) if ob['node'] >= 0 else np.eye(4)
        p_, n_, u_ = [], [], []
        for tr in ob['tris']:
            for v in (tr[0], tr[2], tr[1]):
                if pose:
                    WA, INV, nid2, RX = pose
                    bn = nid2[v.get('bone', ob['node'])]
                    W = WA[bn]
                pp = W @ np.r_[P[v['pos']], 1.0]
                nn = W[:3, :3] @ (N[v['nrm']][:3] if N is not None and 'nrm' in v else np.array([0, 1.0, 0]))
                p_.append(pp[:3])
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
        for n in tx[:1]:
            d, o, pals = texs[n]
            try:
                pal = tex0.palette(d, pals[n]) if n in pals else None
                im = tex0.read_tex0(d, o, pal)
                if not isinstance(im, Image.Image):
                    im = Image.fromarray(np.asarray(im))
                im.save(os.path.join(OUT, 'png', n + '.png'))
            except Exception as e:  # noqa: BLE001
                print('texture echec', n, e)
    allp = np.concatenate([arrays[p['cle'] + '_p'] for p in parts])
    info[nom] = {'fichier': f, 'modele': mn, 'parties': parts, 'min': allp.min(0).round(2).tolist(),
                 'max': allp.max(0).round(2).tolist()}
    print('OBJET %-8s %-16s %d parties  min %s max %s' % (nom, mn, len(parts), info[nom]['min'], info[nom]['max']))
    for p in parts:
        print('     ', p)
np.savez_compressed(os.path.join(OUT, 'objets.npz'), **arrays)
json.dump(info, open(os.path.join(OUT, 'objets.json'), 'w'), indent=1)
