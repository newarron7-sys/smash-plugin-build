"""Inventaire du Final Smash de Black Shadow dans le mod P+ : textes du créateur, contenu de FitBlackShadowFinal.pac,
sous-actions « Final » du PSA (coups, effets, sons), pour décider comment l'adapter à Ultimate.

usage : python3 fs_scan.py <sortie>
"""
import io
import json
import os
import struct
import sys
import urllib.request
import zipfile

sys.path.insert(0, os.path.dirname(__file__))
from arc import arc_entries  # noqa: E402
from brres import brres_files  # noqa: E402
from psa import load, subactions, events, header  # noqa: E402

OUT = sys.argv[1]
os.makedirs(OUT, exist_ok=True)
UA = {'User-Agent': 'Mozilla/5.0'}
get = lambda u: urllib.request.urlopen(urllib.request.Request(u, headers=UA), timeout=300).read()  # noqa: E731
p = json.loads(get('https://gamebanana.com/apiv11/Mod/591515/ProfilePage'))
z = zipfile.ZipFile(io.BytesIO(get(p['_aFiles'][0]['_sDownloadUrl'])))
names = [n for n in z.namelist() if '__MACOSX' not in n and not n.endswith('/')]

print('===== TEXTES')
for n in names:
    if n.lower().endswith('.txt'):
        print('---', n)
        print(z.read(n).decode('utf-8', 'replace')[:3000])

print('===== FICHIERS FINAL')
for n in names:
    if 'final' in n.lower():
        print(n, z.getinfo(n).file_size)


def walk_arc(d, where, depth=0):
    if d[:4] == b'ARC\0':
        for i, (typ, idx, size, red, data) in enumerate(arc_entries(d)):
            print('  ' * depth + 'ARC %s#%d type %d taille %d %s' % (where, i, typ, size, data[:4]))
            walk_arc(data, '%s#%d' % (where, i), depth + 1)
    elif d[:4] == b'bres':
        for folder, items in brres_files(d).items():
            print('  ' * depth + '  %s : %s' % (folder, ', '.join(n for n, _ in items)[:1500]))


for n in names:
    if n.endswith('FitBlackShadowFinal.pac'):
        print('===== CONTENU', n)
        walk_arc(z.read(n), 'Final')

# PSA : sous-actions dont le nom contient Final
mv = next(n for n in names if n.endswith('/FitBlackShadow.pac') and 'P+EX' in n) if any(
    n.endswith('/FitBlackShadow.pac') and 'P+EX' in n for n in names) else next(n for n in names if n.endswith('/FitBlackShadow.pac'))
open('/tmp/fit.pac', 'wb').write(z.read(mv))
m = load('/tmp/fit.pac')
base, subs = subactions(m)
_, h, _ = header(m)


def pv(t, v):
    return v / 60000.0 if t == 1 else v


def dump(label, off):
    fr = 0.0
    for code, params in events(m, base, off):
        if code == 0x00010100:
            fr += pv(*params[0])
            continue
        if code == 0x00020100:
            fr = pv(*params[0])
            continue
        ps = ','.join(str(round(pv(t, x), 2)) if t == 1 else ('%X' % (x & 0xffffffff) if abs(x) > 4096 else str(x)) for t, x in params)
        print('%-26s f=%6.1f %08X [%s]' % (label, fr, code, ps))


print('===== SOUS-ACTIONS FINAL')
for i, n, mo in subs:
    if 'final' in n.lower():
        print('---', i, n)
        for kind, key in (('main', 'subMain'), ('gfx', 'subGFX'), ('sfx', 'subSFX'), ('other', 'subOther')):
            o = struct.unpack_from('>I', m, base + h[key] + 4 * i)[0]
            if o:
                dump('%s %s' % (n, kind), o)
