"""Décor d'un stage de Brawl -> décor d'un stage d'Ultimate (option A : sol, murs et caméra du stage remplacé).

usage : python3 build_stage.py <STGxxx.pac> <dossier de référence stage/<nom> d'un mod Ultimate> <sortie> <préfixe>
        [modèles Brawl à ignorer, séparés par des virgules]

- tout le décor Brawl va dans <nom>_stage_set (le modèle principal du stage) ; les autres modèles du stage d'Ultimate
  (public, tribunes, ciel, lumières…) sont cachés par un model.numdlb vide (même méthode que les mods de stage
  existants) ;
- matériaux : shader « émissif » des stages (SFX_PBS_0000000000080100, texture en Texture5) : la texture s'affiche
  telle quelle, comme dans Brawl où l'éclairage est déjà peint dans les textures ; deux faces visibles ;
  textures avec transparence -> découpe (alpha to coverage), toujours en passe _opaque (la seule de ce shader) ;
- le maillage du sol pour l'encre des Inklings (inkMesh_floorShape, matériau for_ink) est repris du stage d'origine ;
- les textures sortent en PNG (<sortie>/png) : la compilation les convertit en .nutexb.
"""
import os, shutil, sys
import numpy as np
import ssbh_data_py as S

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import brawl_stage  # noqa: E402

PAC, REF, OUT, PFX = sys.argv[1:5]
SKIP = set(sys.argv[5].split(',')) if len(sys.argv) > 5 and sys.argv[5] else set()
STAGE = os.path.basename(os.path.normpath(REF))                 # ex. dolly_stadium
MODELS = os.path.join(REF, 'normal', 'model')
MAIN = '%s_stage_set' % STAGE
DST = os.path.join(OUT, 'mod', 'stage', STAGE, 'normal', 'model')
# v0.4 (forme Ω, méthode du tutoriel de CSharpM7) : le décor remplace UN modèle de fond d'un autre stage, sa forme
# « battle » (Ω / Champ de bataille). La référence ne sert plus que de modèle de fichiers (shader, format).
#   CIBLE=zelda_hyrule/battle/stc_bg_set  -> pas de sol d'encre, rien à cacher (sol, plateformes et collisions du stage)
CIBLE = os.environ.get('CIBLE')
REF_MAIN = os.path.join(MODELS, MAIN)
if CIBLE:
    _st, _form, MAIN = CIBLE.split('/')
    DST = os.path.join(OUT, 'mod', 'stage', _st, _form, 'model')
PNG = os.path.join(OUT, 'png')
os.makedirs(os.path.join(DST, MAIN), exist_ok=True)
os.makedirs(PNG, exist_ok=True)
A = S.mesh_data

TEX, models = brawl_stage.load(PAC)
print('textures Brawl', len(TEX))


def attr(name, data):
    a = A.AttributeData(name)
    a.data = np.asarray(data, dtype=np.float32)
    return a


def tname(t):
    return ('%s_%s_col' % (PFX, t)).lower().replace(' ', '_')


