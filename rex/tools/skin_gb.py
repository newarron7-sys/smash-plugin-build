"""Télécharge un skin GameBanana (usage perso) et en sort le dossier modèle du perso voulu, mis en c00.

usage : python3 skin_gb.py <id GameBanana> <nom interne (dolly…)> <fichiers c00 du jeu (liste)> <dossier de sortie>
Sortie : <sortie>/model (fichiers du modèle) et <sortie>/nouveaux.txt (noms absents du jeu, à déclarer à ARCropolis).
"""
import json, os, re, subprocess, sys, urllib.request
MID, KIND, LISTE, OUT = int(sys.argv[1]), sys.argv[2], sys.argv[3], sys.argv[4]
UA = {'User-Agent': 'Mozilla/5.0'}
get = lambda u: urllib.request.urlopen(urllib.request.Request(u, headers=UA), timeout=600).read()
p = json.loads(get('https://gamebanana.com/apiv11/Mod/%d/ProfilePage' % MID))
print('MOD', MID, p.get('_sName'))
os.makedirs(OUT, exist_ok=True)
best = None
for f in p.get('_aFiles') or []:
    path = '/tmp/skin_%d_%s' % (MID, f['_sFile'])
    open(path, 'wb').write(get(f['_sDownloadUrl']))
    subprocess.run(['rm', '-rf', '/tmp/skinx'])
    subprocess.run(['7z', 'x', '-y', '-o/tmp/skinx', path], capture_output=True)
    for r, ds, fs in os.walk('/tmp/skinx'):
        rr = r.replace('\\', '/').lower()
        if re.search(r'fighter/%s/model/body/c0\d$' % KIND, rr) and any(x.endswith('.numshb') for x in fs):
            best = r
            print('MODELE', f['_sFile'], r, sorted(fs))
            break
    if best:
        break
if not best:
    sys.exit('pas de modele %s dans le mod %d' % (KIND, MID))
subprocess.run(['rm', '-rf', os.path.join(OUT, 'model')])
subprocess.run(['cp', '-r', best, os.path.join(OUT, 'model')])
jeu = set(l.strip() for l in open(LISTE) if l.strip())
nv = sorted(x for x in os.listdir(os.path.join(OUT, 'model')) if x not in jeu)
open(os.path.join(OUT, 'nouveaux.txt'), 'w').write('\n'.join(nv) + '\n')
print('fichiers', len(os.listdir(os.path.join(OUT, 'model'))), '; absents du jeu (a declarer) :', nv)
# autres fichiers du skin (interface…) : listés seulement
for r, ds, fs in os.walk('/tmp/skinx'):
    for x in fs:
        if 'ui/' in os.path.join(r, x).replace('\\', '/'):
            print('UI', os.path.join(r, x))
