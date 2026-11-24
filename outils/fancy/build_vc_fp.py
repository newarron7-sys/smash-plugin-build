"""Fancy Pants Man : vc_mario_c200.nus3audio avec ses sons (banque 21B du mod : fichier 261 = voix, 629 = bruitages).
Noms et ordre des pistes : modèle (mod de voix de Mario de GameBanana) ; même méthode que Primid (vc_mario_c140).
Identifiants de son du mod -> onde : 261 = id - 0xCA94 ; 629 = id - 0xCAC3.
Pistes jouées par le jeu (dégâts, KO, ...) : ses cris ; autres pistes (attaques, 001-015 : jouées par les scripts de
Mario, retirés) : ses bruitages, joués par le plugin aux images de son PSA.
usage : python3 build_vc_fp.py <dossier idsp> <modèle> <sortie>"""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import nus3  # noqa: E402

IDSP, MODELE, OUT = sys.argv[1:4]
V, B = 261, 629
MAP = {
    'vc_mario_damage02': (V, 30), 'vc_mario_damagefly01': (V, 31), 'vc_mario_damagefly02': (V, 32),
    'vc_mario_knockout': (V, 29), 'vc_mario_damage_twinkle': (V, 29), 'vc_mario_furafura': (V, 33),
    'vc_mario_furasleep': (V, 34), 'vc_mario_ottotto': (V, 40), 'vc_mario_heavyget': (V, 39),
    'vc_mario_missfoot01': (V, 30), 'vc_mario_missfoot02': (V, 31), 'vc_mario_passive': (V, 39),
    'vc_mario_swimup': (V, 39), 'vc_mario_cliffcatch': (V, 40),
    'vc_mario_win01': (B, 9), 'vc_mario_win02': (B, 8), 'vc_mario_win03': (B, 12), 'vc_mario_appeal02': (B, 8),
    # bruitages joués par le plugin
    'vc_mario_attack01': (B, 0), 'vc_mario_attack02': (B, 1), 'vc_mario_attack03': (B, 2), 'vc_mario_attack04': (B, 3),
    'vc_mario_attack05': (B, 10), 'vc_mario_attack06': (B, 4), 'vc_mario_attack07': (B, 9),
    'vc_mario_001': (B, 48), 'vc_mario_002': (B, 62), 'vc_mario_005': (B, 63), 'vc_mario_006': (B, 81),
    'vc_mario_007': (B, 82), 'vc_mario_009': (B, 83), 'vc_mario_010': (B, 87), 'vc_mario_011': (B, 88),
    'vc_mario_012': (B, 89), 'vc_mario_013': (B, 84), 'vc_mario_014': (B, 8), 'vc_mario_015': (B, 7),
}
src = nus3.read(MODELE)
noms = [t[1] for t in src]
print('pistes du modèle :', len(noms))
manq = set(noms) - set(MAP)
if manq:
    print('pistes du modèle sans son de Fancy (onde de cri par défaut) :', sorted(manq))
tracks = []
for i, n in enumerate(noms):
    f, w = MAP.get(n, (V, 30))
    data = open(os.path.join(IDSP, 'group_44_file_%d_wave_%d.idsp' % (f, w)), 'rb').read()
    assert data[:4] == b'IDSP', n
    tracks.append((i, n, data))
nus3.write(OUT, tracks)
assert [t[1] for t in nus3.read(OUT)] == noms
print('vc_mario_c200 ok', len(noms), 'pistes ; absentes du modèle :', sorted(set(MAP) - set(noms)))
