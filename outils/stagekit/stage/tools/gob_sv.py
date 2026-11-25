"""Gathering of Old Blood (Brawl, GameBanana 537019) en 2e variante Stage Alts de Smashville (stage/animal_village,
forme normal_s02, touche R ; normal = Sleepless Lamp, normal_s01 = Unreal), v1.

usage : python3 gob_sv.py <STGOLDBLOOD.pac> <réf complète .../stage/animal_village (387162)>
                          <liste des fichiers du jeu stage/animal_village/normal/ (Hashes_FullPath)>
                          <numatb modèle (bg2_set de 392643)> <xvillage00.lvd HDR (normal)> <sortie mod>

Méthode de Sleepless Lamp (lamp_sv.py, même collision de Brawl : sol ±86, murs invisibles ±130) dans un dossier de
variante (générateur de CSK, comme Unreal sur Smashville) :
- décor dans stc_bg_set, maillages regroupés par texture (et mode de mélange), sans os, matériau « Bottom » ;
- matériaux additifs de Brawl (rayons de lumière violets, lueurs : registre de mélange GX source alpha / destination
  alpha = additif sur Wii) : mélange One / One, texture prémultipliée, passe triée (_sort) ;
- tous les autres ensembles cachés par le numdlb à 0 entrée de 387162 ; éclairage « toujours la nuit » (UE_RENDER) ;
- config.json : share_to_vanilla de chaque fichier du jeu de la forme normal non fourni ; image de l'écran des terrains.
"""
import os, shutil, subprocess, sys
import numpy as np
import ssbh_data_py as S

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import brawl_stage  # noqa: E402

PAC, REF, LISTE, MATL_MODELE, LVD, OUT = sys.argv[1:7]
A = S.mesh_data
FORME = os.environ.get('GOB_FORME', 'normal_s02')
NUIT = os.environ.get('GOB_RENDER', '')
RM = os.path.join(REF, 'normal', 'model')
ST = os.path.join(OUT, 'stage', 'animal_village', FORME)
if os.path.isdir(os.path.join(OUT, 'stage')):
    shutil.rmtree(os.path.join(OUT, 'stage'))
ECH = float(os.environ.get('GOB_ECHELLE', '1.0'))
MUR = float(os.environ.get('GOB_MUR_X', '130'))     # murs invisibles de Brawl à ±130
PFX = 'gob_'
DOS = 'stc_bg_set'

from PIL import Image
TEX, models = brawl_stage.load(PAC)
# v3 : le sol (152F3974) est une image de reflet précalculée, posée sans mélange dans Brawl (matériau opaque) ; son alpha
# constant (182) le faisait passer en découpe -> on voyait le vide et le décor en miroir à travers (les « trous »)
for _t in [x for x in os.environ.get('GOB_OPAQUES', '152F3974').split(',') if x]:
    if _t in TEX:
        _im = TEX[_t].convert('RGBA')
        _im.putalpha(255)
        TEX[_t] = _im
# v6 (demande du joueur) : le sol miroir (152F3974) est noir / bleu nuit là où le reflet de Brawl ne montre rien -> on
# repeint ces zones dans le rouge du tapis, et on pose un carré de rouge uni dans un coin (texel du prolongement)
SOL_T = '152F3974'
ROUGE = None
if os.environ.get('GOB_REPEINT', '1') == '1' and SOL_T in TEX:
    _px = np.asarray(TEX[SOL_T].convert('RGB')).astype(np.float32).reshape(-1, 3)
    _rg = _px[(_px[:, 0] > 60) & (_px[:, 0] > 2.0 * _px[:, 1]) & (_px[:, 0] > 2.0 * _px[:, 2])]
    ROUGE = np.percentile(_rg, float(os.environ.get('GOB_ROUGE_PCT', '80')), axis=0) if len(_rg) > 100 else np.array([120.0, 22, 22])
    if os.environ.get('GOB_ROUGE'):
        ROUGE = np.array([float(v) for v in os.environ['GOB_ROUGE'].split(',')])
    print('repeint du sol : rouge %s' % (tuple(int(v) for v in ROUGE),))


