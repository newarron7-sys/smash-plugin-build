"""Télécharge le Zero de P+Ex/REX (GameBanana 700454) et extrait son paquet BrawlInstaller (usage perso)."""
import io, json, os, sys, urllib.request, zipfile
UA = {'User-Agent': 'Mozilla/5.0'}
get = lambda u: urllib.request.urlopen(urllib.request.Request(u, headers=UA), timeout=600).read()  # noqa: E731
out = sys.argv[1]
p = json.loads(get('https://gamebanana.com/apiv11/Mod/700454/ProfilePage'))
z = zipfile.ZipFile(io.BytesIO(get(p['_aFiles'][0]['_sDownloadUrl'])))
pk = zipfile.ZipFile(io.BytesIO(z.read([n for n in z.namelist() if n.endswith('.fighterpackage')][0])))
pk.extractall(out)
print('paquet extrait', len(pk.namelist()), 'fichiers')
