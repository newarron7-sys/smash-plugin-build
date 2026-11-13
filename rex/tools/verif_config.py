"""Vérification d'un mod avant livraison : fichiers de la case (config.json de reslotter.py) déclarés mais ni fournis
par le mod ni partagés avec le jeu.

Piège (Super Sonic, skin GameBanana « Modern Style ») : le skin compte sur des textures du jeu (def_sonic_001_nor, yeux,
bouches...) qu'il ne fournit pas. Sur sa case d'origine (c07) le jeu les trouve ; sur une case ajoutée (c141) ce
reslotter ne partage PAS les .nutexb d'un dossier où le mod a déjà des fichiers : elles n'existent plus, la carte de
normales manque et, comme le modèle est fait en miroir, une moitié du visage est plus sombre que l'autre avec un trait
au milieu (photos du joueur). Correction : copier les textures du jeu dans le dossier, ou ajouter les partages.

usage : python3 verif_config.py <dossier du mod> [Hashes_all.txt] [--partager]
  --partager : ajoute dans config.json un partage « share-to-vanilla » (c00 du jeu) pour chaque fichier manquant
"""
import json, os, re, sys

args = [a for a in sys.argv[1:] if not a.startswith('--')]
Z = args[0]
H = set(l.strip() for l in open(args[1])) if len(args) > 1 else None
p = os.path.join(Z, 'config.json')
c = json.load(open(p))
sv = c.setdefault('share-to-vanilla', {})
sa = c.get('share-to-added', {})
shared = set(x for d in (sv, sa) for v in d.values() for x in (v if isinstance(v, list) else [v]))
manq = [f for fs in c.get('new-dir-files', {}).values() for f in fs if not os.path.exists(os.path.join(Z, f)) and f not in shared]
# textures que les matériaux du mod utilisent vraiment (noms lus dans les .numatb)
util = set()
for d, _, fs in os.walk(Z):
    for f in fs:
        if f.endswith('.numatb'):
            raw = open(os.path.join(d, f), 'rb').read()
            for m in re.findall(rb'[A-Za-z0-9_]{4,}', raw):
                util.add((os.path.relpath(d, Z).replace(os.sep, '/'), m.decode().lower()))
n = 0
for f in manq:
    d, b = os.path.split(f)
    grave = (d, os.path.splitext(b)[0].lower()) in util
    print('%s %s' % ('UTILISE PAR LE MODELE' if grave else 'inutilisé            ', f))
    if grave and '--partager' in sys.argv:
        src = re.sub(r'/c\d+/', '/c00/', f, 1)
        if H is None or src in H:
            sv.setdefault(src, [])
            if f not in sv[src]:
                sv[src].append(f); n += 1
        else:
            print('   pas dans le jeu :', src)
print(len(manq), 'fichiers déclarés ni fournis ni partagés ;', sum(1 for f in manq if (os.path.split(f)[0], os.path.splitext(os.path.basename(f))[0].lower()) in util), 'utilisés par un matériau')
if n:
    json.dump(c, open(p, 'w'), indent=4)
    print(n, 'partages ajoutés')
