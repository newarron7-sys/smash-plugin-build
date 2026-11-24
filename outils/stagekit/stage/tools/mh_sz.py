"""Monster Hunter Tri - Deserted Island Basin 1 (pack « Machinima Stage Pack 01 », STGFINAL.pac) sur le Château de
Suzaku (stage/sf_suzaku), v1.

usage : python3 mh_sz.py <STGFINAL.pac> <réf 379012 .../stage/sf_suzaku> <réf 348709 .../stage/sf_suzaku>
                         <numatb modèle (bg2_set de 392643)> <streetfighter_f00.lvd HDR> <sortie mod>

Méthode de BSL Station / Sleepless Lamp :
- décor de Basin 1 dans stc_bg_m_set (maillages regroupés par texture, sans os ; matériau « Bottom » sans éclairage) ;
- décor d'origine du Château de Suzaku caché par des numdlb à 0 entrée (v2 : plus aucun fichier d'un autre mod) ;
- collisions : sol de Basin 1 tel quel (presque plat, Brawl), murs invisibles aux extrémités (x -782 et +2201),
  plates-formes du toit à y -999 comme HDR ; blast zones et caméra élargies juste derrière les murs (Infinite Azure).
"""
import os, shutil, subprocess, sys
import numpy as np
import ssbh_data_py as S

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import brawl_stage  # noqa: E402

PAC, REF, REF_NU, MATL_MODELE, LVD, OUT = sys.argv[1:7]
A = S.mesh_data
RM = os.path.join(REF, 'normal', 'model')
ST = os.path.join(OUT, 'stage', 'sf_suzaku', 'normal')
if os.path.isdir(os.path.join(OUT, 'stage')):
    shutil.rmtree(os.path.join(OUT, 'stage'))
ECH = float(os.environ.get('MH_ECHELLE', '1.0'))
PFX = 'mh_'
TEST = os.environ.get('MH_TEST', 'C')   # A : collisions seules ; B : + décor de Basin 1 ; C : + château caché
print('version de test', TEST)
DOS = 'stc_bg_m_set'

TEX, models = brawl_stage.load(PAC)
SANS = set()
SANS_TEX = {'m01a05d-t-kira3'}   # reflets de la mer (additifs dans Brawl, noirs en opaque)
LUM = float(os.environ.get('MH_LUM', '1.0'))
SAT = float(os.environ.get('MH_SAT', '1.0'))


def ajuste(im):
    """im : tableau RGBA flottant 0-255 -> couleurs désaturées (SAT) et assombries (LUM)"""
    rgb = im[:, :, :3]
    g = (rgb * np.array([0.299, 0.587, 0.114])).sum(axis=2, keepdims=True)
    im[:, :, :3] = np.clip((g + (rgb - g) * SAT) * LUM, 0, 255)
    return im


garde = []
for _, mn, ol in models:
    if mn in SANS:
        print('ignore (modèle)', mn)
        continue
    for o in ol:
        if not o['tex'] or o['tex'] not in TEX or o['tex'] in SANS_TEX:
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


# sol de Basin 1 (Brawl) : dessus, de gauche à droite (plans 12, 11, 10, 9, 8, 7, 0 ... 6)
SOL_HAUT = [(-781.71, -7.52), (-404.32, -7.62), (-392.38, -6.39), (-353.39, 0.47), (-316.89, 1.80), (-227.80, 2.50),
            (-210.00, 2.70), (37.50, 2.70), (40.18, 2.50), (137.85, 2.00), (202.26, 0.39), (246.47, -2.32),
            (317.51, -7.54), (2201.00, -7.74)]


def sol_coll(x):
    return float(np.interp(x, [p[0] for p in SOL_HAUT], [p[1] for p in SOL_HAUT]))


ecarts = []
for x in (-300, -150, 0, 150, 300):
    h = surface(x)
    if h is not None:
        ecarts.append(h - sol_coll(x))
print('décor - collisions (Brawl) à z 0 :', [round(e, 2) for e in ecarts])
vals = [e for e in ecarts if abs(e) < 10]
Y0 = float(os.environ.get('MH_Y0', str(round(float(np.median(vals)), 3) if vals else 0.0)))
print('Y0 =', Y0)


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
    par_tex.setdefault(o['tex'], []).append(o)

if TEST == 'A':
    par_tex = {}
D = os.path.join(ST, 'model', DOS)
os.makedirs(D, exist_ok=True)
objs, labs, entries, nouveaux = [], [], [], []
for k, (t, ol) in enumerate(sorted(par_tex.items())):
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
    mo = A.MeshObjectData('mh_%02d' % k, 0)
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
    lab = 'mh_%s%s' % (t, '_a' if a else '')
    e = S.matl_data.read_matl(MATL_MODELE).entries[TI]
    e.material_label = lab
    for x in e.textures:
        if x.param_id.name == 'Texture5':
            x.data = tname(t)
    for x in e.rasterizer_states:
        d_ = x.data
        d_.cull_mode = S.matl_data.CullMode.Disabled
        x.data = d_
    if a:
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
    hd = os.path.join(os.environ.get('MH_HD', ''), t + '_x4.png')
    if os.environ.get('MH_HD') and os.path.exists(hd):
        im = np.asarray(Image.open(hd).convert('RGBA')).astype(np.float32)
    im = ajuste(im)
    if a:
        im[:, :, :3] *= im[:, :, 3:] / 255.0
    Image.fromarray(np.clip(im + 0.5, 0, 255).astype(np.uint8), 'RGBA').save(png)
    cible = os.path.join(D, tname(t) + '.nutexb')
    subprocess.run(['ultimate_tex_cli', png, cible, '--format', 'BC7RgbaUnormSrgb'], check=True, capture_output=True)
    nouveaux.append(os.path.relpath(cible, OUT).replace(os.sep, '/'))
    os.remove(png)

