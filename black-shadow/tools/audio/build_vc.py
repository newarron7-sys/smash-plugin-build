"""Construit vc_ganon.nus3audio avec les voix de Black Shadow.

Les 38 pistes gardent les noms et l'ordre de la banque de Ganondorf (identiques dans tous les mods de voix de
Ganondorf), la banque vc_ganon.nus3bank du jeu reste donc valable.

Correspondance retrouvée dans ses scripts du mod : identifiant de son - 0x795D = n° d'onde du fichier 261 (voix).
usage : python3 build_vc.py <dossier idsp> <sortie .nus3audio>
"""
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
import nus3  # noqa: E402

IDSP, OUT = sys.argv[1:3]
# piste d'Ultimate -> onde de ses voix (fichier 261), d'après les sous-actions qui la jouent dans son mod
MAP = [
    ('vc_ganon_attack01', 21),        # Attack11
    ('vc_ganon_attack02', 22),        # AttackAirLw, AttackLw4 2e coup
    ('vc_ganon_attack03', 24),        # voix d'attaque aléatoire du mod
    ('vc_ganon_attack04', 25),        # voix d'attaque aléatoire du mod
    ('vc_ganon_attack05', 26),        # AttackHi3, ThrowB, SpecialNDown
    ('vc_ganon_attack06', 23),        # AttackLw4
    ('vc_ganon_attack07', 27),        # SpecialAirN
    ('vc_ganon_attack08', 21),
    ('vc_ganon_attackhard_H01', 23),
    ('vc_ganon_jump01', 37),          # Jump
    ('vc_ganon_ottotto', 41),         # OttottoWait
    ('vc_ganon_damage01', 39),
    ('vc_ganon_damage02', 40),
    ('vc_ganon_damagefly01', 30),     # DamageFly
    ('vc_ganon_damagefly02', 31),
    ('vc_ganon_missfoot01', 32),
    ('vc_ganon_missfoot02', 33),
    ('vc_ganon_damage_twinkle', 29),  # cri long (KO)
    ('vc_ganon_swimup', 37),
    ('vc_ganon_furafura', 34),        # FuraFura
    ('vc_ganon_furasleep', 35),       # FuraSleepLoop
    ('vc_ganon_wakeup', 36),
    ('vc_ganon_heavyget', 36),        # HeavyGet
    ('vc_ganon_passive', 42),         # Passive
    ('vc_ganon_cliffcatch', 28),      # CliffCatch
    ('vc_ganon_special_N01', 27),     # SpecialAirN
    ('vc_ganon_special_S01', 25),
    ('vc_ganon_special_H01', 24),
    ('vc_ganon_special_H02', 26),
    ('vc_ganon_special_L01', 15),     # SpecialLwAirStart
    ('vc_ganon_special_L02', 16),     # SpecialLwAirLoop
    ('vc_ganon_appeal_H01', 18),      # AppealHi
    ('vc_ganon_appeal_S01', 44),
    ('vc_ganon_appeal_L01', 20),      # AppealLw
    ('vc_ganon_win01', 44),           # Win1
    ('vc_ganon_win02', 45),
    ('vc_ganon_win03', 46),           # Win3
    ('vc_ganon_knockout', 29),
]
tracks = []
for i, (name, w) in enumerate(MAP):
    p = os.path.join(IDSP, 'group_44_file_261_wave_%d.idsp' % w)
    data = open(p, 'rb').read()
    assert data[:4] == b'IDSP', p
    tracks.append((i, name, data))
    print('PISTE %2d %-26s <- onde %2d (%d octets)' % (i, name, w, len(data)))
nus3.write(OUT, tracks)
back = nus3.read(OUT)
assert [t[1] for t in back] == [m[0] for m in MAP]
print('vc_ganon ok', len(back), 'pistes')
