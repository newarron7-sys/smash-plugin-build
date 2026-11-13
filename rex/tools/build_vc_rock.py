"""Construit vc_dolly.nus3audio avec les voix de Rock (REX, banque 213 : fichier 261 = voix, 629 = sons).

Les 54 pistes gardent les noms et l'ordre de la banque de Terry (modèle d'un mod de voix de Terry), la banque
vc_dolly.nus3bank du jeu reste donc valable. Son de REX -> onde : identifiant - 0xC56B (fichier 261) ; les cris des
super-coups sont dans le fichier 629 (identifiant - 0xC59A).
usage : python3 build_vc_rock.py <dossier idsp> <sortie .nus3audio>
"""
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '../../black-shadow/tools/audio'))
import nus3  # noqa: E402

IDSP, OUT = sys.argv[1:3]
V = lambda w: (261, w)          # noqa: E731  voix
S = lambda w: (629, w)          # noqa: E731  sons (cris des super-coups)
MAP = [
    ('vc_dolly_attack01', V(0)), ('vc_dolly_attack02', V(1)), ('vc_dolly_attack03', V(2)),      # Attack11 (au hasard)
    ('vc_dolly_attack04', V(4)), ('vc_dolly_attack05', V(5)), ('vc_dolly_attack06', V(6)),      # Attack13, smashs
    ('vc_dolly_attack07', V(7)),                                                                # AttackHi3
    ('vc_dolly_attackhard_h01', V(5)),
    ('vc_dolly_jump01', V(1)),
    ('vc_dolly_ottotto', V(38)),                                                                # OttottoWait
    ('vc_dolly_escapef', V(17)), ('vc_dolly_escapeb', V(18)),                                   # EscapeF / EscapeB
    ('vc_dolly_damage01', V(32)), ('vc_dolly_damage02', V(33)),                                 # DamageFly
    ('vc_dolly_damagefly01', V(34)), ('vc_dolly_damagefly02', V(35)),
    ('vc_dolly_missfoot01', V(32)), ('vc_dolly_missfoot02', V(33)),
    ('vc_dolly_damage_twinkle', V(29)),                                                         # cri long (KO)
    ('vc_dolly_swimup', V(39)),                                                                 # SwimUp
    ('vc_dolly_furafura', V(34)), ('vc_dolly_furasleep', V(35)), ('vc_dolly_wakeup', V(37)),
    ('vc_dolly_heavyget', V(36)), ('vc_dolly_passive', V(1)), ('vc_dolly_cliffcatch', V(2)),
    ('vc_dolly_attackdash_01', V(9)), ('vc_dolly_attackdash_02', V(8)),                         # AttackDash / AirF
    ('vc_dolly_special_n01', V(19)),                                                            # Reppu Ken !
    ('vc_dolly_special_n02', V(21)), ('vc_dolly_special_n02_short', V(20)),                     # Double Reppu Ken
    ('vc_dolly_special_sf01', V(23)), ('vc_dolly_special_sf02', V(24)), ('vc_dolly_special_sf03', V(25)),  # Hard Edge
    ('vc_dolly_special_sb01', V(31)), ('vc_dolly_special_sb02', V(31)),                         # OverHeadKick
    ('vc_dolly_special_h01', V(26)), ('vc_dolly_special_h02', V(28)), ('vc_dolly_special_h03', V(27)),  # Rising Tackle
    ('vc_dolly_special_l01', V(30)), ('vc_dolly_special_l02', V(22)),                           # B bas
    ('vc_dolly_superspecial01_01', S(2)), ('vc_dolly_superspecial01_02', S(2)),                 # Shining Knuckle
    ('vc_dolly_superspecial02_01', S(0)), ('vc_dolly_superspecial02_02', S(1)),                 # Raging Storm
    ('vc_dolly_final01', S(0)), ('vc_dolly_final02', S(1)), ('vc_dolly_final03', V(45)), ('vc_dolly_final04', V(46)),
    ('vc_dolly_appeal01_02', V(41)), ('vc_dolly_appeal02', V(40)),                              # AppealLw / AppealS
    ('vc_dolly_win01', V(42)), ('vc_dolly_win02', V(44)),                                       # Win1 / Win3
    ('vc_dolly_knockout', V(29)),
]
tracks = []
for i, (name, (f, w)) in enumerate(MAP):
    p = os.path.join(IDSP, 'group_44_file_%d_wave_%d.idsp' % (f, w))
    data = open(p, 'rb').read()
    assert data[:4] == b'IDSP', p
    tracks.append((i, name, data))
    print('PISTE %2d %-28s <- fichier %d onde %2d (%d octets)' % (i, name, f, w, len(data)))
nus3.write(OUT, tracks)
back = nus3.read(OUT)
assert [t[1] for t in back] == [m[0] for m in MAP]
print('vc_dolly ok', len(back), 'pistes')
