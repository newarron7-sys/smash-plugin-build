"""Squelette et modèle de référence de Roy (Ultimate) : pris dans un skin de Roy sur GameBanana (usage perso).

usage : python3 roy_ref.py <sortie>
"""
import json, os, subprocess, sys, urllib.parse, urllib.request
UA = {'User-Agent': 'Mozilla/5.0'}


def get(u):
    return urllib.request.urlopen(urllib.request.Request(u, headers=UA), timeout=600).read()


out = sys.argv[1]
os.makedirs(out, exist_ok=True)
seen = []
for q in ('roy skin', 'roy'):
    u = ('https://gamebanana.com/apiv11/Util/Search/Results?_sModelName=Mod&_sOrder=best_match&_idGameRow=6498'
         '&_sSearchString=%s&_nPage=1' % urllib.parse.quote(q))
    try:
        r = json.loads(get(u))
    except Exception as e:  # noqa
        print('recherche echec', e)
        continue
    for rec in r.get('_aRecords', []):
        if rec['_idRow'] not in seen:
            seen.append(rec['_idRow'])
            print('MOD', rec['_idRow'], rec.get('_sName'))
found = False
for mid in seen[:25]:
    try:
        p = json.loads(get('https://gamebanana.com/apiv11/Mod/%d/ProfilePage' % mid))
    except Exception as e:  # noqa
        continue
    for f in p.get('_aFiles') or []:
        if f.get('_nFilesize', 0) > 80e6:
            continue
        path = os.path.join('/tmp', 'roy_%d_%s' % (mid, f['_sFile']))
        open(path, 'wb').write(get(f['_sDownloadUrl']))
        ls = subprocess.run(['7z', 'l', '-ba', '-slt', path], capture_output=True, text=True).stdout
        names = [l[7:] for l in ls.splitlines() if l.startswith('Path = ')]
        sk = [n for n in names if n.replace('\\', '/').lower().endswith('fighter/roy/model/body/c00/model.nusktb')]
        sk = sk or [n for n in names if 'fighter/roy/model/body/c0' in n.replace('\\', '/').lower() and n.lower().endswith('model.nusktb')]
        print('FICHIER', mid, f['_sFile'], len(names), 'squelette' if sk else '')
        if sk:
            d = sk[0].replace('\\', '/').rsplit('/', 1)[0]
            subprocess.run(['7z', 'x', '-y', '-o/tmp/royx', path], capture_output=True)
            src = os.path.join('/tmp/royx', d)
            dst = os.path.join(out, 'roy_ref')
            subprocess.run(['cp', '-r', src, dst])
            print('REFERENCE', p.get('_sName'), d, sorted(os.listdir(dst)))
            found = True
            break
    if found:
        break
if found:
    import ssbh_data_py as S
    s = S.skel_data.read_skel(os.path.join(out, 'roy_ref', 'model.nusktb'))
    print('OS', len(s.bones))
    for b in s.bones:
        print('  %-24s parent %s' % (b.name, s.bones[b.parent_index].name if b.parent_index is not None else None))
