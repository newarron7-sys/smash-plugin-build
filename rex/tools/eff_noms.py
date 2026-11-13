"""Noms contenus dans un fichier d'effets d'Ultimate (.eff : en-tête EFFN + bloc VFXB) : noms des effets appelés par
les scripts (avant VFXB) et noms des jeux d'émetteurs / textures (dans VFXB). Lecture seule, sans rien modifier.

usage : python3 eff_noms.py <fichier.eff>
"""
import re, sys

d = open(sys.argv[1], 'rb').read()
print('taille', len(d), 'magie', d[:4])
v = d.find(b'VFXB')
tete = d[:v] if v > 0 else d
noms = [m.group().decode() for m in re.finditer(rb'[A-Za-z0-9_]{4,}(?=\0)', tete)]
print('EFFETS (en-tete) :', len(noms))
for n in noms:
    print('  ', n)
if v > 0:
    corps = d[v:]
    em = sorted(set(m.group().decode() for m in re.finditer(rb'[A-Za-z][A-Za-z0-9_]{3,}(?=\0)', corps)))
    print('NOMS DANS VFXB :', len(em))
    for n in em[:400]:
        print('  ', n)
