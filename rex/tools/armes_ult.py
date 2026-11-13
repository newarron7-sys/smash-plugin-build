"""Ajoute les armes de Master Chief (REX, extraites par armes_rex.py) au modèle Ultimate du skin (sur Snake).

Chaque arme devient un maillage « chief_<arme> » lié à 100 % à l'os de la main d'Ultimate (HaveR / HaveL), placé
comme l'effet de REX sur RHaveN / LHaveN : dans REX l'arme suit REX_monde(main) @ A ; les animations convertées donnent
Ult_monde(main) = REX_monde(main) @ KEY (retarget_exo.py) ; donc dans le repère de la main d'Ultimate l'arme est à
inv(KEY) @ A. Le plugin affiche / cache chaque arme (ModelModule::set_mesh_visibility) selon le coup.

usage : python3 armes_ult.py <skin> <ref Snake> <model.json exo> <exo nusktb> <armes> <sortie> [prefixe]
  skin : model.numshb / numdlb du skin GameBanana ; ref : model.nusktb / numatb de Snake ; sortie : modèle complet +
  png des textures (à convertir en nutexb)
"""
import json, os, sys
import numpy as np
import ssbh_data_py as S
from PIL import Image

SKIN, REF, MJSON, EXOSK, ARMES, OUT = sys.argv[1:7]
PFX = sys.argv[7] if len(sys.argv) > 7 else 'chief'
os.makedirs(os.path.join(OUT, 'png'), exist_ok=True)
A = S.mesh_data


def col(m):
    return np.array(m, dtype=float).T


# ------------------------------------------------------------------ squelettes
sk = S.skel_data.read_skel(os.path.join(REF, 'model.nusktb'))
UW = {b.name: col(sk.calculate_world_transform(b)) for b in sk.bones}
print('OS mains Snake :', [n for n in UW if 'Have' in n or 'Throw' in n])
info = json.load(open(MJSON))
EX = info['exo']
NW = {n: np.array(m, dtype=float) for n, m in EX['nw'].items()}
ex = S.skel_data.read_skel(EXOSK)
EXW = {b.name: col(ex.calculate_world_transform(b)) for b in ex.bones}


def orth(m):
    u, _, vt = np.linalg.svd(m[:3, :3])
    r = u @ vt
    if np.linalg.det(r) < 0:
        u[:, -1] *= -1
        r = u @ vt
    return r


ULT_OF = {'RHaveN': 'HaveR', 'LHaveN': 'HaveL', 'ThrowN': 'Throw', 'BustN': 'Bust'}
CONV = {}
for b, u in ULT_OF.items():
    if b in NW and 'H_Exo_' + b in EXW:
        key = orth(np.linalg.inv(NW[b]) @ EXW['H_Exo_' + b])
        C = np.eye(4)
        C[:3, :3] = key.T                                 # inv(KEY) (rotation)
        # échelle REX -> Ultimate du perso (repos : distance hanche -> main)
        CONV[b] = C
        print('KEY', b, np.round(key, 3).tolist())
    else:
        print('KEY absente', b)

# ------------------------------------------------------------------ armes
arm = json.load(open(os.path.join(ARMES, 'armes.json')))
npz = np.load(os.path.join(ARMES, 'armes.npz'))
mesh = A.read_mesh(os.path.join(SKIN, 'model.numshb'))
modl = S.modl_data.read_modl(os.path.join(SKIN, 'model.numdlb'))
print('SKIN objets', [(o.name, o.subindex) for o in mesh.objects])
print('SKIN materiaux', sorted({e.material_label for e in modl.entries}))
ref_matl = S.matl_data.read_matl(os.path.join(REF, 'model.numatb'))
print('REF materiaux', [(e.material_label, [t.data for t in e.textures][:3]) for e in ref_matl.entries])
need = {e.material_label for e in modl.entries}
have = {e.material_label for e in ref_matl.entries}
print('materiaux du skin absents de la reference :', sorted(need - have))


def attr(name, data):
    a = A.AttributeData(name)
    a.data = np.ascontiguousarray(data, dtype=np.float32)   # tableau non contigu (vue transposée) -> panique
    return a


# matériaux du skin absents de la référence (skin du costume 00 : « def_snake_001 », la référence vient d'un autre
# costume : « def_snake_001_c02 ») : repris de leur version de costume, textures renommées en conséquence
import re  # noqa: E402
base_mats = []
for lab in sorted(need):
    src = [e for e in S.matl_data.read_matl(os.path.join(REF, 'model.numatb')).entries
           if e.material_label == lab or re.fullmatch(re.escape(lab) + r'_c\d\d', e.material_label)]
    if not src:
        print('MATERIAU INTROUVABLE', lab)
        continue
    e = src[0]
    suf = e.material_label[len(lab):]
    e.material_label = lab
    for t in e.textures:
        if suf and isinstance(t.data, str):
            t.data = t.data.replace(suf, '')
    base_mats.append(e)
    print('MATERIAU DU SKIN', lab, '<-', lab + suf, [t.data for t in e.textures])
