"""config.json d'ARCropolis pour un mod de stage : les fichiers qui n'existent pas dans le jeu d'origine (nos textures)
doivent être déclarés, sinon le jeu ne les charge pas (v0.1 : décor tout blanc, textures absentes).

usage : python3 make_config.py <dossier du mod> <nom du stage (ex. dolly_stadium)> <motif des nouveaux fichiers (ex. cot_)>
-> "new-dir-files" : { "stage/<nom>/normal": [chemins des nouveaux fichiers] }
"""
import json, os, sys

MOD, STAGE, PAT = sys.argv[1:4]
FORM = sys.argv[4] if len(sys.argv) > 4 else 'normal'          # normal ou battle (forme Ω / Champ de bataille)
new = []
for r, _, fs in os.walk(MOD):
    for f in fs:
        if f.startswith(PAT):
            new.append(os.path.relpath(os.path.join(r, f), MOD).replace(os.sep, '/'))
cfg = {'new-dir-files': {'stage/%s/%s' % (STAGE, FORM): sorted(new)}}
json.dump(cfg, open(os.path.join(MOD, 'config.json'), 'w'), indent=2)
print('config.json :', len(new), 'nouveaux fichiers')
