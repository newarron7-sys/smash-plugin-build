"""Unreal Introduction Map (Brawl, GameBanana 692367) en variante Stage Alts de Smashville (stage/animal_village,
forme normal_sNN), v1. Le Champ de bataille utilise ses modèles au démarrage (plates-formes) ; Smashville a un décor
fixe où cacher les ensembles par un numdlb à 0 entrée est validé (Sleepless Lamp).

usage : python3 ue_sv.py <STGBATTLEFIELD (DOLPHIN).pac> <réf complète .../stage/animal_village (387162)>
                         <liste des fichiers du jeu stage/animal_village/normal/ (Hashes_FullPath)>
                         <numatb modèle (bg2_set de 392643)> <xvillage00.lvd HDR (normal)> <sortie mod>

Méthode de Sleepless Lamp (lamp_sv.py) dans un dossier de variante (générateur de CSK) :
- décor de la carte Unreal dans stc_bg_set (numshb / numdlb / numatb / numshexb de 387162 comme base ; squelette du jeu
  partagé), maillages regroupés par texture, sans os, matériau « Bottom » ; textures sous de nouveaux noms ;
- tous les autres ensembles cachés par le numdlb à 0 entrée de 387162 (comme Lamp) ;
- collisions de la carte (3 niveaux de sol, rebords aux deux bouts, bloc solide) dans le lvd de Smashville d'HDR ;
  plate-forme mobile enterrée (comme Lamp) ;
- config.json : share_to_vanilla de chaque fichier du jeu de la forme normal non fourni ; image de l'écran des terrains
  stage_1/2/4 fournies, 0 et 3 partagées.
"""
import os, shutil, subprocess, sys
import numpy as np
import ssbh_data_py as S

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import brawl_stage  # noqa: E402

PAC, REF, LISTE, MATL_MODELE, LVD, OUT = sys.argv[1:7]
A = S.mesh_data
FORME = os.environ.get('UE_FORME', 'normal_s01')
RM = os.path.join(REF, 'normal', 'model')
ST = os.path.join(OUT, 'stage', 'animal_village', FORME)
if os.path.isdir(os.path.join(OUT, 'stage')):
    shutil.rmtree(os.path.join(OUT, 'stage'))
ECH = float(os.environ.get('UE_ECHELLE', '1.0'))
PFX = 'ue_'
DOS = 'stc_bg_set'

TEX, models = brawl_stage.load(PAC)
SANS = {'EffCommonShield', 'Dummy', 'MShadow'}
SANS_TEX = set()
LUM = float(os.environ.get('UE_LUM', '1.0'))
SAT = float(os.environ.get('UE_SAT', '1.0'))


# v2 : textures éclaircies (UE_LUM) pour retrouver les couleurs des images de référence du joueur (gris ~200, bleu
# ~ (83, 152, 208) au soleil) ; ciel et nuages gardés tels quels
SAUF = set(x for x in os.environ.get('UE_LUM_SAUF', '').split(',') if x)


def ajuste(im, t=None):
    """im : tableau RGBA flottant 0-255 -> saturation (SAT) et luminosité (LUM)"""
    if t in SAUF:
        return im
    rgb = im[:, :, :3]
    g = (rgb * np.array([0.299, 0.587, 0.114])).sum(axis=2, keepdims=True)
    im[:, :, :3] = np.clip((g + (rgb - g) * SAT) * LUM, 0, 255)
    return im


# v2 : faux soleil cuit dans colorSet1 (0,502 = neutre) : dessus clair, côtés plus sombres
SOLEIL = np.array([float(v) for v in os.environ.get('UE_SOLEIL', '-0.3,1,0.35').split(',')])
SOLEIL /= np.linalg.norm(SOLEIL)
OMBRE = float(os.environ.get('UE_OMBRE', '1.0'))      # 1 = pas d'ombrage ; 0,55 = côtés à l'ombre à 55 %


def ombrage(n):
    """n : normales (N, 3) Ultimate (y en haut) -> facteur 0..1 (dessus = 1)"""
    d = np.clip(n @ SOLEIL, 0, None) / SOLEIL[1]
    return np.clip(OMBRE + (1 - OMBRE) * d, 0, 1)


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

