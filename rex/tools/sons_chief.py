"""Sons de Master Chief (banque REX 155, fichier 629) dans la banque de sons de Snake du costume c140.

Son de REX n° id (scripts : 0A000100 id) -> entrée DATA (id - 0x4B24) du fichier 629 -> onde (extraction.txt)
(base vérifiée sur Super Shadow : banque 1BB, base 0x8CE2 = 0x4000 + (banque - 0x14B) * 0xA5 + 0x4B2).
usage : python3 sons_chief.py <dossier 155 (extraction.txt, *.idsp)> <modèle se_snake.nus3audio> <sortie .nus3audio>
"""
import os, re, struct, sys
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, '../../../smash-plugin-build/outils'))
import nus3  # noqa: E402

SRC, MODELE, OUT = sys.argv[1:4]
# étiquette de Snake <- n° du son de Chief dans sa banque (id - 0x4B24)
SONS = {
    # déplacements (joués par les scripts de Snake)
    'se_snake_step_left_l': 103, 'se_snake_step_left_m': 103, 'se_snake_step_left_s': 103,
    'se_snake_step_right_l': 105, 'se_snake_step_right_m': 105, 'se_snake_step_right_s': 105,
    'se_snake_dash_start': 54, 'se_snake_dash_stop': 55, 'se_snake_turn_run': 55,
    'se_snake_jump01': 63, 'se_snake_jump02': 64, 'se_snake_jump03': 64,
    'se_snake_squat': 111, 'se_snake_escape': 113, 'se_snake_escapeb': 113, 'se_snake_escapeair': 113,
    # coups (joués par le plugin, images de REX)
    'se_snake_swing_m': 110, 'se_snake_swing_s': 109, 'se_snake_swing_l': 107,          # jabs
    'se_snake_attackdash_01': 10, 'se_snake_attackdash_02': 33,                        # Spartan Charge / Slide Kick
    'se_snake_attackhard_S01': 16, 'se_snake_attackhard_S02': 84,                      # fusil de combat : rafale, visée
    'se_snake_attackhard_H01': 70,                                                     # mitraillettes
    'se_snake_attackhard_L01': 18, 'se_snake_attackair_N03': 76,                       # pistolet plasma, tir chargé
    'se_snake_smash_S01': 46, 'se_snake_smash_S02': 4,                                 # fusil à pompe : armement, tir
    'se_snake_smash_H01': 5, 'se_snake_smash_H02': 6,                                  # laser : charge, tir
    'se_snake_smash_L01': 15, 'se_snake_smash_L02': 3,                                 # sniper : visée, tir
    'se_snake_attackair_N01': 92, 'se_snake_attackair_N02': 2,                         # coup de pied ; épée
    'se_snake_attackair_F01': 77, 'se_snake_attackair_B01': 8, 'se_snake_attackair_H01': 86,
    'se_snake_attackair_L01': 7, 'se_snake_attackair_L02': 9,                          # Ground Pound : réacteurs, plongée
    # spéciaux (images de REX : Gravity Lift 0, Fusion Coil 9 / 20, Plasma Grenade 3 / 19)
    'se_snake_special_H01': 82, 'se_snake_special_L01': 89, 'se_snake_special_L02': 90,
    'se_snake_special_S01': 98, 'se_snake_special_S02': 99,
    # B neutre (scripts de REX : magnum 19281 / 19253, fusil d'assaut 19328 en boucle, recharge 19331, clic à vide
    # 19332, coup de crosse 19330 / 19333)
    # Final Smash (REX : 9 au souffle, 58 au tir de la roquette, 59 à l'explosion)
    'se_snake_final01': 9, 'se_snake_final02': 58, 'se_snake_final03': 59,
    'se_snake_special_N01': 45, 'se_snake_special_N02': 17, 'se_snake_special_N03': 92,
    'se_snake_special_N04': 95, 'se_snake_special_N05': 96, 'se_snake_special_N06': 94, 'se_snake_special_N07': 97,
}
# rafale du fusil d'assaut : REX joue le son en boucle et l'arrête par script ; le plugin fait pareil (boucle gardée)
GARDER_BOUCLE = {'se_snake_special_N03'}
D = {}
for l in open(os.path.join(SRC, 'extraction.txt')):
    if 'file=629' in l:
        D[int(re.search(r'data=(\d+)', l).group(1))] = int(re.search(r'wave=(\d+)', l).group(1))
pistes = nus3.read_full(MODELE)
noms = {n for _, n, _ in pistes}
manque = [k for k in SONS if k not in noms]
assert not manque, manque
out = []
for i, n, d in pistes:
    if n in SONS:
        f = os.path.join(SRC, 'group_44_file_629_wave_%d.idsp' % D[SONS[n]])
        d = bytearray(open(f, 'rb').read())
        # sons en boucle dans REX (mitraillettes n° 70, coup de pied n° 92 : REX les arrête par script) : boucle
        # retirée (en-tête IDSP : début / fin de boucle à 0x14 / 0x18), sinon ils tournent sans fin (vidéo up tilt)
        boucle = struct.unpack_from('>II', d, 0x14)
        if boucle[1] and n not in GARDER_BOUCLE:
            struct.pack_into('>II', d, 0x14, 0, 0)
            print('BOUCLE RETIREE', n, SONS[n], boucle)
        d = bytes(d)
        print('SON', n, '<-', SONS[n], os.path.basename(f), len(d))
    out.append((i, n, d))
nus3.write(OUT, out)
print('pistes', len(out), 'remplacees', len(SONS))
