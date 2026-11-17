"""Primid v0.18 : armes de REX (épée laser verte, boomerang) dans le modèle de Primid, tenues dans la main droite.

Dans Brawl, une arme tenue a son os HaveN posé sur l'os RHaveN du perso (ici HaveR : le squelette de Primid garde
les os de REX, renommés). Les objets sont donc collés à HaveR, sommets ramenés dans le repère de l'os HaveN de l'arme
(objet collé à un os = sommets dans le repère de cet os). Le plugin les montre pendant les coups qui s'en servent.
Objets : pr_epee (WpnPrimBeamsword : poignée + lame), pr_boomerang (WpnPrimBoomerang).

usage : python3 armes_primid.py <model/body/c00 du mod> <dossier inv/ (sabre, boomerang)>
"""
import os, subprocess, sys
import numpy as np
import ssbh_data_py as S

D, INV = sys.argv[1:3]
M = S.mesh_data
mesh = M.read_mesh(os.path.join(D, 'model.numshb'))
matl = S.matl_data.read_matl(os.path.join(D, 'model.numatb'))
modl = S.modl_data.read_modl(os.path.join(D, 'model.numdlb'))
# v0.21 (le joueur : au tilt avant il ne tient pas l'épée du bon côté) : les coups d'épée de REX sont sur l'os 19 =
# LHaveN (main gauche) : épée dans HaveL. Boomerang : HaveR pour l'instant (côté B à venir).
# v0.23 (le joueur : tourné de l'autre côté, l'épée n'est pas dans la bonne main ; « même erreur qu'avec Zero ») :
# le jeu échange les animations des os gauche / droite quand Primid regarde à gauche : la main qui frappe devient
# HandR. Copie de l'épée en miroir (plan X = 0 du modèle, squelette symétrique) sur HaveR : pr_epee_r ; le plugin
# montre pr_epee (gauche) ou pr_epee_r selon son côté.
ARMES = [('pr_epee', 'sabre', 'WpnPrimBeamSword_HaveN', 'HaveL', False),
         ('pr_epee_r', 'sabre', 'WpnPrimBeamSword_HaveN', 'HaveR', True),
         ('pr_boomerang', 'boomerang', 'WpnPrimBoomerang_HaveN', 'HaveL', False),
         ('pr_boomerang_r', 'boomerang', 'WpnPrimBoomerang_HaveN', 'HaveR', True),
         # v0.31 : Super Scope du neutre B (REX : tenu à LHaveN, comme l'épée ; bouche du canon à +8 le long de Y)
         ('pr_scope', 'scope', 'WpnPrimSuperScope_HaveN', 'HaveL', False),
         ('pr_scope_r', 'scope', 'WpnPrimSuperScope_HaveN', 'HaveR', True)]
# v0.30 : boomerang en vol : copie centrée sur l'os Hat (chapeau de Mario : animé partout, inutilisé par le maillage
# de Primid ; déplacé par le plugin, tourné sur lui-même autour de X = dans le plan de l'écran)
# v0.31 : tirs du Super Scope (WpnPrimBeamS : petit tir, WpnPrimBeamL : tir chargé) sur d'autres os libres
# (mustacheL, Face) : pointe du tir (origine de l'objet de REX) sur l'os, traînée le long de -Z (derrière lui)
# v0.36 (le joueur : « le projectile va dans tous les sens ») : le boomerang en vol tournait par set_joint_rotate
# autour de l'axe X de l'os Hat, lui-même penché par l'animation de la tête : il basculait dans tous les sens. À la
# place : 8 copies déjà tournées (0, 45... 315° autour de X = dans le plan de l'écran), le plugin en montre une par
# image (pr_boom_vol0 à pr_boom_vol7), sans rotation de l'os.
TOURS = 8
# v0.38 (le joueur, vidéo du tir chargé : « le projectile va dans tous les sens au lieu d'être tout droit ») : sur
# Face et mustacheL (enfants de la tête), les tirs tournaient avec la tête de Primid. Sur l'os Throw (enfant de Rot,
# sans rotation au repos ; libre depuis que les invocations sont dans le J.E.T.) : il ne tourne qu'avec le perso.
# Copies _r tournées de 180° (autour de Y) pour un tir qui part dans l'autre sens que Primid (il s'est retourné).
VOLS = [('pr_boom_vol%d' % k, 'boomerang', 'Hat', True) for k in range(TOURS)] + [
    ('pr_beam_s', 'beams', 'Throw', False), ('pr_beam_s_r', 'beams', 'Throw', False),
    ('pr_beam_l', 'beaml', 'Throw', False), ('pr_beam_l_r', 'beaml', 'Throw', False)]