# v3 : carte d'ombres d'origine (BakeMap de la carte Brawl, 2e texture + 2e jeu d'UV de Blue / Grid) dans un matériau
# du jeu à texture d'ombre : Texture0 (couleur, map1) + Texture9 (éclairage cuit, bake1), modèle pris dans
# stc_bg_set de 387162 (même dossier que le décor)
CUIT = os.environ.get('UE_CUIT', '1') == '1'
GAB_CUIT = None
if CUIT:
    ATTR_CUIT = None
    _rm = S.matl_data.read_matl(os.path.join(RM, DOS, 'model.numatb'))
    _dl = S.modl_data.read_modl(os.path.join(RM, DOS, 'model.numdlb'))
    _sh = A.read_mesh(os.path.join(RM, DOS, 'model.numshb'))
    _obj = {(o.name, o.subindex): o for o in _sh.objects}
    for pref in ('SFX_PBS_0000000008088280', 'SFX_PBS_0000000000088280', 'SFX_PBS_0000000000080280', 'SFX_PBS_0000000008080280'):
        for e in _rm.entries:
            if not (e.shader_label.startswith(pref) and sorted(t.param_id.name for t in e.textures) == ['Texture0', 'Texture9']):
                continue
            utilis = [(en.mesh_object_name, en.mesh_object_subindex) for en in _dl.entries if en.material_label == e.material_label]
            o = next((_obj[k] for k in utilis if k in _obj), None)
            if o is not None:
                GAB_CUIT = e
                ATTR_CUIT = ([x.name for x in o.texture_coordinates], [x.name for x in o.color_sets])
                break
        if GAB_CUIT is not None:
            break
    print('matériau à texture d\'ombre :', GAB_CUIT.shader_label if GAB_CUIT else None, GAB_CUIT.material_label if GAB_CUIT else None, ATTR_CUIT)

# regroupement par texture
par_tex = {}
for mn, o in garde:
    par_tex.setdefault(o['tex'], []).append(o)

# autres ensembles cachés par le numdlb à 0 entrée de 387162 (méthode de Sleepless Lamp, validée sur Smashville)
vide = os.path.join(RM, 'dys_guest_asami_set', 'model.numdlb')
assert len(S.modl_data.read_modl(vide).entries) == 0
ens = set(d for d in os.listdir(RM) if os.path.isdir(os.path.join(RM, d)))
for l in open(LISTE):
    p = l.strip()
    if p.startswith('stage/animal_village/normal/model/'):
        ens.add(p.split('/')[4])
caches = sorted(ens - {DOS})
for d_ in caches:
    os.makedirs(os.path.join(ST, 'model', d_), exist_ok=True)
    shutil.copy(vide, os.path.join(ST, 'model', d_, 'model.numdlb'))
print('cachés (0 entrée) :', len(caches), ' '.join(caches))
BN = os.path.join(REF, 'normal')
# v2 : toujours le jour (Smashville suit l'horloge de la console) : fichiers render/ d'un mod « toujours le jour »
# (Islandville 382440 : la même série d'éclairage copiée sur matin / jour / soir / nuit, comme Siege's Smashville)
JOUR = os.environ.get('UE_JOUR', '')
if JOUR and os.path.isdir(JOUR):
    shutil.copytree(JOUR, os.path.join(ST, 'render'))
    print('render : jour permanent', sorted(os.listdir(os.path.join(ST, 'render'))))
D = os.path.join(ST, 'model', DOS)
os.makedirs(D, exist_ok=True)
objs, labs, entries, nouveaux = [], [], [], []
for k, (t, ol) in enumerate(sorted(par_tex.items())):
    pos = np.concatenate([o['pos'][:, [0, 2, 1]] for o in ol]).reshape(-1, 3).astype(np.float64)
    nrm = np.concatenate([o['nrm'][:, [0, 2, 1]] for o in ol]).reshape(-1, 3)
    uv = np.concatenate([(o['uv'][:, [0, 2, 1]] if o['uv'] is not None else np.zeros(o['pos'].shape[:2] + (2,)))
                         for o in ol]).reshape(-1, 2)
    cuit = GAB_CUIT is not None and ATTR_CUIT is not None and all(o.get('uv1') is not None and 'BakeMap' in o.get('texs', []) for o in ol)
    uv1 = np.concatenate([(o['uv1'][:, [0, 2, 1]] if o.get('uv1') is not None else np.zeros(o['pos'].shape[:2] + (2,)))
                          for o in ol]).reshape(-1, 2)
    key = np.round(np.c_[pos, nrm * 100, uv * 4096, uv1 * 4096], 3)
    uk, inv = np.unique(key, axis=0, return_inverse=True)
    first = np.zeros(len(uk), dtype=np.int64)
    first[inv.ravel()[::-1]] = np.arange(len(inv.ravel()))[::-1]
    pos, nrm, uv, uv1 = pos[first], nrm[first], uv[first], uv1[first]
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
    if cuit:
        mo.texture_coordinates = [attr(n_, uv1 if n_ == 'bake1' else uv) for n_ in ATTR_CUIT[0]]
        mo.color_sets = [attr(n_, np.full((nv, 4), 0.502)) for n_ in ATTR_CUIT[1]]
    else:
        mo.texture_coordinates = [attr('map1', uv), attr('bake1', uv), attr('uvSet', uv)]
        c1 = np.full((nv, 4), 0.502)
        c1[:, :3] *= ombrage(nrm)[:, None]
        mo.color_sets = [attr('colorSet1', c1), attr('colorSet4', np.full((nv, 4), 0.502))]
    mo.vertex_indices = idx
    a = alpha(t)
    lab = 'ue_%s%s%s' % (t, '_a' if a else '', '_cuit' if cuit else '')
    if cuit:
        e = next(x for x in S.matl_data.read_matl(os.path.join(RM, DOS, 'model.numatb')).entries
                 if x.material_label == GAB_CUIT.material_label)
    else:
        e = S.matl_data.read_matl(MATL_MODELE).entries[TI]
    e.material_label = lab
    for x in e.textures:
        if x.param_id.name in ('Texture5', 'Texture0'):
            x.data = tname(t)
        elif x.param_id.name == 'Texture9':
            x.data = tname('BakeMap')
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
    im = ajuste(im, t) if not cuit else im
    if a:
        im[:, :, :3] *= im[:, :, 3:] / 255.0
    Image.fromarray(np.clip(im + 0.5, 0, 255).astype(np.uint8), 'RGBA').save(png)
    cible = os.path.join(D, tname(t) + '.nutexb')
    subprocess.run(['ultimate_tex_cli', png, cible, '--format', 'BC7RgbaUnormSrgb'], check=True, capture_output=True)
    os.remove(png)

