"""Images d'interface manquantes d'un skin (chara_0, chara_6, chara_7) faites à partir de celles qu'il fournit
(chara_1 portrait 512x512, chara_3 rendu, chara_4 visage 224x224), aux tailles d'Ultimate (comme zero/tools/build_ui.py).
usage : python3 ui_manquantes.py <dossier png (chara_N.png)> <nom (cloudmario)> <sortie du mod>
"""
import os, subprocess, sys
from PIL import Image

SRC, NOM, OUT = sys.argv[1:4]
L = lambda n: Image.open(os.path.join(SRC, n + '.png')).convert('RGBA')  # noqa: E731
c1, c4 = L('chara_1'), L('chara_4')


def boite(im):
    b = im.getchannel('A').point(lambda a: 255 if a > 40 else 0).getbbox()
    return b or (0, 0, im.width, im.height)


def cadre(im, centre, w, h, size):
    """découpe w x h autour de centre (bords transparents si besoin), mise à la taille voulue"""
    x0, y0 = int(centre[0] - w / 2), int(centre[1] - h / 2)
    fond = Image.new('RGBA', (int(w), int(h)), (0, 0, 0, 0))
    fond.alpha_composite(im.crop((max(0, x0), max(0, y0), min(im.width, x0 + int(w)), min(im.height, y0 + int(h)))),
                         (max(0, -x0), max(0, -y0)))
    return fond.resize(size, Image.LANCZOS)


# portrait (chara_1) : la tête = la partie opaque du haut de l'image (les bras tendus faussent le centre de tout le
# corps) ; les cases sont centrées sur elle
b = boite(c1)
haut = b[1]
bande = c1.crop((0, haut, c1.width, haut + int(c1.height * 0.30)))
bt = boite(bande)
cx = (bt[0] + bt[2]) / 2
tete = bt[2] - bt[0]                    # largeur de la tête (casquette comprise)
imgs = {
    'chara_0': c4.resize((128, 128), Image.LANCZOS),
    # case du CSS (456x300, 3:2) : tête entière et début du corps
    'chara_7': cadre(c1, (cx, haut + tete * 0.72), tete * 1.7, tete * 1.7 * 300 / 456, (456, 300)),
    # bandeau (512x256, 2:1) : visage
    'chara_6': cadre(c1, (cx, haut + tete * 0.55), tete * 1.45, tete * 1.45 / 2, (512, 256)),
}
os.makedirs(os.path.join(OUT, 'png'), exist_ok=True)
for k, im in imgs.items():
    png = os.path.join(OUT, 'png', '%s_%s_00.png' % (k, NOM))
    im.save(png)
    d = os.path.join(OUT, 'ui/replace/chara', k)
    os.makedirs(d, exist_ok=True)
    rr = subprocess.run(['ultimate_tex_cli', png, os.path.join(d, '%s_%s_00.bntx' % (k, NOM)),
                         '--format', 'BC7RgbaUnormSrgb', '--no-mipmaps'], capture_output=True, text=True)
    print(k, im.size, 'ok' if rr.returncode == 0 else 'ECHEC ' + rr.stderr[:200])
