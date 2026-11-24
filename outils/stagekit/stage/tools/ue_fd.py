"""Unreal Introduction Map (Brawl, GameBanana 692367, BraveDragonWolf, version Dolphin) sur la Destination Finale
(stage/end), v1.

usage : python3 ue_fd.py <STGBATTLEFIELD (DOLPHIN).pac> <réf « Brawl Final Destination » 454060 .../stage/end>
                         <config.json de 454060> <numatb modèle (bg2_set de 392643)> <battlefield_00.lvd HDR (modèle de lvd)> <sortie mod>

Structure de « Brawl Final Destination » (454060, mod de la Destination Finale qui remplace tout le décor) :
- tout ce qui n'est pas un modèle copié tel quel (effet, motion, render) ; paysages cachés par ses numdlb / nusrcmdlb ;
- ring_set : décor de la carte Unreal (maillages regroupés par texture, sans os ; matériau « Bottom » sans éclairage),
  nusktb / nuhlpb / xmb / nusrcmdlb de 454060 ;
- ring_decorations_set : fichiers complets de 454060 (textures comprises), sommets ramenés en un point sous le sol ;
- collisions : celles de la carte (3 niveaux de sol + bloc solide en haut à gauche), rebords aux deux bouts (lignes
  hang_l / hang_r comme le Studio Wii Fit), dans un lvd fait sur le modèle de celui du Champ de bataille d'HDR.
"""
import os, shutil, subprocess, sys
import numpy as np
import ssbh_data_py as S

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import brawl_stage  # noqa: E402

PAC, REF, CFG_REF, MATL_MODELE, LVD, OUT = sys.argv[1:7]
A = S.mesh_data
RM = os.path.join(REF, 'normal', 'model')
ST = os.path.join(OUT, 'stage', 'end', 'normal')
if os.path.isdir(os.path.join(OUT, 'stage')):
    shutil.rmtree(os.path.join(OUT, 'stage'))
ECH = float(os.environ.get('UE_ECHELLE', '1.0'))
PFX = 'ue_'
TEST = 'C'
DOS = 'ring_set'

TEX, models = brawl_stage.load(PAC)
SANS = {'EffCommonShield', 'Dummy', 'MShadow'}
SANS_TEX = set()
LUM = float(os.environ.get('UE_LUM', '1.0'))
SAT = float(os.environ.get('UE_SAT', '1.0'))


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


# sol de la carte (Brawl, plans 8, 7, 0 ... 6) de gauche à droite
SOL_HAUT = [(-213.48, 11.44), (-110.73, 11.44), (-87.29, 0.0), (111.85, 0.0), (159.06, -23.60), (180.64, -23.77),
            (182.61, -23.01), (216.80, -23.01), (218.89, -23.82), (266.65, -23.82)]
BLOC = [(-110.87, 52.08), (-214.10, 52.08), (-216.79, 70.49), (-110.87, 70.49), (-110.87, 52.08)]   # plans 9-11


def sol_coll(x):
    return float(np.interp(x, [p[0] for p in SOL_HAUT], [p[1] for p in SOL_HAUT]))


ecarts = []
for x in (-180, -50, 0, 60, 240):
    h = surface(x)
    if h is not None:
        ecarts.append(h - sol_coll(x))
print('décor - collisions (Brawl) à z 0 :', [round(e, 2) for e in ecarts])
vals = [e for e in ecarts if abs(e) < 10]
Y0 = float(os.environ.get('UE_Y0', str(round(float(np.median(vals)), 3) if vals else 0.0)))
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

# 454060 : tout sauf les maillages tel quel ; ring_decorations_set complet (textures comprises) ramené en un point
BN = os.path.join(REF, 'normal')
for racine, _, fichiers in os.walk(BN):
    rel = os.path.relpath(racine, BN)
    if rel.startswith('model') or rel.startswith('param'):
        continue
    for f in fichiers:
        if f.lower() == 'desktop.ini':
            continue
        os.makedirs(os.path.join(ST, rel), exist_ok=True)
        shutil.copy(os.path.join(racine, f), os.path.join(ST, rel, f))
