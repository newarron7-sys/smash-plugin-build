"""BSL Station Main Deck (Brawl / P+ FD 1:1, GameBanana 406762, YorchUZW) sur le Studio Wii Fit (stage/wiifit), v1.

usage : python3 bsl_wf.py <STGBSLSTATION_FD.pac> <réf « Legal » .../stage/wiifit (319081)> <réf Bowser's Keep .../stage/wiifit (482979)>
                          <réf mvc2_training .../stage/wiifit (476217)> <numatb modèle (bg2_set de 392643)> <wiifit00.lvd HDR> <sortie mod>

Méthode de Sleepless Lamp (lamp_sv.py) + structure des mods qui remplacent tout le Studio Wii Fit (Bowser's Keep 482979,
mvc2_training 476217) :
- décor de BSL dans stc_floor_set (numshb / numdlb / numatb / numshexb ; squelette d'origine), regroupé par texture ;
- cachés par un numdlb à 0 entrée, exactement comme ces deux mods : la prof et son reflet, la Balance Board, le vélo,
  les machines, les murs, la salle, les effets, la bâche ;
- plateau du stage (dyr_board01_hmdl_set) gardé entier mais tous ses sommets ramenés en un point sous le sol ;
- objets posés par animation dans Brawl (portes x3 au fond) : première image des CHR0 appliquée (BRAWL_CHR0=1) ;
- vitres et reflets retirés (vue sur SR388) ; couleurs désaturées et décor assombri (BSL_SAT / BSL_LUM) ;
- pas de SA-X ni de parasites X (animés dans Brawl, couchés / figés sans leur animation), pas d'ombres TShadow1 ;
- collisions : celles de BSL (sol plat ±85,56, rebords) posées dans le lvd du Studio Wii Fit d'HDR.
"""
import os, shutil, subprocess, sys
import numpy as np
import ssbh_data_py as S

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import brawl_stage  # noqa: E402

PAC, REF, REF_BK, REF_MVC, MATL_MODELE, LVD, OUT = sys.argv[1:8]
A = S.mesh_data
RM = os.path.join(REF, 'normal', 'model')
ST = os.path.join(OUT, 'stage', 'wiifit', 'normal')
if os.path.isdir(os.path.join(OUT, 'stage')):
    shutil.rmtree(os.path.join(OUT, 'stage'))
ECH = float(os.environ.get('BSL_ECHELLE', '1.0'))
PFX = 'bsl_'
DOS = 'stc_floor_set'

TEX, models = brawl_stage.load(PAC)
SANS = {'SA-X', 'SA-X_shadow', 'X', 'X2', 'X3', 'X4', 'Shadow', 'SR388'}   # animés dans Brawl / ombres multiplicatives / planète SR388 verte sous le stage (demande du joueur)
SANS_TEX = {'2802kd_glass01', '2802kd_rifrect01'}   # vitres / reflets (transparents ou additifs dans Brawl) : vitres ouvertes sur SR388
LUM = float(os.environ.get('BSL_LUM', '0.8'))        # éclairage du décor baissé (demande du joueur)
SAT = float(os.environ.get('BSL_SAT', '0.8'))        # couleurs moins vives (demande du joueur)


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


hs = [surface(x) for x in (-70, -35, 0, 35, 70)]
print('sol visible (Brawl) à z 0 :', hs)
vals = [h for h in hs if h is not None and abs(h) < 10]
Y0 = float(os.environ.get('BSL_Y0', str(round(float(np.median(vals)), 3) if vals else 0.0)))
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
    mo = A.MeshObjectData('bsl_%02d' % k, 0)
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
    lab = 'bsl_%s%s' % (t, '_a' if a else '')
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
    hd = os.path.join(os.environ.get('BSL_HD', ''), t + '_x4.png')
    if os.environ.get('BSL_HD') and os.path.exists(hd):
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
if os.environ.get('BSL_SCENE'):
    import json as _j
    from PIL import Image
    SC = os.environ['BSL_SCENE']
    os.makedirs(os.path.join(SC, 'tex'), exist_ok=True)
    noms, scene = {}, []
    for i, t in enumerate(sorted(par_tex)):
        noms[t] = 't%02d' % i
        hd = os.path.join(os.environ.get('BSL_HD', ''), t + '_x4.png')
        im_ = np.asarray((Image.open(hd) if os.environ.get('BSL_HD') and os.path.exists(hd) else TEX[t]).convert('RGBA')).astype(np.float32)
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

