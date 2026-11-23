"""Central Organ Tower (Brawl) sur le Petit Champ de bataille (battlefield_s), v1 : le sol seulement.

usage : python3 cot_sbf.py <STGCENTRALORGANTOWER.pac> <réf Neonfield .../battlefield_s> <réf Snowy SBF .../battlefield_s> <sortie mod>

Méthode (copiée de mods de battlefield_s qui marchent, rien d'inventé) :
- ring_nocastshadow_set (anneau + dessous du terrain) : remplacé par les maillages du sol de COT, comme Neonfield ;
  mêmes attributs de sommets que les maillages de Neonfield (Position0, Normal0, Tangent0, map1, uvSet, uvSet1,
  colorSet1), matériau « Material 1 » de Neonfield copié tel quel (seule Texture0 change), sol d'encre d'origine
  (inkMesh1Shape + sfx_Lightmap_INK) gardé ;
- textures : AUCUN nouveau fichier, les textures de COT écrasent des textures d'origine du dossier (pas de config.json) ;
- fond caché : numdlb + numatb vides de Neonfield (bg_*, ring_effects*, ring_flag_set) ; plantes : petit maillage de
  Snowy SBF (ring_plant_set) ;
- gardés tels quels : ciel (bg_skydome_set), plates-formes (floating_plate_set / floating_plate_sub_set), collisions.
"""
import os, shutil, subprocess, sys
import numpy as np
import ssbh_data_py as S

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import brawl_stage  # noqa: E402

PAC, NEON, SNOW, OUT = sys.argv[1:5]
A = S.mesh_data
NM = os.path.join(NEON, 'normal', 'model')
SM = os.path.join(SNOW, 'normal', 'model')
FORME = os.environ.get('COT_FORME', 'normal')            # normal_s01 = variante (Stage Alts, touche L)
DST = os.path.join(OUT, 'stage', 'battlefield_s', FORME, 'model')
RING = os.path.join(DST, 'ring_nocastshadow_set')
if os.path.isdir(os.path.join(OUT, 'stage')):
    shutil.rmtree(os.path.join(OUT, 'stage'))
os.makedirs(RING)

ECH = float(os.environ.get('COT_ECHELLE', '0.9'))     # sol de COT -86..86 (Brawl) -> bords du Petit Champ ±78
Y0 = float(os.environ.get('COT_Y0', '1.0'))            # dessus du sol (Brawl) -> y 0
MODELE = os.environ.get('COT_MODELE', 'Tower')          # modèle(s) du stage Brawl (Royal Court : Royal ; virgules)
MODELES = set(MODELE.split(','))
SOL = ['mesh_177', 'mesh_178', 'mesh_183', 'mesh_184', 'mesh_185', 'mesh_186', 'mesh_187',
       'mesh_188', 'mesh_189', 'mesh_190', 'mesh_191', 'mesh_192', 'mesh_193']
TOUT = os.environ.get('COT_TOUT', '1') == '1'          # v2 : tout le décor (v1 : le sol seul)
CIEL = {'mesh_175', 'mesh_180', 'mesh_168'}             # ciel / nuages / lointain : matériau émissif (sans ombrage)
GRAND = float(os.environ.get('COT_GRAND', '0'))         # >0 : objets plus grands que ça = ciel / lointain
# dossiers qui reçoivent le décor, avec leurs textures d'origine (écrasées : aucun nouveau fichier)
HASH = os.environ.get('COT_HASHES', os.path.join(os.path.dirname(OUT.rstrip('/')), '..', '..', 'Hashes_FullPath'))
DOSSIERS = ['ring_nocastshadow_set', 'bg_ground_set', 'bg_falls_set', 'ring_effects_set']
PLACES = {d: [] for d in DOSSIERS}
for l in open(HASH):
    l = l.strip()
    for d in DOSSIERS:
        pre = 'stage/battlefield_s/normal/model/%s/' % d
        if l.startswith(pre) and l.endswith('.nutexb') and '/' not in l[len(pre):] and 'cube' not in l:
            PLACES[d].append(l[len(pre):-7])