def repeint(t, im):
    # im : tableau RGBA float ; renvoie le tableau repeint (sol uniquement)
    if ROUGE is None or t != SOL_T:
        return im
    rgb = im[:, :, :3]
    mx = rgb.max(2)
    T = float(os.environ.get('GOB_SEUIL', '75'))
    w = np.clip((T - mx) / (T * 0.5), 0, 1)                       # sombre -> repeint
    w = np.maximum(w, np.clip((rgb[:, :, 2] - rgb[:, :, 0]) / 40.0, 0, 1))  # bleu -> repeint
    ton = (0.85 + 0.15 * np.clip(mx / T, 0, 1))[:, :, None] * ROUGE[None, None, :]
    rgb = rgb * (1 - w[:, :, None]) + ton * w[:, :, None]
    h, wd = rgb.shape[:2]
    c = max(16, wd // 16)
    rgb[:c, :c] = ROUGE
    im = im.copy()
    im[:, :, :3] = rgb
    im[:, :, 3] = 255
    print('  repeint %s : %.1f %% des pixels, carré %d px' % (t, 100.0 * float((w > 0.5).mean()), c))
    return im


if ROUGE is not None:
    TEX[SOL_T] = Image.fromarray(np.clip(repeint(SOL_T, np.asarray(TEX[SOL_T].convert('RGBA')).astype(np.float32)) + 0.5, 0, 255).astype(np.uint8), 'RGBA')
SANS = set()
# v2 : lueur sur le mur près de la porte (mesh_29, additive, collée au mur) qui clignote en jeu (demande du joueur)
SANS_OBJ = set(x for x in os.environ.get('GOB_SANS_OBJ', 'mesh_29').split(',') if x)
garde = []
for _, mn, ol in models:
    if mn in SANS:
        print('ignore (modèle)', mn)
        continue
    for o in ol:
        if o['name'] in SANS_OBJ:
            print('ignore (objet)', mn, o['name'])
            continue
        if not o['tex'] or o['tex'] not in TEX:
            print('ignore', mn, o['name'], o['tex'])
            continue
        garde.append((mn, o))


def surface(x, z=0.0):
    """hauteur la plus haute du décor sous y +15 à la verticale de (x, z) (Brawl)"""
    best = None
    for _, o in garde:
        P = o['pos']
        a, b, c = P[:, 0], P[:, 1], P[:, 2]
        v0, v1 = c - a, b - a
        d = v0[:, 0] * v1[:, 2] - v1[:, 0] * v0[:, 2]
        ok = np.abs(d) > 1e-9
        px, pz = x - a[:, 0], z - a[:, 2]
        with np.errstate(divide='ignore', invalid='ignore'):
            u = (px * v1[:, 2] - v1[:, 0] * pz) / d
            v = (v0[:, 0] * pz - px * v0[:, 2]) / d
        m = ok & (u >= -1e-6) & (v >= -1e-6) & (u + v <= 1 + 1e-6)
        y = a[m, 1] + u[m] * v0[m, 1] + v[m] * v1[m, 1]
        y = y[y < 15]
        if len(y) and (best is None or y.max() > best):
            best = float(y.max())
    return best


hs = [surface(x) for x in (-80, -40, 0, 40, 80)]
print('sol visible (Brawl) à z 0 :', hs)
vals = [h for h in hs if h is not None and abs(h) < 10]
Y0 = float(os.environ.get('GOB_Y0', str(round(float(np.median(vals)), 3) if vals else 0.0)))
print('Y0 =', Y0)


# cour prolongée vers l'écran (demande du joueur : on voyait le bord avant du sol quand la caméra recule) :
# le bord avant du dallage (z ~322, y ~11, texture 197F49C0, v 0,4933, u linéaire en x) est étiré jusqu'à z 2500,
# et sur les côtés jusqu'à x ±2500 (u du bout du bord).
def prolongement():
    bord = []
    for _, o in garde:
        if o['tex'] != '197F49C0' or o['uv'] is None:
            continue
        P, U = o['pos'], o['uv']
        nn = np.cross(P[:, 1] - P[:, 0], P[:, 2] - P[:, 0])
        nn /= np.maximum(np.linalg.norm(nn, axis=1, keepdims=True), 1e-9)
        for t in np.where(np.abs(nn[:, 1]) > 0.5)[0]:
            for k in range(3):
                if P[t, k, 2] > 318 and abs(U[t, k, 1] - 0.4933) < 0.002:
                    bord.append((P[t, k, 0], P[t, k, 1], P[t, k, 2], U[t, k, 0], U[t, k, 1]))
    bord = np.array(sorted(bord))
    xs = np.unique(np.round(bord[:, 0], 1))
    pts = []
    for x in xs:
        m = np.abs(bord[:, 0] - x) < 0.2
        pts.append((x, bord[m, 1].min() - 0.05, bord[m, 2].min() - 0.5, bord[m, 3].mean(), bord[m, 4].mean()))
    pts = [(-2500.0,) + pts[0][1:]] + pts + [(2500.0,) + pts[-1][1:]]
    Z2 = 2500.0
    pos, uv = [], []
    for (x0, y0, z0, u0, v0), (x1, y1, z1, u1, v1) in zip(pts, pts[1:]):
        a, b, c, d = (x0, y0, z0), (x1, y1, z1), (x1, y1, Z2), (x0, y0, Z2)
        pos += [[a, b, c], [a, c, d]]
        uv += [[(u0, v0), (u1, v1), (u1, v1)], [(u0, v0), (u1, v1), (u0, v0)]]
    pos = np.array(pos, dtype=np.float64)
    print('cour prolongée : bord x %.0f..%.0f, y %.1f..%.1f, z %.0f -> %.0f, %d triangles'
          % (xs.min(), xs.max(), min(p[1] for p in pts), max(p[1] for p in pts), min(p[2] for p in pts), Z2, len(pos)))
    return {'name': 'prolongement', 'tex': '197F49C0', 'pos': pos, 'uv': np.array(uv, dtype=np.float64),
            'nrm': np.tile(np.array([0.0, 1.0, 0.0]), pos.shape[:2] + (1,))}


if os.environ.get('GOB_PROLONGE', '0') == '1':
    garde.append(('prolongement', prolongement()))

# v2 (demande du joueur) : trous dans le sol (le sol de Brawl est translucide, avec le décor en miroir dessous ; là où
# le miroir ne couvre pas, on voyait le vide) et bord du sol visible devant :
# - sol translucide (152F3974) prolongé devant et sur les côtés jusqu'à ±3000 (UV prolongés par un ajustement affine) ;
# - plan opaque sombre sous le décor en miroir (fond du reflet), qui bouche tous les trous
if os.environ.get('GOB_SOL', '1') == '1':
    from PIL import Image as _I
    sols = [o for _, o in garde if o['tex'] == '152F3974']
    Ps = np.concatenate([o['pos'].reshape(-1, 3) for o in sols])
    Us = np.concatenate([o['uv'].reshape(-1, 2) for o in sols])
    co = np.linalg.lstsq(np.c_[Ps[:, 0], Ps[:, 2], np.ones(len(Ps))], Us, rcond=None)[0]
    ys = float(np.median(Ps[:, 1]))
    x0s, x1s, z0s, z1s = Ps[:, 0].min(), Ps[:, 0].max(), Ps[:, 2].min(), Ps[:, 2].max()
    G = 3000.0

    def quad(xa, xb, za, zb, y):
        pos = np.array([[[xa, y, za], [xb, y, zb], [xb, y, za]], [[xa, y, za], [xa, y, zb], [xb, y, zb]]], dtype=np.float64)
        uv = np.c_[pos.reshape(-1, 3)[:, 0], pos.reshape(-1, 3)[:, 2], np.ones(6)] @ co
        return pos, uv.reshape(2, 3, 2)
    # v4 : les grands plans séparés (texture unie à part) restaient invisibles / transparents en jeu -> prolongement
    # découpé en dalles de 200 unités, dans le MÊME objet que le sol (texture du sol, UV fixés sur un texel de la
    # couleur moyenne du sol), comme le prolongement de Sleepless Lamp qui marche ; jusqu'à ±4000 ; plus de fond séparé
    G = float(os.environ.get('GOB_SOL_G', '4000'))
    PAS = 200.0
    _a = np.asarray(TEX['152F3974'].convert('RGB')).astype(np.float32)
    _h, _w = _a.shape[:2]
    _med = np.median(_a.reshape(-1, 3), axis=0)
    if os.environ.get('GOB_SOL_ROUGE', '1') == '1':
        # v5 (demande du joueur) : la couleur médiane était presque noire -> le prolongement ressemblait à du vide ;
        # on vise le rouge du tapis : médiane des pixels nettement rouges, un peu assombrie (rouge foncé du bord)
        _px = _a.reshape(-1, 3)
        _rg = _px[(_px[:, 0] > 60) & (_px[:, 0] > 2.0 * _px[:, 1]) & (_px[:, 0] > 2.0 * _px[:, 2])]
        if len(_rg) > 100:
            _med = np.median(_rg, axis=0) * float(os.environ.get('GOB_SOL_ROUGE_F', '0.85'))
        print('prolongement : couleur visée %s (%d pixels rouges)' % (tuple(int(v) for v in _med), len(_rg)))
    best, buv = 1e9, (0.5, 0.5)
    for _y in range(3, _h - 3, 4):
        for _x in range(3, _w - 3, 4):
            blk = _a[_y - 3:_y + 4, _x - 3:_x + 4].reshape(-1, 3)
            sc = np.abs(blk.mean(0) - _med).sum() + blk.std(0).sum()
            if sc < best:
                best, buv = sc, ((_x + 0.5) / _w, (_y + 0.5) / _h)
    if ROUGE is not None:
        _c = max(16, _w // 16)
        buv = ((_c / 2.0) / _w, (_c / 2.0) / _h)
    def dalles(xa, xb, za, zb):
        out = []
        xs_ = np.arange(xa, xb, PAS).tolist() + [xb]
        zs_ = np.arange(za, zb, PAS).tolist() + [zb]
        for x0_, x1_ in zip(xs_, xs_[1:]):
            for z0_, z1_ in zip(zs_, zs_[1:]):
                out += [[[x0_, ys, z0_], [x1_, ys, z1_], [x1_, ys, z0_]], [[x0_, ys, z0_], [x0_, ys, z1_], [x1_, ys, z1_]]]
        return out
    # v4b : le sol d'origine n'est pas un rectangle (trous à l'intérieur de son emprise, devant à droite) -> dalles sur
    # TOUTE la surface, juste sous le point le plus bas du sol d'origine (cachées là où il existe)
    ys = float(Ps[:, 1].min()) - 0.3
    tri_ = dalles(-G, G, -G, G)
    pos = np.array(tri_, dtype=np.float64)
    sol0 = sols[0]
    sol0['pos'] = np.concatenate([sol0['pos'], pos])
    sol0['uv'] = np.concatenate([sol0['uv'], np.tile(np.array(buv), pos.shape[:2] + (1,))])
    sol0['nrm'] = np.concatenate([sol0['nrm'], np.tile(np.array([0.0, 1.0, 0.0]), pos.shape[:2] + (1,))])
    yf = float('nan')
    print('prolongement : %d dalles dans l\'objet du sol, texel %s (écart %.1f, couleur %s)' % (len(pos) // 2, tuple(round(v, 3) for v in buv), best, tuple(int(v) for v in _a[int(buv[1] * _h), int(buv[0] * _w)])))
    print('sol prolongé à ±%.0f (y %.2f, sol d\'origine x %.0f..%.0f z %.0f..%.0f) ; fond du reflet y %.1f' % (G, ys, x0s, x1s, z0s, z1s, yf))


def attr(name, data):
    a = A.AttributeData(name)
    a.data = np.asarray(data, dtype=np.float32)
    return a


def alpha(t):
    return int(np.asarray(TEX[t].convert('RGBA'))[:, :, 3].min()) < 200


def tname(t):
    return (PFX + t).lower().replace(' ', '_')


tm = S.matl_data.read_matl(MATL_MODELE)
TI = next(i for i, e in enumerate(tm.entries) if e.material_label == 'Bottom')

# regroupement par texture
par_tex = {}
for mn, o in garde:
    par_tex.setdefault((o['tex'], bool(o.get('add'))), []).append(o)

D = os.path.join(ST, 'model', DOS)
os.makedirs(D, exist_ok=True)
objs, labs, entries, nouveaux = [], [], [], []
for k, ((t, add), ol) in enumerate(sorted(par_tex.items())):
    pos = np.concatenate([o['pos'][:, [0, 2, 1]] for o in ol]).reshape(-1, 3).astype(np.float64)
    nrm = np.concatenate([o['nrm'][:, [0, 2, 1]] for o in ol]).reshape(-1, 3)
    uv = np.concatenate([(o['uv'][:, [0, 2, 1]] if o['uv'] is not None else np.zeros(o['pos'].shape[:2] + (2,)))
                         for o in ol]).reshape(-1, 2)
    key = np.round(np.c_[pos, nrm * 100, uv * 4096], 3)
    uk, inv = np.unique(key, axis=0, return_inverse=True)
    first = np.zeros(len(uk), dtype=np.int64)
    first[inv.ravel()[::-1]] = np.arange(len(inv.ravel()))[::-1]
    pos, nrm, uv = pos[first], nrm[first], uv[first]
    pos[:, 1] -= Y0
    pos *= ECH
    nrm = nrm / np.maximum(np.linalg.norm(nrm, axis=1, keepdims=True), 1e-6)
    idx = inv.astype(np.uint32).ravel()
    nv = len(pos)
    mo = A.MeshObjectData('gob_%02d' % k, 0)
    mo.positions = [attr('Position0', pos)]
    mo.normals = [attr('Normal0', np.c_[nrm, np.zeros(nv)])]
    try:
        tg = A.calculate_tangents_vec4(pos, nrm, uv, idx)
    except Exception:  # noqa: BLE001
        up = np.where(np.abs(nrm[:, 1:2]) > 0.9, np.array([[1, 0, 0]]), np.array([[0, 1, 0]]))
        tg = np.c_[np.cross(up, nrm), np.ones(nv)]
    mo.tangents = [attr('Tangent0', tg)]
    mo.texture_coordinates = [attr('map1', uv), attr('bake1', uv), attr('uvSet', uv)]
    mo.color_sets = [attr('colorSet1', np.full((nv, 4), 0.502)), attr('colorSet4', np.full((nv, 4), 0.502))]
    mo.vertex_indices = idx
    a = alpha(t)
    lab = 'gob_%s%s%s' % (t, '_a' if a else '', '_add' if add else '')
    e = S.matl_data.read_matl(MATL_MODELE).entries[TI]
    e.material_label = lab
    for x in e.textures:
        if x.param_id.name == 'Texture5':
            x.data = tname(t)
    for x in e.rasterizer_states:
        d_ = x.data
        d_.cull_mode = S.matl_data.CullMode.Disabled
        x.data = d_
    if add:
        # additif : One / One sur la texture prémultipliée, passe triée, sans écriture de profondeur gênante
        for x in e.blend_states:
            d_ = x.data
            d_.source_color = S.matl_data.BlendFactor.One
            d_.destination_color = S.matl_data.BlendFactor.One
            d_.alpha_sample_to_coverage = False
            x.data = d_
        e.shader_label = e.shader_label.rsplit('_', 1)[0] + '_sort'
    elif t == SOL_T and os.environ.get('GOB_SOL_OMBRE', '1') == '1':
        # v7 : le shader « Bottom » (000010000000c100) n'affiche pas l'ombre des persos ; 0000000008008100 a exactement les
        # mêmes paramètres (Texture5, CustomVector3/8/27) et reçoit l'ombre (base de shaders ssbh_wgpu)
        e.shader_label = 'SFX_PBS_0000000008008100_opaque'
        print('  sol : shader qui reçoit l\'ombre des persos', e.shader_label)
    elif a:
        for x in e.blend_states:
            d_ = x.data
            d_.alpha_sample_to_coverage = True
            x.data = d_
    entries.append(e)
    objs.append(mo)
    labs.append(lab)
    print('%-8s %-12s %6d tri %6d sommets %s' % (mo.name, t, len(idx) // 3, nv, lab))
    from PIL import Image
    png = os.path.join(OUT, '_tmp.png')
    im = np.asarray(TEX[t].convert('RGBA')).astype(np.float32)
    hd = os.path.join(os.environ.get('GOB_HD', ''), t + '_x4.png')
    if os.environ.get('GOB_HD') and os.path.exists(hd):
        im = np.asarray(Image.open(hd).convert('RGBA')).astype(np.float32)
        im = repeint(t, im)
    if a or add:
        im[:, :, :3] *= im[:, :, 3:] / 255.0
    Image.fromarray(np.clip(im + 0.5, 0, 255).astype(np.uint8), 'RGBA').save(png)
    cible = os.path.join(D, tname(t) + '.nutexb')
    if os.path.exists(cible):
        os.remove(png)
        print('  (texture déjà écrite)')
        continue
    subprocess.run(['ultimate_tex_cli', png, cible, '--format', 'BC7RgbaUnormSrgb'], check=True, capture_output=True)
    nouveaux.append(os.path.relpath(cible, OUT).replace(os.sep, '/'))
    os.remove(png)

# scène three.js (aperçus / image de l'écran des terrains), même décor que le mod
if os.environ.get('GOB_SCENE'):
    import json as _j
    from PIL import Image
    SC = os.environ['GOB_SCENE']
    os.makedirs(os.path.join(SC, 'tex'), exist_ok=True)
    noms, scene = {}, []
    for i, t in enumerate(sorted(set(k_[0] for k_ in par_tex))):
        noms[t] = 't%02d' % i
        hd = os.path.join(os.environ.get('GOB_HD', ''), t + '_x4.png')
        if os.environ.get('GOB_HD') and os.path.exists(hd):
            Image.fromarray(np.clip(repeint(t, np.asarray(Image.open(hd).convert('RGBA')).astype(np.float32)) + 0.5, 0, 255).astype(np.uint8), 'RGBA').save(os.path.join(SC, 'tex', noms[t] + '.png'))
        else:
            TEX[t].convert('RGBA').save(os.path.join(SC, 'tex', noms[t] + '.png'))
    for (t, add), ol in par_tex.items():
        for o in ol:
            p = o['pos'][:, [0, 2, 1]].astype(np.float64)
            p[:, :, 1] -= Y0
            p *= ECH
            u = o['uv'][:, [0, 2, 1]] if o['uv'] is not None else np.zeros(p.shape[:2] + (2,))
            scene.append({'p': np.round(p.reshape(-1), 4).tolist(), 'uv': np.round(u.reshape(-1), 5).tolist(), 't': noms[t], 'a': bool(alpha(t)),
                          'add': bool(add)})
    _j.dump(scene, open(os.path.join(SC, 'scene.json'), 'w'))
    print('scène', len(scene), 'objets')

matl = S.matl_data.read_matl(os.path.join(RM, DOS, 'model.numatb'))
matl.entries = entries
matl.save(os.path.join(D, 'model.numatb'))
mesh = A.read_mesh(os.path.join(RM, DOS, 'model.numshb'))
mesh.objects = objs
mesh.save(os.path.join(D, 'model.numshb'))
modl = S.modl_data.read_modl(os.path.join(RM, DOS, 'model.numdlb'))
modl.entries = [S.modl_data.ModlEntryData(o.name, o.subindex, l) for o, l in zip(objs, labs)]
modl.save(os.path.join(D, 'model.numdlb'))
S.meshex_data.MeshExData.from_mesh_objects(objs).save(os.path.join(D, 'model.numshexb'))
print('DOSSIER', DOS, len(objs), 'objets', sum(len(o.vertex_indices) // 3 for o in objs), 'triangles')

# tous les autres ensembles cachés : numdlb à 0 entrée (celui de la réf)
vide = os.path.join(RM, 'dys_guest_asami_set', 'model.numdlb')
assert len(S.modl_data.read_modl(vide).entries) == 0
ens = set(d for d in os.listdir(RM) if os.path.isdir(os.path.join(RM, d)))
for l in open(LISTE):
    if l.startswith('stage/animal_village/normal/model/'):
        ens.add(l.strip().split('/')[4])
caches = sorted(ens - {DOS})
for d in caches:
    os.makedirs(os.path.join(ST, 'model', d), exist_ok=True)
    shutil.copy(vide, os.path.join(ST, 'model', d, 'model.numdlb'))
print('cachés (0 entrée) :', len(caches), ' '.join(caches))

# persos éclairés de nuit (Always Night)
src = os.path.join(NUIT, 'normal', 'render')
if os.path.isdir(src):
    shutil.copytree(src, os.path.join(ST, 'render'))
    print('render : nuit permanente', len(os.listdir(os.path.join(ST, 'render'))), 'fichiers')

# collisions
import yaml
P_ = os.path.join(ST, 'param')
os.makedirs(P_, exist_ok=True)
tmp = os.path.join(OUT, '_lvd.yml')
subprocess.run(['yamlvd', LVD, tmp], check=True, capture_output=True)
d = yaml.safe_load(open(tmp))
bz = d['blast_zone'][0]
X, XM, YB = MUR * ECH, (MUR + 10) * ECH, -30.0
YH = min(170.0, bz['top'] - 10.0)
pts = [(XM, YB), (-XM, YB), (-XM, YH), (-X, YH), (-X, 0.0), (X, 0.0), (X, YH), (XM, YH), (XM, YB)]
sol = next(c for c in d['collisions'] if c['entry']['name'] == 'COL_00_Floor01')
sol['vertices'] = [{'x': float(x), 'y': float(y)} for x, y in pts]
nrm = []
for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
    dx, dy = x1 - x0, y1 - y0
    n = (dx * dx + dy * dy) ** 0.5
    nrm.append({'x': float(-dy / n), 'y': float(dx / n)})
sol['normals'] = nrm
sol['cliffs'] = []
sol['materials'] = [sol['materials'][0]] * len(nrm)
for c in d['collisions']:
    if c is sol:
        continue
    # plate-forme mobile (relative à son os) : petit segment 45 sous elle -> toujours dans le bloc du sol
    c['vertices'] = [{'x': -5.0, 'y': -45.0}, {'x': 5.0, 'y': -45.0}]
    c['normals'] = [{'x': 0.0, 'y': 1.0}]
    print('enterrée :', c['entry']['name'])
for s in d['spawns']:
    s['pos']['y'] = 0.1
for r in d['respawns']:
    r['pos']['x'] = max(-X + 15, min(X - 15, r['pos']['x']))
cb = d['camera_boundary'][0]
cb['left'], cb['right'] = min(cb['left'], -XM - 10), max(cb['right'], XM + 10)
print('sol plat x ±%.0f, murs jusqu\'à y %.0f ; caméra %s ; zones de K.O. %s' % (X, YH, {k: cb[k] for k in ('left', 'right', 'top', 'bottom')},
                                                                      {k: bz[k] for k in ('left', 'right', 'top', 'bottom')}))
yaml.safe_dump(d, open(tmp, 'w'), sort_keys=False)
subprocess.run(['yamlvd', tmp, os.path.join(P_, 'xvillage00.lvd')], check=True, capture_output=True)
os.remove(tmp)

import json
# config.json (Stage Alts) : chaque fichier du jeu de stage/animal_village/normal non fourni est partagé vers la variante
fournis = set()
for racine, _, fichiers in os.walk(ST):
    for f in fichiers:
        fournis.add(os.path.relpath(os.path.join(racine, f), OUT).replace(os.sep, '/'))
partage = {}
for l in open(LISTE):
    p = l.strip()
    if not p.startswith('stage/animal_village/normal/'):
        continue
    v = p.replace('stage/animal_village/normal/', 'stage/animal_village/%s/' % FORME, 1)
    if v not in fournis:
        partage[p] = [v]
UI_FOURNIS = [int(x) for x in os.environ.get('GOB_UI_FOURNIS', '1,2,4').split(',') if x]
for n in range(5):
    if n not in UI_FOURNIS:
        partage['ui/replace/stage/stage_%d/stage_%d_animal_village.bntx' % (n, n)] = \
            ['ui/replace/stage/stage_%d/stage_%d_animal_village_%s.bntx' % (n, n, FORME.split('_')[-1])]
json.dump({'share_to_vanilla': partage}, open(os.path.join(OUT, 'config.json'), 'w'), indent=2)
print('fournis', len(fournis), '; partagés avec le jeu', len(partage), '; additifs', [l for l in labs if l.endswith('_add')])
