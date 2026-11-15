"""Metal Sonic : piste de visibilité « NeoMetal » (objets NeoMetal_VIS_O_OBJShape ajoutés par add_neo.py) dans les
animations du Final Smash, comme le canon de Zero (zero/tools/retarget_roy.py).

usage : python3 neo_vis.py <dossier des animations (d04final*.nuanmb)>
- d04finalstart / d04finalairstart (REX FinalStart) : Neo caché jusqu'à l'image 50 (REX : Metal disparaît et le
  Landmaster / Neo apparaît à 50), montré ensuite ; le plugin fige l'animation à la fin pendant tout le Final Smash ;
- d04finalend / d04finalairend : Neo caché (Metal revient).
"""
import os
import sys

import ssbh_data_py as S

DOS = sys.argv[1]
APPARITION = 50
A = S.anim_data
for fn, debut in (('d04finalstart', APPARITION), ('d04finalairstart', APPARITION), ('d04finalend', None), ('d04finalairend', None)):
    p = os.path.join(DOS, fn + '.nuanmb')
    if not os.path.exists(p):
        print('absent', fn)
        continue
    a = A.read_anim(p)
    n = int(a.final_frame_index) + 1
    vals = [debut is not None and f >= debut for f in range(n)]
    t = A.TrackData('Visibility')
    t.values = vals
    nd = A.NodeData('NeoMetal')
    nd.tracks = [t]
    g = A.GroupData(A.GroupType.Visibility)
    g.nodes = [nd]
    a.groups = [x for x in a.groups if x.group_type != A.GroupType.Visibility] + [g]
    a.save(p)
    print(fn, n, 'images, Neo montre a partir de', debut)
