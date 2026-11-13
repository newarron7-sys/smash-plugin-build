"""Trouve les dimensions et le format des images d'interface d'Ultimate (chara_0 à chara_7) dans un mod de
référence (CHAO5 Lite, branche chao5-lite-dl), pour produire les images de Black Shadow aux mêmes réglages.

usage : python3 ref_dims.py <dossier des zips> <sortie dims.json> <dossier temporaire>
"""
import glob
import json
import os
import re
import struct
import subprocess
import sys
import zipfile

zdir, out, tmp = sys.argv[1:4]
os.makedirs(tmp, exist_ok=True)
found = {}
for zp in sorted(glob.glob(os.path.join(zdir, '*.zip'))):
    try:
        z = zipfile.ZipFile(zp)
    except zipfile.BadZipFile:
        print('zip illisible', zp)
        continue
    for n in z.namelist():
        m = re.search(r'ui/replace/chara/(chara_\d+)/chara_\d+_([a-z_]+?)_0(\d)\.bntx$', n)
        if not m or m.group(1) in found:
            continue
        data = z.read(n)
        p = os.path.join(tmp, os.path.basename(n))
        open(p, 'wb').write(data)
        info = {'source': n, 'size': len(data)}
        i = data.find(b'BRTI')
        if i >= 0:
            # BRTI : drapeaux, dim, tile mode, swizzle, nb mips (u16 à +0x16), format (u32 à +0x1C), largeur, hauteur
            info['mips'] = struct.unpack_from('<H', data, i + 0x16)[0]
            info['format'] = '%08X' % struct.unpack_from('<I', data, i + 0x1C)[0]
            info['width'], info['height'] = struct.unpack_from('<II', data, i + 0x24)
        png = p + '.png'
        r = subprocess.run(['ultimate_tex_cli', p, png], capture_output=True, text=True)
        info['png'] = os.path.exists(png)
        found[m.group(1)] = info
        print(m.group(1), info)
json.dump(found, open(out, 'w'), indent=1)
