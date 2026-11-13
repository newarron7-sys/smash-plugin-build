"""Télécharge le Zero de _Silent_ (WIP GameBanana 52403, lien Google Drive de la page) et l'extrait (usage perso).

usage : python3 fetch_silent.py <dossier de sortie>
"""
import os, re, subprocess, sys, urllib.request, http.cookiejar
FID = '1EqdemmMi0Xr7zHU-alWUYfKqP5kwpFKZ'
OUT = sys.argv[1]
os.makedirs(OUT, exist_ok=True)
cj = http.cookiejar.CookieJar()
op = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(cj))
op.addheaders = [('User-Agent', 'Mozilla/5.0')]
url = 'https://drive.usercontent.google.com/download?id=%s&export=download&confirm=t' % FID
r = op.open(url, timeout=900)
cd = r.headers.get('Content-Disposition', '')
data = r.read()
if data[:15].lower().startswith(b'<!doctype html') or data[:5] == b'<html':
    h = data.decode('utf8', 'ignore')
    form = dict(re.findall(r'name="([^"]+)" value="([^"]*)"', h))
    act = re.search(r'action="([^"]+)"', h)
    print('page de confirmation', act and act.group(1), form)
    u = (act.group(1) if act else 'https://drive.usercontent.google.com/download') + '?' + '&'.join('%s=%s' % kv for kv in form.items())
    r = op.open(u, timeout=900)
    cd = r.headers.get('Content-Disposition', '')
    data = r.read()
name = (re.findall(r'filename="?([^";]+)', cd) or ['silent_zero.bin'])[0]
print('fichier', name, len(data))
arc = os.path.join('/tmp', name)
open(arc, 'wb').write(data)
print(subprocess.run(['7z', 'x', '-y', '-o' + OUT, arc], capture_output=True, text=True).stdout[-600:])
