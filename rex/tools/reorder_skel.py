"""Remet les os d'un squelette de skin dans l'ordre du squelette d'origine du perso, os ajoutés à la fin.

Ultimate exige les os d'origine aux mêmes places : un skin qui insère des os au milieu (oreilles, émeraudes…)
décale tous les suivants, et le corps s'étire vers un seul os en jeu. Les maillages (numshb) et le nuhlpb
désignent les os par leur nom : seul le nusktb change.

usage : python3 reorder_skel.py <nusktb du skin> <nusktb d'origine du perso> <sortie.nusktb>
"""
import sys
import ssbh_data_py as S

SKIN, ORIG, OUT = sys.argv[1:4]
skin = S.skel_data.read_skel(SKIN)
orig = S.skel_data.read_skel(ORIG)
names = [b.name for b in skin.bones]
by = {b.name: b for b in skin.bones}
parent = {b.name: (names[b.parent_index] if b.parent_index is not None else None) for b in skin.bones}

order = [b.name for b in orig.bones if b.name in by]
manque = [b.name for b in orig.bones if b.name not in by]
extras = [n for n in names if n not in set(order)]
order += extras
pos = {n: i for i, n in enumerate(order)}
for n in extras:  # un os ajouté doit venir après son parent
    p = parent[n]
    assert p is None or pos[p] < pos[n], (n, p)

new = []
for n in order:
    b = by[n]
    b.parent_index = pos[parent[n]] if parent[n] is not None else None
    new.append(b)
skin.bones = new
skin.save(OUT)
print('os d\'origine remis en place :', len(order) - len(extras), '/', len(orig.bones))
print('os ajoutes a la fin :', extras)
if manque:
    print('ATTENTION os d\'origine absents du skin :', manque)
deplaces = [n for i, n in enumerate(names) if n in pos and pos[n] != i]
print('os qui ont change de place :', len(deplaces))