if any(l.endswith('_cuit') for l in labs):
    from PIL import Image
    png = os.path.join(OUT, '_tmp.png')
    # v4 : en jeu, beaucoup trop lumineux (soleil de Smashville + éclairage cuit + textures déjà éclaircies) ->
    # textures d'origine pour les objets à texture d'ombre et carte d'ombres assombrie (UE_CUIT_LUM)
    bk = np.asarray(TEX['BakeMap'].convert('RGBA')).astype(np.float32)
    bk[:, :, :3] *= float(os.environ.get('UE_CUIT_LUM', '1.0'))
    Image.fromarray(np.clip(bk + 0.5, 0, 255).astype(np.uint8), 'RGBA').save(png)
    subprocess.run(['ultimate_tex_cli', png, os.path.join(D, tname('BakeMap') + '.nutexb'), '--format', 'BC7RgbaUnormSrgb'],
                   check=True, capture_output=True)
    os.remove(png)
    print('texture d\'ombre :', tname('BakeMap'), TEX['BakeMap'].size, [l for l in labs if l.endswith('_cuit')])

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
        Image.fromarray(np.clip(ajuste(im_, t) + 0.5, 0, 255).astype(np.uint8), 'RGBA').save(os.path.join(SC, 'tex', noms[t] + '.png'))
    TEX['BakeMap'].convert('RGBA').save(os.path.join(SC, 'tex', 'bake.png'))
    for t, ol in par_tex.items():
        for o in ol:
            p = o['pos'][:, [0, 2, 1]].astype(np.float64)
            p[:, :, 1] -= Y0
            p *= ECH
            u = o['uv'][:, [0, 2, 1]] if o['uv'] is not None else np.zeros(p.shape[:2] + (2,))
            el = {'p': np.round(p.reshape(-1), 4).tolist(), 'uv': np.round(u.reshape(-1), 5).tolist(), 't': noms[t], 'a': bool(alpha(t))}
            if GAB_CUIT is not None and o.get('uv1') is not None and 'BakeMap' in o.get('texs', []):
                el['b'] = 'bake'
                el['u1'] = np.round(o['uv1'][:, [0, 2, 1]].reshape(-1), 5).tolist()
            else:
                nn = o['nrm'][:, [0, 2, 1]].reshape(-1, 3)
                nn = nn / np.maximum(np.linalg.norm(nn, axis=1, keepdims=True), 1e-6)
                el['c'] = np.round(np.repeat(ombrage(nn)[:, None], 3, axis=1).reshape(-1), 3).tolist()
            scene.append(el)
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

# collisions : sol de la carte, rebords aux deux bouts, bloc solide ; lvd de Smashville d'HDR
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
sol = next(c for c in d['collisions'] if c['entry']['name'] == 'COL_00_Floor01')
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
for c in autres:              # plate-forme mobile : petit segment enterré (comme Lamp ; la retirer peut faire planter)
    c['vertices'] = [{'x': -5.0, 'y': -45.0}, {'x': 5.0, 'y': -45.0}]
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
UI_FOURNIS = [int(x) for x in os.environ.get('UE_UI_FOURNIS', '1,2,4').split(',') if x]
for n in range(5):
    if n not in UI_FOURNIS:
        partage['ui/replace/stage/stage_%d/stage_%d_animal_village.bntx' % (n, n)] = \
            ['ui/replace/stage/stage_%d/stage_%d_animal_village_%s.bntx' % (n, n, FORME.split('_')[-1])]
json.dump({'share_to_vanilla': partage}, open(os.path.join(OUT, 'config.json'), 'w'), indent=2)
print('fournis', len(fournis), '; partagés avec le jeu', len(partage))