for d in DOSSIERS:
    PLACES[d].sort()
    os.makedirs(os.path.join(DST, d), exist_ok=True)

TEX, models = brawl_stage.load(PAC)


def attr(name, data):
    a = A.AttributeData(name)
    a.data = np.asarray(data, dtype=np.float32)
    return a


def alpha(t):
    return int(np.asarray(TEX[t].convert('RGBA'))[:, :, 3].min()) < 200


objs, texs, libres = [], {}, [(d, n) for d in DOSSIERS for n in PLACES[d]]
# atlas (Royal Court : plus de textures que de places) : textures non répétées, opaques, 256x256, deux par deux
from PIL import Image
uvs = {}
for _, mn, ol in models:
    for o in ol:
        if mn in MODELES and o['tex'] in TEX:
            uvs.setdefault(o['tex'], []).append(o['uv'].reshape(-1, 2))
manque = max(0, len(uvs) - len(libres))
cand = [t for t in sorted(uvs) if TEX[t].size == (256, 256) and not alpha(t)
        and np.concatenate(uvs[t]).min() > -0.01 and np.concatenate(uvs[t]).max() < 1.01]
ATLAS = {}
for k in range(manque):
    t0, t1 = cand[2 * k], cand[2 * k + 1]
    nom = 'ATLAS%02d' % k
    TEX[nom] = Image.new('RGBA', (512, 256))
    TEX[nom].paste(TEX[t0].convert('RGBA'), (0, 0))
    TEX[nom].paste(TEX[t1].convert('RGBA'), (256, 0))
    hd = os.environ.get('COT_HD')
    if hd and os.path.exists(os.path.join(hd, t0 + '_x4.png')):
        a0, a1 = Image.open(os.path.join(hd, t0 + '_x4.png')), Image.open(os.path.join(hd, t1 + '_x4.png'))
        im = Image.new('RGBA', (a0.width * 2, a0.height))
        im.paste(a0.convert('RGBA'), (0, 0))
        im.paste(a1.convert('RGBA'), (a0.width, 0))
        im.save(os.path.join(hd, nom + '_x4.png'))
    ATLAS[t0], ATLAS[t1] = (nom, 0), (nom, 1)
