"""Zero (REX) : génère zero/plugin/src/generated.rs (coups, hitbox, enchaînements) depuis son moveset P+Ex, PENDANT la
compilation (rien du mod n'est dans le dépôt).

usage : python3 gen_zero.py <paquet extrait> <sortie.rs>
Ses animations étant les siennes (même squelette), les images et positions des hitbox sont reprises telles quelles ;
os Brawl -> os de son squelette Ultimate (noms en minuscules), TopN / TransN -> top.
"""
import os, re, struct, sys
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, '../../black-shadow/tools'))
sys.path.insert(0, HERE)
import gen  # noqa: E402
from arc import arc_entries  # noqa: E402
from brres import brres_files, chr0_info  # noqa: E402
from psa import load, subactions, header, events  # noqa: E402
from lz import decompress  # noqa: E402

PKG, OUT = sys.argv[1:3]
# v0.1 (skin sur Roy c00) : squelette de Roy -> les hitbox du sabre vont sur l'os sword1 de Roy (lame le long de son z)
ROY = '--roy' in sys.argv
tmp = '/tmp/gen_zero_model.pac'
open(tmp, 'wb').write(decompress(open(os.path.join(PKG, 'Costumes/PacFiles/0000/FitZero00.pac'), 'rb').read()))
bones = gen.bone_names(tmp)                     # n° d'os -> nom Brawl
CORE = {
    'TransN': 'top', 'TopN': 'top', 'XRotN': 'xrot', 'YRotN': 'rot', 'HipN': 'hip', 'WaistN': 'waist', 'BustN': 'bust',
    'NeckN': 'neck', 'HeadN': 'head', 'LShoulderN': 'claviclel', 'LShoulderJ': 'shoulderl', 'LArmJ': 'arml',
    'LHandN': 'handl', 'LHaveN': 'havel', 'RShoulderN': 'clavicler', 'RShoulderJ': 'shoulderr', 'RArmJ': 'armr',
    'RHandN': 'handr', 'RHaveN': 'haver', 'LLegJ': 'legl', 'LKneeJ': 'kneel', 'LFootJ': 'footl', 'LToeN': 'toel',
    'RLegJ': 'legr', 'RKneeJ': 'kneer', 'RFootJ': 'footr', 'RToeN': 'toer', 'ThrowN': 'throw',
}
gen.BONES = {n: CORE.get(n, n.lower()) for n in bones.values()}

# position le long du sabre de Zero (au repos) : SwordN = poignée, la lame suit son axe y
sys.path.insert(0, os.path.join(HERE, '../../black-shadow/tools/anim'))
import brawl_anim  # noqa: E402
import numpy as np  # noqa: E402
BR = brawl_anim.read_bones(tmp)
BLADE = {}
for n in BR:
    if n.startswith('Sword') and n != 'SwordSheathedM':
        BLADE[n.lower()] = float(np.linalg.norm(BR[n].world[:3, 3] - BR['SwordN'].world[:3, 3]))
print('lame de Zero (distance a la poignee)', {k: round(v, 2) for k, v in BLADE.items()})
# épée de Roy (son ACMD) : coups de la lame de z = 1.5 (garde) à 8.4 (bout), traînée jusqu'à 14.5.
# Lame de Zero : SwordBladeN (1.9) -> 1.5, SwordBladeN2 (13.5) -> 8.4, linéaire au-delà (bout de son tilt à 19.7)
Z0, Z1 = BLADE.get('swordbladen', 1.9), BLADE.get('swordbladen2', 13.5)


def on_roy_sword(line):
    m = re.search(r'bone: "([^"]+)".*?x: ([-\d.]+), y: ([-\d.]+), z: ([-\d.]+)', line)
    # v0.5 des animations : plus de miroir, ses bras / jambes gardent leur côté (arml reste arml)
    if not m or m.group(1) not in BLADE:
        return line
    d = BLADE[m.group(1)] + float(m.group(3))
    # v0.9 : lame du skin agrandie à la taille de P+ (facteur 1.19 sur Sword2, mesuré par retarget_roy.py) :
    # le long de Sword1, la lame va de z = 0.9 (garde) à 11.6 (bout) -> hitbox de P+ reportées sur cette lame
    zr = 0.9 + (d - Z0) * (11.6 - 0.9) / (Z1 - Z0)
    line = line.replace('bone: "%s"' % m.group(1), 'bone: "sword1"')
    return re.sub(r'x: [-\d.]+, y: [-\d.]+, z: [-\d.]+', 'x: 0.000, y: 0.000, z: %.3f' % max(zr, 0.0), line)