# scène three.js (aperçus / image de l'écran des terrains), même décor que le mod
if os.environ.get('MH_SCENE'):
    import json as _j
    from PIL import Image
    SC = os.environ['MH_SCENE']
    os.makedirs(os.path.join(SC, 'tex'), exist_ok=True)
    noms, scene = {}, []
    for i, t in enumerate(sorted(par_tex)):
        noms[t] = 't%02d' % i
        hd = os.path.join(os.environ.get('MH_HD', ''), t + '_x4.png')
        im_ = np.asarray((Image.open(hd) if os.environ.get('MH_HD') and os.path.exists(hd) else TEX[t]).convert('RGBA')).astype(np.float32)
        Image.fromarray(np.clip(ajuste(im_) + 0.5, 0, 255).astype(np.uint8), 'RGBA').save(os.path.join(SC, 'tex', noms[t] + '.png'))
    for t, ol in par_tex.items():
        for o in ol:
            p = o['pos'][:, [0, 2, 1]].astype(np.float64)
            p[:, :, 1] -= Y0
            p *= ECH
            u = o['uv'][:, [0, 2, 1]] if o['uv'] is not None else np.zeros(p.shape[:2] + (2,))
            scene.append({'p': np.round(p.reshape(-1), 4).tolist(), 'uv': np.round(u.reshape(-1), 5).tolist(), 't': noms[t], 'a': bool(alpha(t))})
    _j.dump(scene, open(os.path.join(SC, 'scene.json'), 'w'))
    print('scène', len(scene), 'objets')

if TEST == 'A':
    shutil.rmtree(D)
else:
  matl = S.matl_data.read_matl(MATL_MODELE)
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

# décor d'origine : tous les ensembles cachés par un numdlb à 0 entrée (méthode des mods Wii Fit / Smashville qui marchent) ;
# aucun fichier d'un autre mod (v1 : squelettes / matériaux copiés -> plantage au chargement)
vide = S.modl_data.read_modl(os.path.join(RM, DOS, 'model.numdlb'))
vide.entries = []
caches = ['stc_ring_s_set', 'stc_ring_under1_set', 'stc_ring_under_set', 'dyr_cloud01_set', 'dyr_hurinkazan_set', 'floating_plate_set']
for nom in (caches if TEST == 'C' else []):
    os.makedirs(os.path.join(ST, 'model', nom), exist_ok=True)
    vide.save(os.path.join(ST, 'model', nom, 'model.numdlb'))
print('cachés (0 entrée) :', ' '.join(caches) if TEST == 'C' else 'aucun (test %s)' % TEST)

# collisions : sol de Basin 1 tel quel, murs invisibles aux extrémités, plates-formes à -999 (HDR)
import yaml
P_ = os.path.join(ST, 'param')
os.makedirs(P_, exist_ok=True)
tmp = os.path.join(OUT, '_lvd.yml')
subprocess.run(['yamlvd', LVD, tmp], check=True, capture_output=True)
d = yaml.safe_load(open(tmp))
XL, XR = SOL_HAUT[0][0] * ECH, SOL_HAUT[-1][0] * ECH
BZ = {'left': XL - 120.0, 'right': XR + 100.0, 'top': 400.0, 'bottom': -250.0}
CAM = {'left': XL - 100.0, 'right': XR + 80.0, 'top': 330.0, 'bottom': -150.0}
YB, YH = -40.0, BZ['top'] - 10.0
haut = [(x * ECH, y * ECH) for x, y in SOL_HAUT]
pts = [(XR + 10, YB), (XL - 10, YB), (XL - 10, YH), (XL, YH)] + haut + [(XR, YH), (XR + 10, YH), (XR + 10, YB)]
sol = max((c for c in d['collisions'] if not c['col_flags']['drop_through']), key=lambda c: len(c['vertices']))
m_rien = next(m for m in sol['materials'] if not any(m['line_flags'].values()))
sol['vertices'] = [{'x': float(x), 'y': float(y)} for x, y in pts]
nrm = []
for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
    dx, dy = x1 - x0, y1 - y0
    n = (dx * dx + dy * dy) ** 0.5
    nrm.append({'x': float(-dy / n), 'y': float(dx / n)})
sol['normals'] = nrm
sol['materials'] = [m_rien] * len(nrm)
sol['cliffs'] = []
for c in d['collisions']:
    if c is sol:
        continue
    for v in c['vertices']:
        v['y'] = -999.0
    print('plate-forme à -999 :', c['entry']['name'])
for k, v in BZ.items():
    d['blast_zone'][0][k] = v
for k, v in CAM.items():
    d['camera_boundary'][0][k] = v
xs = [-50, 50, -25, 25, -75, 75, -100, 100]
for s_, x in zip(d['spawns'], xs):
    s_['pos']['x'], s_['pos']['y'] = float(x), sol_coll(x) * ECH + 0.1
for r_, x in zip(d['respawns'], xs):
    r_['pos']['x'], r_['pos']['y'] = float(x), 100.0
print('sol x %.0f..%.0f, murs jusqu\'à y %.0f ; caméra %s ; zones de K.O. %s' % (XL, XR, YH, CAM, BZ))
yaml.safe_dump(d, open(tmp, 'w'), sort_keys=False)
subprocess.run(['yamlvd', tmp, os.path.join(P_, 'streetfighter_f00.lvd')], check=True, capture_output=True)
os.remove(tmp)

import json
cfg = {'new-dir-files': {}}
for f in nouveaux:
    cfg['new-dir-files'].setdefault(os.path.dirname(f), []).append(f)
json.dump(cfg, open(os.path.join(OUT, 'config.json'), 'w'), indent=2)
print('nouvelles textures', len(nouveaux))