if ATLAS:
    print('atlas', len(ATLAS) // 2)
liste = [(mn, o) for _, mn, ol in models for o in ol if mn in MODELES and (TOUT or o['name'] in SOL) and o['tex'] in TEX]
liste.sort(key=lambda mo: (mo[1]['name'] not in SOL))      # le sol d'abord (ses textures dans ring_nocastshadow_set)
# optimisation : triangles jamais vus par la caméra du jeu retirés (test prudent : un triangle n'est retiré que s'il
# est entièrement hors du champ de TOUTES les positions de caméra échantillonnées), HD seulement pour le proche
CULL = os.environ.get('COT_CULL', '0') == '1'
HD_DIST = float(os.environ.get('COT_HD_DIST', '1e9'))


def _cams():
    out = []
    for cx in (-140, -70, 0, 70, 140):
        for cy in (0, 40, 90, 150):
            for cz in (110, 180, 280, 420):
                eye = np.array([cx, cy, cz], float)
                tgt = np.array([cx * 0.8, cy * 0.6, 0.0])
                f = tgt - eye
                f /= np.linalg.norm(f)
                r = np.cross(f, [0, 1, 0])
                r /= np.linalg.norm(r)
                u = np.cross(r, f)
                out.append((eye, f, r, u))
    return out


CAMS = _cams()
TV = np.tan(np.radians(55 / 2))          # champ vertical large (55°) + marge
TH = TV * 16 / 9 * 1.1


def visibles(tri):                        # tri : (n, 3, 3) en unités Ultimate
    vis = np.zeros(len(tri), bool)
    for eye, f, r, u in CAMS:
        d = tri - eye
        z = d @ f
        x = d @ r
        y = d @ u
        dehors = ((z < 1).all(1) | (x > TH * z).all(1) | (x < -TH * z).all(1) |
                  (y > TV * z).all(1) | (y < -TV * z).all(1))
        vis |= ~dehors
    return vis


# COT_VIS : triangles réellement vus (rendu d'identifiants depuis les caméras du jeu, avec ce qui les cache), un
# booléen par triangle dans l'ordre des maillages ; on garde aussi leurs voisins (pas de trous)
VIS = np.load(os.environ['COT_VIS']) if os.environ.get('COT_VIS') else None
base_tri = 1


def dilate(garde, tri):
    cle = np.round(tri.reshape(-1, 3), 2)
    _, idv = np.unique(cle, axis=0, return_inverse=True)
    idv = idv.reshape(-1, 3)
    marque = np.zeros(idv.max() + 1, bool)
    marque[idv[garde].ravel()] = True
    return marque[idv].any(1)


# COT_SCROLL : défilement de textures (SRT0 de Brawl) {matériau: [su, sv, tu, tv, du par image, images]}
import json as _json
SCROLL = _json.loads(os.environ.get('COT_SCROLL', '{}'))
if SCROLL and 'bg_skydome_set' not in DOSSIERS:
    DOSSIERS.append('bg_skydome_set')
    os.makedirs(os.path.join(DST, 'bg_skydome_set'), exist_ok=True)
PROCHES, n_avant, n_apres = set(), 0, 0
for mn, o in liste:
    pos = o['pos'][:, [0, 2, 1]].reshape(-1, 3).astype(np.float64)
    nrm = o['nrm'][:, [0, 2, 1]].reshape(-1, 3)
    uv = o['uv'][:, [0, 2, 1]].reshape(-1, 2).copy()
    if CULL:
        tri = ((pos - [0, Y0, 0]) * ECH).reshape(-1, 3, 3)
        if VIS is not None:
            garde = VIS[base_tri:base_tri + len(tri)].copy()
            base_tri += len(tri)
            garde = dilate(garde, tri)
        else:
            garde = visibles(tri)
        n_avant += len(tri)
        n_apres += int(garde.sum())
        if not garde.any():
            continue
        g3 = np.repeat(garde, 3)
        pos, nrm, uv = pos[g3], nrm[g3], uv[g3]
        if np.linalg.norm(tri[garde].reshape(-1, 3), axis=1).min() < HD_DIST:
            PROCHES.add(o['tex'])
    else:
        PROCHES.add(o['tex'])
    if o['tex'] in ATLAS:
        nom, moitie = ATLAS[o['tex']]
        uv[:, 0] = moitie * 0.5 + 0.004 + np.clip(uv[:, 0], 0, 1) * 0.492
        o = dict(o, tex=nom)
    grand = GRAND > 0 and np.ptp(pos, 0).max() > GRAND
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
    defil = SCROLL.get(o.get('mat'))
    if defil:
        uv = uv * [defil[0], defil[1]] + [defil[2], defil[3]]   # échelle / décalage fixes du SRT0, cuits (fond qui défile : bg_skydome_set)
        dos, cible = 'bg_skydome_set', 'he_' + o['tex']
        texs.setdefault(o['tex'], (dos, cible))
        genre = 'a' if alpha(o['tex']) else 'e'
    else:
        if o['tex'] not in texs:
            texs[o['tex']] = libres.pop(0)
        dos, cible = texs[o['tex']]
        genre = 'a' if alpha(o['tex']) else ('e' if (o['name'] in CIEL if MODELE == 'Tower' else grand) else 'o')
    pref = 'cot_' if MODELE == 'Tower' else ('rc_' if MODELE == 'Royal' else 'he_%s_' % mn)
    mo = A.MeshObjectData(pref + o['name'], 0)
    mo.positions = [attr('Position0', pos)]
    mo.normals = [attr('Normal0', np.c_[nrm, np.zeros(nv)])]
    try:
        tg = A.calculate_tangents_vec4(pos.astype(np.float32), nrm.astype(np.float32), uv.astype(np.float32), idx)
    except Exception:  # noqa: BLE001
        tg = np.tile([1.0, 0.0, 0.0, 1.0], (nv, 1))
    mo.tangents = [attr('Tangent0', tg)]
    mo.texture_coordinates = [attr('map1', uv), attr('uvSet', np.zeros((nv, 2))), attr('uvSet1', np.zeros((nv, 2)))]
    mo.color_sets = [attr('colorSet1', np.full((nv, 4), 0.502))]
    mo.vertex_indices = idx
    lab = ('he_' + o['mat']) if defil else 'cot_%s_%s' % (genre, cible)
    objs.append((dos, mo, lab, o['tex'], genre))
    print('%-14s %5d tri %5d sommets %s tex %s -> %s/%s' % (mo.name, len(idx) // 3, nv, genre, o['tex'], dos, cible))

# matériaux de Neonfield copiés tels quels (seule la texture change) :
#   o = « Material 1 » (SFX_PBS_0100000008008269_opaque, Texture0)
#   a = « Material 2 » (SFX_PBS_0000000000080100_sort, Texture5, transparence prémultipliée) : textures à transparence
#   e = « Material 2 » aussi : ciel et lointain, sans ombrage
neon_ring = os.path.join(NM, 'ring_nocastshadow_set')
P = os.path.join(neon_ring, 'model.numatb')
REFMAT = {'o': ('Material 1', 'Texture0'), 'a': ('Material 2', 'Texture5'), 'e': ('Material 2', 'Texture5')}
# optimisation (Royal Court) : transparence et ciel dans la passe opaque (« Material 3 » de Neonfield,
# SFX_PBS_0000000800098100_opaque, émissif Texture5, alpha-to-coverage) au lieu de la passe triée : moins de
# superpositions transparentes plein écran (ciel, dôme, colonnes de lumière)
EMI_OPAQUE = os.environ.get('COT_EMI_OPAQUE', '0') == '1'
if EMI_OPAQUE:
    REFMAT['a'] = REFMAT['e'] = ('Material 3', 'Texture5')
faites = set()
for dos in DOSSIERS:
    sel = [x for x in objs if x[0] == dos]
    if dos == 'ring_nocastshadow_set':
        mesh = A.read_mesh(os.path.join(neon_ring, 'model.numshb'))
        modl = S.modl_data.read_modl(os.path.join(neon_ring, 'model.numdlb'))
        ink = [o for o in mesh.objects if o.name.startswith('inkMesh')]
        ink_lab = {(e.mesh_object_name, e.mesh_object_subindex): e.material_label for e in modl.entries}
        matl = S.matl_data.read_matl(P)
        entries = [e for e in matl.entries if e.material_label in set(ink_lab[(o.name, o.subindex)] for o in ink)]
        base = [(o, ink_lab[(o.name, o.subindex)]) for o in ink]
    else:
        if not sel:
            continue
        mesh = A.read_mesh(os.path.join(SM, dos, 'model.numshb'))
        modl = S.modl_data.read_modl(os.path.join(SM, dos, 'model.numdlb'))
        matl = S.matl_data.read_matl(P)
        entries, base = [], []
    mesh.objects = [o for o, _ in base] + [x[1] for x in sel]
    mesh.save(os.path.join(DST, dos, 'model.numshb'))
    modl.entries = ([S.modl_data.ModlEntryData(o.name, o.subindex, lab) for o, lab in base] +
                    [S.modl_data.ModlEntryData(x[1].name, x[1].subindex, x[2]) for x in sel])
    modl.save(os.path.join(DST, dos, 'model.numdlb'))
    if dos == 'ring_nocastshadow_set':
        S.meshex_data.MeshExData.from_mesh_objects(mesh.objects).save(os.path.join(DST, dos, 'model.numshexb'))
    vus = set()
    for _, _, lab, t, genre in sel:
        if lab in vus:
            continue
        vus.add(lab)
        nom, slot = REFMAT[genre]
        e = next(x for x in S.matl_data.read_matl(P).entries if x.material_label == nom)
        e.material_label = lab
        cible = 'he_' + t if lab.startswith('he_') else texs[t][1]
        for x in e.textures:
            if x.param_id.name == slot:
                x.data = cible
        if nom == 'Material 3':
            for v in e.vectors:
                if v.param_id.name == 'CustomVector6':
                    v.data = [1.0, 1.0, 0.0, 0.0]          # pas de décalage d'UV
                    if lab.startswith('he_'):                # ZW = vitesse de défilement (unités / s), auto
                        du = SCROLL[lab[3:]][4]
                        v.data = [1.0, 1.0, du * 60.0 * float(os.environ.get('COT_SCROLL_K', '1')), 0.0]
            if lab.startswith('he_Layer'):                   # V bloqué au bord (comme Brawl) : pas d'immeubles
                for sp in e.samplers:                        # répétés dans le ciel ; U répété (défilement)
                    sd = sp.data
                    sd.wrapt = S.matl_data.WrapMode.ClampToEdge
                    sp.data = sd
        entries.append(e)
        if (dos, cible) not in faites:
            faites.add((dos, cible))
            hd = os.path.join(os.environ.get('COT_HD', ''), t + '_x4.png')     # texture agrandie (upscale_tex.py)
            if os.environ.get('COT_HD') and os.path.exists(hd) and (t in PROCHES or t.startswith('ATLAS')):
                from PIL import Image as _I
                im = np.asarray(_I.open(hd).convert('RGBA')).astype(np.float32)
                print('texture HD', t)
            else:
                im = np.asarray(TEX[t].convert('RGBA')).astype(np.float32)
            if genre == 'a':                       # transparence prémultipliée (fondu One / OneMinusSrcAlpha)
                im[:, :, :3] *= im[:, :, 3:] / 255.0
            png = os.path.join(OUT, '_tmp.png')
            from PIL import Image
            Image.fromarray(np.clip(im + 0.5, 0, 255).astype(np.uint8), 'RGBA').save(png)
            subprocess.run(['ultimate_tex_cli', png, os.path.join(DST, dos, cible + '.nutexb'), '--format',
                            'BC7RgbaUnormSrgb'], check=True, capture_output=True)
            os.remove(png)
    matl.entries = entries
    matl.save(os.path.join(DST, dos, 'model.numatb'))
    print('DOSSIER', dos, len(sel), 'objets', len(vus), 'materiaux')

if SCROLL and os.environ.get('COT_SCROLL_MOTION') == '1':   # ancienne méthode (trop rapide en jeu)
    an = S.anim_data.AnimData()
    nb = max(int(v[5]) for v in SCROLL.values())
    an.final_frame_index = float(nb - 1)
    g = S.anim_data.GroupData(S.anim_data.GroupType.Material)
    for m, (su, sv, tu, tv, du, _) in SCROLL.items():
        if not any(x[2] == 'he_' + m for x in objs):
            continue
        nd = S.anim_data.NodeData('he_' + m)
        tr = S.anim_data.TrackData('CustomVector6')
        tr.values = [[su, sv, tu + du * f, tv] for f in range(nb)]
        nd.tracks = [tr]
        g.nodes.append(nd)
    an.groups = [g]
    mdir = os.path.join(OUT, 'stage', 'battlefield_s', FORME, 'motion', 'bg_skydome_set')
    os.makedirs(mdir, exist_ok=True)
    an.save(os.path.join(mdir, 'bg_skydome.nuanmb'))
    print('motion', len(g.nodes), 'materiaux qui defilent,', nb, 'images')

# fond caché (fichiers de Neonfield tels quels) et plantes (fichiers de Snowy SBF tels quels)
for d in ('bg_birds_set', 'bg_falls_set', 'bg_far_skycity_set', 'bg_ground_set', 'bg_solar_eclipse_line_set',
          'bg_solar_eclipse_set', 'ring_effects_r_set', 'ring_effects_set', 'ring_flag_set'):
    if os.path.exists(os.path.join(DST, d, 'model.numshb')):
        continue                                   # dossier qui reçoit du décor
    os.makedirs(os.path.join(DST, d), exist_ok=True)
    for f in ('model.numdlb', 'model.numatb'):
        shutil.copy(os.path.join(NM, d, f), os.path.join(DST, d, f))
os.makedirs(os.path.join(DST, 'ring_plant_set'))
for f in ('model.numshb', 'model.numdlb', 'model.numatb'):
    shutil.copy(os.path.join(SM, 'ring_plant_set', f), os.path.join(DST, 'ring_plant_set', f))
print('objets', len(objs), 'textures', len(texs))
if CULL:
    print('triangles : %d -> %d (%.0f%%) ; textures HD %d' % (n_avant, n_apres, 100.0 * n_apres / max(n_avant, 1), len(PROCHES)))

# ---- v3 : collisions de COT (sol plat, murs invisibles), plates-formes retirées (méthode HDR validée) ----
#   usage : ... <sortie mod> <battlefield_s_00.lvd d'origine> <floating_plate_set/model.numshb d'origine>
if len(sys.argv) > 6:
    import yaml
    LVD, PLAT = sys.argv[5:7]
    tmp = os.path.join(OUT, '_lvd.yml')
    subprocess.run(['yamlvd', LVD, tmp], check=True, capture_output=True)
    d = yaml.safe_load(open(tmp))
    MX = float(os.environ.get('COT_MUR_X', '130'))         # murs (Brawl) : COT / Royal Court 130, Hammer Express 75
    X, XM, YB, YH = MX * ECH, (MX + 10) * ECH, -30.0, 170.0  # dans les zones de K.O.
    sol = next(c for c in d['collisions'] if c['entry']['name'] == 'COL_00_Floor01')
    XD = X + float(os.environ.get('COT_MUR_DROIT', '20'))  # COT : 20 ; Royal Court : 0  # mur droit un peu plus loin (demande du joueur)
    XDM = XD + (XM - X)
    pts = [(XDM, YB), (-XM, YB), (-XM, YH), (-X, YH), (-X, 0.0), (XD, 0.0), (XD, YH), (XDM, YH), (XDM, YB)]
    sol['vertices'] = [{'x': float(x), 'y': float(y)} for x, y in pts]
    nrm = []
    for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
        dx, dy = x1 - x0, y1 - y0
        n = (dx * dx + dy * dy) ** 0.5
        nrm.append({'x': float(-dy / n), 'y': float(dx / n)})
    sol['normals'] = nrm
    sol['cliffs'] = []                                      # pas de bords : sol fermé par les murs
    sol['materials'] = [sol['materials'][0]] * len(nrm)
    # plates-formes : petits segments DANS le bloc du sol (les retirer fait planter)
    for c in d['collisions']:
        n = c['entry']['name']
        if 'Platform01' in n:
            c['vertices'] = [{'x': -50.0, 'y': -25.0}, {'x': -30.0, 'y': -25.0}]
        elif 'Platform03' in n:
            c['vertices'] = [{'x': 30.0, 'y': -25.0}, {'x': 50.0, 'y': -25.0}]
    for s in d['spawns']:                                   # départs sur les plates-formes -> au sol
        if s['pos']['y'] > 10:
            s['pos']['y'] = 0.111259505
    yaml.safe_dump(d, open(tmp, 'w'), sort_keys=False)
    P_ = os.path.join(OUT, 'stage', 'battlefield_s', FORME, 'param')
    os.makedirs(P_, exist_ok=True)
    subprocess.run(['yamlvd', tmp, os.path.join(P_, 'battlefield_s_00.lvd')], check=True, capture_output=True)
    os.remove(tmp)
    # plates-formes visibles : tous les sommets en un point (0, -25, 0) dans le sol
    m = A.read_mesh(PLAT)
    for o in m.objects:
        p = np.asarray(o.positions[0].data, dtype=np.float32)
        q = np.zeros_like(p)
        q[:, 1] = -25.0
        o.positions[0].data = q
    os.makedirs(os.path.join(DST, 'floating_plate_set'), exist_ok=True)
    m.save(os.path.join(DST, 'floating_plate_set', 'model.numshb'))
    os.makedirs(os.path.join(DST, 'floating_plate_sub_set'), exist_ok=True)
    for f in ('model.numdlb', 'model.numatb'):
        shutil.copy(os.path.join(NM, 'floating_plate_sub_set', f), os.path.join(DST, 'floating_plate_sub_set', f))
    print('collisions COT : murs x -%.0f / +%.0f jusqu\'a y %.0f ; plates-formes retirees' % (X, XD, YH))