# v0.19 (vidéo du joueur : un grand disque vert autour de l'épée) : polygon2 de l'épée = halo de la lame (12 de large,
# rendu additif dans Brawl), opaque ici : retiré ; la lame (polygon3) et la poignée (polygon0, 1) restent
SAUF = {'sabre': {'polygon2'}}
# la lame (texture A) : additive dans Brawl (noir = invisible), sans alpha : alpha = luminosité, matériau à test d'alpha
# (comme le portail, validé en v0.7 : shader 0100000008048269, CustomVector0.x = 0)
# v0.32 (le joueur : « on voit pas très bien le projectile ») : tirs x 3,5 (plus de l'éclat de REX gardé)
LUMI = {'sabre_wpnprimbeamsworda_col.png': 1.6, 'beams_wpnprimbeam_col.png': 3.5, 'beaml_wpnprimbeam_col.png': 3.5}
TEST_ALPHA = {'WpnPrimBeamSword_BeamSwordA', 'WpnPrimBeamS_Beam', 'WpnPrimBeamL_Beam'}


def test_alpha(m):
    m.shader_label = 'SFX_PBS_0100000008048269_opaque'
    vs = m.vectors
    for v in vs:
        if v.param_id.name == 'CustomVector0':
            v.data = [0.0, 0.0, 0.0, 0.0]
    m.vectors = vs
    return m


def att(n, d):
    a = M.AttributeData(n)
    a.data = np.asarray(d, np.float32)
    return a


objs = [o for o in mesh.objects if not any(o.name == a[0] for a in ARMES) and o.name != 'pr_boom_vol']
ents = [x for x in modl.entries if not any(x.mesh_object_name == a[0] for a in ARMES) and x.mesh_object_name != 'pr_boom_vol']
mats = [e for e in matl.entries if not e.material_label.startswith('PrArme_')]
mats_vus = set()
textures = []
SK = S.skel_data.read_skel(os.path.join(D, 'model.nusktb'))


def monde_os(n):
    return np.asarray(SK.calculate_world_transform(next(b for b in SK.bones if b.name == n)), np.float32)


def materiaux_de(dos, at):
    """matériaux d'un objet (ajoutés une seule fois, préfixe PrArme_) -> nom d'origine : nom dans Primid"""
    if dos not in mats_vus:
        for e in at.entries:
            if e.material_label in TEST_ALPHA:
                e = test_alpha(e)
            l = 'PrArme_' + e.material_label
            e.material_label = l
            rs = e.rasterizer_states
            for p in rs:
                d = p.data
                d.cull_mode = S.matl_data.CullMode.Disabled
                p.data = d
            e.rasterizer_states = rs
            mats.append(e)
        mats_vus.add(dos)
    return {e.material_label.replace('PrArme_', ''): 'PrArme_' + e.material_label.replace('PrArme_', '')
            for e in at.entries}


def textures_de(dos):
    if dos in tex_vus:
        return
    tex_vus.add(dos)
    pd = os.path.join(INV, dos, 'png')
    for f in sorted(os.listdir(pd)):
        if f.endswith('.png'):
            fmt = 'BC7RgbaUnormSrgb' if f.endswith('_col.png') else 'BC7RgbaUnorm'
            src_png = os.path.join(pd, f)
            if f in LUMI:
                from PIL import Image
                im = Image.open(src_png).convert('RGBA')
                a = np.asarray(im).astype(np.float32)
                a[..., 3] = np.clip(a[..., :3].max(-1) * LUMI[f], 0, 255)
                src_png = os.path.join(D, '_lumi_' + f)
                Image.fromarray(a.astype(np.uint8)).save(src_png)
            subprocess.run(['ultimate_tex_cli', src_png, os.path.join(D, f[:-4] + '.nutexb'), '--format', fmt],
                           check=True, capture_output=True)
            textures.append(f[:-4] + '.nutexb')
            if src_png.startswith(os.path.join(D, '_lumi_')):
                os.remove(src_png)


