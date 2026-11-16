"""Retire des partages (share-to-vanilla / share-to-added) du config.json tout fichier que le mod fournit lui-même,
et l'ajoute aux nouveaux fichiers. Sinon ARCropolis charge le fichier partagé (Ness vanilla) à la place du nôtre :
Knuckle Joe v2 (16/11) = spéciaux, dégâts, garde, bord... sans ses animations.
usage : python3 fix_partages.py <dossier du mod> <dossier new-dir-files, ex. fighter/ness/c100>"""
import json, os, sys

mod, cle = sys.argv[1], sys.argv[2]
p = os.path.join(mod, 'config.json')
c = json.load(open(p))
fournis = {os.path.relpath(os.path.join(r, f), mod) for r, _, fs in os.walk(os.path.join(mod, 'fighter')) for f in fs}
n = 0
for sec in ('share-to-vanilla', 'share-to-added'):
    for src in list(c.get(sec, {})):
        v = c[sec][src]
        l = v if isinstance(v, list) else [v]
        reste = [x for x in l if x not in fournis]
        n += len(l) - len(reste)
        if not reste:
            del c[sec][src]
        else:
            c[sec][src] = reste if isinstance(v, list) else reste[0]
ndf = c['new-dir-files'].setdefault(cle, [])
ajout = [f for f in sorted(fournis) if f not in ndf and '/c1' in f]
ndf += ajout
json.dump(c, open(p, 'w'), indent=2)
print('retirés des partages :', n, '; ajoutés aux nouveaux fichiers :', len(ajout))
