"""Interface de Zero (REX) : images du CSS / combat / résultats tirées du paquet (rendus HD fournis), aux tailles
d'Ultimate, converties en .bntx, et ses noms (msg_name.xmsbt). Case ui_chara_zerorex, costume 00 = c130.

usage : python3 build_ui.py <dossier ui_src> <sortie du mod>
"""
import os, subprocess, sys
from PIL import Image

SRC, OUT = sys.argv[1:3]
L = lambda n: Image.open(os.path.join(SRC, n + '.png')).convert('RGBA')  # noqa: E731
res = L('CSP_Result_HD_0000')       # 384x480, corps entier
face = L('CSSIcon_P__HD_0000')      # 240x240, visage
stock = L('StockIcon_P__HD_0000')   # 128x128
grid = L('CSSIcon_vBrawl_HD_0000')  # 240x168


def fit(im, size):
    return im.resize(size, Image.LANCZOS)


def crop_ratio(im, box, size):
    return im.crop(box).resize(size, Image.LANCZOS)


big = Image.new('RGBA', (3028, 2036), (0, 0, 0, 0))
r = res.resize((int(384 * 2036 / 480), 2036), Image.LANCZOS)
big.alpha_composite(r, ((3028 - r.width) // 2, 0))
imgs = {
    'chara_0': fit(face, (128, 128)),
    'chara_1': crop_ratio(res, (0, 0, 384, 384), (512, 512)),
    'chara_2': fit(stock, (64, 64)),
    'chara_3': big,
    'chara_4': fit(face, (224, 224)),
    'chara_6': crop_ratio(face, (0, 80, 240, 200), (512, 256)),
    'chara_7': crop_ratio(grid, (0, 4, 240, 162), (456, 300)),
}
os.makedirs(os.path.join(OUT, '..', 'ui_png'), exist_ok=True)
for k, im in imgs.items():
    png = os.path.join(OUT, '..', 'ui_png', '%s_zerorex_00.png' % k)
    im.save(png)
    d = os.path.join(OUT, 'ui/replace/chara', k)
    os.makedirs(d, exist_ok=True)
    dst = os.path.join(d, '%s_zerorex_00.bntx' % k)
    rr = subprocess.run(['ultimate_tex_cli', png, dst, '--format', 'BC7RgbaUnormSrgb', '--no-mipmaps'], capture_output=True, text=True)
    print(k, im.size, 'ok' if rr.returncode == 0 else 'ECHEC ' + rr.stderr[:200])
NAMES = {
    'nam_chr0_00_zerorex': 'Zero', 'nam_chr1_00_zerorex': 'Zero', 'nam_chr2_00_zerorex': 'ZERO',
    'nam_chr3_00_zerorex': 'ZERO', 'nam_stage_name_00_zerorex': 'The Crimson\r\nMaverick Hunter',
}
os.makedirs(os.path.join(OUT, 'ui/message'), exist_ok=True)
xml = ['<?xml version="1.0" encoding="utf-16"?>', '<xmsbt>']
for k, v in NAMES.items():
    xml.append('    <entry label="%s">' % k)
    xml.append('        <text>%s</text>' % v.replace('\r\n', '&#xD;&#xA;'))
    xml.append('    </entry>')
xml.append('</xmsbt>')
with open(os.path.join(OUT, 'ui/message/msg_name.xmsbt'), 'w', encoding='utf-16') as fh:
    fh.write('\r\n'.join(xml) + '\r\n')
print('interface ok')