md = decompress(open(os.path.join(PKG, 'PacFiles/FitZeroMotion.pac'), 'rb').read())
lens = {}
for e in arc_entries(md):
    if e[4][:4] == b'bres':
        for folder, items in brres_files(e[4]).items():
            if 'Chr' in folder:
                for n, o in items:
                    lens.setdefault(n, chr0_info(e[4], o)[1])

m = load(os.path.join(PKG, 'PacFiles/FitZero.pac'))
base, subs = subactions(m)
byname = {}


def follow(mo):
    # v0.15 : sous-action qui commence par un « Goto » (00090100) : on suit vers son vrai code (côté B aérien)
    for _ in range(4):
        ev_ = events(m, base, mo, 2)
        if ev_ and ev_[0][0] == 0x00090100 and ev_[0][1]:
            mo = ev_[0][1][0][1]
        else:
            break
    return mo


for i, n, mo in subs:
    if not n or not mo:
        continue
    mo = follow(mo)
    tl = gen.timeline(m, base, mo)
    has = any(k in ('hit', 'catch', 'throw') for _, k, _ in tl)
    if n not in byname or (has and not byname[n][1]):
        byname[n] = (mo, has)

MAP = [
    ("game_attackdash", "AttackDash", 0), ("game_attacks3", "AttackS3S", 0),
    ("game_attackhi3", "AttackHi3", 0), ("game_attacklw3", "AttackLw3", 0),
    ("game_attacks4", "AttackS4S", 1), ("game_attackhi4", "AttackHi4", 1), ("game_attacklw4", "AttackLw4", 1),
    ("game_attackairn", "AttackAirN", 2), ("game_attackairf", "AttackAirF", 2), ("game_attackairb", "AttackAirB", 2),
    ("game_attackairhi", "AttackAirHi", 2), ("game_attackairlw", "AttackAirLw", 2),
    # v0.2 : spéciaux (kind 3 : joués par leurs propres scripts dans le plugin, pas d'enchaînement)
    ("zero_sp_n4", "SpecialN4", 3), ("zero_sp_s", "SpecialSAttack", 3), ("zero_sp_hi", "SpecialHi", 3),
    # v0.4 : bas B (coup au sol, coup en l'air, onde de choc à l'atterrissage)
    ("zero_sp_lw", "SpecialLw", 3), ("zero_sp_airlw", "SpecialAirLw", 3), ("zero_sp_lwland", "SpecialLwLanding", 3),
    # v0.13 : jabs 1 à 3 de P+ (joués par le plugin : un seul fichier d'animation de Roy, les 3 bout à bout)
    ("zero_jab1", "Attack11", 3), ("zero_jab2", "Attack12", 3), ("zero_jab3", "Attack13", 3),
    # v0.15 : smash chargés (mouvements special_s3* de Roy, joués quand le smash a été maintenu) et côté B aérien
    ("game_specials3hi", "AttackHi4Charged", 0), ("game_specials3lw", "AttackLw4Charged", 0),
    ("game_specials3s", "AttackS4SCharged", 0), ("zero_sp_airs", "SpecialAirSAttack", 3),
    # v0.18 : Final Smash de P+ (sol, air)
    ("zero_final", "Final", 3), ("zero_finalair", "FinalAir", 3),
]
START = {"AttackS4S": "AttackS4Start", "AttackHi4": "AttackHi4Start", "AttackLw4": "AttackLw4Start"}
MOTION = {'game_attack11': 'attack_11', 'game_attackdash': 'attack_dash', 'game_attacks3': 'attack_s3_s',
          'game_attackhi3': 'attack_hi3', 'game_attacklw3': 'attack_lw3', 'game_attacks4': 'attack_s4_s',
          'game_attackhi4': 'attack_hi4', 'game_attacklw4': 'attack_lw4', 'game_attackairn': 'attack_air_n',
          'game_attackairf': 'attack_air_f', 'game_attackairb': 'attack_air_b', 'game_attackairhi': 'attack_air_hi',
          'game_attackairlw': 'attack_air_lw', 'game_specials3hi': 'special_s3_hi', 'game_specials3lw': 'special_s3_lw',
          'game_specials3s': 'special_s3_s'}
