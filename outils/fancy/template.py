"""Cherche sur GameBanana un mod de voix de Ganondorf qui contient un vc_ganon.nus3audio complet (modèle des noms
et de l'ordre des pistes), le télécharge et liste ses pistes.

usage : python3 template.py <dossier de sortie>
"""
import glob
import json
import os
import shutil
import subprocess
import sys
import urllib.parse
import urllib.request

sys.path.insert(0, os.path.dirname(__file__))
import nus3  # noqa: E402

OUT = sys.argv[1]
# v2 : autre perso (nom interne, recherches) : template.py <sortie> dolly "terry voice,terry voice pack"
KIND = sys.argv[2] if len(sys.argv) > 2 else 'ganon'
QUERIES = sys.argv[3].split(',') if len(sys.argv) > 3 else ['ganondorf voice', 'ganondorf voice pack', 'ganon voice', 'ganondorf sound']
os.makedirs(OUT, exist_ok=True)
UA = {'User-Agent': 'Mozilla/5.0'}


def get(u):
    return urllib.request.urlopen(urllib.request.Request(u, headers=UA), timeout=300).read()


seen = set()
cands = []
for q in QUERIES:
    for model in ('Mod', 'Sound'):
        u = ('https://gamebanana.com/apiv11/Util/Search/Results?_sModelName=%s&_sOrder=best_match&_idGameRow=6498'
             '&_sSearchString=%s&_nPage=1' % (model, urllib.parse.quote(q)))
        try:
            r = json.loads(get(u))
        except Exception as e:  # noqa: BLE001
            print('RECHERCHE echec', model, q, e)
            continue
        for rec in r.get('_aRecords', []):
            key = (rec.get('_sModelName'), rec.get('_idRow'))
            if key in seen:
                continue
            seen.add(key)
            cands.append(key + (rec.get('_sName', ''),))
print('CANDIDATS', len(cands))
for c in cands[:40]:
    print('CAND', c)

best = None
for model, idr, name in cands[:25]:
    try:
        p = json.loads(get('https://gamebanana.com/apiv11/%s/%d/ProfilePage' % (model, idr)))
    except Exception as e:  # noqa: BLE001
        print('PROFIL echec', idr, e)
        continue
    for f in p.get('_aFiles', []) or []:
        fn = f.get('_sFile', '')
        if f.get('_nFilesize', 0) > 150_000_000:
            continue
        d = os.path.join(OUT, 'dl', str(idr))
        os.makedirs(d, exist_ok=True)
        arc = os.path.join(d, fn)
        try:
            open(arc, 'wb').write(get(f['_sDownloadUrl']))
        except Exception as e:  # noqa: BLE001
            print('DL echec', fn, e)
            continue
        x = os.path.join(d, 'x')
        subprocess.run(['7z', 'x', '-y', '-o' + x, arc], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        for v in glob.glob(os.path.join(x, '**', 'vc_%s*.nus3audio' % KIND), recursive=True):
            try:
                tr = nus3.read(v)
            except Exception as e:  # noqa: BLE001
                print('LECTURE echec', v, e)
                continue
            print('MODELE', idr, name, os.path.relpath(v, x), len(tr), 'pistes')
            if best is None or len(tr) > best[0]:
                best = (len(tr), v, idr, name)
        for v in glob.glob(os.path.join(x, '**', 'vc_%s*.nus3bank' % KIND), recursive=True):
            print('BANQUE', idr, os.path.relpath(v, x))
            shutil.copy(v, os.path.join(OUT, 'vc_%s.nus3bank.' % KIND + str(idr)))
    if best and best[0] >= 40:
        break
if best:
    shutil.copy(best[1], os.path.join(OUT, 'vc_%s_modele.nus3audio' % KIND))
    print('CHOISI', best[2], best[3], best[0], 'pistes')
    for t in nus3.read(best[1]):
        print('TRACK %4d %-40s %8d %s' % t)
else:
    print('AUCUN MODELE')
