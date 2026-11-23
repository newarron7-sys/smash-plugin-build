"""GameBanana (usage perso) : télécharge et extrait les fichiers d'un mod, ou cherche des mods Ultimate qui touchent
un chemin donné (ex. stage/dolly_stadium) et en liste l'arborescence.

usage :
  python3 gb_fetch.py mod <id> <sortie>                          -> fichiers du mod extraits dans <sortie>/<id>/
  python3 gb_fetch.py cherche "<recherche>" <chemin> <sortie> [n] -> les n premiers mods Ultimate (jeu 6498) dont une
                                                                    archive contient <chemin>, extraits dans <sortie>/<id>/
"""
import json, os, subprocess, sys, urllib.parse, urllib.request

UA = {'User-Agent': 'Mozilla/5.0'}


def get(u):
    return urllib.request.urlopen(urllib.request.Request(u, headers=UA), timeout=600).read()


def files_of(mid):
    p = json.loads(get('https://gamebanana.com/apiv11/Mod/%d/ProfilePage' % mid))
    return p.get('_sName', ''), p.get('_aGame', {}).get('_sName', ''), p.get('_aFiles') or []


def download(mid, f, out):
    os.makedirs(out, exist_ok=True)
    path = os.path.join(out, f['_sFile'])
    open(path, 'wb').write(get(f['_sDownloadUrl']))
    return path


def listing(path):
    ls = subprocess.run(['7z', 'l', '-ba', '-slt', path], capture_output=True, text=True).stdout
    return [l[7:].replace('\\', '/') for l in ls.splitlines() if l.startswith('Path = ')]


def extract(path, dst):
    os.makedirs(dst, exist_ok=True)
    subprocess.run(['7z', 'x', '-y', '-o' + dst, path], capture_output=True)


def main():
    mode = sys.argv[1]
    if mode == 'mod':
        mid, out = int(sys.argv[2]), sys.argv[3]
        name, game, fl = files_of(mid)
        print('MOD', mid, name, '| jeu :', game)
        for f in fl:
            print('FICHIER', f['_sFile'], f.get('_nFilesize'))
            p = download(mid, f, os.path.join(out, 'archives'))
            names = listing(p)
            print('  %d entrees' % len(names))
            for n in names[:400]:
                print('   ', n)
            extract(p, os.path.join(out, str(mid)))
        return
    query, want, out = sys.argv[2], sys.argv[3].lower(), sys.argv[4]
    nmax = int(sys.argv[5]) if len(sys.argv) > 5 else 1
    seen = []
    for q in (query, query + ' stage'):
        u = ('https://gamebanana.com/apiv11/Util/Search/Results?_sModelName=Mod&_sOrder=best_match&_idGameRow=6498'
             '&_sSearchString=%s&_nPage=1' % urllib.parse.quote(q))
        try:
            for rec in json.loads(get(u)).get('_aRecords', []):
                if rec['_idRow'] not in seen:
                    seen.append(rec['_idRow'])
                    print('MOD', rec['_idRow'], rec.get('_sName'))
        except Exception as e:  # noqa: BLE001
            print('recherche echec', e)
    found = 0
    for mid in seen[:30]:
        try:
            name, _, fl = files_of(mid)
        except Exception:  # noqa: BLE001
            continue
        for f in fl:
            if f.get('_nFilesize', 0) > 150e6:
                continue
            p = download(mid, f, '/tmp/gbdl')
            names = listing(p)
            hit = [n for n in names if want in n.lower()]
            print('FICHIER', mid, f['_sFile'], len(names), ('-> %d chemins %s' % (len(hit), want)) if hit else '')
            if hit:
                print('REFERENCE', mid, name)
                for n in names:
                    print('   ', n)
                extract(p, os.path.join(out, str(mid)))
                found += 1
                break
            os.remove(p)
        if found >= nmax:
            break


if __name__ == '__main__':
    main()