tex_vus = set()
WL, F = monde_os('HaveL'), np.diag([-1.0, 1.0, 1.0, 1.0]).astype(np.float32)
for nom, dos, have, OS, miroir in ARMES:
    md = os.path.join(INV, dos, 'model')
    am = M.read_mesh(os.path.join(md, 'model.numshb'))
    at = S.matl_data.read_matl(os.path.join(md, 'model.numatb'))
    ad = S.modl_data.read_modl(os.path.join(md, 'model.numdlb'))
    sk = S.skel_data.read_skel(os.path.join(md, 'model.nusktb'))
    hb = next(b for b in sk.bones if b.name == have)
    inv = np.linalg.inv(np.asarray(sk.calculate_world_transform(hb), np.float32))
    if miroir:
        # local main gauche -> monde (repos) -> miroir X -> local main droite (convention ssbh : p_monde = p @ M)
        inv = (inv @ WL @ F @ np.linalg.inv(monde_os(OS))).astype(np.float32)
    lab = materiaux_de(dos, at)
    mat_de = {x.mesh_object_name + '/%d' % x.mesh_object_subindex: x.material_label for x in ad.entries}
    garde = [o for o in am.objects if o.name not in SAUF.get(dos, set())]
    for k, src in enumerate(garde):
        o = M.MeshObjectData(nom, k)
        o.positions = [att('Position0', M.transform_points(np.asarray(src.positions[0].data)[:, :3], inv))]
        o.normals = [att('Normal0', M.transform_vectors(np.asarray(src.normals[0].data)[:, :3], inv))]
        if src.tangents:
            t = np.asarray(src.tangents[0].data, np.float32)
            t3 = np.asarray(M.transform_vectors(t[:, :3], inv), np.float32)
            o.tangents = [att('Tangent0', np.c_[t3, t[:, 3:4] if t.shape[1] > 3 else np.ones((len(t), 1))])]
        o.texture_coordinates = [att(u.name, u.data) for u in src.texture_coordinates]
        o.color_sets = [att(c.name, c.data) for c in src.color_sets]
        o.vertex_indices = np.asarray(src.vertex_indices, np.uint32)
        o.parent_bone_name = OS
        o.bone_influences = []
        o.sort_bias = src.sort_bias
        objs.append(o)
        ents.append(S.modl_data.ModlEntryData(nom, k, lab[mat_de['%s/%d' % (src.name, src.subindex)]]))
    textures_de(dos)
# --- objets lancés (boomerang en vol, tirs) : sur un os libre, repère du modèle au repos (sommets = monde - centre,
# ou origine de l'objet de REX sur l'os)
for nom, dos, OSV, centrer in VOLS:
    md = os.path.join(INV, dos, 'model')
    am = M.read_mesh(os.path.join(md, 'model.numshb'))
    ad = S.modl_data.read_modl(os.path.join(md, 'model.numdlb'))
    lab = materiaux_de(dos, S.matl_data.read_matl(os.path.join(md, 'model.numatb')))
    textures_de(dos)
    WH = monde_os(OSV)
    objs = [o for o in objs if o.name != nom]
    ents = [x for x in ents if x.mesh_object_name != nom]
    mat_de = {x.mesh_object_name + '/%d' % x.mesh_object_subindex: x.material_label for x in ad.entries}
    toutes = np.concatenate([np.asarray(o.positions[0].data)[:, :3] for o in am.objects])
    centre = (toutes.min(0) + toutes.max(0)) / 2 if centrer else np.zeros(3, np.float32)
    IH = np.linalg.inv(WH)
    ang = np.radians(int(nom[11:]) * 360.0 / TOURS) if nom.startswith('pr_boom_vol') else 0.0
    c, s_ = np.cos(ang), np.sin(ang)
    RX = np.array([[1, 0, 0], [0, c, s_], [0, -s_, c]], np.float32)       # rotation autour de X (convention ligne)
    if nom.endswith('_r'):
        RX = RX @ np.diag([-1.0, 1.0, -1.0]).astype(np.float32)          # demi-tour autour de Y
    for k, src in enumerate(am.objects):
        o = M.MeshObjectData(nom, k)
        p = (np.asarray(src.positions[0].data, np.float32)[:, :3] - centre) @ RX + WH[3, :3]
        o.positions = [att('Position0', M.transform_points(p, IH))]
        o.normals = [att('Normal0', M.transform_vectors(np.asarray(src.normals[0].data)[:, :3] @ RX, IH))]
        if src.tangents:
            t = np.asarray(src.tangents[0].data, np.float32)
            o.tangents = [att('Tangent0', np.c_[np.asarray(M.transform_vectors(t[:, :3] @ RX, IH), np.float32), t[:, 3:4]])]
        o.texture_coordinates = [att(u.name, u.data) for u in src.texture_coordinates]
        o.color_sets = [att(c.name, c.data) for c in src.color_sets]
        o.vertex_indices = np.asarray(src.vertex_indices, np.uint32)
        o.parent_bone_name = OSV
        o.bone_influences = []
        o.sort_bias = src.sort_bias
        objs.append(o)
        ents.append(S.modl_data.ModlEntryData(nom, k, lab[mat_de['%s/%d' % (src.name, src.subindex)]]))
mesh.objects = objs
modl.entries = ents
matl.entries = mats
mesh.save(os.path.join(D, 'model.numshb'))
matl.save(os.path.join(D, 'model.numatb'))
modl.save(os.path.join(D, 'model.numdlb'))
S.meshex_data.MeshExData.from_mesh_objects(mesh.objects).save(os.path.join(D, 'model.numshexb'))
print('armes :', [a[0] for a in ARMES] + [v[0] for v in VOLS], 'textures', textures)
