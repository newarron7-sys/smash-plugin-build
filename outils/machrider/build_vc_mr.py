"""Mach Rider : vc_captain_c190.nus3audio avec ses sons (banque 21C du mod : fichier 261 = voix, 629 = bruitages).
Noms et ordre des pistes : modèle (mod de voix de Captain Falcon de GameBanana, template.py captain).
Identifiants du mod -> onde : voix n = id - 0xCB38 (261) ; bruitages n = id - 0xCB67 (629).
Pistes jouées par le jeu (dégâts, KO, rebord ...) : ses cris ; pistes des coups / spéciaux (jouées par les scripts de
Falcon, retirés) : ses cris et bruitages, joués par le plugin aux images de son PSA. Pistes sans son de Mach Rider :
muettes (onde vide du mod), pour ne plus entendre Falcon.
usage : python3 build_vc_mr.py <dossier idsp 21C> <modèle> <sortie> [carte.json [clé : vc / se]]"""
import json, os, re, struct, sys
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
import nus3  # noqa: E402

IDSP, MODELE, OUT = sys.argv[1:4]
MAP = json.load(open(sys.argv[4]))[sys.argv[5] if len(sys.argv) > 5 else 'vc'] if len(sys.argv) > 4 else {}
D = {}
for l in open(os.path.join(IDSP, 'extraction.txt')):
    if l.startswith('DATA'):
        f = int(re.search(r'file=(\d+)', l).group(1))
        D[(f, int(re.search(r'data=(\d+)', l).group(1)))] = int(re.search(r'wave=(\d+)', l).group(1))


def onde(f, n):
    return open(os.path.join(IDSP, 'group_44_file_%d_wave_%d.idsp' % (f, D[(f, n)])), 'rb').read()


muet = onde(261, 0)          # voix 0 du mod : vide
src = nus3.read(MODELE)
noms = [t[1] for t in src]
print('pistes du modèle :', len(noms))
for t in src:
    print('TRACK %4d %-40s %8d %s' % t)
tracks = []
for i, n in enumerate(noms):
    if n in MAP:
        f, w = MAP[n]
        data = bytearray(onde(f, w))
        # sons en boucle dans le mod (le PSA les arrête) : boucle retirée (comme Master Chief), sinon sans fin
        boucle = struct.unpack_from('>II', data, 0x14)
        if boucle[1]:
            struct.pack_into('>II', data, 0x14, 0, 0)
            print('BOUCLE RETIREE', n, boucle)
        data = bytes(data)
        print('SON', n, '<-', f, w, len(data))
    else:
        data = muet
    assert data[:4] == b'IDSP', n
    tracks.append((i, n, data))
manque = sorted(set(MAP) - set(noms))
if manque:
    print('ABSENTES du modèle :', manque)
nus3.write(OUT, tracks)
assert [t[1] for t in nus3.read(OUT)] == noms
print('ok', len(tracks), 'pistes')
