"""Télécharge le pack d'effets « Zero MMZ Effects » de Tofu Yi (GameBanana 434366) et garde la couleur c00 (usage perso).

usage : python3 fetch_effects.py <dossier de sortie>
"""
import json, os, shutil, subprocess, sys, urllib.request
UA = {'User-Agent': 'Mozilla/5.0'}
OUT = sys.argv[1]
get = lambda u: urllib.request.urlopen(urllib.request.Request(u, headers=UA), timeout=900).read()  # noqa: E731
p = json.loads(get('https://gamebanana.com/apiv11/Mod/434366/ProfilePage'))
f = next(f for f in p['_aFiles'] if 'effect' in f['_sFile'].lower())
print('fichier', f['_sFile'], f['_nFilesize'])
arc = '/tmp/' + f['_sFile']
open(arc, 'wb').write(get(f['_sDownloadUrl']))
subprocess.run(['7z', 'x', '-y', '-o/tmp/effx', arc], check=True, capture_output=True)
root = next(os.path.join(dp) for dp, dn, fn in os.walk('/tmp/effx') if 'config.json' in fn)
print('racine', root)
print(open(os.path.join(root, 'config.json')).read())
for dp, dn, fn in os.walk(root):
    for n in fn:
        src = os.path.join(dp, n)
        rel = os.path.relpath(src, root)
        if 'c00' in rel or rel == 'config.json' or not any('c0%d' % i in rel for i in range(1, 8)):
            dst = os.path.join(OUT, rel)
            os.makedirs(os.path.dirname(dst), exist_ok=True)
            shutil.copy(src, dst)
            print('garde', rel, os.path.getsize(src))
