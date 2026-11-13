"""Voix de Master Chief (banque REX 155, fichier 261 = voix) dans la banque de voix de Snake du costume c140.

Son de REX n° id (scripts : 0A000100 / 0A020100 id) -> onde (id - 0x4AF5) du fichier 261 (0x4AF5 = 0x4B24 - 47 : les 47
voix sont juste avant les 118 sons du fichier 629). Dans REX, Chief parle peu : 13 voix non vides seulement (provocation
bas : 7 répliques au hasard, provocation côté, rebord, chutes, éjections, sommeil, objet lourd, au bord, victoire 3).
Les pistes de Snake sans voix de Chief dans REX sont rendues muettes (piste silencieuse du modèle).
usage : python3 sons_chief_voix.py <dossier 155 (extraction.txt, *.idsp)> <modèle vc_snake.nus3audio> <sortie>
"""
import os, re, struct, sys
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, '../../../smash-plugin-build/outils'))
import nus3  # noqa: E402

SRC, MODELE, OUT = sys.argv[1:4]
VOIX = {
    # provocation bas (REX AppealLw, image 40 : une des 7 au hasard) ; joué par le plugin
    'vc_snake_001': 5, 'vc_snake_002': 6, 'vc_snake_003': 7, 'vc_snake_004': 8, 'vc_snake_005': 9,
    'vc_snake_006': 10, 'vc_snake_007': 11,
    # provocation côté (REX AppealS, image 30) ; aussi l'entrée de Snake (vc_snake_008)
    'vc_snake_008': 19,
    'vc_snake_cliffcatch': 28,                                   # CliffCatch
    'vc_snake_damagefly01': 30, 'vc_snake_damagefly02': 31,      # DamageFly* (4 au hasard)
    'vc_snake_damage01': 32, 'vc_snake_damage02': 33,
    'vc_snake_missfoot01': 32, 'vc_snake_missfoot02': 33,
    'vc_snake_damage_twinkle': 29, 'vc_snake_knockout': 29,      # cri long (REX : noyade / chute)
    'vc_snake_furasleep': 35, 'vc_snake_heavyget': 36, 'vc_snake_ottotto': 41,
    'vc_snake_win01': 46, 'vc_snake_win02': 46, 'vc_snake_win03': 46,  # Win3 (seule victoire avec voix)
}
D = {}
for l in open(os.path.join(SRC, 'extraction.txt')):
    if 'file=261' in l:
        D[int(re.search(r'data=(\d+)', l).group(1))] = int(re.search(r'wave=(\d+)', l).group(1))
pistes = nus3.read_full(MODELE)
noms = [n for _, n, _ in pistes]
manque = [k for k in VOIX if k not in noms]
assert not manque, manque
muet = min((d for _, n, d in pistes if d[:4] == b'IDSP'), key=len)
print('piste muette', len(muet))
out = []
for i, n, d in pistes:
    if n in VOIX:
        f = os.path.join(SRC, 'group_44_file_261_wave_%d.idsp' % D[VOIX[n]])
        d = bytearray(open(f, 'rb').read())
        struct.pack_into('>II', d, 0x14, 0, 0)                  # pas de boucle
        d = bytes(d)
        print('VOIX', n, '<-', VOIX[n], len(d))
    elif d:
        d = muet
    out.append((i, n, d))
nus3.write(OUT, out)
print('pistes', len(out), 'voix', len(VOIX))