objects, labels, used = [], [], {}
for _, mn, objs in models:
    if mn in SKIP:
        print('modele ignore', mn)
        continue
    for o in objs:
        if not o['tex'] or o['tex'] not in TEX:
            print('sans texture, ignore', mn, o['name'])
            continue
        pos = o['pos'][:, [0, 2, 1]].reshape(-1, 3)          # sens des triangles : Brawl -> Ultimate
        nrm = o['nrm'][:, [0, 2, 1]].reshape(-1, 3)
        uv = (o['uv'][:, [0, 2, 1]].reshape(-1, 2) if o['uv'] is not None else np.zeros((len(pos), 2)))
        # sommets en double fusionnés (position, normale, UV identiques)
        key = np.round(np.c_[pos, nrm * 100, uv * 4096], 3)
        uk, inv = np.unique(key, axis=0, return_inverse=True)
        first = np.zeros(len(uk), dtype=np.int64)
        first[inv[::-1]] = np.arange(len(inv))[::-1]
        pos, nrm, uv = pos[first], nrm[first], uv[first]
        idx = inv.astype(np.uint32).ravel()
        nv = len(pos)
        mo = A.MeshObjectData('%s_%s' % (mn, o['name']), 0)
        mo.positions = [attr('Position0', pos)]
        mo.normals = [attr('Normal0', np.c_[nrm, np.zeros(nv)])]
        try:
            tg = A.calculate_tangents_vec4(pos, nrm, uv, idx)
        except Exception:  # noqa: BLE001
            up = np.where(np.abs(nrm[:, 1:2]) > 0.9, np.array([[1, 0, 0]]), np.array([[0, 1, 0]]))
            tg = np.c_[np.cross(up, nrm), np.ones(nv)]
        mo.tangents = [attr('Tangent0', tg)]
        mo.texture_coordinates = [attr('map1', uv)]
        mo.color_sets = [attr('colorSet1', np.full((nv, 4), 0.502))]
        mo.vertex_indices = idx
        alpha = int(np.asarray(TEX[o['tex']])[:, :, 3].min()) < 200
        lab = '%s_%s%s' % (PFX, o['tex'], '_a' if alpha else '')
        used[lab] = (o['tex'], alpha)
        objects.append(mo)
        labels.append(lab)
        print('OBJET %-26s %6d triangles %6d sommets  %s' % (mo.name, len(idx) // 3, nv, lab))

# versions de test (recherche du crash au chargement) :
#   STAGE_DECOR=0       -> aucun objet du décor Brawl (seulement le sol d'encre et les modèles cachés)
#   STAGE_MAX_VERTS=N   -> on garde les objets les plus proches de la zone de combat jusqu'à N sommets en tout
if os.environ.get('STAGE_DECOR') == '0':
    objects, labels, used = [], [], {}
    print('TEST : aucun objet du decor')
elif os.environ.get('STAGE_MAX_VERTS'):
    cap = int(os.environ['STAGE_MAX_VERTS'])
    dist = [float(np.linalg.norm(np.asarray(o.positions[0].data).mean(0) * [1, 1, 0.5])) for o in objects]
    keep, tot = [], 0
    for i in sorted(range(len(objects)), key=lambda i: dist[i]):
        n_ = len(objects[i].positions[0].data)
        if tot + n_ <= cap:
            keep.append(i)
            tot += n_
    keep.sort()
    objects = [objects[i] for i in keep]
    labels = [labels[i] for i in keep]
    used = {k: v for k, v in used.items() if k in set(labels)}
    print('TEST : %d objets gardes, %d sommets' % (len(objects), tot))

# sol pour l'encre (repris du stage d'origine)
ref_main = REF_MAIN
ref_mesh = A.read_mesh(os.path.join(ref_main, 'model.numshb'))
ref_matl = S.matl_data.read_matl(os.path.join(ref_main, 'model.numatb'))
ink = [] if CIBLE else [o for o in ref_mesh.objects if o.name.startswith('inkMesh')]
ref_modl = S.modl_data.read_modl(os.path.join(ref_main, 'model.numdlb'))
ink_lab = {(e.mesh_object_name, e.mesh_object_subindex): e.material_label for e in ref_modl.entries}
for o in ink:
    objects.insert(0, o)
    labels.insert(0, ink_lab.get((o.name, o.subindex), 'for_ink'))
print('sol pour l\'encre :', [o.name for o in ink])

# matériaux
tmpl = next(e for e in ref_matl.entries if e.shader_label.startswith('SFX_PBS_0000000000080100'))
entries = [e for e in ref_matl.entries if e.material_label in set(labels)]       # for_ink
for lab, (t, alpha) in sorted(used.items()):
    e = S.matl_data.read_matl(os.path.join(ref_main, 'model.numatb')).entries[ref_matl.entries.index(tmpl)]
    e.material_label = lab
    for x in e.textures:
        if x.param_id.name == 'Texture5':
            x.data = tname(t)
    for x in e.vectors:
        if x.param_id.name == 'CustomVector3':
            x.data = [1.0, 1.0, 1.0, 1.0]
    for x in e.rasterizer_states:
        d_ = x.data
        d_.cull_mode = S.matl_data.CullMode.Disabled
        x.data = d_                      # (les sous-objets sont des copies : il faut les réaffecter)
    if alpha:
        for x in e.blend_states:
            # v0.1.2 : le shader du stage n'existe qu'en passe _opaque (la passe _sort de la v0.1 faisait planter
            # le jeu au chargement) : on garde _opaque et la transparence se fait par découpe (alpha to coverage)
            d_ = x.data
            d_.alpha_sample_to_coverage = True
            x.data = d_
    entries.append(e)
    TEX[t].save(os.path.join(PNG, tname(t) + '.png'))
ref_matl.entries = entries
ref_matl.save(os.path.join(DST, MAIN, 'model.numatb'))

ref_mesh.objects = objects
ref_mesh.save(os.path.join(DST, MAIN, 'model.numshb'))
ref_modl.entries = [S.modl_data.ModlEntryData(o.name, o.subindex, lab) for o, lab in zip(objects, labels)]
ref_modl.save(os.path.join(DST, MAIN, 'model.numdlb'))
S.meshex_data.MeshExData.from_mesh_objects(objects).save(os.path.join(DST, MAIN, 'model.numshexb'))

# les autres modèles du stage d'origine : cachés.
# v0.3 (guide de stages de CSharpM7 : « si l'objet a une animation, utiliser un numshb vide ») : un numdlb vide fait
# planter le jeu au chargement sur ce stage (public, drapeaux, lumières, écran géant sont animés / pilotés par le jeu).
# On garde donc leur numdlb, numatb, squelette et animations d'origine et on ne remplace QUE le maillage, par un
# maillage « vide » (un seul sommet, comme le fichier vide du guide) : le jeu trouve tout ce qu'il attend, rien ne
# s'affiche.
null = A.MeshObjectData('nullShape', 0)
null.positions = [attr('Position0', np.zeros((1, 3)))]
null.normals = [attr('Normal0', np.zeros((1, 4)))]
null.tangents = [attr('Tangent0', np.zeros((1, 4)))]
null.texture_coordinates = [attr('map1', np.zeros((1, 2)))]
null.vertex_indices = np.zeros(3, dtype=np.uint32)
blank = A.read_mesh(os.path.join(ref_main, 'model.numshb'))
blank.objects = [null]
hidden = []
for d in sorted(os.listdir(MODELS)):
    if d == MAIN or not os.path.isdir(os.path.join(MODELS, d)):
        continue
    if os.environ.get('STAGE_HIDE') == '0' or CIBLE:  # version de test, ou forme Ω : on ne cache rien
        continue
    # versions de test : STAGE_KEEP = débuts de noms des modèles d'origine à laisser tels quels (ex. ch_dolly,auroravision)
    if any(d.startswith(k) for k in os.environ.get('STAGE_KEEP', '').split(',') if k):
        continue
    os.makedirs(os.path.join(DST, d), exist_ok=True)
    blank.save(os.path.join(DST, d, 'model.numshb'))
    hidden.append(d)
print('modeles du stage d\'origine caches :', hidden)
print('OBJETS', len(objects), 'MATERIAUX', len(entries), 'TEXTURES', len(used))