lines = ["// généré pendant la compilation à partir du mod (ne pas publier)", "pub static MOVES: &[Mv] = &["]
faf = []
for script, sub, kind in MAP:
    if sub not in byname:
        print('absent', sub)
        continue
    mo = byname[sub][0]
    tl = gen.timeline(m, base, mo)
    start = float(lens.get(START[sub], 0)) if sub in START else 0.0
    total = float(lens.get(sub, 0)) + start
    evs = []
    for f, k, p in tl:
        if k == 'clear':
            evs.append("Ev { f: %.1f, clear: true, ..E }" % (f + start))
        elif k == 'hit':
            ln = gen.ev_line(f + start, p, bones)
            evs.append(on_roy_sword(ln) if ROY else ln)
    if not evs:
        print('sans coup', sub)
        continue
    fa = next((f + start for f, k, _ in tl if k == 'faf'), -1.0)
    if kind != 3:
        faf.append((MOTION[script], fa if fa > 0 else total))
    lines.append("    Mv { script: \"%s\", total: %.1f, hold: %.1f, kind: %d, evs: &[\n        %s,\n    ] },"
                 % (script, total, start, kind, ",\n        ".join(evs)))
    print('%-18s <- %-12s %5.1f images, %2d evenements, enchainement %.1f, os %s'
          % (script, sub, total, len(evs), fa, sorted({e.split('bone: "')[1].split('"')[0] for e in evs if 'bone:' in e})))
