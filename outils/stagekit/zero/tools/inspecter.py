"""Inspecte un mod GameBanana (fiche, crédits, licence, contenu de l'archive) sans rien publier du mod.

usage : python3 inspecter.py <id du mod> <dossier de travail>
"""
import json, os, subprocess, sys, urllib.request, re, html
UA = {'User-Agent': 'Mozilla/5.0'}


def get(u):
    return urllib.request.urlopen(urllib.request.Request(u, headers=UA), timeout=600).read()


mid, work = sys.argv[1], sys.argv[2]
os.makedirs(work, exist_ok=True)
p = json.loads(get('https://gamebanana.com/apiv11/Mod/%s/ProfilePage' % mid))
print('NOM', p.get('_sName'))
print('AUTEUR', (p.get('_aSubmitter') or {}).get('_sName'))
print('CATEGORIE', (p.get('_aCategory') or {}).get('_sName'), '| JEU', (p.get('_aGame') or {}).get('_sName'))
txt = re.sub(r'<br\s*/?>', '\n', p.get('_sText') or '')
txt = html.unescape(re.sub(r'<[^>]+>', ' ', txt))
print('DESCRIPTION', p.get('_sDescription'))
print('TEXTE_DEBUT'); print(txt[:6000]); print('TEXTE_FIN')
print('CREDITS', json.dumps(p.get('_aCredits'), ensure_ascii=False)[:3000])
print('LICENCE', p.get('_sLicense'), json.dumps(p.get('_aLicenseChecklist'), ensure_ascii=False)[:1500])
for f in p.get('_aFiles') or []:
    print('FICHIER', f.get('_sFile'), f.get('_nFilesize'), f.get('_sDescription'), f.get('_sDownloadUrl'))
for i, f in enumerate(p.get('_aFiles') or []):
    path = os.path.join(work, f['_sFile'])
    open(path, 'wb').write(get(f['_sDownloadUrl']))
    print('== contenu de', f['_sFile'])
    r = subprocess.run(['7z', 'l', '-ba', '-slt', path], capture_output=True, text=True)
    cur = {}
    for line in r.stdout.splitlines():
        if line.startswith('Path = '):
            cur = {'p': line[7:]}
        elif line.startswith('Size = ') and cur:
            print('  %10s  %s' % (line[7:], cur['p']))

# paquet BrawlInstaller (.fighterpackage = zip) : liste des fichiers et infos du perso
import zipfile, glob
for z in glob.glob(os.path.join(work, '*.zip')):
    zz = zipfile.ZipFile(z)
    for n in zz.namelist():
        if n.endswith('.fighterpackage'):
            fp = os.path.join(work, 'pkg.fighterpackage')
            open(fp, 'wb').write(zz.read(n))
            pk = zipfile.ZipFile(fp)
            print('== contenu du paquet', n)
            for i in pk.infolist():
                print('  %10d  %s' % (i.file_size, i.filename))
            for i in pk.infolist():
                if i.filename.lower().endswith(('.txt', '.ini', '.json', '.xml', '.cfg')) and i.file_size < 20000:
                    print('== texte', i.filename)
                    print(pk.read(i.filename).decode('utf-8', 'replace')[:4000])
    for n in zz.namelist():
        if n.lower().endswith('readme.txt'):
            print('== ReadMe'); print(zz.read(n).decode('utf-8', 'replace'))
