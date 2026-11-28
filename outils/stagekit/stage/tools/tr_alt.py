"""Training Rooms - PT Edit (Brawl, GameBanana 149289) en variante Stage Alts du stage d'entraînement.

usage : python3 tr_alt.py <STGONLINETRAINING.pac> <liste stage/training> <dossier YS bg2_set> <dossier Neonfield
        ring_nocastshadow_set (ou vide)> <lvd modèle> <sortie mod>

Méthode (celle de Gathering of Old Blood / Unreal, qui marchent chez le joueur) :
- tout le décor de Brawl (salle StgTrain + 3 plates-formes) dans training_base_set, sous de nouveaux noms de textures
  (sans new-dir-files, comme les variantes d'HDR) ; les 3 autres ensembles d'origine cachés (numdlb à 0 entrée) ;
- matériaux : « Bottom » de Yoshi's Story (émissif Texture5) ; sol / murs avec le shader qui reçoit l'ombre des persos
  (SFX_PBS_0000000008008100_opaque, mêmes paramètres) ; quadrillage noir (BlackSquare) en découpe ;
- lignes vertes (texture GreenLine) : animation de Brawl = SRT0 du matériau « Line », translation V 0 -> -1 en 200
  images (boucle) -> « Material 3 » de Neonfield (SFX_PBS_0000000800098100_opaque, émissif Texture5) dont
  CustomVector6 = [échelle u, échelle v, vitesse u, vitesse v] (unités / s) fait défiler les UV tout seul ;
- collisions de Brawl (brawl_coll.py), caméra / zones de K.O. d'après l'étendue de la salle ;
- config.json : chaque fichier de stage/training/normal non fourni partagé vers la variante (share_to_vanilla).
"""
import copy, json, os, shutil, subprocess, sys
import numpy as np
import ssbh_data_py as S
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import brawl_stage  # noqa: E402
import brawl_coll  # noqa: E402

PAC, LISTE, YS, NEON, LVD, OUT = sys.argv[1:7]
A = S.mesh_data
FORME = os.environ.get('TR_FORME', 'normal_s01')
STAGE = os.environ.get('TR_STAGE', 'training')          # v2 : wiifit (Studio Wii Fit, méthode de BSL Station)
ST = os.path.join(OUT, 'stage', STAGE, FORME)
REF_DOS = os.environ.get('TR_REF_DOS', '')              # dossier du même ensemble dans une réf du stage (conteneurs)
LVD_NOM = os.environ.get('TR_LVD_NOM', 'blank_00.lvd')
if os.path.isdir(os.path.join(OUT, 'stage')):
    shutil.rmtree(os.path.join(OUT, 'stage'))
ECH = float(os.environ.get('TR_ECHELLE', '1.0'))
Y0 = float(os.environ.get('TR_Y0', '0.0'))
DOS = os.environ.get('TR_DOS', 'training_base_set')
PFX = 'tra_'
MODELES = set(os.environ.get('TR_MODELES', 'StgTrain,zStgBattleFieldAshiba01,zStgBattleFieldAshiba02,'
                                           'zStgBattleFieldAshiba03').split(','))
SANS_TEX = set(os.environ.get('TR_SANS_TEX', 'TShadow1').split(','))     # ombres plaquées de Brawl (fondu spécial)
LIGNE = 'GreenLine'
# Brawl : V 0 -> -1 en 200 images à 60 i/s = -0,3 / s (TR_VITESSE_K pour ajuster en jeu)
VIT_V = -60.0 / 200.0 * float(os.environ.get('TR_VITESSE_K', '1'))
LUEUR = float(os.environ.get('TR_LUEUR', '1.35'))                       # lignes un peu plus lumineuses (lueur Brawl)

TEX, models = brawl_stage.load(PAC)
# quadrillage noir : bord transparent de 1 px sur 32 -> 4 px sur 128 (sinon les mipmaps effacent les lignes)
if 'BlackSquare' in TEX:
    TEX['BlackSquare'] = TEX['BlackSquare'].convert('RGBA').resize((128, 128), Image.NEAREST)
