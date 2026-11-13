"""Squelette et modèle de référence d'un perso d'Ultimate : pris dans un skin de ce perso sur GameBanana (usage perso).

usage : python3 ref_fighter.py <nom interne (captain…)> <recherche GameBanana> <sortie>
"""
import json, os, subprocess, sys, urllib.parse, urllib.request
UA = {'User-Agent': 'Mozilla/5.0'}


def get(u):
    return urllib.request.urlopen(urllib.request.Request(u, headers=UA), timeout=600).read()


KIND, QUERY, out = sys.argv[1], sys.argv[2], sys.argv[3]
os.makedirs(out, exist_ok=True)
seen = []
# skin choisi par le propriétaire (REF_MOD=id GameBanana) : essayé en premier, la recherche ne sert que de secours
if os.environ.get('REF_MOD'):
    seen.append(int(os.environ['REF_MOD']))
    print('MOD choisi', seen[0])
for q in (QUERY + ' skin', QUERY):
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
# v0.7 : plusieurs skins valides sont gardés, puis on prend le squelette le plus courant (= celui du jeu : les skins qui
# changent le squelette sont minoritaires). MAJORITE=0 : le premier valide, comme avant.
CANDS = []
NMAX = int(os.environ.get('MAJORITE', '6')) or 1
for mid in seen[:25]:
    if len(CANDS) >= NMAX:
        break
    try:
        p = json.loads(get('https://gamebanana.com/apiv11/Mod/%d/ProfilePage' % mid))
    except Exception as e:  # noqa
        continue
    for f in p.get('_aFiles') or []:
        if f.get('_nFilesize', 0) > 80e6:
            continue
        path = os.path.join('/tmp', 'ref_%d_%s' % (mid, f['_sFile']))
        try:
            open(path, 'wb').write(get(f['_sDownloadUrl']))
        except Exception as e:  # noqa
            print('telechargement echec', mid, e)
            continue
        ls = subprocess.run(['7z', 'l', '-ba', '-slt', path], capture_output=True, text=True).stdout
        names = [l[7:] for l in ls.splitlines() if l.startswith('Path = ')]
        sk = [n for n in names if n.replace('\\', '/').lower().endswith('fighter/%s/model/body/c00/model.nusktb' % KIND)]
        sk = sk or [n for n in names if ('fighter/%s/model/body/c0' % KIND) in n.replace('\\', '/').lower() and n.lower().endswith('model.nusktb')]
        print('FICHIER', mid, f['_sFile'], len(names), 'squelette' if sk else '')
        if sk:
            _d = sk[0].replace('\\', '/').rsplit('/', 1)[0].lower()
            _need = ('model.numatb', 'model.numdlb', 'model.numshb', 'model.nuhlpb')
            _have = [n.replace('\\', '/').lower() for n in names]
            if not all((_d + '/' + x) in _have for x in _need):
                print('  incomplet (il faut', _need, ')')
                sk = []
        if sk:
            d = sk[0].replace('\\', '/').rsplit('/', 1)[0]
            xd = '/tmp/refx_%d' % mid
            subprocess.run(['rm', '-rf', xd])
            subprocess.run(['7z', 'x', '-y', '-o' + xd, path], capture_output=True)
            src = os.path.join(xd, d)
            try:
                import ssbh_data_py as _S
                _labs = [e.material_label for e in _S.matl_data.read_matl(os.path.join(src, 'model.numatb')).entries]
                _sk = _S.skel_data.read_skel(os.path.join(src, 'model.nusktb'))
                _sig = tuple((b.name, b.parent_index) for b in _sk.bones)
            except Exception as _e:  # noqa: BLE001
                _labs, _sig = [], None
            # matériaux d'origine : noms d'Ultimate (def_/skin_/alp_<perso>, EyeL…) ; ceux d'un logiciel 3D = fait maison
            _bad = [l for l in _labs if any(k in l.lower() for k in ('material.', 'lambert', 'blinn'))]
            print('  materiaux :', _labs[:8], '; os :', len(_sig) if _sig else '?')
            if _bad or not _labs or not _sig:
                print('  materiaux faits maison ou illisible, on passe')
                continue
            CANDS.append((mid, p.get('_sName'), src, _sig))
            break
if CANDS:
    from collections import Counter
    cnt = Counter(c[3] for c in CANDS)
    for c in CANDS:
        print('CANDIDAT', c[0], c[1], len(c[3]), 'os ; meme squelette que', cnt[c[3]] - 1, 'autre(s)')
    best = max(CANDS, key=lambda c: (cnt[c[3]], 1 if (os.environ.get('REF_MOD') and c[0] == int(os.environ['REF_MOD'])) else 0))
    dst = os.path.join(out, 'ref')
    subprocess.run(['rm', '-rf', dst])
    subprocess.run(['cp', '-r', best[2], dst])
    print('REFERENCE', best[0], best[1], '(squelette partage par %d skin(s) sur %d)' % (cnt[best[3]], len(CANDS)), sorted(os.listdir(dst)))
    found = True
if found:
    import ssbh_data_py as S
    s = S.skel_data.read_skel(os.path.join(out, 'ref', 'model.nusktb'))
    print('OS', len(s.bones))
    for b in s.bones:
        print('  %-24s parent %s' % (b.name, s.bones[b.parent_index].name if b.parent_index is not None else None))