TEMPLATE = lab if base_mats else ref_matl.entries[0].material_label
print('MATERIAU MODELE', TEMPLATE)
saved = set()


def png(name, im):
    if name not in saved:
        # tailles en puissances de 2 (textures de REX : 60x60, 63x30, 5x5…) : sûr pour la compression du nutexb
        p2 = [max(8, 1 << (int(v) - 1).bit_length()) for v in im.size]
        if list(im.size) != p2:
            im = im.resize(tuple(p2), Image.LANCZOS)
        im.save(os.path.join(OUT, 'png', name + '.png'))
        saved.add(name)
    return name


png(PFX + '_wnor', Image.new('RGBA', (8, 8), (128, 128, 255, 255)))
LAME_ECH = float(os.environ.get('LAME_ECH', '1.7'))
new_mats = []
objs, ents = list(mesh.objects), list(modl.entries)
for nom, it in arm.items():
    if nom == 'laser':
        continue                                       # rayon : effet du plugin, pas un maillage
    b = it['os']
    u = ULT_OF[b]
    if u not in UW or b not in CONV:
        print('ARME ignoree (os)', nom, b, u)
        continue
    M = UW[u] @ CONV[b]
    for k, p in enumerate(it['parties']):
        key = p['cle']
        P = npz[key + '_p'].astype(float)
        N = npz[key + '_n'].astype(float)
        UV = npz[key + '_u'].astype(float)
        if nom == 'swordb':
            # v25 (vidéo : « son épée doit être plus grande, là elle est à peine visible ») : les lames de REX sont un
            # seul plan (x constant), vu par la tranche en jeu ; agrandies (x LAME_ECH autour de la poignée) et
            # doublées d'un plan croisé (tourné de 90° autour de leur axe, z) : visibles sous tous les angles
            x0 = float(np.median(P[:, 0]))
            P = P.copy()
            P[:, 1:] *= LAME_ECH
            Q = np.c_[x0 - P[:, 1], P[:, 0] - x0, P[:, 2]]
            NQ = np.c_[-N[:, 1], N[:, 0], N[:, 2]]
            P = np.r_[P, Q]
            N = np.r_[N, NQ]
            UV = np.r_[UV, UV]
        pos = np.ascontiguousarray((M @ np.c_[P, np.ones(len(P))].T).T[:, :3])
        nrm = np.ascontiguousarray((M[:3, :3] @ N.T).T)
        nrm /= np.linalg.norm(nrm, axis=1, keepdims=True) + 1e-9
        UV = np.nan_to_num(UV)
        UV = np.ascontiguousarray(UV)
        UV[:, 1] = 1 - UV[:, 1]                             # V inversé (Brawl -> Ultimate)
        nv = len(pos)
        o = A.MeshObjectData('%s_%s' % (PFX, nom), k)
        o.positions = [attr('Position0', pos)]
        o.normals = [attr('Normal0', np.c_[nrm, np.zeros(nv)])]
        idx = np.arange(nv, dtype=np.uint32)
        try:
            tg = A.calculate_tangents_vec4(pos.astype(np.float32), nrm.astype(np.float32), UV.astype(np.float32), idx)
        except BaseException:  # noqa: BLE001
            print('tangentes non calculees', nom, k)
            tg = np.tile([1.0, 0, 0, 1], (nv, 1))
        tg = np.nan_to_num(np.asarray(tg, dtype=float), nan=0.0)
        o.tangents = [attr('Tangent0', tg)]
        o.texture_coordinates = [attr('map1', UV)]
        o.color_sets = [attr('colorSet1', np.full((nv, 4), 0.5))]
        o.vertex_indices = idx
        o.bone_influences = [A.BoneInfluence(u, [A.VertexWeight(i, 1.0) for i in range(nv)])]
        objs.append(o)
        tx = p['textures'][0] if p['textures'] else None
        lab = '%s_%s_%s' % (PFX, nom, (tx or 'blanc').lower().replace(' ', '_').replace('[', '').replace(']', ''))
        ents.append(S.modl_data.ModlEntryData(o.name, o.subindex, lab))
        if lab in {m.material_label for m in new_mats}:
            continue
        lame = nom == 'swordb'                     # lames de l'épée à énergie : transparentes et lumineuses
        _all = S.matl_data.read_matl(os.path.join(REF, 'model.numatb')).entries
        e = ([x for x in _all if x.material_label.startswith('alp_')] if lame else [])
        e = (e or [x for x in _all if x.material_label == TEMPLATE or x.material_label.startswith(TEMPLATE + '_c')])[0]
        e.material_label = lab
        src = os.path.join(ARMES, 'png', tx + '.png') if tx else None
        if src and os.path.exists(src):
            im = Image.open(src).convert('RGBA')
            if 'laser' in nom:
                pass
            c = png(lab + '_col', im)
        else:
            c = png(lab + '_col', Image.new('RGBA', (8, 8), (90, 90, 90, 255)))
        met, rough = (0.6, 0.35) if nom not in ('laser',) else (0.0, 0.5)
        prm = png(lab + '_prm', Image.new('RGBA', (8, 8), (int(met * 255), int(rough * 255), 255, 90)))
        for t in e.textures:
            n = t.param_id.name
            if n == 'Texture0':
                t.data = c
            elif n == 'Texture4':
                t.data = PFX + '_wnor'
            elif n == 'Texture6':
                t.data = prm
            elif lame and n in ('Texture5', 'Texture14'):
                t.data = c
        if lame:
            for v in e.vectors:
                if v.param_id.name == 'CustomVector3':
                    v.data = [1.6, 1.6, 1.6, 1.0]
            print('LAME', lab, [t.param_id.name for t in e.textures], e.shader_label)
        new_mats.append(e)
        print('ARME', nom, k, o.name, lab, nv // 3, 'triangles')

bad = []
for o in objs:
    t = A.read_mesh(os.path.join(SKIN, 'model.numshb'))
    t.objects = [o]
    try:
        t.save('/tmp/_essai.numshb')
    except BaseException as ex:  # noqa: BLE001
        bad.append(o)
        nv_ = len(o.vertex_indices)
        print('  pos min/max', np.asarray(o.positions[0].data).min(0), np.asarray(o.positions[0].data).max(0),
              'nan', [np.isnan(np.asarray(a_.data)).any() for a_ in o.positions + o.normals + o.tangents + o.texture_coordinates])
        for champ in ['positions', 'normals', 'tangents', 'texture_coordinates', 'color_sets', 'bone_influences',
                      'vertex_indices']:
            o2 = A.MeshObjectData(o.name, o.subindex)
            rnd = np.random.rand(nv_, 3)
            o2.positions = [attr('Position0', rnd)]
            o2.normals = [attr('Normal0', np.c_[rnd, np.zeros(nv_)])]
            o2.tangents = [attr('Tangent0', np.tile([1, 0, 0, 1.0], (nv_, 1)))]
            o2.texture_coordinates = [attr('map1', np.random.rand(nv_, 2))]
            o2.color_sets = [attr('colorSet1', np.full((nv_, 4), 0.5))]
            o2.vertex_indices = np.arange(nv_, dtype=np.uint32)
            o2.bone_influences = [A.BoneInfluence(o.bone_influences[0].bone_name,
                                                  [A.VertexWeight(i, 1.0) for i in range(nv_)])]
            setattr(o2, champ, getattr(o, champ))
            t.objects = [o2]
            try:
                t.save('/tmp/_essai.numshb')
                print('   champ', champ, 'OK')
            except BaseException as ex2:  # noqa: BLE001
                print('   champ', champ, 'ECHEC', str(ex2)[:60])
        print('OBJET REFUSE', o.name, o.subindex, [a_.name for a_ in o.texture_coordinates],
              [np.asarray(a_.data).shape for a_ in o.positions + o.normals + o.tangents + o.texture_coordinates],
              np.isnan(np.asarray(o.tangents[0].data)).any() if o.tangents else None, str(ex)[:80])
objs = [o for o in objs if all(o is not b_ for b_ in bad)]
mesh.objects = objs
mesh.save(os.path.join(OUT, 'model.numshb'))
modl.entries = ents
modl.save(os.path.join(OUT, 'model.numdlb'))
matl = S.matl_data.read_matl(os.path.join(REF, 'model.numatb'))
matl.entries = base_mats + new_mats
matl.save(os.path.join(OUT, 'model.numatb'))
try:
    S.meshex_data.MeshExData.from_mesh_objects(objs).save(os.path.join(OUT, 'model.numshexb'))
except Exception as e:  # noqa: BLE001
    print('numshexb non genere :', e)
print('MODELE AVEC ARMES ok :', len(objs), 'objets,', len(matl.entries), 'materiaux')
