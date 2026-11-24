"""Sleepless Lamp (Brawl, GameBanana 692696) sur Smashville (stage/animal_village), v1.

usage : python3 lamp_sv.py <STGSLEEPLESSLAMP.pac> <réf complète .../stage/animal_village (387162)>
                           <réf « toujours la nuit » .../stage/animal_village (145592)> <numatb modèle (bg2_set de 392643)>
                           <xvillage00.lvd HDR (normal)> <sortie mod>

Méthode de Papetoon (papetoon_ys.py) + structure d'un mod de Smashville qui remplace tout le décor (Greenville
Restoration, 387162) :
- décor de Sleepless Lamp dans stc_bg_set (numshb / numdlb / numatb / numshexb ; squelette d'origine), maillages
  regroupés par texture, sans os ; matériau « Bottom » (SFX_PBS_000010000000c100, texture en Texture5, sans
  éclairage), deux faces, transparence par découpe ;
- tous les autres ensembles de Smashville cachés par un numdlb à 0 entrée (méthode de 387162) : scène, plate-forme
  mobile, villageois, ballons, taxi, horloge, ciels... ;
- éclairage des persos de nuit en permanence (render/ de 145592 « Always Night ») ;
- collisions : sol plat de Sleepless Lamp (y 0) fermé par des murs invisibles à x ±130 (comme dans Brawl, méthode de
  COT) ; plate-forme mobile de Smashville enterrée dans le sol (la retirer du lvd peut faire planter).
"""
import os, shutil, subprocess, sys
import numpy as np
import ssbh_data_py as S

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import brawl_stage  # noqa: E402

PAC, REF, NUIT, MATL_MODELE, LVD, OUT = sys.argv[1:7]
A = S.mesh_data
RM = os.path.join(REF, 'normal', 'model')
ST = os.path.join(OUT, 'stage', 'animal_village', 'normal')
if os.path.isdir(os.path.join(OUT, 'stage')):
    shutil.rmtree(os.path.join(OUT, 'stage'))
ECH = float(os.environ.get('LAMP_ECHELLE', '1.0'))
MUR = float(os.environ.get('LAMP_MUR_X', '130'))     # murs invisibles de Brawl à ±130
PFX = 'lamp_'
DOS = 'stc_bg_set'

TEX, models = brawl_stage.load(PAC)
garde = []
for _, mn, ol in models:
    for o in ol:
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
Y0 = float(os.environ.get('LAMP_Y0', str(round(float(np.median(vals)), 3) if vals else 0.0)))
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
    mo = A.MeshObjectData('lamp_%02d' % k, 0)
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
    lab = 'lamp_%s%s' % (t, '_a' if a else '')
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
    hd = os.path.join(os.environ.get('LAMP_HD', ''), t + '_x4.png')
    if os.environ.get('LAMP_HD') and os.path.exists(hd):
        im = np.asarray(Image.open(hd).convert('RGBA')).astype(np.float32)
    if a:
        im[:, :, :3] *= im[:, :, 3:] / 255.0
    Image.fromarray(np.clip(im + 0.5, 0, 255).astype(np.uint8), 'RGBA').save(png)
    cible = os.path.join(D, tname(t) + '.nutexb')
    subprocess.run(['ultimate_tex_cli', png, cible, '--format', 'BC7RgbaUnormSrgb'], check=True, capture_output=True)
    nouveaux.append(os.path.relpath(cible, OUT).replace(os.sep, '/'))
    os.remove(png)

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
caches = sorted(d for d in os.listdir(RM) if os.path.isdir(os.path.join(RM, d)) and d != DOS)
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
cfg = {'new-dir-files': {}}
for f in nouveaux:
    cfg['new-dir-files'].setdefault(os.path.dirname(f), []).append(f)
json.dump(cfg, open(os.path.join(OUT, 'config.json'), 'w'), indent=2)
print('nouvelles textures', len(nouveaux))