if LIGNE in TEX:
    _g = np.asarray(TEX[LIGNE].convert('RGBA')).astype(np.float32)
    _g[:, :, :3] = np.clip(_g[:, :, :3] * LUEUR, 0, 255)
    _g[:, :, 3] = 255
    TEX[LIGNE] = Image.fromarray(_g.astype(np.uint8), 'RGBA').resize((32, 256), Image.BICUBIC)

garde = []
for _, mn, ol in models:
    if mn not in MODELES:
        print('ignore (modèle)', mn)
        continue
    for o in ol:
        if not o['tex'] or o['tex'] not in TEX or o['tex'] in SANS_TEX:
            print('ignore', mn, o['name'], o['tex'])
            continue
        garde.append((mn, o))


ADD = set(o['tex'] for _, o in garde if o.get('add'))   # bords lumineux des plates-formes : additifs dans Brawl


SUR = os.environ.get('TR_SUR', '0') == '1'   # v3 : seulement les matériaux de BSL / Lamp (Bottom _opaque + découpe)


def genre(t):
    if t == LIGNE:
        return 'l'
    if t in ADD:
        return 'a' if SUR else 'add'
    a = np.asarray(TEX[t].convert('RGBA'))[:, :, 3]
    return 'a' if int(a.min()) < 200 else 'o'


def attr(name, data):
    a = A.AttributeData(name)
    a.data = np.asarray(data, dtype=np.float32)
    return a


def tname(t):
    return (PFX + t).lower().replace(' ', '_')


YM = S.matl_data.read_matl(os.path.join(YS, 'model.numatb'))
BOTTOM = next(e.material_label for e in YM.entries if e.material_label == 'Bottom')
NEON_M = os.path.join(NEON, 'model.numatb') if NEON else ''
M3 = None
if NEON_M and os.path.exists(NEON_M) and os.environ.get('TR_LIGNES_FIXES', '0') != '1':
    M3 = next((e for e in S.matl_data.read_matl(NEON_M).entries if e.material_label == 'Material 3'), None)
print('Material 3 de Neonfield :', M3.shader_label if M3 else 'ABSENT (lignes sans défilement)')

par_tex = {}
for mn, o in garde:
    par_tex.setdefault(o['tex'], []).append(o)

