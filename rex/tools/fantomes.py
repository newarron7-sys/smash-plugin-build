"""Knuckle Joe : copies transparentes de son corps pour le haut B (Rising Break). Dans le showcase de REX, en montant
il laisse plusieurs images de lui-même, transparentes, en dessous de lui.
Ultimate n'a pas de « traînée du corps » générique : on ajoute au modèle 3 copies du corps (objets kjoe_fantome1 à 3),
liées aux mêmes os mais décalées vers le bas (1, 2 et 3 fois DECALAGE de sa hauteur), de plus en plus transparentes
et teintées bleu-blanc, dans un matériau transparent (alp_ du modèle de référence de Ness, passe _sort). Cachées tout
le temps, le plugin les montre pendant la montée (set_mesh_visibility).
usage : python3 fantomes.py <dossier direct (numshb)> <dossier model (numdlb, numatb)> <référence Ness> <dossier png>"""
import os
import sys

import numpy as np
import ssbh_data_py as S
from PIL import Image

DIRECT, MODEL, REF, PNG = sys.argv[1:5]
A = S.mesh_data
NB = 3
DECALAGE = 0.42                      # fraction de sa hauteur entre deux copies
ALPHAS = [120, 75, 40]
TEINTE = (175, 215, 255)


def attr(name, data):
    a = A.AttributeData(name)
    a.data = np.asarray(data, dtype=np.float32)
    return a


def copie_attrs(lst, decal=None):
    out = []
    for a in lst:
        d = np.asarray(a.data, dtype=np.float32).copy()
        if decal is not None:
            d[:, 1] += decal
        out.append(attr(a.name, d))
    return out


mesh = A.read_mesh(os.path.join(DIRECT, 'model.numshb'))
modl = S.modl_data.read_modl(os.path.join(MODEL, 'model.numdlb'))
matl = S.matl_data.read_matl(os.path.join(MODEL, 'model.numatb'))
base = [o for o in mesh.objects if not o.name.startswith('kjoe_fantome')]
mat_de = {(e.mesh_object_name, e.mesh_object_subindex): e.material_label for e in modl.entries}
ys = np.concatenate([np.asarray(o.positions[0].data)[:, 1] for o in base])
haut = float(ys.max() - ys.min())
print('hauteur du modèle', round(haut, 2), '; objets', len(base))

# matériau transparent : alp_ de la référence de Ness (sinon le matériau du corps, passé en _sort)
ref = S.matl_data.read_matl(os.path.join(REF, 'model.numatb')).entries
gab = next((e for e in ref if e.shader_label.endswith('_sort')), None) or \
    next((e for e in ref if e.material_label.startswith('alp_')), None)
GAB_REF = gab is not None
if gab is None:
    gab = matl.entries[0]
print('gabarit', gab.material_label, gab.shader_label, '(référence de Ness)' if GAB_REF else '(corps)')
corps = matl.entries[0]
tex_corps = {t.param_id.name: t.data for t in corps.textures}
col_corps = tex_corps.get('Texture0', 'kjoe_0_model0_col')
src = os.path.join(PNG, col_corps + '.png')
im0 = Image.open(src).convert('RGBA') if os.path.exists(src) else Image.new('RGBA', (8, 8), (255, 255, 255, 255))

new_objs, new_ents, new_mats = [], [], []
for k in range(1, NB + 1):
    col = 'kjoe_fantome%d_col' % k
    a = np.asarray(im0).astype(np.float32)
    a[..., :3] = a[..., :3] * 0.45 + np.array(TEINTE, dtype=np.float32) * 0.55
    a[..., 3] = ALPHAS[k - 1]
    Image.fromarray(a.clip(0, 255).astype(np.uint8), 'RGBA').save(os.path.join(PNG, col + '.png'))
    lab = 'alp_kjoe_fantome%d' % k
    e = next(x for x in S.matl_data.read_matl(os.path.join(REF if GAB_REF else MODEL, 'model.numatb')).entries
             if x.material_label == gab.material_label)
    e.material_label = lab
    if not e.shader_label.endswith('_sort'):
        e.shader_label = e.shader_label.rsplit('_', 1)[0] + '_sort'
    for t in e.textures:
        n = t.param_id.name
        if n in ('Texture0', 'Texture5'):
            t.data = col
        elif n in tex_corps:
            t.data = tex_corps[n]
    try:
        for b in e.blend_states:
            b.data.source_color = S.matl_data.BlendFactor.SourceAlpha
            b.data.destination_color = S.matl_data.BlendFactor.OneMinusSourceAlpha
    except Exception as ex:  # noqa: BLE001
        print('blend non réglé :', ex)
    for v in e.vectors:
        if v.param_id.name == 'CustomVector0':
            v.data = [0.0, v.data[1], v.data[2], v.data[3]]   # pas de test d'alpha (sinon les copies disparaissent)
    new_mats.append(e)
    dy = -DECALAGE * haut * k
    for o in base:
        c = A.MeshObjectData('kjoe_fantome%d' % k, len([x for x in new_objs if x.name == 'kjoe_fantome%d' % k]))
        c.positions = copie_attrs(o.positions, dy)
        c.normals = copie_attrs(o.normals)
        c.binormals = copie_attrs(o.binormals)
        c.tangents = copie_attrs(o.tangents)
        c.texture_coordinates = copie_attrs(o.texture_coordinates)
        c.color_sets = copie_attrs(o.color_sets)
        c.vertex_indices = np.asarray(o.vertex_indices, dtype=np.uint32).copy()
        c.bone_influences = [A.BoneInfluence(b.bone_name, [A.VertexWeight(w.vertex_index, w.vertex_weight)
                                                          for w in b.vertex_weights]) for b in o.bone_influences]
        c.parent_bone_name = o.parent_bone_name
        c.sort_bias = 10 * k
        try:
            c.disable_depth_write = True
        except Exception:  # noqa: BLE001
            pass
        new_objs.append(c)
        new_ents.append(S.modl_data.ModlEntryData(c.name, c.subindex, lab))
    print('copie', k, ': décalage', round(dy, 2), ', alpha', ALPHAS[k - 1], ',', lab, e.shader_label)

mesh.objects = base + new_objs
mesh.save(os.path.join(DIRECT, 'model.numshb'))
modl.entries = [x for x in modl.entries if not x.mesh_object_name.startswith('kjoe_fantome')] + new_ents
modl.save(os.path.join(MODEL, 'model.numdlb'))
matl.entries = [x for x in matl.entries if not x.material_label.startswith('alp_kjoe_fantome')] + new_mats
matl.save(os.path.join(MODEL, 'model.numatb'))
try:
    S.meshex_data.MeshExData.from_mesh_objects(mesh.objects).save(os.path.join(MODEL, 'model.numshexb'))
except Exception as ex:  # noqa: BLE001
    print('numshexb non généré :', ex)
print('FANTOMES OK', len(new_objs), 'objets')
