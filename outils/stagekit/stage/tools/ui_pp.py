"""Images de l'écran des terrains pour Planet Papetoon (Yoshi's Story) : rendu three.js de la scène (rendu/, serveur
http sur 8765 déjà lancé dans le dossier du rendu), aux dimensions des .bntx d'un mod de référence, puis greffe des
pixels dans ces .bntx (nom interne du jeu gardé).
usage : python3 ui_pp.py <dossier du rendu> <dossier des bntx de réf (stage_N_yoshi_cartboard.bntx)> <dossier ui du mod> <dossier aperçus>"""
import glob, os, re, struct, subprocess, sys
from PIL import Image

R, REFD, UI, AP = sys.argv[1:5]
OUTILS = os.path.dirname(os.path.abspath(__file__))
os.makedirs(AP, exist_ok=True)
# caméra (x, y, z) -> cible, champ vertical ; stage_1 = petite vignette, plus serrée
CAM = {1: ((0, 26, 118), (0, 12, 0), 40), 'def': ((0, 30, 150), (0, 15, 0), 38)}

for ref in sorted(glob.glob(os.path.join(REFD, 'stage_*_yoshi_cartboard.bntx'))):
    n = int(re.search(r'stage_(\d+)_', os.path.basename(ref)).group(1))
    if n == 0:
        continue
    d = open(ref, 'rb').read()
    i = d.find(b'BRTI')
    mips = struct.unpack_from('<H', d, i + 0x16)[0]
    fmt = struct.unpack_from('<I', d, i + 0x1C)[0]
    w, h = struct.unpack_from('<II', d, i + 0x24)
    print('stage_%d : %dx%d mips %d format %08X' % (n, w, h, mips, fmt))
    subprocess.run(['ultimate_tex_cli', ref, os.path.join(AP, 'ref_stage_%d.png' % n)], capture_output=True)
    (cx, cy, cz), (tx, ty, tz), fov = CAM.get(n, CAM['def'])
    big = os.path.join(AP, '_big.png')
    k = 2 if max(w, h) <= 1024 else 1
    r = subprocess.run(['node', os.path.join(R, 'rendu.js'), big, str(w * k), str(h * k), str(cx), str(cy), str(cz),
                        str(tx), str(ty), str(tz), str(fov)], capture_output=True, text=True, cwd=R)
    print(r.stdout[-400:], r.stderr[-400:])
    png = os.path.join(AP, 'stage_%d.png' % n)
    Image.open(big).convert('RGBA').resize((w, h), Image.LANCZOS).save(png)
    os.remove(big)
    neuf = os.path.join(AP, '_neuf.bntx')
    fmtname = {0x2006: 'BC7RgbaUnormSrgb', 0x2001: 'BC7RgbaUnorm', 0x1A06: 'BC1RgbaUnormSrgb', 0x0B06: 'Rgba8UnormSrgb',
               0x0B01: 'Rgba8Unorm'}.get(fmt, 'BC7RgbaUnormSrgb')
    args = ['ultimate_tex_cli', png, neuf, '--format', fmtname] + (['--no-mipmaps'] if mips == 1 else [])
    subprocess.run(args, check=True, capture_output=True)
    dst = os.path.join(UI, 'replace', 'stage', 'stage_%d' % n, 'stage_%d_yoshi_cartboard.bntx' % n)
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    subprocess.run([sys.executable, os.path.join(OUTILS, 'greffe_bntx.py'), ref, neuf, dst], check=True)
    os.remove(neuf)