D = os.path.join(ST, 'model', DOS)
os.makedirs(D, exist_ok=True)
objs, labs, entries = [], [], []
scene = []
for k, (t, ol) in enumerate(sorted(par_tex.items())):
    g = genre(t)
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
    mo = A.MeshObjectData('tra_%02d' % k, 0)
    mo.positions = [attr('Position0', pos)]
    mo.normals = [attr('Normal0', np.c_[nrm, np.zeros(nv)])]
    try:
        tg = A.calculate_tangents_vec4(pos, nrm, uv, idx)
    except Exception:  # noqa: BLE001
        up = np.where(np.abs(nrm[:, 1:2]) > 0.9, np.array([[1, 0, 0]]), np.array([[0, 1, 0]]))
        tg = np.c_[np.cross(up, nrm), np.ones(nv)]
    mo.tangents = [attr('Tangent0', tg)]
    lab = 'tra_%s' % t
    if g == 'l' and M3 is not None:
        # mêmes attributs que les maillages de Neonfield (cot_sbf.py)
        mo.texture_coordinates = [attr('map1', uv), attr('uvSet', np.zeros((nv, 2))), attr('uvSet1', np.zeros((nv, 2)))]
        mo.color_sets = [attr('colorSet1', np.full((nv, 4), 0.502))]
        e = next(x for x in S.matl_data.read_matl(NEON_M).entries if x.material_label == 'Material 3')
        e.material_label = lab
        for x in e.textures:
            if x.param_id.name == 'Texture5':
                x.data = tname(t)
        for v in e.vectors:
            if v.param_id.name == 'CustomVector6':
                v.data = [1.0, 1.0, 0.0, VIT_V]
        for x in e.rasterizer_states:
            d_ = x.data
            d_.cull_mode = S.matl_data.CullMode.Disabled
            x.data = d_
        print('  lignes : %s, CustomVector6 = %s' % (e.shader_label, [1.0, 1.0, 0.0, VIT_V]))
    else:
        mo.texture_coordinates = [attr('map1', uv), attr('bake1', uv), attr('uvSet', uv)]
        mo.color_sets = [attr('colorSet1', np.full((nv, 4), 0.502)), attr('colorSet4', np.full((nv, 4), 0.502))]
        e = next(x for x in S.matl_data.read_matl(os.path.join(YS, 'model.numatb')).entries if x.material_label == BOTTOM)
        e.material_label = lab
        for x in e.textures:
            if x.param_id.name == 'Texture5':
                x.data = tname(t)
        for x in e.rasterizer_states:
            d_ = x.data
            d_.cull_mode = S.matl_data.CullMode.Disabled
            x.data = d_
        if g == 'add':
            # additif (méthode Gathering of Old Blood) : One / One, passe triée
            for x in e.blend_states:
                d_ = x.data
                d_.source_color = S.matl_data.BlendFactor.One
                d_.destination_color = S.matl_data.BlendFactor.One
                d_.alpha_sample_to_coverage = False
                x.data = d_
            e.shader_label = e.shader_label.rsplit('_', 1)[0] + '_sort'
        elif g == 'a':
            for x in e.blend_states:
                d_ = x.data
                d_.alpha_sample_to_coverage = True
                x.data = d_
        elif not SUR:
            # reçoit l'ombre des persos (même méthode que Gathering of Old Blood v7)
            e.shader_label = 'SFX_PBS_0000000008008100_opaque'
    entries.append(e)
    objs.append(mo)
    labs.append(lab)
    print('%-7s %-14s %s %6d tri %6d sommets' % (mo.name, t, g, len(idx) // 3, nv))
    png = os.path.join(OUT, '_tmp.png')
    im = TEX[t].convert('RGBA')
    if g in ('o', 'add'):
        im.putalpha(255)
    elif SUR and t in ADD:
        _q = np.asarray(im).astype(np.float32)
        _q[:, :, 3] = np.clip(_q[:, :, :3].max(axis=2) * 3.0, 0, 255)
        im = Image.fromarray(_q.astype(np.uint8), 'RGBA')
    im.save(png)
    subprocess.run(['ultimate_tex_cli', png, os.path.join(D, tname(t) + '.nutexb'), '--format', 'BC7RgbaUnormSrgb'],
                   check=True, capture_output=True)
    os.remove(png)
    for o in ol:
        p = o['pos'][:, [0, 2, 1]].astype(np.float64)
        p[:, :, 1] -= Y0
        p *= ECH
        u = o['uv'][:, [0, 2, 1]] if o['uv'] is not None else np.zeros(p.shape[:2] + (2,))
        scene.append({'p': np.round(p.reshape(-1), 4).tolist(), 'uv': np.round(u.reshape(-1), 5).tolist(),
                      't': tname(t), 'a': g == 'a', 'add': g == 'add', 'l': g == 'l'})

CONT = REF_DOS or YS
matl = S.matl_data.read_matl(os.path.join(CONT, 'model.numatb'))
matl.entries = entries
matl.save(os.path.join(D, 'model.numatb'))
mesh = A.read_mesh(os.path.join(CONT, 'model.numshb'))
mesh.objects = objs
mesh.save(os.path.join(D, 'model.numshb'))
modl = S.modl_data.read_modl(os.path.join(CONT, 'model.numdlb'))
modl.entries = [S.modl_data.ModlEntryData(o.name, o.subindex, l) for o, l in zip(objs, labs)]
modl.save(os.path.join(D, 'model.numdlb'))
S.meshex_data.MeshExData.from_mesh_objects(objs).save(os.path.join(D, 'model.numshexb'))
print('DOSSIER', DOS, len(objs), 'objets', sum(len(o.vertex_indices) // 3 for o in objs), 'triangles')

# autres ensembles d'origine cachés : numdlb à 0 entrée
if os.environ.get('TR_VIDE'):
    # Studio Wii Fit : exactement comme BSL Station (numdlb à 0 entrée de Bowser's Keep, liste de BSL)
    assert len(S.modl_data.read_modl(os.environ['TR_VIDE']).entries) == 0
    caches = [x for x in os.environ['TR_CACHES'].split(',') if x]
    if caches == ['*']:
        # Smashville (méthode Gathering of Old Blood) : tous les ensembles du stage (réf + liste du jeu) sauf le nôtre
        ens = set()
        RMD = os.path.dirname(REF_DOS.rstrip('/'))
        ens.update(d_ for d_ in os.listdir(RMD) if os.path.isdir(os.path.join(RMD, d_)))
        for l in open(LISTE):
            if l.startswith('stage/%s/normal/model/' % STAGE):
                ens.add(l.strip().split('/')[4])
        caches = sorted(ens - {DOS})
    for d in caches:
        os.makedirs(os.path.join(ST, 'model', d), exist_ok=True)
        shutil.copy(os.environ['TR_VIDE'], os.path.join(ST, 'model', d, 'model.numdlb'))
    print('cachés (0 entrée) :', ' '.join(caches))
else:
    ens = set()
    for l in open(LISTE):
        if l.startswith('stage/training/normal/model/'):
            ens.add(l.strip().split('/')[4])
    vide = S.modl_data.read_modl(os.path.join(YS, 'model.numdlb'))
    vide.entries = []
    for d in sorted(ens - {DOS}):
        os.makedirs(os.path.join(ST, 'model', d), exist_ok=True)
        vide.save(os.path.join(ST, 'model', d, 'model.numdlb'))
    print('cachés (0 entrée) :', ' '.join(sorted(ens - {DOS})))
if os.environ.get('TR_PLATEAU'):
    # plateau du Studio Wii Fit : modèle entier (fichiers de mvc2_training) aux sommets ramenés en (0, -30, 0) (BSL)
    PL = os.environ['TR_PLATEAU']
    DPL = os.path.join(ST, 'model', os.path.basename(PL.rstrip('/')))
    os.makedirs(DPL, exist_ok=True)
    for f in ('model.numdlb', 'model.numatb'):
        shutil.copy(os.path.join(PL, f), DPL)
    m = A.read_mesh(os.path.join(PL, 'model.numshb'))
    for o in m.objects:
        q = np.zeros_like(np.asarray(o.positions[0].data, dtype=np.float32))
        q[:, 1] = -30.0
        o.positions[0].data = q
    m.save(os.path.join(DPL, 'model.numshb'))
    print('plateau ramené en un point :', len(m.objects), 'objets')

# scène three.js (aperçu)
if os.environ.get('TR_SCENE'):
    SC = os.environ['TR_SCENE']
    os.makedirs(os.path.join(SC, 'tex'), exist_ok=True)
    for t in par_tex:
        im = TEX[t].convert('RGBA')
        if genre(t) in ('o', 'add'):
            im.putalpha(255)
        im.save(os.path.join(SC, 'tex', tname(t) + '.png'))
    json.dump(scene, open(os.path.join(SC, 'scene.json'), 'w'))
    print('scène', len(scene), 'objets')

# collisions de Brawl
import yaml  # noqa: E402
P_ = os.path.join(ST, 'param')
os.makedirs(P_, exist_ok=True)
tmp = os.path.join(OUT, '_lvd.yml')
subprocess.run(['yamlvd', LVD, tmp], check=True, capture_output=True)
d = yaml.safe_load(open(tmp))
PTS, PLS = None, None
_out = []
brawl_coll.blobs(open(PAC, 'rb').read(), 'pac', _out)
for _n, _d in _out:
    _r = brawl_coll.essai(_d)
    if _r and (PLS is None or len(_r[1]) > len(PLS)):
        PTS, PLS = _r[0], _r[1]
segs = [(PTS[a], PTS[b], ty, fl) for a, b, ty, fl in PLS]


def _k(p):
    return (round(p[0], 2), round(p[1], 2))


def _thr(j):
    return bool(segs[j][3] & 0x100)


libres = list(range(len(segs)))
chaines = []
while libres:
    i = libres.pop(0)
    ch = [i]
    while True:
        fin = _k(segs[ch[-1]][1])
        nx = [j for j in libres if _k(segs[j][0]) == fin and _thr(j) == _thr(ch[-1])]
        if not nx:
            break
        ch.append(nx[0])
        libres.remove(nx[0])
    while True:
        deb = _k(segs[ch[0]][0])
        pv = [j for j in libres if _k(segs[j][1]) == deb and _thr(j) == _thr(ch[0])]
        if not pv:
            break
        ch.insert(0, pv[0])
        libres.remove(pv[0])
    chaines.append(ch)

# modèle de collision : la plus longue collision statique (sans os) du lvd modèle
stat = [c for c in d['collisions'] if not (c['entry'].get('bone_name') or '').strip() and not c['col_flags'].get('drop_through')] \
    or d['collisions']
sol0 = max(stat, key=lambda c: len(c['vertices']))
print('collision modèle :', sol0['entry']['name'], {k_: v_ for k_, v_ in sol0['col_flags'].items()})
mat0 = copy.deepcopy(sol0['materials'][0])
for k_ in mat0.get('line_flags', {}):
    mat0['line_flags'][k_] = False


def _poly(c, pts):
    c['vertices'] = [{'x': float(x * ECH), 'y': float((y - Y0) * ECH)} for x, y in pts]
    nrm = []
    for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
        dx, dy = x1 - x0, y1 - y0
        n = (dx * dx + dy * dy) ** 0.5 or 1.0
        nrm.append({'x': float(-dy / n), 'y': float(dx / n)})
    c['normals'] = nrm
    c['cliffs'] = []
    c['materials'] = [copy.deepcopy(mat0) for _ in nrm]


cols = []
for k_, ch in enumerate(chaines):
    pts = [segs[ch[0]][0]] + [segs[j][1] for j in ch]
    c = copy.deepcopy(sol0)
    c['entry']['name'] = 'COL_00_Brawl%02d' % k_
    c['entry']['subname'] = 'Brawl%02d' % k_
    c['col_flags']['drop_through'] = bool(all(segs[j][3] & 0x100 for j in ch))
    _poly(c, pts)
    # rebords de Brawl : drapeau 0x2000 = rebord à gauche du segment (début), 0x4000 = à droite (fin)
    for li, j in enumerate(ch):
        for bit, cote, ang, p_ in ((0x2000, 'hang_l', -1.0, segs[j][0]), (0x4000, 'hang_r', 1.0, segs[j][1])):
            if segs[j][3] & bit and not c['col_flags']['drop_through']:
                c['materials'][li]['line_flags'][cote] = True
                en = copy.deepcopy(sol0['entry'])
                n_ = len([x for cc in cols for x in cc['cliffs']]) + len(c['cliffs'])
                en['name'], en['subname'] = 'CLIFF_brawl%02d' % n_, 'brawl%02d' % n_
                en['start_pos'] = {'x': float(p_[0] * ECH), 'y': float((p_[1] - Y0) * ECH), 'z': 0.0}
                c['cliffs'].append({'entry': en, 'pos': {'x': float(p_[0] * ECH), 'y': float((p_[1] - Y0) * ECH)},
                                    'angle': ang, 'line_index': li})
    cols.append(c)
    print('chaîne %d : rebords %s, %s %s' % (k_, [(cl['pos']['x'], cl['pos']['y'], cl['angle']) for cl in c['cliffs']], 'traversable' if c['col_flags']['drop_through'] else 'pleine',
                                  [(round(x, 1), round(y, 1)) for x, y in pts]))
autres = [c for c in d['collisions'] if c is not sol0]
if os.environ.get('TR_ENTERRE', '0') == '1':
    # comme BSL Station : les autres collisions du lvd gardées, enterrées sous le sol
    for c in autres:
        _ye = float(os.environ.get('TR_ENTERRE_Y', '-30'))
        c['vertices'] = [{'x': -5.0, 'y': _ye}, {'x': 5.0, 'y': _ye}]
        c['normals'] = [{'x': 0.0, 'y': 1.0}]
        c['cliffs'] = []
        c['materials'] = c['materials'][:1]
        print('enterrée :', c['entry']['name'])
    cols = cols + autres
if os.environ.get('TR_COLL_BSL', '0') == '1':
    # test B : collisions faites comme BSL Station (rien de nouveau dans le lvd) : seul le sol d'origine (COL_00_Floor01)
    # reçoit la chaîne pleine de Brawl, avec ses propres matériaux (rien / hang_l / hang_r) et ses propres rebords
    # déplacés ; les autres collisions d'origine enterrées sous le sol (sommets / normales seulement) ; pas de plates-formes
    ch = next(c for c in chaines if not _thr(c[0]))
    pts = [segs[ch[0]][0]] + [segs[j][1] for j in ch]
    mats = sol0['materials']
    m_rien = next(m for m in mats if not any(m['line_flags'].values()))
    m_l = next((m for m in mats if m['line_flags']['hang_l']), m_rien)
    m_r = next((m for m in mats if m['line_flags']['hang_r']), m_rien)
    sol0['vertices'] = [{'x': float(x * ECH), 'y': float((y - Y0) * ECH)} for x, y in pts]
    nrm, mm, gauche, droite = [], [], [], []
    for li, j in enumerate(ch):
        (x0, y0), (x1, y1) = segs[j][0], segs[j][1]
        n = ((x1 - x0) ** 2 + (y1 - y0) ** 2) ** 0.5 or 1.0
        nrm.append({'x': float(-(y1 - y0) / n), 'y': float((x1 - x0) / n)})
        fl = segs[j][3]
        mm.append(m_l if fl & 0x2000 else m_r if fl & 0x4000 else m_rien)
        if fl & 0x2000:
            gauche.append((segs[j][0], li))
        if fl & 0x4000:
            droite.append((segs[j][1], li))
    sol0['normals'] = nrm
    sol0['materials'] = mm
    for cl in sol0['cliffs']:
        lst = droite if cl['angle'] > 0 else gauche
        if lst:
            (x_, y_), li = lst.pop(0)
            cl['pos'] = {'x': float(x_ * ECH), 'y': float((y_ - Y0) * ECH)}
            cl['entry']['start_pos'] = {'x': float(x_ * ECH), 'y': float((y_ - Y0) * ECH), 'z': 0.0}
            cl['line_index'] = li
    print('test B : sol d\'origine = chaîne de Brawl, rebords d\'origine déplacés :',
          [(cl['entry']['name'], cl['pos'], cl['angle'], cl['line_index']) for cl in sol0['cliffs']])
    for c in d['collisions']:
        if c is sol0:
            continue
        c['vertices'] = [{'x': -5.0, 'y': -30.0}, {'x': 5.0, 'y': -30.0}]
        c['normals'] = [{'x': 0.0, 'y': 1.0}]
        print('enterrée :', c['entry']['name'], c['entry'].get('bone_name'))
    cols = d['collisions']
d['collisions'] = cols
# corniches : bords des sols pleins (accrochables) laissés au jeu (cliffs vides, comme Gathering of Old Blood)
allp = np.array(PTS)
xmin, xmax, ymin, ymax = allp[:, 0].min(), allp[:, 0].max(), allp[:, 1].min(), allp[:, 1].max()
cb = d['camera_boundary'][0]
bz = d['blast_zone'][0]
cb.update({'left': float((xmin - 30) * ECH), 'right': float((xmax + 60) * ECH), 'top': float((ymax - Y0 + 20) * ECH),
           'bottom': float((ymin - Y0 - 60) * ECH)})
bz.update({'left': float((xmin - 150) * ECH), 'right': float((xmax + 170) * ECH), 'top': float((ymax - Y0 + 150) * ECH),
           'bottom': float((ymin - Y0 - 160) * ECH)})
for env, cible in (('TR_CAMERA', cb), ('TR_BLAST', bz)):
    if os.environ.get(env):
        l_, r_, t_, b_ = [float(v) for v in os.environ[env].split(',')]
        cible.update({'left': l_ * ECH, 'right': r_ * ECH, 'top': t_ * ECH, 'bottom': b_ * ECH})
for k_, ref in (('shrunken_camera_boundary', cb), ('shrunken_blast_zone', bz)):
    for x in d.get(k_) or []:
        x.update({kk: ref[kk] for kk in ('left', 'right', 'top', 'bottom')})
xs_sp = [float(v) for v in os.environ.get('TR_SPAWNS', '-60,20,-100,60').split(',')]
for i_, s_ in enumerate(d['spawns']):
    s_['pos']['x'] = xs_sp[i_ % len(xs_sp)] * ECH
    s_['pos']['y'] = 0.1
for i_, r_ in enumerate(d['respawns']):
    r_['pos']['x'] = xs_sp[i_ % len(xs_sp)] * ECH
    r_['pos']['y'] = 40.0 * ECH
print('caméra %s ; K.O. %s' % ({k: cb[k] for k in ('left', 'right', 'top', 'bottom')},
                               {k: bz[k] for k in ('left', 'right', 'top', 'bottom')}))
if os.environ.get('TR_COLL_PNG'):
    from PIL import ImageDraw
    sc = 1.5
    W_, H_ = int((bz['right'] - bz['left']) * sc), int((bz['top'] - bz['bottom']) * sc)
    im_ = Image.new('RGB', (W_, H_), (20, 24, 40))
    dr = ImageDraw.Draw(im_)
    P = lambda x, y: ((x - bz['left']) * sc, (bz['top'] - y) * sc)  # noqa: E731
    dr.rectangle([P(cb['left'], cb['top']), P(cb['right'], cb['bottom'])], outline=(90, 90, 140))
    for c in cols:
        v = [(q['x'], q['y']) for q in c['vertices']]
        col = (80, 200, 255) if c['col_flags']['drop_through'] else (255, 255, 255)
        for a_, b_ in zip(v, v[1:]):
            dr.line([P(*a_), P(*b_)], fill=col, width=3)
    for s_ in d['spawns']:
        x_, y_ = P(s_['pos']['x'], s_['pos']['y'])
        dr.ellipse([x_ - 5, y_ - 10, x_ + 5, y_], fill=(80, 255, 80))
    im_.save(os.environ['TR_COLL_PNG'])
yaml.safe_dump(d, open(tmp, 'w'), sort_keys=False)
subprocess.run(['yamlvd', tmp, os.path.join(P_, LVD_NOM)], check=True, capture_output=True)
os.remove(tmp)
if os.environ.get('TR_LVD_ORIGINE', '0') == '1':
    # test : lvd du stage d'origine (HDR) sans aucune modification
    shutil.copy(LVD, os.path.join(P_, LVD_NOM))
    print('test : lvd d\'origine tel quel', LVD)

# config.json (Stage Alts)
fournis = set()
for racine, _, fichiers in os.walk(ST):
    for f in fichiers:
        fournis.add(os.path.relpath(os.path.join(racine, f), OUT).replace(os.sep, '/'))
partage = {}
for l in open(LISTE):
    p = l.strip()
    if not p.startswith('stage/%s/normal/' % STAGE):
        continue
    v = p.replace('stage/%s/normal/' % STAGE, 'stage/%s/%s/' % (STAGE, FORME), 1)
    if v not in fournis:
        partage[p] = [v]
UI_FOURNIS = [int(x) for x in os.environ.get('TR_UI_FOURNIS', '').split(',') if x]
for n in range(5):
    if n not in UI_FOURNIS:
        partage['ui/replace/stage/stage_%d/stage_%d_%s.bntx' % (n, n, STAGE)] = \
            ['ui/replace/stage/stage_%d/stage_%d_%s_%s.bntx' % (n, n, STAGE, FORME.split('_')[-1])]
if FORME == 'normal':
    # à la place du terrain normal (comme BSL Station) : pas de partage, nouvelles textures déclarées (new-dir-files)
    vanille = set(l.strip() for l in open(LISTE))
    cfg = {'new-dir-files': {}}
    for f in sorted(fournis):
        if f not in vanille:
            cfg['new-dir-files'].setdefault(os.path.dirname(f), []).append(f)
    json.dump(cfg, open(os.path.join(OUT, 'config.json'), 'w'), indent=2)
    print('fournis', len(fournis), '; nouveaux fichiers', sum(len(v) for v in cfg['new-dir-files'].values()))
else:
    json.dump({'share_to_vanilla': partage}, open(os.path.join(OUT, 'config.json'), 'w'), indent=2)
    print('fournis', len(fournis), '; partagés avec le jeu', len(partage))
