"""Planet Papetoon : textures utilisées (pour l'agrandissement x4) et scène three.js pour l'image de l'écran des terrains.
Même tri et même transformation que papetoon_ys.py (objets gardés, Y0 / ECH).
usage : python3 pp_scene.py textures <STGPAPETOON.pac> <dossier png>
        python3 pp_scene.py scene <STGPAPETOON.pac> <dossier HD (_x4.png) ou ''> <dossier du rendu>"""
import json, os, sys
import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import brawl_stage  # noqa: E402

MODE, PAC = sys.argv[1:3]
ECH = float(os.environ.get('PP_ECHELLE', str(58.0 / 56.0)))
Y0 = float(os.environ.get('PP_Y0', '-4.98'))
SANS = {'stGimmickYoshiBumper', 'EffFoxReflectorStart', 'EffFoxReflector', 'EffFoxReflectorRef', 'Path'}
SANS_OBJ = {'Rail', 'MainShadow', 'PlatMidShadow', 'PlatLeftShadow', 'PlatRightShadow'}

TEX, models = brawl_stage.load(PAC)
garde = []
for _, mn, ol in models:
    if mn in SANS:
        continue
    for o in ol:
        if o['name'] in SANS_OBJ or not o['tex'] or o['tex'] not in TEX:
            continue
        garde.append((mn, o))
utilisees = sorted({o['tex'] for _, o in garde})


def alpha(t):
    return int(np.asarray(TEX[t].convert('RGBA'))[:, :, 3].min()) < 200


if MODE == 'textures':
    D = sys.argv[3]
    os.makedirs(D, exist_ok=True)
    for t in utilisees:
        TEX[t].convert('RGBA').save(os.path.join(D, t + '.png'))
        print(t, TEX[t].size)
    sys.exit(0)

HD, OUT = sys.argv[3:5]
from PIL import Image  # noqa: E402
os.makedirs(os.path.join(OUT, 'tex'), exist_ok=True)
noms = {}
for i, t in enumerate(utilisees):
    p = os.path.join(HD, t + '_x4.png') if HD else ''
    im = Image.open(p).convert('RGBA') if p and os.path.exists(p) else TEX[t].convert('RGBA')
    noms[t] = 't%02d' % i
    im.save(os.path.join(OUT, 'tex', noms[t] + '.png'))
scene = []
for mn, o in garde:
    pos = o['pos'][:, [0, 2, 1]].astype(np.float64)
    pos[:, :, 1] -= Y0
    pos *= ECH
    uv = o['uv'][:, [0, 2, 1]] if o['uv'] is not None else np.zeros(pos.shape[:2] + (2,))
    scene.append({'p': np.round(pos.reshape(-1), 4).tolist(), 'uv': np.round(uv.reshape(-1), 5).tolist(),
                  't': noms[o['tex']], 'a': bool(alpha(o['tex']))})
json.dump(scene, open(os.path.join(OUT, 'scene.json'), 'w'))
print('scène', len(scene), 'objets', sum(len(s['p']) // 9 for s in scene), 'triangles', len(noms), 'textures')
