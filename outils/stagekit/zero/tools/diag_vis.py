"""Diagnostic : conventions de visibilité d'Ultimate (model.nuanmb et maillages de Roy) comparées aux fichiers de Zero."""
import sys
import ssbh_data_py as S

ROY, ZM, ZA = sys.argv[1:4]   # dossier modèle de Roy, dossier modèle de Zero, une animation de Zero
for lab, d in (('ROY', ROY), ('ZERO', ZM)):
    try:
        a = S.anim_data.read_anim(d + '/model.nuanmb')
        print(lab, 'model.nuanmb : images', a.final_frame_index, 'groupes', [(str(g.group_type), len(g.nodes)) for g in a.groups])
        for g in a.groups:
            for nd in g.nodes[:40]:
                print('   ', str(g.group_type), nd.name, [t.name for t in nd.tracks], [str(t.values[:2])[:80] for t in nd.tracks][:2])
    except Exception as e:  # noqa: BLE001
        print(lab, 'model.nuanmb illisible', e)
    m = S.mesh_data.read_mesh(d + '/model.numshb')
    print(lab, 'maillages', [(o.name, o.subindex) for o in m.objects][:60])
    md = S.modl_data.read_modl(d + '/model.numdlb')
    print(lab, 'numdlb', md.model_name, md.skeleton_file_name, md.material_file_names, md.animation_file_name, md.mesh_file_name)
    mt = S.matl_data.read_matl(d + '/model.numatb')
    for e in mt.entries:
        print(lab, 'MAT', e.material_label, e.shader_label, {t.param_id.name: t.data for t in e.textures})
import os
names = set(x[:-7] for x in os.listdir(ZM) if x.endswith('.nutexb'))
mt = S.matl_data.read_matl(ZM + '/model.numatb')
miss = sorted({t.data for e in mt.entries for t in e.textures if not t.data.startswith('#') and not t.data.startswith('/') and t.data not in names})
print('TEXTURES MANQUANTES', miss)
a = S.anim_data.read_anim(ZA)
for g in a.groups:
    print('ZERO ANIM groupe', str(g.group_type), len(g.nodes), [nd.name for nd in g.nodes][:8])
