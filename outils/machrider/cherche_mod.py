"""Cherche sur GameBanana (Smash Ultimate) des mods par mots-clés, télécharge ceux qui contiennent un dossier
sound pour un perso donné, et copie leur config.json + la liste de leurs fichiers sound (pour comprendre comment une
case ajoutée a ses sons). usage : python3 cherche_mod.py <sortie> <perso> <recherches séparées par des virgules>"""
import glob, json, os, shutil, subprocess, sys, urllib.parse, urllib.request
OUT, KIND, QS = sys.argv[1], sys.argv[2], sys.argv[3].split(',')
os.makedirs(OUT, exist_ok=True)
UA = {'User-Agent': 'Mozilla/5.0'}
def get(u):
    return urllib.request.urlopen(urllib.request.Request(u, headers=UA), timeout=300).read()
seen, cands = set(), []
for q in QS:
    u = ('https://gamebanana.com/apiv11/Util/Search/Results?_sModelName=Mod&_sOrder=best_match&_idGameRow=6498'
         '&_sSearchString=%s&_nPage=1' % urllib.parse.quote(q))
    try:
        r = json.loads(get(u))
    except Exception as e:  # noqa: BLE001
        print('RECHERCHE echec', q, e); continue
    for rec in r.get('_aRecords', []):
        if rec['_idRow'] not in seen:
            seen.add(rec['_idRow']); cands.append((rec['_idRow'], rec.get('_sName', '')))
print('CANDIDATS', cands[:20])
n = 0
for idr, name in cands[:12]:
    try:
        p = json.loads(get('https://gamebanana.com/apiv11/Mod/%d/ProfilePage' % idr))
    except Exception as e:  # noqa: BLE001
        print('PROFIL echec', idr, e); continue
    for f in p.get('_aFiles', []) or []:
        if f.get('_nFilesize', 0) > 300_000_000:
            continue
        d = os.path.join('/tmp/dl', str(idr)); os.makedirs(d, exist_ok=True)
        arc = os.path.join(d, f['_sFile'])
        try:
            open(arc, 'wb').write(get(f['_sDownloadUrl']))
        except Exception as e:  # noqa: BLE001
            print('DL echec', e); continue
        x = os.path.join(d, 'x')
        subprocess.run(['7z', 'x', '-y', '-o' + x, arc], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        snd = [os.path.relpath(v, x) for v in glob.glob(os.path.join(x, '**', '*%s*' % KIND), recursive=True)
               if '/sound/' in v.replace('\\', '/')]
        if not snd:
            continue
        o = os.path.join(OUT, '%d' % idr); os.makedirs(o, exist_ok=True)
        open(os.path.join(o, 'nom.txt'), 'w').write(name + '\n' + f['_sFile'] + '\n')
        open(os.path.join(o, 'sons.txt'), 'w').write('\n'.join(sorted(snd)) + '\n')
        for c in glob.glob(os.path.join(x, '**', 'config.json'), recursive=True):
            shutil.copy(c, os.path.join(o, 'config_%d.json' % len(os.listdir(o))))
        for b in glob.glob(os.path.join(x, '**', 'sound', '**', '*.nus3bank'), recursive=True)[:6]:
            shutil.copy(b, os.path.join(o, os.path.basename(b)))
        n += 1
        print('MOD avec sons', idr)
print('fini', n)
