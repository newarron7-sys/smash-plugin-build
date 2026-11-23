"""Inspecte un mod qui remplace le Ganon du Final Smash (dossier ganonboss) : fichiers, squelette, maillages.

usage : python3 ganonboss_scan.py <dossier extrait>
"""
import os
import sys

import numpy as np
import ssbh_data_py as S

root = sys.argv[1]
for dp, dn, fn in os.walk(root):
    for f in fn:
        p = os.path.join(dp, f)
        print('FICHIER', os.path.relpath(p, root), os.path.getsize(p))
for dp, dn, fn in os.walk(root):
    if 'model.nusktb' in fn:
        sk = S.skel_data.read_skel(os.path.join(dp, 'model.nusktb'))
        print('SQUELETTE', os.path.relpath(dp, root), len(sk.bones), 'os')
        for b in sk.bones:
            w = np.array(sk.calculate_world_transform(b)).T
            par = sk.bones[b.parent_index].name if b.parent_index is not None and b.parent_index >= 0 else None
            print('OS %-24s parent %-20s pos %s' % (b.name, par, np.round(w[:3, 3], 2).tolist()))
    if 'model.numshb' in fn:
        m = S.mesh_data.read_mesh(os.path.join(dp, 'model.numshb'))
        for o in m.objects:
            infl = sorted({i.bone_name for i in o.bone_influences})
            attrs = [a.name for a in o.positions + o.normals + o.tangents + o.texture_coordinates + o.color_sets]
            print('MAILLAGE %s/%d parent=%s attributs=%s os=%s' % (o.name, o.subindex, o.parent_bone_name, attrs, infl[:12]))
    if 'model.numdlb' in fn:
        md = S.modl_data.read_modl(os.path.join(dp, 'model.numdlb'))
        for e in md.entries:
            print('MODL', e.mesh_object_name, e.mesh_object_subindex, e.material_label)
    if 'model.numatb' in fn:
        mt = S.matl_data.read_matl(os.path.join(dp, 'model.numatb'))
        for e in mt.entries:
            print('MATL', e.material_label, e.shader_label, [(str(t.param_id), t.data) for t in e.textures])
