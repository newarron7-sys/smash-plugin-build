"""Planet Papetoon (Brawl, GameBanana 395060, MarioDox) sur Yoshi's Story (stage/yoshi_cartboard), v1.

usage : python3 papetoon_ys.py <STGPAPETOON.pac> <réf Super Bell Island .../stage/yoshi_cartboard> <yoshi_cartboard_00.lvd HDR> <sortie mod>

Même méthode que COT / Royal Court (cot_sbf.py, rc_fd.py), avec la structure d'un mod de Yoshi's Story qui marche
(Super Bell Island, 392643 : décor remplacé par ses propres maillages dans ground_set / bg1_set, nouvelles textures
déclarées dans config.json) :
- ground_set : le terrain de Papetoon (YoshiStory/*) + ses 3 plates-formes ; bg1_set : le désert (240mAdvWild01,
  Dust, DolHalberd) ; maillages sans os, numdlb / numatb / nusktb de la réf comme modèle de fichiers ;
- matériau : « Bottom » de la réf (SFX_PBS_000010000000c100, texture en Texture5, sans éclairage : l'éclairage est déjà
  peint dans les textures de Brawl), deux faces, transparence par découpe (alpha to coverage) ;
- cachés (maillage d'un seul point, numdlb d'origine gardé) : floatingfloor_set (plates-formes d'origine), bg2_set
  (fond d'origine). Gardés : Randall (cloudrift_set), Maskass (heiho_set), tosanzo_set, ombres ;
- pas de Shine (stGimmickYoshiBumper, EffFoxReflector*, Rail, Path) : demande du joueur ;
- collisions : celles du Yoshi's Story d'HDR (même forme que Brawl / P+), plates-formes déplacées sur celles de Papetoon ;
  décor mis à l'échelle pour que le bord du sol tombe sur celui des collisions (±58).
"""
import os, shutil, subprocess, sys
import numpy as np
import ssbh_data_py as S

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import brawl_stage  # noqa: E402

PAC, REF, LVD, OUT = sys.argv[1:5]
A = S.mesh_data
RM = os.path.join(REF, 'normal', 'model')
ST = os.path.join(OUT, 'stage', 'yoshi_cartboard', 'normal')
if os.path.isdir(os.path.join(OUT, 'stage')):
    shutil.rmtree(os.path.join(OUT, 'stage'))
ECH = float(os.environ.get('PP_ECHELLE', str(58.0 / 56.0)))   # bord du sol Brawl ±56 -> collisions HDR ±58
Y0 = float(os.environ.get('PP_Y0', '-1.0'))                     # dessus du sol de Papetoon (Brawl) -> y 0
PFX = 'pptn_'
SANS = {'stGimmickYoshiBumper', 'EffFoxReflectorStart', 'EffFoxReflector', 'EffFoxReflectorRef', 'Path'}
SANS_OBJ = {'Rail'}
DOSSIER = {'YoshiStory': 'ground_set'}                          # le reste -> bg1_set
CACHES = ['floatingfloor_set', 'bg2_set']

TEX, models = brawl_stage.load(PAC)


def attr(name, data):
    a = A.AttributeData(name)
    a.data = np.asarray(data, dtype=np.float32)
    return a


def alpha(t):
    return int(np.asarray(TEX[t].convert('RGBA'))[:, :, 3].min()) < 200


def tname(t):
    return (PFX + t).lower().replace(' ', '_')


# matériau modèle : « Bottom » (bg2_set de la réf)
tm = S.matl_data.read_matl(os.path.join(RM, 'bg2_set', 'model.numatb'))
TI = next(i for i, e in enumerate(tm.entries) if e.material_label == 'Bottom')

par_dossier = {}
for _, mn, ol in models:
    if mn in SANS:
        print('ignore (Shine)', mn)
        continue
    for o in ol:
        if o['name'] in SANS_OBJ or not o['tex'] or o['tex'] not in TEX:
            print('ignore', mn, o['name'], o['tex'])
            continue
        par_dossier.setdefault(DOSSIER.get(mn, 'bg1_set'), []).append((mn, o))

