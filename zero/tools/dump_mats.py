"""Matériaux d'un modèle Ultimate : shader, textures, états de mélange (pour choisir des modèles de matériaux)."""
import sys
import ssbh_data_py as S
m = S.matl_data.read_matl(sys.argv[1])
for e in m.entries:
    tx = {t.param_id.name: t.data for t in e.textures}
    bl = [(b.param_id.name, str(b.data.source_color), str(b.data.destination_color)) for b in e.blend_states]
    vec = {v.param_id.name: [round(x, 3) for x in v.data] for v in e.vectors}
    fl = {f.param_id.name: round(f.data, 3) for f in e.floats}
    bo = {b.param_id.name: b.data for b in e.booleans}
    print('MAT', e.material_label, e.shader_label)
    print('   tex', tx)
    print('   blend', bl, 'bool', bo, 'float', fl)
    print('   vec', vec)