for d_ in sorted(os.listdir(os.path.join(BN, 'model'))):
    src = os.path.join(BN, 'model', d_)
    dst = os.path.join(ST, 'model', d_)
    os.makedirs(dst, exist_ok=True)
    for f in os.listdir(src):
        if f.lower() == 'desktop.ini':
            continue
        if d_ == DOS and (f.endswith('.nutexb') or f in ('model.numshb', 'model.numatb', 'model.numdlb', 'model.numshexb')):
            continue
        shutil.copy(os.path.join(src, f), dst)
    if d_ == 'ring_decorations_set':
        m = A.read_mesh(os.path.join(src, 'model.numshb'))
        for o in m.objects:
            p = np.asarray(o.positions[0].data, dtype=np.float32)
            q = np.zeros_like(p)
            q[:, 1] = -100.0
            o.positions[0].data = q
        m.save(os.path.join(dst, 'model.numshb'))
        S.meshex_data.MeshExData.from_mesh_objects(m.objects).save(os.path.join(dst, 'model.numshexb'))
        print('ring_decorations_set (454060) ramené en un point :', len(m.objects), 'objets')
print('copié de 454060 :', sorted(os.listdir(os.path.join(BN, 'model'))))
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
    mo = A.MeshObjectData('ue_%02d' % k, 0)
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
    lab = 'ue_%s%s' % (t, '_a' if a else '')
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
    hd = os.path.join(os.environ.get('UE_HD', ''), t + '_x4.png')
    if os.environ.get('UE_HD') and os.path.exists(hd):
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
if os.environ.get('UE_SCENE'):
    import json as _j
    from PIL import Image
    SC = os.environ['UE_SCENE']
    os.makedirs(os.path.join(SC, 'tex'), exist_ok=True)
    noms, scene = {}, []
    for i, t in enumerate(sorted(par_tex)):
        noms[t] = 't%02d' % i
        hd = os.path.join(os.environ.get('UE_HD', ''), t + '_x4.png')
        im_ = np.asarray((Image.open(hd) if os.environ.get('UE_HD') and os.path.exists(hd) else TEX[t]).convert('RGBA')).astype(np.float32)
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

if False:
    pass
