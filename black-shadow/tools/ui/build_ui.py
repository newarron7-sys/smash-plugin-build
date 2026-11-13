"""Assemble le dossier ui/ de Black Shadow : images .bntx (converties avec ultimate_tex_cli) et nom (msg_name.xmsbt).

usage : python3 build_ui.py <dossier png> <dims.json> <sortie BlackShadow>
"""
import json
import os
import subprocess
import sys

PNG, DIMS, OUT = sys.argv[1:4]
# format BNTX (BRTI) -> nom de format d'ultimate_tex_cli
FMT = {'00002001': 'BC7RgbaUnorm', '00002006': 'BC7RgbaUnormSrgb', '00001A01': 'BC1RgbaUnorm',
       '00001A06': 'BC1RgbaUnormSrgb', '00001C01': 'BC3RgbaUnorm', '00001C06': 'BC3RgbaUnormSrgb',
       '00000B01': 'R8G8B8A8Unorm', '00000B06': 'R8G8B8A8UnormSrgb'}
try:
    ref = json.load(open(DIMS))
except (OSError, ValueError):
    ref = {}

for f in sorted(os.listdir(PNG)):
    if not f.endswith('.png'):
        continue
    key = f.split('_ganon')[0]
    dst_dir = os.path.join(OUT, 'ui/replace/chara', key)
    os.makedirs(dst_dir, exist_ok=True)
    # v0.15 : sa propre case sur l'écran des personnages (ui_chara_blackshadow, costume 00 = c120)
    dst = os.path.join(dst_dir, f[:-4].replace('_ganon_', '_blackshadow_') + '.bntx')
    # chara_7 absent du mod de référence : réglages lus dans chara_7_allmight_00.bntx (BC7 sRGB, sans mipmaps)
    info = ref.get(key) or ({'format': '00002006', 'mips': 1} if key == 'chara_7' else {})
    args = ['ultimate_tex_cli', os.path.join(PNG, f), dst]
    if info.get('format') in FMT:
        args += ['--format', FMT[info['format']]]
    if info.get('mips') == 1:
        args += ['--no-mipmaps']
    r = subprocess.run(args, capture_output=True, text=True)
    if r.returncode != 0:
        print('reessai sans format', f, r.stderr.strip()[:300])
        r = subprocess.run(['ultimate_tex_cli', os.path.join(PNG, f), dst] + (['--no-mipmaps'] if info.get('mips') == 1 else []),
                           capture_output=True, text=True)
    print(f, '->', dst, 'ok' if r.returncode == 0 else ('ECHEC ' + r.stderr[:300]), args[3:])

# nom affiché (tous les costumes de Ganondorf)
NAMES = {
    'nam_chr0_00_blackshadow': 'Black Shadow',
    'nam_chr1_00_blackshadow': 'Black Shadow',
    'nam_chr2_00_blackshadow': 'BLACK SHADOW',
    'nam_chr3_00_blackshadow': 'BLACK SHADOW',
    'nam_chr_no_00_blackshadow': '23',
    'nam_stage_name_00_blackshadow': 'Leader of the\r\nDark Million',
    'nam_final_arts_blackshadow': 'Black Bull',
    'nam_final_arts_help_blackshadow': 'Black Shadow jumps into his Black Bull\r\nand charges forward with full force,\r\nhitting every opponent in his path.',
}
xml = ['<?xml version="1.0" encoding="utf-16"?>', '<xmsbt>']
for k, v in NAMES.items():
    xml.append('  <entry label="%s">' % k)
    xml.append('    <text>%s</text>' % v.replace('\r\n', '&#xD;&#xA;'))
    xml.append('  </entry>')
xml.append('</xmsbt>')
os.makedirs(os.path.join(OUT, 'ui/message'), exist_ok=True)
with open(os.path.join(OUT, 'ui/message/msg_name.xmsbt'), 'w', encoding='utf-16') as fh:
    fh.write('\r\n'.join(xml) + '\r\n')
print('msg_name.xmsbt ok')
