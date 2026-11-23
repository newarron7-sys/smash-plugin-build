"""Inspecte les modèles d'un dossier stage/<nom> d'un mod Ultimate : objets du numdlb, matériaux (shader, textures,
états), squelette, nombre de sommets. usage : python3 inspect_ref.py <dossier stage/<nom>>"""
import os, sys
import ssbh_data_py as S

R = sys.argv[1]
for forme in sorted(os.listdir(R)):
    M = os.path.join(R, forme, 'model')
    if not os.path.isdir(M):
        continue
    for d in sorted(os.listdir(M)):
        P = os.path.join(M, d)
        fs = sorted(os.listdir(P))
        print('=== %s/%s : %s' % (forme, d, ' '.join(f for f in fs if not f.endswith('.nutexb'))))
        print('    textures :', ' '.join(f[:-7] for f in fs if f.endswith('.nutexb')))
        try:
            modl = S.modl_data.read_modl(os.path.join(P, 'model.numdlb'))
            print('    numdlb : %d entrées, squelette %s, matériaux %s' % (len(modl.entries), modl.skeleton_file_name, modl.material_file_names))
            for e in modl.entries[:12]:
                print('      ', e.mesh_object_name, e.mesh_object_subindex, e.material_label)
        except Exception as ex:
            print('    numdlb illisible', ex)
        try:
            mesh = S.mesh_data.read_mesh(os.path.join(P, 'model.numshb'))
            for o in mesh.objects[:8]:
                print('      maillage %s/%d sommets %d attributs %s %s os %s' % (o.name, o.subindex, len(o.positions[0].data),
                      [a.name for a in o.texture_coordinates], [a.name for a in o.color_sets], [b.name for b in o.bone_influences][:3]))
            print('      (%d maillages)' % len(mesh.objects))
        except Exception as ex:
            print('    numshb :', ex)
        try:
            matl = S.matl_data.read_matl(os.path.join(P, 'model.numatb'))
            for e in matl.entries[:10]:
                print('      mat %s | %s | %s | blend %s | cull %s' % (e.material_label, e.shader_label,
                      ' '.join('%s=%s' % (t.param_id.name, t.data) for t in e.textures),
                      [(b.data.source_color.name, b.data.destination_color.name) for b in e.blend_states],
                      [r.data.cull_mode.name for r in e.rasterizer_states]))
            print('      (%d matériaux)' % len(matl.entries))
        except Exception as ex:
            print('    numatb :', ex)
