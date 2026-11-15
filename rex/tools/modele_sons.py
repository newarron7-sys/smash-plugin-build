"""Cherche sur GameBanana des mods qui contiennent une banque complète du perso de base (se_<perso>*.nus3audio et
vc_<perso>*.nus3audio) : modèles des noms et de l'ordre des pistes (comme black-shadow/tools/audio/template.py, qui ne
cherchait que la voix). Les pistes qu'on ne remplace pas gardent le contenu du modèle.

usage : python modele_sons.py <sortie> <perso> "<recherche 1>,<recherche 2>,..."
"""
import glob, json, os, shutil, subprocess, sys, urllib.parse, urllib.request
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '../../outils'))
import nus3  # noqa: E402

OUT, KIND = sys.argv[1:3]
QUERIES = sys.argv[3].split(',')
os.makedirs(OUT, exist_ok=True)
UA = {'User-Agent': 'Mozilla/5.0'}


def get(u):
    return urllib.request.urlopen(urllib.request.Request(u, headers=UA), timeout=300).read()


seen, cands = set(), []
for q in QUERIES:
    for model in ('Mod', 'Sound'):
        u = ('https://gamebanana.com/apiv11/Util/Search/Results?_sModelName=%s&_sOrder=best_match&_idGameRow=6498'
             '&_sSearchString=%s&_nPage=1&_nPerpage=30' % (model, urllib.parse.quote(q)))
        try:
            r = json.loads(get(u))
        except Exception as e:  # noqa: BLE001
            print('RECHERCHE echec', model, q, e)
            continue
        for rec in r.get('_aRecords', []):
            key = (rec.get('_sModelName'), rec.get('_idRow'))
            if key not in seen:
                seen.add(key)
                cands.append(key + (rec.get('_sName', ''),))
print('CANDIDATS', len(cands))
best = {'se': None, 'vc': None}
vus = 0
for model, idr, name in cands:
    if vus >= 30 or (best['se'] and best['vc'] and best['se'][0] >= 80 and best['vc'][0] >= 40):
        break
    try:
        p = json.loads(get('https://gamebanana.com/apiv11/%s/%d/ProfilePage' % (model, idr)))
    except Exception as e:  # noqa: BLE001
        print('PROFIL echec', idr, e)
        continue
    for f in p.get('_aFiles', []) or []:
        if f.get('_nFilesize', 0) > 200_000_000:
            continue
        # arborescence de l'archive (sans la télécharger) : seulement celles qui ont une banque du perso
        try:
            tree = json.dumps(json.loads(get('https://gamebanana.com/apiv11/File/%d' % f['_idRow'])).get('_aMetadata', {}))
        except Exception:  # noqa: BLE001
            tree = ''
        if 'se_%s' % KIND not in tree and 'vc_%s' % KIND not in tree:
            continue
        vus += 1
        d = os.path.join(OUT, 'dl', str(f['_idRow']))
        os.makedirs(d, exist_ok=True)
        arc = os.path.join(d, 'a.bin')
        try:
            open(arc, 'wb').write(get(f['_sDownloadUrl']))
        except Exception as e:  # noqa: BLE001
            print('DL echec', idr, e)
            continue
        x = os.path.join(d, 'x')
        subprocess.run(['7z', 'x', '-y', '-o' + x, arc], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        for k in ('se', 'vc'):
            for v in glob.glob(os.path.join(x, '**', '%s_%s*.nus3audio' % (k, KIND)), recursive=True):
                if 'cheer' in os.path.basename(v):
                    continue
                try:
                    tr = nus3.read(v)
                except Exception as e:  # noqa: BLE001
                    print('LECTURE echec', os.path.basename(v), e)
                    continue
                print('MODELE', k, idr, name, '|', os.path.relpath(v, x), len(tr), 'pistes', os.path.getsize(v))
                if best[k] is None or len(tr) > best[k][0]:
                    best[k] = (len(tr), v, idr, name)
                    shutil.copy(v, os.path.join(OUT, '%s_%s_modele.nus3audio' % (k, KIND)))
                    with open(os.path.join(OUT, '%s_%s_pistes.txt' % (k, KIND)), 'w') as o:
                        o.write('MOD %s %s %s\n' % (idr, name, os.path.relpath(v, x)))
                        for t in tr:
                            o.write('TRACK %4d %-44s %8d\n' % tuple(t[:3]))
                # banque (nus3bank) du même mod, pour référence
                for b_ in glob.glob(os.path.join(os.path.dirname(v), '%s_%s*.nus3bank' % (k, KIND))):
                    shutil.copy(b_, os.path.join(OUT, '%s_%s_modele.nus3bank' % (k, KIND)))
        shutil.rmtree(x, ignore_errors=True)
        os.remove(arc)
for k in ('se', 'vc'):
    if best[k]:
        # (le dossier x a été retiré : la copie est faite au moment du choix ci-dessous)
        print('CHOISI', k, best[k][2], best[k][3], best[k][0], 'pistes')
    else:
        print('AUCUN MODELE', k)
