"""Primid v0.39 : Cymul en chrome (le joueur, vidéo de REX : « il faut qu'il soit plus brillant »).

Dans Brawl, son corps est une texture presque noire éclairée par un reflet d'environnement (sphère chromée violette).
Ultimate : couleur de base argent-lavande (texture d'origine gardée pour les détails) et PRM métal 1, rugosité 0,15,
spéculaire 0,5 : le cubemap de la scène se reflète dessus.
usage : python3 cymul_chrome.py <inv/cymul/png> <dossier de sortie (model/pump/c00)>
"""
import os, subprocess, sys
import numpy as np
from PIL import Image

PNG, OUT = sys.argv[1:3]
TMP = os.path.join(OUT, '_chrome_tmp.png')
a = np.asarray(Image.open(os.path.join(PNG, 'cymul_enmcymalbody00_col.png')).convert('RGBA')).astype(np.float32)
teinte = np.array([0.92, 0.88, 1.10], np.float32)
a[..., :3] = np.clip((175.0 + (a[..., :3] - 19.0) * 1.6) * teinte, 0, 255)
Image.fromarray(a.astype(np.uint8)).save(TMP)
subprocess.run(['ultimate_tex_cli', TMP, os.path.join(OUT, 'cymul_enmcymalbody00_col.nutexb'), '--format', 'BC7RgbaUnormSrgb'],
               check=True, capture_output=True)
p = np.asarray(Image.open(os.path.join(PNG, 'cymul_enmcymalbody00_m00_r45_s35_prm.png')).convert('RGBA')).astype(np.float32)
p[..., 0] = 90           # métal 0,35 (v0.40 : métal 1 reflétait la couleur du stage : rouge sur un stage de lave)
p[..., 1] = 51           # rugosité 0,2
p[..., 3] = 128          # spéculaire 0,5 (AO gardé)
Image.fromarray(p.astype(np.uint8)).save(TMP)
subprocess.run(['ultimate_tex_cli', TMP, os.path.join(OUT, 'cymul_enmcymalbody00_m00_r45_s35_prm.nutexb'), '--format', 'BC7RgbaUnorm'],
               check=True, capture_output=True)
os.remove(TMP)
print('Cymul chromé')
