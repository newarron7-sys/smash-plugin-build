"""Primid v0.28 : fichier d'effets de Mario (effect/fighter/mario/ef_mario.eff) pris dans des mods GameBanana (usage
perso), pour une traînée d'épée propre à Primid (ef_mario_c00.eff + trail_c00, comme la traînée de Zero sur Roy).
Le fichier le plus courant parmi les mods trouvés = celui du jeu (les recolorations sont minoritaires).

usage : python3 fetch_eff.py <perso> <sortie>
"""
import hashlib, json, os, subprocess, sys, urllib.parse, urllib.request
from collections import Counter
UA = {'User-Agent': 'Mozilla/5.0'}


def get(u):
    return urllib.request.urlopen(urllib.request.Request(u, headers=UA), timeout=600).read()


K, OUT = sys.argv[1:3]
os.makedirs(OUT, exist_ok=True)
seen = []
for q in ('%s effects' % K, '%s effect' % K, '%s fireball' % K, '%s moveset' % K, K):
    try:
        r = json.loads(get('https://gamebanana.com/apiv11/Util/Search/Results?_sModelName=Mod&_sOrder=best_match'
                           '&_idGameRow=6498&_sSearchString=%s&_nPage=1' % urllib.parse.quote(q)))
    except Exception as e:  # noqa
        print('recherche echec', q, e)
        continue
    for rec in r.get('_aRecords', []):
        if rec['_idRow'] not in seen:
            seen.append(rec['_idRow'])
CANDS = []
cible = 'effect/fighter/%s/ef_%s' % (K, K)
for mid in seen[:40]:
    if len(CANDS) >= 6:
        break
    try:
        p = json.loads(get('https://gamebanana.com/apiv11/Mod/%d/ProfilePage' % mid))
    except Exception:  # noqa
        continue
    for f in p.get('_aFiles') or []:
        if f.get('_nFilesize', 0) > 60e6:
            continue
        path = '/tmp/eff_%d_%s' % (mid, f['_sFile'])
        try:
            open(path, 'wb').write(get(f['_sDownloadUrl']))
        except Exception as e:  # noqa
            continue
        ls = subprocess.run(['7z', 'l', '-ba', '-slt', path], capture_output=True, text=True).stdout
        names = [l[7:] for l in ls.splitlines() if l.startswith('Path = ')]
        ef = [n for n in names if cible in n.replace('\\', '/').lower() and n.lower().endswith('.eff')]
        print('MOD', mid, p.get('_sName'), f['_sFile'], len(names), ef[:2])
        if ef:
            xd = '/tmp/effx_%d' % mid
            subprocess.run(['rm', '-rf', xd])
            subprocess.run(['7z', 'x', '-y', '-o' + xd, path, ef[0]], capture_output=True)
            src = os.path.join(xd, ef[0])
            if os.path.exists(src):
                d = open(src, 'rb').read()
                CANDS.append((mid, p.get('_sName'), d, hashlib.md5(d).hexdigest()))
                print('  eff', len(d), CANDS[-1][3])
            break
if not CANDS:
    sys.exit('aucun ef_%s.eff trouve' % K)
cnt = Counter(c[3] for c in CANDS)
best = max(CANDS, key=lambda c: cnt[c[3]])
open(os.path.join(OUT, 'ef_%s.eff' % K), 'wb').write(best[2])
print('CHOIX', best[0], best[1], len(best[2]), 'meme fichier dans %d mod(s) sur %d' % (cnt[best[3]], len(CANDS)))