else:
  matl = S.matl_data.read_matl(MATL_MODELE)
  matl.entries = entries
  matl.save(os.path.join(D, 'model.numatb'))
  mesh = A.read_mesh(os.path.join(BN, 'model', DOS, 'model.numshb'))
  mesh.objects = objs
  mesh.save(os.path.join(D, 'model.numshb'))
  modl = S.modl_data.read_modl(os.path.join(BN, 'model', DOS, 'model.numdlb'))
  modl.entries = [S.modl_data.ModlEntryData(o.name, o.subindex, l) for o, l in zip(objs, labs)]
  modl.save(os.path.join(D, 'model.numdlb'))
  S.meshex_data.MeshExData.from_mesh_objects(objs).save(os.path.join(D, 'model.numshexb'))
  print('DOSSIER', DOS, len(objs), 'objets', sum(len(o.vertex_indices) // 3 for o in objs), 'triangles')

# collisions : sol de la carte, rebords aux deux bouts, bloc solide ; lvd fait sur le modèle du Champ de bataille d'HDR
import yaml, copy
P_ = os.path.join(ST, 'param')
os.makedirs(P_, exist_ok=True)
tmp = os.path.join(OUT, '_lvd.yml')
subprocess.run(['yamlvd', LVD, tmp], check=True, capture_output=True)
d = yaml.safe_load(open(tmp))
YB = -60.0
haut = [(x * ECH, y * ECH) for x, y in SOL_HAUT]
(x0, y0), (x1, y1) = haut[0], haut[-1]
haut = [haut[0], (x0 + 5, y0)] + haut[1:-1] + [(x1 - 5, y1), haut[-1]]
pts = [(x1, YB), (x0, YB)] + haut + [(x1, YB)]
sol = next(c for c in d['collisions'] if not c['col_flags']['drop_through'])
m_rien = copy.deepcopy(sol['materials'][0])
for k in m_rien['line_flags']:
    m_rien['line_flags'][k] = False
m_l, m_r = copy.deepcopy(m_rien), copy.deepcopy(m_rien)
m_l['line_flags']['hang_l'] = True
m_r['line_flags']['hang_r'] = True


def poly(c, pts, mats):
    c['vertices'] = [{'x': float(x), 'y': float(y)} for x, y in pts]
    nrm = []
    for (a0, b0), (a1, b1) in zip(pts, pts[1:]):
        dx, dy = a1 - a0, b1 - b0
        n = (dx * dx + dy * dy) ** 0.5
        nrm.append({'x': float(-dy / n), 'y': float(dx / n)})
    c['normals'] = nrm
    c['materials'] = mats
    c['cliffs'] = []


n_l = len(pts) - 1
i_l = 2                         # segment (x0, y0) -> (x0 + 5, y0)
i_r = n_l - 2                   # segment (x1 - 5, y1) -> (x1, y1)
poly(sol, pts, [m_l if i == i_l else m_r if i == i_r else m_rien for i in range(n_l)])
bloc = copy.deepcopy(sol)
bloc['entry']['name'] = 'COL_00_Block01'
bloc['entry']['subname'] = 'Block01'
bp = [(x * ECH, y * ECH) for x, y in BLOC]
poly(bloc, bp, [copy.deepcopy(m_rien) for _ in range(len(bp) - 1)])
autres = [c for c in d['collisions'] if c is not sol]
for c in autres:              # plates-formes du Champ de bataille : petits segments dans le sol (les retirer peut faire planter)
    c['vertices'] = [{'x': -20.0, 'y': -40.0}, {'x': 20.0, 'y': -40.0}]
    c['normals'] = [{'x': 0.0, 'y': 1.0}]
    print('enterrée :', c['entry']['name'])
d['collisions'] = [sol, bloc] + autres
BZ = {'left': x0 - 130.0, 'right': x1 + 130.0, 'top': 260.0, 'bottom': -170.0}
CAM = {'left': x0 - 90.0, 'right': x1 + 90.0, 'top': 190.0, 'bottom': -110.0}
for k, v in BZ.items():
    d['blast_zone'][0][k] = v
for k, v in CAM.items():
    d['camera_boundary'][0][k] = v
xs = [-60, 60, -20, 20, -80, 100, -40, 40]
for s_, x in zip(d['spawns'], xs):
    s_['pos']['x'], s_['pos']['y'] = float(x), sol_coll(x) * ECH + 0.1
for r_, x in zip(d['respawns'], xs):
    r_['pos']['x'], r_['pos']['y'] = float(x), 100.0
print('sol x %.1f..%.1f (rebords lignes %d et %d), bloc ; caméra %s ; zones de K.O. %s' % (x0, x1, i_l, i_r, CAM, BZ))
yaml.safe_dump(d, open(tmp, 'w'), sort_keys=False)
subprocess.run(['yamlvd', tmp, os.path.join(P_, 'end_00.lvd')], check=True, capture_output=True)
os.remove(tmp)

import json
cfg = {'new-dir-files': {}}
for f in nouveaux:
    cfg['new-dir-files'].setdefault(os.path.dirname(f), []).append(f)
ref_cfg = json.load(open(CFG_REF)).get('new-dir-files', {})
for k, v in ref_cfg.items():
    if k.endswith('ring_decorations_set'):
        cfg['new-dir-files'].setdefault(k, []).extend(v)
json.dump(cfg, open(os.path.join(OUT, 'config.json'), 'w'), indent=2)
print('nouvelles textures', len(nouveaux))
