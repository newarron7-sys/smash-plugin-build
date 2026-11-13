"""v0.29 : matériaux plus proches de ceux d'Ultimate : peau avec diffusion sous la surface (shader de peau d'Ultimate,
CustomVector11 = couleur sous la peau, CustomVector30 = intensité / douceur), cheveux avec reflet étiré (shader
anisotrope, CustomFloat10). Le canal rouge de la PRM de la peau sert de masque de peau (mis à 1).

usage : python3 materiaux_rock.py <model.numatb> [sortie]
"""
import sys
import ssbh_data_py as S

src = sys.argv[1]
dst = sys.argv[2] if len(sys.argv) > 2 else src
mt = S.matl_data.read_matl(src)
P = S.matl_data.ParamId
for e in mt.entries:
    if e.material_label == 'Skin':
        e.shader_label = 'SFX_PBS_010000000808826b_opaque'
        e.vectors.append(S.matl_data.Vector4Param(P.CustomVector11, [0.25, 0.035, 0.03, 1.0]))
        e.vectors.append(S.matl_data.Vector4Param(P.CustomVector30, [0.6, 1.5, 0.0, 0.0]))
    if e.material_label == 'Hair':
        e.shader_label = 'SFX_PBS_0100080008088269_opaque'
        e.floats.append(S.matl_data.FloatParam(P.CustomFloat10, 0.7))
mt.save(dst)
print('materiaux : peau et cheveux mis a jour')
