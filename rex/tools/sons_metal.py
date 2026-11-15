"""Sons et voix de Metal Sonic (banque REX 18E : fichier 629 = 118 sons, 261 = 47 voix) dans les banques de Sonic de
sa case (se_sonic / vc_sonic, costume c00 de la source, renommé par reslotter.py), comme Master Chief
(sons_chief.py / sons_chief_voix.py).

Son de REX n° id (scripts : 0A000100 id) : sons = id - 28641, voix = id - 28594
(base = 0x4000 + (0x18E - 0x14B) * 0xA5 + 0x4B2 ; les 47 voix sont juste avant les sons).
Modèles (noms et ordre des pistes) : mods GameBanana trouvés par modele_sons.py ; seules les 52 / 32 pistes du jeu
sont gardées (les modèles en ajoutent). Les pistes de sons sans équivalent dans REX gardent le contenu du modèle ;
toutes les voix de Sonic sont remplacées (voix de REX, ou silence : Metal Sonic ne doit pas parler comme Sonic).
usage : python3 sons_metal.py <dossier 18E> <se modèle> <vc modèle> <se sortie> <vc sortie>
"""
import os, re, struct, sys
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '../../outils'))
import nus3  # noqa: E402

SRC, SE_M, VC_M, SE_O, VC_O = sys.argv[1:6]
SONS = {
    # déplacements (joués par les scripts de Sonic)
    'se_sonic_step_left_m': 8, 'se_sonic_step_left_s': 8, 'se_sonic_step_right_m': 9, 'se_sonic_step_right_s': 9,
    'se_sonic_step_left_m_win': 8, 'se_sonic_step_right_m_win': 9,
    'se_sonic_dash_start': 54, 'se_sonic_dash_stop': 55, 'se_sonic_dash_stop_win': 55,
    'se_sonic_jump01': 63, 'se_sonic_jump02': 64, 'se_sonic_escape': 1, 'se_sonic_escapeair': 1,
    # coups (joués par le plugin aux images de REX : sons.rs)
    'se_sonic_swing_s': 110, 'se_sonic_swing_m': 108, 'se_sonic_swing_l': 109, 'se_sonic_swing_ll': 107,
    'se_sonic_swing_s_win': 110, 'se_sonic_swing_m_win': 108, 'se_sonic_swing_l_win': 109,
    'se_sonic_attackair_H01': 47,      # sifflement (tilt avant, aériens haut / bas)
    'se_sonic_attackair_L01': 98,      # réacteur (arrière aérien, lancer haut, side B)
    'se_sonic_smash_H01': 3, 'se_sonic_smash_L01': 0, 'se_sonic_smash_S01': 10, 'se_sonic_smash_S02': 49,
    # spéciaux
    'se_sonic_special_N01': 82,        # Ring Spark Field : charge
    'se_sonic_special_N02': 93,        # décharge chargée
    'se_sonic_special_H02': 83, 'se_sonic_special_H03': 86, 'se_sonic_rounddash': 84, 'se_sonic_special_S02': 85,
    'se_sonic_special_H01': 6,         # Jet Boost : charge
    'se_sonic_special_S01': 97,        # V.Maximum Overdrive : départ
    'se_sonic_special_S03': 87,        # Black Shield
    'se_sonic_final01': 58, 'se_sonic_final03': 92, 'se_sonic_final04': 99,
}
VOIX = {
    'vc_sonic_001': 3, 'vc_sonic_002': 7, 'vc_sonic_003': 19, 'vc_sonic_004': 26,
    'vc_sonic_appeal01': 2, 'vc_sonic_appeal02': 5, 'vc_sonic_appeal03': 20,
    'vc_sonic_attack01': 8, 'vc_sonic_attack02': 9, 'vc_sonic_attack03': 21, 'vc_sonic_attack04': 39,
    'vc_sonic_attack05': 8, 'vc_sonic_attack06': 9, 'vc_sonic_attack07': 46,
    'vc_sonic_damage01': 30, 'vc_sonic_damage02': 33, 'vc_sonic_damagefly01': 31, 'vc_sonic_damagefly02': 32,
    'vc_sonic_damage_twinkle': 32, 'vc_sonic_knockout': 31, 'vc_sonic_missfoot01': 30, 'vc_sonic_missfoot02': 33,
    'vc_sonic_furafura': 34, 'vc_sonic_furasleep': 34, 'vc_sonic_ottotto': 41, 'vc_sonic_passive': 42,
    'vc_sonic_swimup': 43, 'vc_sonic_win01': 13, 'vc_sonic_win02': 15, 'vc_sonic_win03': 4,
}
D = {261: {}, 629: {}}
for l in open(os.path.join(SRC, 'extraction.txt')):
    m = re.search(r'DATA group=\d+ file=(\d+) data=(\d+) wave=(\d+)', l)
    if m:
        D[int(m.group(1))][int(m.group(2))] = int(m.group(3))


def onde(fichier, n):
    d = bytearray(open(os.path.join(SRC, 'group_44_file_%d_wave_%d.idsp' % (fichier, D[fichier][n])), 'rb').read())
    if struct.unpack_from('>I', d, 0x10)[0] == 0:
        return None
    # sons en boucle dans REX (arrêtés par script) : boucle retirée, sinon ils tournent sans fin (piège de Chief)
    if struct.unpack_from('>II', d, 0x14)[1]:
        struct.pack_into('>II', d, 0x14, 0, 0)
        print('BOUCLE RETIREE', fichier, n)
    return bytes(d)


def silence():
    """piste muette : une onde courte de REX dont les données sont mises à zéro (ADPCM : zéro = silence)"""
    d = bytearray(open(os.path.join(SRC, 'group_44_file_629_wave_%d.idsp' % D[629][7]), 'rb').read())
    chans, _, nsamp, _, _, _, hdr, cinfo, off = struct.unpack_from('>9I', d, 0x08)
    for c in range(chans):
        b = hdr + c * cinfo + 0x20
        d[b:b + 14] = bytes(14)
    d[off:] = bytes(len(d) - off)
    return bytes(d)


MUET = silence()
for modele, sortie, table, fichier, n_jeu in ((SE_M, SE_O, SONS, 629, 52), (VC_M, VC_O, VOIX, 261, 32)):
    pistes = nus3.read_full(modele)[:n_jeu]
    noms = [n for _, n, _ in pistes]
    manque = [k for k in table if k not in noms]
    assert not manque, manque
    out = []
    for i, n, d in pistes:
        if n in table:
            w = onde(fichier, table[n])
            print('PISTE', n, '<-', table[n], 'vide : silence' if w is None else len(w))
            d = w or MUET
        elif fichier == 261:
            print('PISTE', n, '<- silence')
            d = MUET
        out.append((i, n, d))
    nus3.write(sortie, out)
    print(os.path.basename(sortie), len(out), 'pistes')