# cachés par un numdlb à 0 entrée (comme Bowser's Keep / mvc2_training)
vide = os.path.join(REF_BK, 'normal', 'model', 'dys_trainer_set', 'model.numdlb')
assert len(S.modl_data.read_modl(vide).entries) == 0
caches = ['dys_balancer_set', 'dys_trainer_far_set', 'dys_trainer_set', 'dys_trainerm_far_set', 'dys_trainerm_set',
          'exercisebike_set', 'stc_cover_set', 'stc_eff_set', 'stc_machine_set', 'stc_room_set', 'stc_wall_set']
for d_ in caches:
    os.makedirs(os.path.join(ST, 'model', d_), exist_ok=True)
    shutil.copy(vide, os.path.join(ST, 'model', d_, 'model.numdlb'))
print('cachés (0 entrée) :', ' '.join(caches))
# plateau : modèle entier (fichiers de mvc2_training) aux sommets ramenés en (0, -30, 0)
PL = os.path.join(REF_MVC, 'normal', 'model', 'dyr_board01_hmdl_set')
DPL = os.path.join(ST, 'model', 'dyr_board01_hmdl_set')
os.makedirs(DPL, exist_ok=True)
for f in ('model.numdlb', 'model.numatb'):
    shutil.copy(os.path.join(PL, f), DPL)
m = A.read_mesh(os.path.join(PL, 'model.numshb'))
for o in m.objects:
    p = np.asarray(o.positions[0].data, dtype=np.float32)
    q = np.zeros_like(p)
    q[:, 1] = -30.0
    o.positions[0].data = q
m.save(os.path.join(DPL, 'model.numshb'))
print('plateau ramené en un point :', len(m.objects), 'objets')

# collisions : sol de BSL (Brawl, plans 0-14 de l'objet 0), rebords comme le Studio Wii Fit (lignes hang_l / hang_r de 5)
import yaml
P_ = os.path.join(ST, 'param')
os.makedirs(P_, exist_ok=True)
tmp = os.path.join(OUT, '_lvd.yml')
subprocess.run(['yamlvd', LVD, tmp], check=True, capture_output=True)
d = yaml.safe_load(open(tmp))
X = 85.56
SOL = [(47.47, -55.39), (-47.47, -55.39), (-53.77, -54.26), (-61.42, -47.37), (-65.84, -31.36), (-65.80, -20.45),
       (-85.56, -10.51), (-X, 0.0), (-X + 5, 0.0), (X - 5, 0.0), (X, 0.0), (85.56, -10.51), (65.80, -20.45),
       (65.84, -31.36), (61.42, -47.37), (53.77, -54.26), (47.47, -55.39)]
pts = [(x * ECH, y * ECH) for x, y in SOL]
sol = next(c for c in d['collisions'] if c['entry']['name'] == 'COL_00_Floor01')
mats = sol['materials']
m_rien = next(m for m in mats if not any(m['line_flags'].values()))
m_l = next(m for m in mats if m['line_flags']['hang_l'])
m_r = next(m for m in mats if m['line_flags']['hang_r'])
sol['vertices'] = [{'x': float(x), 'y': float(y)} for x, y in pts]
nrm, mm = [], []
for i, ((x0, y0), (x1, y1)) in enumerate(zip(pts, pts[1:])):
    dx, dy = x1 - x0, y1 - y0
    n = (dx * dx + dy * dy) ** 0.5
    nrm.append({'x': float(-dy / n), 'y': float(dx / n)})
    mm.append(m_l if i == 7 else m_r if i == 9 else m_rien)
sol['normals'] = nrm
sol['materials'] = mm
for c in d['collisions']:
    if c is sol:
        continue
    c['vertices'] = [{'x': -5.0, 'y': -30.0}, {'x': 5.0, 'y': -30.0}]
    c['normals'] = [{'x': 0.0, 'y': 1.0}]
    print('enterrée :', c['entry']['name'])
cb, bz = d['camera_boundary'][0], d['blast_zone'][0]
print('sol x ±%.2f, rebords lignes 7 et 9 ; caméra %s ; zones de K.O. %s' % (X * ECH, {k: cb[k] for k in ('left', 'right', 'top', 'bottom')},
                                                                  {k: bz[k] for k in ('left', 'right', 'top', 'bottom')}))
yaml.safe_dump(d, open(tmp, 'w'), sort_keys=False)
subprocess.run(['yamlvd', tmp, os.path.join(P_, 'wiifit00.lvd')], check=True, capture_output=True)
os.remove(tmp)

import json
cfg = {'new-dir-files': {}}
for f in nouveaux:
    cfg['new-dir-files'].setdefault(os.path.dirname(f), []).append(f)
json.dump(cfg, open(os.path.join(OUT, 'config.json'), 'w'), indent=2)
print('nouvelles textures', len(nouveaux))