nouveaux = []
for dos, liste in par_dossier.items():
    D = os.path.join(ST, 'model', dos)
    os.makedirs(D, exist_ok=True)
    objs, labs, mats = [], [], {}
    for mn, o in liste:
        pos = o['pos'][:, [0, 2, 1]].reshape(-1, 3).astype(np.float64)
        nrm = o['nrm'][:, [0, 2, 1]].reshape(-1, 3)
        uv = (o['uv'][:, [0, 2, 1]].reshape(-1, 2) if o['uv'] is not None else np.zeros((len(pos), 2)))
        key = np.round(np.c_[pos, nrm * 100, uv * 4096], 3)
        uk, inv = np.unique(key, axis=0, return_inverse=True)
        first = np.zeros(len(uk), dtype=np.int64)
        first[inv[::-1]] = np.arange(len(inv))[::-1]
        pos, nrm, uv = pos[first], nrm[first], uv[first]
        pos[:, 1] -= Y0
        pos *= ECH
        nrm = nrm / np.maximum(np.linalg.norm(nrm, axis=1, keepdims=True), 1e-6)
        idx = inv.astype(np.uint32).ravel()
        nv = len(pos)
        mo = A.MeshObjectData(('%s_%s' % (mn, o['name'])).replace(' ', '_'), 0)
        mo.positions = [attr('Position0', pos)]
        mo.normals = [attr('Normal0', np.c_[nrm, np.zeros(nv)])]
        try:
            tg = A.calculate_tangents_vec4(pos, nrm, uv, idx)
        except Exception:  # noqa: BLE001
            up = np.where(np.abs(nrm[:, 1:2]) > 0.9, np.array([[1, 0, 0]]), np.array([[0, 1, 0]]))
            tg = np.c_[np.cross(up, nrm), np.ones(nv)]
        mo.tangents = [attr('Tangent0', tg)]
        mo.texture_coordinates = [attr('map1', uv), attr('bake1', uv), attr('uvSet', uv)]   # comme « bottom » de la réf
        mo.color_sets = [attr('colorSet1', np.full((nv, 4), 0.502)), attr('colorSet4', np.full((nv, 4), 0.502))]
        mo.vertex_indices = idx
        a = alpha(o['tex'])
        lab = 'pptn_%s%s' % (o['tex'], '_a' if a else '')
        mats[lab] = (o['tex'], a)
        objs.append(mo)
        labs.append(lab)
        print('%-10s %-34s %6d tri %6d sommets %s' % (dos, mo.name, len(idx) // 3, nv, lab))
    entries = []
    for lab, (t, a) in sorted(mats.items()):
        e = S.matl_data.read_matl(os.path.join(RM, 'bg2_set', 'model.numatb')).entries[TI]
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
        png = os.path.join(OUT, '_tmp.png')
        im = np.asarray(TEX[t].convert('RGBA')).astype(np.float32)
        hd = os.path.join(os.environ.get('PP_HD', ''), t + '_x4.png')
        if os.environ.get('PP_HD') and os.path.exists(hd):
            from PIL import Image
            im = np.asarray(Image.open(hd).convert('RGBA')).astype(np.float32)
        if a:
            im[:, :, :3] *= im[:, :, 3:] / 255.0
        from PIL import Image
        Image.fromarray(np.clip(im + 0.5, 0, 255).astype(np.uint8), 'RGBA').save(png)
        cible = os.path.join(D, tname(t) + '.nutexb')
        if not os.path.exists(cible):
            subprocess.run(['ultimate_tex_cli', png, cible, '--format', 'BC7RgbaUnormSrgb'], check=True, capture_output=True)
            nouveaux.append(os.path.relpath(cible, OUT).replace(os.sep, '/'))
        os.remove(png)
    matl = S.matl_data.read_matl(os.path.join(RM, dos, 'model.numatb'))
    matl.entries = entries
    matl.save(os.path.join(D, 'model.numatb'))
    mesh = A.read_mesh(os.path.join(RM, dos, 'model.numshb'))
    mesh.objects = objs
    mesh.save(os.path.join(D, 'model.numshb'))
    modl = S.modl_data.read_modl(os.path.join(RM, dos, 'model.numdlb'))
    modl.entries = [S.modl_data.ModlEntryData(o.name, o.subindex, l) for o, l in zip(objs, labs)]
    modl.save(os.path.join(D, 'model.numdlb'))
    S.meshex_data.MeshExData.from_mesh_objects(objs).save(os.path.join(D, 'model.numshexb'))
    for f in ('model.nusktb', 'model.nusrcmdlb'):
        if os.path.exists(os.path.join(RM, dos, f)):
            shutil.copy(os.path.join(RM, dos, f), D)
    print('DOSSIER', dos, len(objs), 'objets', len(entries), 'matériaux')

# modèles d'origine cachés : maillage d'un seul point (méthode de build_stage.py / guide de CSharpM7)
null = A.MeshObjectData('nullShape', 0)
null.positions = [attr('Position0', np.zeros((1, 3)))]
null.normals = [attr('Normal0', np.zeros((1, 4)))]
null.tangents = [attr('Tangent0', np.zeros((1, 4)))]
null.texture_coordinates = [attr('map1', np.zeros((1, 2)))]
null.vertex_indices = np.zeros(3, dtype=np.uint32)
for dos in CACHES:
    blank = A.read_mesh(os.path.join(RM, dos, 'model.numshb'))
    blank.objects = [null]
    os.makedirs(os.path.join(ST, 'model', dos), exist_ok=True)
    blank.save(os.path.join(ST, 'model', dos, 'model.numshb'))
print('cachés :', CACHES)

# collisions : HDR, plates-formes de Papetoon (dessus des plates-formes Brawl, même transformation que le décor)
import yaml
P_ = os.path.join(ST, 'param')
os.makedirs(P_, exist_ok=True)
tmp = os.path.join(OUT, '_lvd.yml')
subprocess.run(['yamlvd', LVD, tmp], check=True, capture_output=True)
d = yaml.safe_load(open(tmp))


def t(x, y):
    return round(x * ECH, 3), round((y - Y0) * ECH, 3)


PLATS = {'gauche': ((-60.0, 18.0), (-28.0, 18.0)), 'droite': ((28.0, 18.0), (60.0, 18.0)), 'haut': ((-16.0, 37.0), (16.0, 37.0))}
plats = [c for c in d['collisions'] if len(c['vertices']) == 2 and c['vertices'][0]['y'] > 10]
plats.sort(key=lambda c: (c['vertices'][0]['y'] > 35, c['vertices'][0]['x']))
for c, nom in zip(plats, ['gauche', 'droite', 'haut']):
    (x0, y0), (x1, y1) = PLATS[nom]
    a_, b_ = t(x0, y0), t(x1, y1)
    print('plate-forme', nom, [(v['x'], v['y']) for v in c['vertices']], '->', a_, b_)
    c['vertices'][0]['x'], c['vertices'][0]['y'] = a_
    c['vertices'][1]['x'], c['vertices'][1]['y'] = b_
yaml.safe_dump(d, open(tmp, 'w'), sort_keys=False)
subprocess.run(['yamlvd', tmp, os.path.join(P_, 'yoshi_cartboard_00.lvd')], check=True, capture_output=True)
os.remove(tmp)

# config.json : nouvelles textures
import json
cfg = {'new-dir-files': {}}
for f in nouveaux:
    dos = os.path.dirname(f)
    cfg['new-dir-files'].setdefault(dos, []).append(f)
json.dump(cfg, open(os.path.join(OUT, 'config.json'), 'w'), indent=2)
print('nouvelles textures', len(nouveaux))
