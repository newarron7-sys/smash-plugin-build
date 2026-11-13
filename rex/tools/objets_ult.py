"""Modèles des objets de Master Chief (REX, extraits par objets_rex.py) pour les objets d'Ultimate du plugin :
  grenade (Plasma Grenade) -> item/chewing (bombe collante) ; coil (Fusion Coil) -> item/powderbox (caisse explosive)

Modèle de référence d'objet : un mod de la caisse explosive (squelette à un os « Have », matériau d'objet, nuhlpb /
xmb / nusrcmdlb) ; le maillage de REX est mis à l'échelle et centré comme la référence (bas posé à la même hauteur).

usage : python3 objets_ult.py <ref powderbox (dossier model/body/c00)> <objets (npz, json, png)> <sortie>
  -> <sortie>/<chewing|powderbox>/ (fichiers du modèle + png à convertir en nutexb)
"""
import json, os, shutil, sys
import numpy as np
import ssbh_data_py as S
from PIL import Image

REF, OBJ, OUT = sys.argv[1:4]
A = S.mesh_data
# objet de REX -> (dossier d'Ultimate, taille visée (plus grande dimension), posé au sol comme la référence)
# grenade : centrée sur l'origine de l'objet (point collé sur le perso touché ; centrée avant sur le milieu de la caisse
# de référence, donc affichée plusieurs unités au-dessus de la vraie bombe : vidéo « décalé »)
CIBLES = {'grenade': ('chewing', 3.6, 'centre0'), 'coil': ('powderbox', None, True),
          # Gravity Lift : sur la C4 de Snake (objet posé au sol, dossier par costume : seulement Chief c140) ;
          # à plat, pattes au sol (origine de la C4 = point de pose), 16 de large (vidéo de REX : à peu près la
          # taille de Chief)
          'lift': ('c4', None, 'bas0')}
VARIANTES = [('_py', np.eye(3)), ('_ny', np.diag([1.0, -1.0, -1.0])),
             ('_px', np.array([[0, 1, 0], [-1, 0, 0], [0, 0, 1]], float)),
             ('_nx', np.array([[0, -1, 0], [1, 0, 0], [0, 0, 1]], float)),
             ('_pz', np.array([[1, 0, 0], [0, 0, -1], [0, 1, 0]], float)),
             ('_nz', np.array([[1, 0, 0], [0, 0, 1], [0, -1, 0]], float))]
ECRASE = {'lift': 0.35}  # v18 : demandé « ça doit ressembler à » l'ascenseur de Spiritlinx : étoile basse, presque à plat
LARGEUR = {'lift': 22.0}  # 16 en v11 : trop petit en jeu (vidéo)
# couleurs des parties sans texture (batterie covenante : métal violet, lueurs bleues)
COUL = {'cov_battery_shiny': (95, 60, 140), 'cov_battery_dull': (70, 50, 100),
        'cov_battery_illum': (120, 200, 255), 'cov_battery_illum_core': (170, 230, 255)}

info = json.load(open(os.path.join(OBJ, 'objets.json')))
npz = np.load(os.path.join(OBJ, 'objets.npz'))
ref_mesh = A.read_mesh(os.path.join(REF, 'model.numshb'))
rp = np.concatenate([np.asarray(o.positions[0].data)[:, :3] for o in ref_mesh.objects])
rmin, rmax = rp.min(0), rp.max(0)
print('REF objets', [(o.name, o.subindex, [b.bone_name for b in o.bone_influences]) for o in ref_mesh.objects])
print('REF boite', rmin.round(2).tolist(), rmax.round(2).tolist())
ref_matl = S.matl_data.read_matl(os.path.join(REF, 'model.numatb'))
print('REF materiaux', [(e.material_label, e.shader_label, [(t.param_id.name, t.data) for t in e.textures]) for e in ref_matl.entries])
sk = S.skel_data.read_skel(os.path.join(REF, 'model.nusktb'))
os_ = sk.bones[0].name
print('REF os', [b.name for b in sk.bones])


def attr(name, data):
    a = A.AttributeData(name)
    a.data = np.ascontiguousarray(data, dtype=np.float32)
    return a


def p2(im):
    s = [max(8, 1 << (int(v) - 1).bit_length()) for v in im.size]
    return im.resize(tuple(s), Image.LANCZOS) if list(im.size) != s else im


