"""Primid v0.30 : textures de traînée d'épée d'un perso qui en a (Roy, Marth…) prises dans des mods GameBanana (usage
perso), pour voir leur format et faire la traînée verte de Primid dans le même format.

usage : python3 fetch_trail.py <sortie>
"""
import json, os, subprocess, sys, urllib.parse, urllib.request
UA = {'User-Agent': 'Mozilla/5.0'}


def get(u):
    return urllib.request.urlopen(urllib.request.Request(u, headers=UA), timeout=600).read()


OUT = sys.argv[1]
os.makedirs(OUT, exist_ok=True)
seen = []
for q in ('Roy sword trail', 'Marth sword trail', 'sword trail', 'Roy effects', 'Marth effects', 'Lucina sword trail'):
    try:
        r = json.loads(get('https://gamebanana.com/apiv11/Util/Search/Results?_sModelName=Mod&_sOrder=best_match'
                           '&_idGameRow=6498&_sSearchString=%s&_nPage=1' % urllib.parse.quote(q)))
    except Exception as e:  # noqa
        continue
    for rec in r.get('_aRecords', []):
        if rec['_idRow'] not in seen:
            seen.append(rec['_idRow'])
n = 0
for mid in seen[:30]:
    if n >= 4:
        break
    try:
        p = json.loads(get('https://gamebanana.com/apiv11/Mod/%d/ProfilePage' % mid))
    except Exception:  # noqa
        continue
    for f in p.get('_aFiles') or []:
        if f.get('_nFilesize', 0) > 60e6:
            continue
        path = '/tmp/tr_%d_%s' % (mid, f['_sFile'])
        try:
            open(path, 'wb').write(get(f['_sDownloadUrl']))
        except Exception:  # noqa
            continue
        ls = subprocess.run(['7z', 'l', '-ba', '-slt', path], capture_output=True, text=True).stdout
        names = [l[7:] for l in ls.splitlines() if l.startswith('Path = ')]
        tr = [x for x in names if '/trail' in x.replace('\\', '/').lower() and x.lower().endswith('.nutexb')]
        eff = [x for x in names if x.lower().endswith('.eff')]
        print('MOD', mid, p.get('_sName'), f['_sFile'], len(names), tr[:6], eff[:2])
        if tr:
            xd = os.path.join(OUT, 'mod_%d' % mid)
            subprocess.run(['7z', 'x', '-y', '-o' + xd, path] + tr, capture_output=True)
            n += 1
            break