lines.append("];")
# v0.3 : tirs de buster (article du moveset) : la liste d'événements hors personnage qui choisit sa hitbox selon la
# charge (3 hitbox derrière des « si »). Rangés du plus faible au plus fort : petit, moyen, gros.
known = set()
_, hd, _ = header(m)
for tbl in ('subMain', 'subGFX', 'subSFX', 'subOther'):
    for i in range((hd['subGFX'] - hd['subMain']) // 4):
        o = struct.unpack_from('>I', m, base + hd[tbl] + 4 * i)[0]
        if o:
            known.add(o)
buster = []
for o in range(base, len(m) - 8, 4):
    if struct.unpack_from('>I', m, o)[0] != 0x000A0400:
        continue
    st = o - base
    if st in known:
        continue
    evs_ = events(m, base, st, 40)
    hits = [p for c, p in evs_ if c in (0x06000D00, 0x06150F00)]
    if len(hits) == 3 and any(c == 0x000E0000 for c, _ in evs_):
        buster = hits
        break
bl = sorted((gen.ev_line(0.0, p, bones) for p in buster), key=lambda l: float(re.search(r'dmg: ([\d.]+)', l).group(1)))
# v0.6 : éclair du Raikousen (bas B) : article dont toutes les hitbox font 2 % (au moins 5), une par angle, dans
# l'ordre, en gardant pour chaque angle la version au recul le plus faible (l'autre est une variante)
raiko = []
seen_ = set()
# adresses des événements des sous-actions du personnage (à exclure : ce sont ses coups, pas l'article)
for st_ in known:
    q_ = base + st_
    for _ in range(400):
        if struct.unpack_from('>Q', m, q_)[0] == 0:
            break
        seen_.add(q_)
        q_ += 8
for o in range(base, len(m) - 8, 4):
    if struct.unpack_from('>I', m, o)[0] not in (0x06000D00, 0x06150F00):
        continue
    np_ = struct.unpack_from('>B', m, o + 2)[0]
    po = struct.unpack_from('>I', m, o + 4)[0]
    if np_ < 13 or not po or base + po + 8 * np_ > len(m):
        continue
    pr = [struct.unpack_from('>Ii', m, base + po + 8 * i) for i in range(np_)]
    if abs(gen.pv(*pr[1]) - 2.0) > 1e-3 or o in seen_:
        continue
    seen_.add(o)
    raiko.append((o, pr))
# regroupe les hitbox proches (même liste d'événements : écart de moins de 0x40 octets)
groups_ = []
for o, pr in sorted(raiko, key=lambda x: x[0]):
    if groups_ and o - groups_[-1][-1][0] < 0x40:
        groups_[-1].append((o, pr))
    else:
        groups_.append([(o, pr)])
best = max(groups_, key=len) if groups_ else []
# paires « si / sinon » (même angle, deux reculs) : on garde la plus faible ; puis le dernier coup
picked = []
srt = sorted(best, key=lambda x: x[0])
i_ = 0
while i_ < len(srt):
    pr = srt[i_][1]
    a = int(gen.pv(*pr[2])) & 0xffff
    if i_ + 1 < len(srt) and (int(gen.pv(*srt[i_ + 1][1][2])) & 0xffff) == a:
        q = srt[i_ + 1][1]
        pr = pr if (pr[3][1] & 0xffff) <= (q[3][1] & 0xffff) else q
        i_ += 2
    else:
        i_ += 1
    picked.append(pr)
rl = [re.sub(r'bone: "[^"]+"', 'bone: "top"', gen.ev_line(0.0, pr, bones)) for pr in picked] if len(best) >= 5 else []
# v0.7 : chopes (sol, course, demi-tour), coup pendant la prise et lancers : zones, images, dégâts, angles, recul
lines.append("pub static SUBS: &[Sub] = &[")
for name in ('Catch', 'CatchDash', 'CatchTurn', 'CatchAttack', 'ThrowF', 'ThrowB', 'ThrowHi', 'ThrowLw'):
    if name not in byname:
        print('absent', name)
        continue
    mo_ = byname[name][0]
    # codes de tous les événements (pour repérer un demi-tour, etc.)
    print('EVTS %-12s' % name, ' '.join('%08X' % c for c, _ in events(m, base, mo_, 80)))
    ln = gen.sub_lines(name, mo_, m, base, bones, float(lens.get(name, 0)))
    if ROY:
        ln = re.sub(r'Ev \{[^}]*\}', lambda q: on_roy_sword(q.group(0)), ln)
    lines.append(ln)
lines.append("];")
# lancers où il se retourne (événement 05000000 « Reverse Direction ») : image du demi-tour
lines.append("pub static TURN: &[(&str, f32)] = &[")
for name in ('ThrowF', 'ThrowB', 'ThrowHi', 'ThrowLw'):
    if name not in byname:
        continue
    f_ = 0.0
    for c_, p_ in events(m, base, byname[name][0], 80):
        if c_ == 0x00010100:
            f_ += gen.pv(*p_[0])
        elif c_ == 0x00020100:
            f_ = gen.pv(*p_[0])
        elif c_ == 0x05000000:
            print('DEMI-TOUR', name, f_)
            lines.append("    (\"%s\", %.1f)," % (name, f_))
            break
lines.append("];")
# diagnostic (v0.8) : sabre sorti ou rangé ? événements « Model Changer » (0B00 / 0B01) de chaque sous-action
for i_, n_, mo_ in subs:
    if not n_ or not mo_:
        continue
    f_ = 0.0
    vis = []
    for c_, p_ in events(m, base, mo_, 120):
        if c_ == 0x00010100:
            f_ += gen.pv(*p_[0])
        elif c_ == 0x00020100:
            f_ = gen.pv(*p_[0])
        elif (c_ >> 16) in (0x0B00, 0x0B01, 0x0B02):
            vis.append('%g:%04X(%s)' % (f_, c_ >> 16, ','.join(str(x[1]) for x in p_)))
    print('VIS %-22s %s' % (n_, ' '.join(vis) if vis else '-'))
# diagnostic B neutre (v0.10) : tous les événements (principal, GFX, son, autre) de SpecialN1-4 / SpecialAirN1-4
_, hd_, _ = header(m)
nsub_ = (hd_['subGFX'] - hd_['subMain']) // 4
for i_, n_, mo_ in subs:
    if not re.match(r'^(Attack(S4S|Hi4|Lw4)\w*|SpecialAirS\w*|Final\w*|Entry\w*)$', n_):
        continue
    for tbl_ in ('subMain', 'subGFX', 'subSFX', 'subOther'):
        o_ = struct.unpack_from('>I', m, base + hd_[tbl_] + 4 * i_)[0] if i_ < nsub_ else 0
        if not o_:
            continue
        f_ = 0.0
        out_ = []
        for c_, p_ in events(m, base, o_, 300):
            if c_ == 0x00010100:
                f_ += gen.pv(*p_[0])
                continue
            if c_ == 0x00020100:
                f_ = gen.pv(*p_[0])
                continue
            out_.append('%g:%08X(%s)' % (f_, c_, ','.join('%d/%d' % (t, v) for t, v in p_[:6])))
        print('BASB %-14s %-8s %s' % (n_, tbl_, ' '.join(out_)[:6000]))
print('CHR0', ' '.join('%s:%s' % (k, v) for k, v in sorted(lens.items())))
print('SUBS', ' '.join('%d:%s' % (i, n) for i, n, mo in subs if n))
# v0.9 : montée du lancer haut (P+) : déplacement de l'os XRotN image par image (avant, haut), à la taille de Roy
# (échelle des animations converties : hanches Roy 11.24 / Zero 12.13). Le plugin déplace vraiment Zero.
hi_move = []
for e_ in arc_entries(md):
    if e_[4][:4] != b'bres' or hi_move:
        continue
    for folder_, items_ in brres_files(e_[4]).items():
        if 'Chr' not in folder_:
            continue
        for n_, o_ in items_:
            if n_ == 'ThrowHi' and not hi_move:
                an_ = brawl_anim.read_chr0(e_[4], o_)
                p0_ = None
                for k_ in range(max(1, an_['frames'])):
                    w_ = brawl_anim.world_pose(BR, an_, float(k_))['XRotN'][:3, 3]
                    if p0_ is None:
                        p0_ = w_.copy()
                    d_ = (w_ - p0_) * (11.24 / 12.13)
                    hi_move.append((float(d_[2]), float(d_[1])))
# diagnostic bas B (v0.11) : déplacement de la racine (XRotN, TransN) dans ses animations
for e_ in arc_entries(md):
    if e_[4][:4] != b'bres':
        continue
    for folder_, items_ in brres_files(e_[4]).items():
        if 'Chr' not in folder_:
            continue
        for n_, o_ in items_:
            if n_ in ('SpecialLw', 'SpecialLwAir', 'SpecialAirLw', 'SpecialLwLanding'):
                an_ = brawl_anim.read_chr0(e_[4], o_)
                pts_ = []
                for k_ in range(0, max(1, an_['frames']), 3):
                    w_ = brawl_anim.world_pose(BR, an_, float(k_))
                    pts_.append('%d:%s/%s' % (k_, ','.join('%.1f' % v for v in w_['XRotN'][:3, 3][[2, 1]]),
                                              ','.join('%.1f' % v for v in w_['TransN'][:3, 3][[2, 1]])))
                print('RACINE %-16s %d images (XRotN z,y / TransN z,y) %s' % (n_, an_['frames'], ' '.join(pts_)))
print('MONTEE lancer haut', [(round(a, 1), round(b, 1)) for a, b in hi_move[::6]])
lines.append("pub static THROW_HI_MOVE: &[(f32, f32)] = &[")
for a_, b_ in hi_move:
    lines.append("    (%.3f, %.3f)," % (a_, b_))
lines.append("];")
lines.append("pub static RAIKO: &[Ev] = &[")
for l in rl:
    lines.append("    " + l + ",")
lines.append("];")
print('eclair raikousen', [(re.search(r'dmg: ([\d.]+)', l).group(1), re.search(r'angle: (\d+)', l).group(1),
                            re.search(r'kbg: (\d+)', l).group(1)) for l in rl])
lines.append("pub static BUSTER: &[Ev] = &[")
for l in bl:
    lines.append("    " + re.sub(r'bone: "[^"]+"', 'bone: "top"', l) + ",")
lines.append("];")
print('buster', [re.search(r'dmg: ([\d.]+)', l).group(1) for l in bl])
# v0.13 : jabs : (début dans le fichier, durée, enchaînement P+) de Attack11, Attack12, Attack13
jab = []
o_ = 0.0
for n_ in ('Attack11', 'Attack12', 'Attack13'):
    if n_ not in byname:
        continue
    ln_ = float(lens.get(n_, 0))
    fa_ = next((f for f, k, _ in gen.timeline(m, base, byname[n_][0]) if k == 'faf'), -1.0)
    jab.append((o_, ln_, fa_ if fa_ > 0 else ln_))
    o_ += ln_
print('JABS', jab)
lines.append("pub static JAB: &[(f32, f32, f32)] = &[")
for a_, b_, c_ in jab:
    lines.append("    (%.1f, %.1f, %.1f)," % (a_, b_, c_))
lines.append("];")
lines.append("pub static FAF: &[(&str, f32)] = &[")
for mname, f in faf:
    lines.append("    (\"%s\", %.1f)," % (mname, f))
lines.append("];")
open(OUT, 'w').write("\n".join(lines) + "\n")
print('generated.rs ok :', len(faf), 'coups')