for nom, (dos, taille, sol) in CIBLES.items():
    it = info[nom]
    d = os.path.join(OUT, dos)
    os.makedirs(os.path.join(d, 'png'), exist_ok=True)
    P_all = np.concatenate([npz[p['cle'] + '_p'] for p in it['parties']]).astype(float)
    mn, mx = P_all.min(0), P_all.max(0)
    if nom in LARGEUR:
        k = LARGEUR[nom] / float((mx - mn)[0])
    elif taille:
        k = taille / float((mx - mn).max())
    else:
        k = float((rmax - rmin)[1] / (mx - mn)[1])          # même hauteur que la référence
    # centre en x / z comme la référence ; en y : bas sur celui de la référence (sol) ou centré
    c_src = (mn + mx) / 2
    c_ref = (rmin + rmax) / 2
    if sol == 'centre0':
        off = -c_src * k
    elif sol == 'bas0':
        # v22 : ascenseur centré sur son moyeu (origine du modèle de REX, d'où part le faisceau), pas sur le milieu de
        # sa boîte : ses pattes et pétales ne sont pas symétriques (Z de -14,2 à 9,3), le jet bleu tombait à côté du
        # centre (vidéo v21)
        off = np.array([0.0, -mn[1] * k, 0.0])
    else:
        dy = (rmin[1] - mn[1] * k) if sol else (c_ref[1] - c_src[1] * k)
        off = np.array([c_ref[0] - c_src[0] * k, dy, c_ref[2] - c_src[2] * k])
    print('OBJET', nom, '->', dos, 'echelle', round(k, 3), 'decalage', off.round(2).tolist())
    # os du squelette : « Have » de la caisse de référence pour la caisse ; pour la bombe collante et la C4, un nom à
    # nous (« ChiefRoot ») : les animations d'origine de ces objets bougent leurs propres os (vidéo : bombe affichée
    # décalée de l'objet réel) ; un os qu'aucune animation ne connaît reste sur l'origine de l'objet
    os_cible = 'ChiefRoot' if dos in ('chewing', 'c4') else os_
    mesh = A.read_mesh(os.path.join(REF, 'model.numshb'))
    modl = S.modl_data.read_modl(os.path.join(REF, 'model.numdlb'))
    objs, ents, mats = [], [], []
    for i, p in enumerate(it['parties']):
        P = npz[p['cle'] + '_p'].astype(float) * k + off
        N = npz[p['cle'] + '_n'].astype(float)
        if nom in ECRASE:
            # hauteur écrasée (le bas reste au sol : bas0 met le point le plus bas à 0) ; normales corrigées
            P[:, 1] *= ECRASE[nom]
            N[:, 1] /= ECRASE[nom]
            N /= np.maximum(np.linalg.norm(N, axis=1, keepdims=True), 1e-6)
        UV = np.ascontiguousarray(np.nan_to_num(npz[p['cle'] + '_u'].astype(float)))
        UV[:, 1] = 1 - UV[:, 1]
        # ascenseur (C4) : vidéos v13 à v18, toujours debout : le jeu tourne la C4 (orientation de la main de Snake
        # gardée) d'un angle que je ne peux pas mesurer d'ici. 6 copies du maillage, le haut de l'ascenseur (+Y) mis
        # sur chacun des 6 axes ; le plugin mesure en jeu quel axe du modèle pointe vers le haut (os axe_x / y / z) et
        # n'affiche que la copie qui est alors à plat
        for vnom, R in (VARIANTES if dos == 'c4' else [('', np.eye(3))]):
          P_, N_ = P, N
          P = P_ @ R.T
          N = N_ @ R.T
          self_nom = '%s_%d%s' % (nom, i, vnom)
          nv = len(P)
          idx = np.arange(nv, dtype=np.uint32)
          o = A.MeshObjectData(self_nom, 0)
          o.positions = [attr('Position0', P)]
          o.normals = [attr('Normal0', np.c_[N, np.zeros(nv)])]
          try:
              tg = A.calculate_tangents_vec4(P.astype(np.float32), N.astype(np.float32), UV.astype(np.float32), idx)
          except BaseException:  # noqa: BLE001
              tg = np.tile([1.0, 0, 0, 1], (nv, 1))
          o.tangents = [attr('Tangent0', np.nan_to_num(np.asarray(tg, dtype=float)))]
          o.texture_coordinates = [attr('map1', UV)]
          o.color_sets = [attr('colorSet1', np.full((nv, 4), 0.5))]
          o.vertex_indices = idx
          o.bone_influences = [A.BoneInfluence(os_cible, [A.VertexWeight(j, 1.0) for j in range(nv)])]
          objs.append(o)
          lab = '%s_%s' % (nom, p['materiau'].lower().replace(' ', '_'))
          ents.append(S.modl_data.ModlEntryData(o.name, 0, lab))
          if lab in [m.material_label for m in mats]:
              P, N = P_, N_
              continue
          e = S.matl_data.read_matl(os.path.join(REF, 'model.numatb')).entries[0]
          e.material_label = lab
          tx = p['textures'][0] if p['textures'] else None
          src = os.path.join(OBJ, 'png', tx + '.png') if tx else None
          if src and os.path.exists(src):
              col = p2(Image.open(src).convert('RGBA'))
          else:
              col = Image.new('RGBA', (8, 8), COUL.get(p['materiau'], (128, 128, 128)) + (255,))
          col.save(os.path.join(d, 'png', lab + '_col.png'))
          Image.new('RGBA', (8, 8), (128, 128, 255, 255)).save(os.path.join(d, 'png', lab + '_nor.png'))
          Image.new('RGBA', (8, 8), (100, 110, 255, 90)).save(os.path.join(d, 'png', lab + '_prm.png'))
          for t in e.textures:
              n = t.param_id.name
              if n == 'Texture0':
                  t.data = lab + '_col'
              elif n == 'Texture4':
                  t.data = lab + '_nor'
              elif n == 'Texture6':
                  t.data = lab + '_prm'
          mats.append(e)
          print('  partie', o.name, lab, nv // 3, 'triangles', 'texture', tx)
          P, N = P_, N_
    mesh.objects = objs
    mesh.save(os.path.join(d, 'model.numshb'))
    modl.entries = ents
    modl.save(os.path.join(d, 'model.numdlb'))
    matl = S.matl_data.read_matl(os.path.join(REF, 'model.numatb'))
    matl.entries = mats
    matl.save(os.path.join(d, 'model.numatb'))
    S.meshex_data.MeshExData.from_mesh_objects(objs).save(os.path.join(d, 'model.numshexb'))
    skc = S.skel_data.read_skel(os.path.join(REF, 'model.nusktb'))
    if os_cible != os_:
        racine = skc.bones[0]
        racine.name = os_cible
        if dos == 'c4':
            # C4 : « Have » gardé (enfant de ChiefRoot, sans maillage). Pas pour la bombe collante : avec « Have », le
            # jeu colle cet os sur la cible et le modèle se retrouve décalé (vidéos v7 / v9) ; sans, il est à la bonne
            # place mais son explosion d'origine ne s'affiche plus : le plugin la fait (speciaux.rs, lueur_grenade).
            # « rot » et « top » : os que le code de la C4 du jeu lit (statut ESTABLISH_GROUND : à chaque image elle se
            # replace sur la position de « rot » ; SEARCH sur « rot », effets sur « top »). Sans eux la position lue
            # est (0, 0) : l'ascenseur s'affichait au centre du terrain (vidéos v8 à v10). Enfants de ChiefRoot, sur
            # l'origine du modèle, sans maillage
            # (l'os de référence n'est pas sur l'origine du modèle : translation 0 / -5,2 / -6,2 ; « rot » et « top »
            # reçoivent l'inverse pour être exactement sur l'origine = la position de l'objet)
            # v14 : l'os racine aussi mis sur l'origine (identité) : vidéo v13, le modèle dessiné à quelques unités du
            # point de pose (jet et zone de propulsion à côté de l'ascenseur) ; avec l'os racine décalé de
            # 0 / -5,2 / -6,2, le décalage dépend de la façon dont le jeu place cet os. Identité = aucun décalage possible
            # v15 : identité = ascenseur debout (vidéo v14) : la rotation de l'os de référence compense celle que le jeu
            # donne à la C4 posée ; on garde donc sa rotation et on ne retire que sa translation (le décalage de la v13)
            t_ = np.array(racine.transform, dtype=float)
            t_[3, :3] = 0.0
            racine.transform = np.ascontiguousarray(t_, dtype=np.float32)
            inv = np.ascontiguousarray(np.eye(4), dtype=np.float32)
            fils = []
            for nom_os in ('Have', 'rot', 'top', 'axe_x', 'axe_y', 'axe_z'):
                o_ = S.skel_data.read_skel(os.path.join(REF, 'model.nusktb')).bones[0]
                o_.name = nom_os
                o_.parent_index = 0
                t_ = inv.copy()
                if nom_os.startswith('axe_'):
                    # repères : 10 le long de chaque axe du modèle (le plugin lit leur position dans le monde)
                    t_[3, 'xyz'.index(nom_os[-1])] = 10.0
                o_.transform = t_
                fils.append(o_)
            skc.bones = [racine] + fils
    skc.save(os.path.join(d, 'model.nusktb'))
    for f in ('model.nuhlpb', 'model.xmb', 'model.nusrcmdlb'):
        if os.path.exists(os.path.join(REF, f)):
            shutil.copy(os.path.join(REF, f), os.path.join(d, f))
    print('MODELE', dos, 'ok', len(objs), 'objets', len(mats), 'materiaux')
