"""Matériaux d'un modèle Ultimate de référence (nom, shader, textures, mélange, paramètres)."""
import sys
import ssbh_data_py as S
for path in sys.argv[1:]:
    m = S.matl_data.read_matl(path)
    print('==', path)
    for e in m.entries:
        print('MAT', e.material_label, e.shader_label)
        print('   tex', {t.param_id.name: t.data for t in e.textures})
        for b in e.blend_states:
            print('   blend', b.param_id.name, b.data.source_color, b.data.destination_color, getattr(b.data, 'alpha_sample_to_coverage', ''))
        for r in e.rasterizer_states:
            print('   rast', r.param_id.name, r.data.cull_mode, r.data.fill_mode)
        print('   vec', {v.param_id.name: [round(x, 3) for x in v.data] for v in e.vectors})
        print('   flt', {v.param_id.name: round(v.data, 3) for v in e.floats}, 'bool', {v.param_id.name: v.data for v in e.booleans})
