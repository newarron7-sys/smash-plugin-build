"""Unreal Introduction Map (Brawl, GameBanana 692367) en variante Stage Alts du Champ de bataille (stage/battlefield,
forme normal_s02, touche R sur l'écran des terrains d'HDR), v2 (la v1 sur la Destination Finale plantait au chargement :
lvd du Champ de bataille sur la DF).

usage : python3 ue_bf.py <STGBATTLEFIELD (DOLPHIN).pac> <hdr-stages/stage/battlefield (HDR)> <liste des fichiers du jeu
                         stage/battlefield/normal/ (Hashes_FullPath)> <numatb modèle (bg2_set de 392643)> <sortie mod>

Méthode (Stage Alts, générateur de CSK) : dossier stage/battlefield/normal_s02 ; tout fichier de la forme non fourni est
partagé avec le jeu (share_to_vanilla). Fournis :
- ring_nocastshadow_set : décor de la carte Unreal (maillages regroupés par texture, sans os ; matériau « Bottom »),
  numshb / numdlb / nusktb d'HDR comme base, textures sous de nouveaux noms (comme les variantes d'HDR) ;
- floating_plate_set : fichiers d'HDR (textures comprises), sommets en un point (0, -25, 0) dans le sol (méthode validée) ;
- fond, effets de l'anneau, drapeaux, plantes, dessous des plates-formes : numdlb + numatb vides (méthode Neonfield) ;
- param : battlefield_00.lvd (collisions de la carte, fait sur le lvd du Champ de bataille d'HDR = même terrain),
  battlefield.stdat d'HDR.
"""
import os, shutil, subprocess, sys
import numpy as np
import ssbh_data_py as S

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import brawl_stage  # noqa: E402

PAC, HBF, LISTE, MATL_MODELE, OUT = sys.argv[1:6]
A = S.mesh_data
FORME = os.environ.get('UE_FORME', 'normal_s02')
HN = os.path.join(HBF, 'normal')
LVD = os.path.join(HN, 'param', 'battlefield_00.lvd')
ST = os.path.join(OUT, 'stage', 'battlefield', FORME)
if os.path.isdir(os.path.join(OUT, 'stage')):
    shutil.rmtree(os.path.join(OUT, 'stage'))
ECH = float(os.environ.get('UE_ECHELLE', '1.0'))
PFX = 'ue_'
TEST = 'C'
DOS = 'ring_nocastshadow_set'
VIDES = ['bg_birds_set', 'bg_falls_set', 'bg_far_skycity_set', 'bg_ground_set', 'bg_skydome_set', 'bg_solar_eclipse_line_set',
         'bg_solar_eclipse_set', 'ring_effects_set', 'ring_effects_r_set', 'ring_flag_set', 'ring_plant_set', 'floating_plate_sub_set']

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

# fichiers fournis (le reste est partagé avec le jeu par config.json)
MD = os.path.join(ST, 'model')
# plates-formes : fichiers d'HDR, sommets en un point dans le sol
src = os.path.join(HN, 'model', 'floating_plate_set')
dst = os.path.join(MD, 'floating_plate_set')
os.makedirs(dst, exist_ok=True)
for f in os.listdir(src):
    shutil.copy(os.path.join(src, f), dst)
m = A.read_mesh(os.path.join(src, 'model.numshb'))
for o in m.objects:
    q = np.zeros_like(np.asarray(o.positions[0].data, dtype=np.float32))
    q[:, 1] = -25.0
    o.positions[0].data = q
m.save(os.path.join(dst, 'model.numshb'))
S.meshex_data.MeshExData.from_mesh_objects(m.objects).save(os.path.join(dst, 'model.numshexb'))
print('floating_plate_set (HDR) en un point :', len(m.objects), 'objets,', sorted(os.listdir(dst)))
# fond caché : numdlb + numatb vides
gab_dl = os.path.join(HN, 'model', 'bg_ground_set', 'model.numdlb')
gab_ma = os.path.join(HN, 'model', 'bg_ground_set', 'model.numatb')
for d_ in VIDES:
    os.makedirs(os.path.join(MD, d_), exist_ok=True)
    md = S.modl_data.read_modl(gab_dl)
    md.entries = []
    md.save(os.path.join(MD, d_, 'model.numdlb'))
    ma = S.matl_data.read_matl(gab_ma)
    ma.entries = []
    ma.save(os.path.join(MD, d_, 'model.numatb'))
print('vides :', VIDES)
# param : stdat d'HDR
os.makedirs(os.path.join(ST, 'param'), exist_ok=True)
shutil.copy(os.path.join(HN, 'param', 'battlefield.stdat'), os.path.join(ST, 'param', 'battlefield.stdat'))
BN = HN
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
  shutil.copy(os.path.join(BN, 'model', DOS, 'model.nusktb'), os.path.join(D, 'model.nusktb'))
  print('DOSSIER', DOS, len(objs), 'objets', sum(len(o.vertex_indices) // 3 for o in objs), 'triangles')

# collisions : sol de la carte, rebords aux deux bouts, bloc solide ; lvd du Champ de bataille d'HDR (même terrain)
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
subprocess.run(['yamlvd', tmp, os.path.join(P_, 'battlefield_00.lvd')], check=True, capture_output=True)
os.remove(tmp)

import json
# config.json (Stage Alts) : chaque fichier du jeu de stage/battlefield/normal non fourni est partagé vers la variante
fournis = set()
for racine, _, fichiers in os.walk(ST):
    for f in fichiers:
        fournis.add(os.path.relpath(os.path.join(racine, f), OUT).replace(os.sep, '/'))
partage = {}
for l in open(LISTE):
    p = l.strip()
    if not p.startswith('stage/battlefield/normal/'):
        continue
    v = p.replace('stage/battlefield/normal/', 'stage/battlefield/%s/' % FORME, 1)
    if v not in fournis:
        partage[p] = [v]
UI_FOURNIS = [int(x) for x in os.environ.get('UE_UI_FOURNIS', '1,2,4').split(',') if x]
for n in range(5):
    if n not in UI_FOURNIS:
        partage['ui/replace/stage/stage_%d/stage_%d_battlefield.bntx' % (n, n)] = \
            ['ui/replace/stage/stage_%d/stage_%d_battlefield_%s.bntx' % (n, n, FORME.split('_')[-1])]
json.dump({'share_to_vanilla': partage}, open(os.path.join(OUT, 'config.json'), 'w'), indent=2)
print('fournis', len(fournis), '; partagés avec le jeu', len(partage))
