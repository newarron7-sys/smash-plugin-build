"""Textures d'interface de REX (fichiers .brres / .pac des dossiers info, menu, menu2) -> .png (usage perso).
Reprend le décodeur TEX0 de black-shadow/tools/ui/mod_ui.py.

usage : python3 rex_ui_tex.py <sortie> <fichier>...
"""
import os, sys
HERE = os.path.dirname(os.path.abspath(__file__))
SRC = open(os.path.join(HERE, '../../black-shadow/tools/ui/mod_ui.py')).read()
OUT = sys.argv[1]
os.makedirs(os.path.join(OUT, 'textures'), exist_ok=True)
sys.path.insert(0, os.path.join(HERE, '../../black-shadow/tools'))
# seulement les fonctions (pas le téléchargement du mod de Black Shadow)
code = SRC[SRC.index('def lz77'):SRC.rindex('for n in names:')]
g = {'__name__': 'rex_ui', 'OUT': OUT}
exec('import io, json, os, struct, sys\nfrom brres import brres_files\nfrom PIL import Image, ImageDraw\n' + code, g)
for f in sys.argv[2:]:
    g['scan'](open(f, 'rb').read(), os.path.basename(f))
for fn, w, h, fmt in g['found']:
    print('TEX %-60s %4dx%-4d fmt %s' % (fn, w, h, fmt))
