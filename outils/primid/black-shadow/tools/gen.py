"""Génère src/generated.rs à partir des fichiers du mod Brawl, PENDANT la compilation.

Le dépôt ne contient aucune donnée du mod : ce script lit les fichiers téléchargés depuis
GameBanana au moment de la compilation (licence CC BY-NC-ND : rien de dérivé n'est publié).

usage : python3 gen.py <FitXXX.pac (moveset)> <FitXXXMotionEtc.pac> <FitXXX00.pac (modèle)> <sortie.rs>
"""
import sys, struct
sys.path.insert(0, __file__.rsplit('/', 1)[0])
from arc import arc_entries
from brres import brres_files, dict_entries, chr0_info
from psa import load, subactions, events

# script Ultimate de Ganondorf  <-  sous-action Brawl, type (0 normal, 1 smash, 2 aérien)
MAP = [
    ("game_attack11", "Attack11", 0),
    ("game_attackdash", "AttackDash", 0),
    ("game_attacks3hi", "AttackS3Hi", 0),
    ("game_attacks3", "AttackS3S", 0),
    ("game_attacks3lw", "AttackS3Lw", 0),
    ("game_attackhi3", "AttackHi3", 0),
    ("game_attacklw3", "AttackLw3", 0),
    ("game_attacks4", "AttackS4S", 1),
    ("game_attackhi4", "AttackHi4", 1),
    ("game_attacklw4", "AttackLw4", 1),
]

# v0.3 : l'animation Ultimate d'un smash = début (Start) + coup mis bout à bout
START = {"AttackS4S": "AttackS4Start", "AttackHi4": "AttackHi4Start", "AttackLw4": "AttackLw4Start"}
MAP += [
    ("game_attackairn", "AttackAirN", 2),
    ("game_attackairf", "AttackAirF", 2),
    ("game_attackairb", "AttackAirB", 2),
    ("game_attackairhi", "AttackAirHi", 2),
    ("game_attackairlw", "AttackAirLw", 2),
]

# os du squelette Brawl de Ganondorf -> os d'Ultimate
BONES = {
    'TopN': 'top', 'TransN': 'top', 'XRotN': 'top', 'YRotN': 'top', 'HipN': 'hip',
    'LKoshi01N': 'hip', 'LLegJ': 'legl', 'LKneeJ': 'kneel', 'LFootJ': 'footl', 'LToeN': 'footl',
    'RKoshi01N': 'hip', 'RLegJ': 'legr', 'RKneeJ': 'kneer', 'RFootJ': 'footr', 'RToeN': 'footr',
    'WaistN': 'waist', 'BustN': 'bust', 'NeckN': 'head', 'HeadN': 'head',
    'LShoulderN': 'shoulderl', 'LShoulderJ': 'shoulderl', 'LShoulderJb': 'shoulderl', 'LArmJ': 'arml', 'LHandN': 'handl', 'LHaveN': 'handl',
    'RShoulderN': 'shoulderr', 'RShoulderJ': 'shoulderr', 'RShoulderJb': 'shoulderr', 'RArmJ': 'armr', 'RHandN': 'handr', 'RHaveN': 'handr',
}

# élément Brawl (5 bits bas des drapeaux) -> attribut Ultimate
ELEM = {0: 'collision_attr_normal', 2: 'collision_attr_cutup', 3: 'collision_attr_elec', 4: 'collision_attr_ice',
        5: 'collision_attr_fire', 0x0B: 'collision_attr_bury', 0x13: 'collision_attr_purple', 0x14: 'collision_attr_paralyze', 0x15: 'collision_attr_aura'}


def bone_names(model_pac):
    d = open(model_pac, 'rb').read()
    dd = [x for x in arc_entries(d) if x[0] == 2][0][4]
    n, o = brres_files(dd)['3DModels(NW4R)'][0]
    offs = struct.unpack_from('>11i', dd, o + 0x10)
    idx = {}
    for name, bo in dict_entries(dd, o + offs[1]):
        idx[struct.unpack_from('>i', dd, bo + 0xC)[0]] = name
    return idx


def anim_lengths(motion_pac):
    d = open(motion_pac, 'rb').read()
    res = {}
    for typ, idx, size, red, data in arc_entries(d):
        if data[:4] != b'ARC\0':
            continue
        for t2, i2, s2, r2, dd in arc_entries(data):
            if dd[:4] != b'bres':
                continue
            for folder, items in brres_files(dd).items():
                if 'Chr' in folder:
                    for n, o in items:
                        res[n] = chr0_info(dd, o)[1]
    return res


def pv(t, v):
    return v / 60000.0 if t == 1 else float(v)


def timeline(m, base, off):
    """liste (frame, 'hit', params) / (frame, 'clear', None)"""
    f = 0.0
    out = []
    for code, params in events(m, base, off):
        if code == 0x00010100:
            f += pv(*params[0])
        elif code == 0x00020100:
            f = pv(*params[0])
        elif code in (0x06000D00, 0x06150F00) and len(params) >= 13:
            out.append((f, 'hit', params))
        elif code == 0x06040000:
            out.append((f, 'clear', None))
        elif code in (0x060A0800, 0x060A0900) and len(params) >= 8:
            out.append((f, 'catch', params))
        elif code == 0x060E1100 and len(params) >= 8:
            out.append((f, 'throw', params))
        elif code == 0x060F0500:
            out.append((f, 'release', params))
        elif code == 0x64000000:
            out.append((f, 'faf', params))
    return out


# sous-actions des spéciaux dont on lit les coups / chopes / projections
SPECIAL_SUBS = ['SpecialN', 'SpecialNDown', 'SpecialAirN', 'SpecialSStart', 'SpecialS', 'SpecialAirSStart',
                'SpecialAirS', 'SpecialHiStart', 'SpecialHiLand', 'SpecialLwHit', 'SpecialLwAirLoop', 'SpecialLwAirLand',
                # v0.7 : prises (chope au sol, course, demi-tour ; coup pendant la prise et projections pour la suite)
                'Catch', 'CatchDash', 'CatchTurn', 'CatchAttack', 'ThrowF', 'ThrowB', 'ThrowHi', 'ThrowLw']


def ev_line(f, p, bones):
    v = [pv(t, x) for t, x in p]
    raw0 = p[0][1] & 0xffffffff
    bone = BONES.get(bones.get(raw0 >> 16, ''), 'top')
    hid = raw0 & 0xffff
    kbg = p[3][1] & 0xffff
    wdsk = (p[3][1] >> 16) & 0xffff
    bkb = p[4][1] & 0xffff
    flags = p[12][1] & 0xffffffff
    attr = ELEM.get(flags & 0x1f, 'collision_attr_normal')
    bz, by, bx = v[6], v[7], v[8]
    dmg = v[1] if 0 <= v[1] < 200 else -1.0  # -1 : dégâts donnés par une variable (bas B chargé)
    special = 'true' if len(p) > 13 else 'false'
    return ("Ev { f: %.1f, id: %d, bone: \"%s\", dmg: %.2f, angle: %d, kbg: %d, fkb: %d, bkb: %d, size: %.3f, x: %.3f, y: %.3f, z: %.3f, hitlag: %.3f, sdi: %.3f, trip: %.3f, attr: \"%s\", special: %s, ..E }"
            % (f, min(hid, 7), bone, dmg, int(v[2]) & 0xffff, kbg, wdsk, bkb, v[5], bz, by, bx,
               v[10] if v[10] > 0 else 1.0, v[11] if v[11] > 0 else 1.0, v[9] / 100.0, attr, special))


def sub_lines(name, mo, m, base, bones, total):
    tl = timeline(m, base, mo)
    evs, cats, thr = [], [], []
    release = -1.0
    for f, k, p in tl:
        if k == 'clear':
            evs.append("Ev { f: %.1f, clear: true, ..E }" % f)
        elif k == 'hit':
            evs.append(ev_line(f, p, bones))
        elif k == 'catch':
            v = [pv(t, x) for t, x in p]
            # id, os, taille, Z, Y, X, ?, situations (1 = au sol, 3 = sol + air)
            # comme pour les coups : Brawl Z -> x (profondeur), Brawl X (vers l'avant) -> z (vers l'avant pour top)
            cats.append("Catch { f: %.1f, id: %d, size: %.3f, x: %.3f, y: %.3f, z: %.3f, ground_only: %s }"
                        % (f, int(v[0]), v[2], v[3], v[4], v[5], 'true' if int(v[7]) == 1 else 'false'))
        elif k == 'release':
            if release < 0:
                release = f
        elif k == 'throw':
            v = [int(x) if t == 0 else pv(t, x) for t, x in p]
            thr.append("Throw { id: %d, dmg: %.1f, angle: %d, kbg: %d, fkb: %d, bkb: %d, attr: \"%s\" }"
                       % (int(v[0]), float(v[2]), int(v[3]) & 0xffff, int(v[4]), int(v[5]), int(v[6]), ELEM.get(int(v[7]) & 0x1f, 'collision_attr_normal')))
    if cats:
        print(f"DEBUG_CATCH {name}: " + " | ".join(cats))
    faf = next((f for f, k, _ in tl if k == 'faf'), -1.0)
    print("FAF %-18s %-12s anim %5.1f  enchainement %5.1f" % (name, name, total, faf))
    if name == 'ThrowB' and faf > 0:
        FAFS.append(('throw_b', faf))
    if name.startswith(('Catch', 'Throw')):
        print("PRISE %-12s anim=%s lacher=%s throws=%s | %s" % (name, total, release, thr, " | ".join(evs)))
    return ("    Sub { name: \"%s\", evs: &[%s], catches: &[%s], throws: &[%s], release: %.1f, total: %.1f },"
            % (name, ", ".join(evs), ", ".join(cats), ", ".join(thr), release, total))


# script Ultimate -> nom du mouvement (motion) d'Ultimate
MOTION = {'game_attack11': 'attack_11', 'game_attackdash': 'attack_dash', 'game_attacks3hi': 'attack_s3_hi',
          'game_attacks3': 'attack_s3_s', 'game_attacks3lw': 'attack_s3_lw', 'game_attackhi3': 'attack_hi3',
          'game_attacklw3': 'attack_lw3', 'game_attacks4': 'attack_s4_s', 'game_attackhi4': 'attack_hi4',
          'game_attacklw4': 'attack_lw4', 'game_attackairn': 'attack_air_n', 'game_attackairf': 'attack_air_f',
          'game_attackairb': 'attack_air_b', 'game_attackairhi': 'attack_air_hi', 'game_attackairlw': 'attack_air_lw'}
FAFS = []


def main():
    moveset, motion, model, out = sys.argv[1:5]
    bones = bone_names(model)
    lens = anim_lengths(motion)
    m = load(moveset)
    base, subs = subactions(m)
    byname = {}
    for i, n, mo in subs:
        if n and mo and n not in byname:
            byname[n] = mo
    # pour le smash avant, la sous-action 0x58 est celle qui frappe
    for i, n, mo in subs:
        if n == 'AttackS4S' and mo:
            tl = timeline(m, base, mo)
            if any(k == 'hit' for _, k, _ in tl):
                byname['AttackS4S'] = mo
                break
    lines = ["// généré pendant la compilation à partir du mod (ne pas publier)", "pub static MOVES: &[Mv] = &["]
    for script, sub, kind in MAP:
        mo = byname.get(sub)
        if not mo:
            continue
        tl = timeline(m, base, mo)
        if not any(k == 'hit' for _, k, _ in tl):
            continue
        start = float(lens.get(START[sub], 0)) if sub in START else 0.0
        total = float(lens.get(sub, 0)) + start
        evs = []
        for f, k, p in tl:
            f += start
            if k not in ('hit', 'clear'):
                continue
            if k == 'clear':
                evs.append("Ev { f: %.1f, clear: true, ..E }" % f)
                continue
            v = [pv(t, x) for t, x in p]
            raw0 = p[0][1] & 0xffffffff
            bone = BONES.get(bones.get(raw0 >> 16, ''), 'top')
            hid = raw0 & 0xffff
            kbg = p[3][1] & 0xffff
            wdsk = (p[3][1] >> 16) & 0xffff
            bkb = p[4][1] & 0xffff
            flags = p[12][1] & 0xffffffff
            attr = ELEM.get(flags & 0x1f, 'collision_attr_normal')
            # Brawl : décalages Z, Y, X (X = vers l'avant pour TopN) -> Ultimate : x, y, z (z = vers l'avant pour top)
            bz, by, bx = v[6], v[7], v[8]
            evs.append("Ev { f: %.1f, id: %d, bone: \"%s\", dmg: %.2f, angle: %d, kbg: %d, fkb: %d, bkb: %d, size: %.3f, x: %.3f, y: %.3f, z: %.3f, hitlag: %.3f, sdi: %.3f, trip: %.3f, attr: \"%s\", ..E }"
                       % (f, min(hid, 7), bone, v[1], int(v[2]) & 0xffff, kbg, wdsk, bkb, v[5], bz, by, bx,
                          v[10] if v[10] > 0 else 1.0, v[11] if v[11] > 0 else 1.0, v[9] / 100.0, attr))
        faf = next((f + start for f, k, _ in tl if k == 'faf'), -1.0)
        FAFS.append((MOTION[script], faf if faf > 0 else total))
        print("FAF %-18s %-12s anim %5.1f  enchainement %5.1f" % (script, sub, total, faf))
        lines.append("    Mv { script: \"%s\", total: %.1f, hold: %.1f, kind: %d, evs: &[\n        %s,\n    ] }," % (script, total, start, kind, ",\n        ".join(evs)))
        print("%-18s <- %-12s %3d images, %d événements" % (script, sub, total, len(evs)))
    lines.append("];")
    # spéciaux
    lines.append("pub static SUBS: &[Sub] = &[")
    for name in SPECIAL_SUBS:
        mo = byname.get(name)
        if not mo:
            print('spécial absent :', name)
            continue
        lines.append(sub_lines(name, mo, m, base, bones, float(lens.get(name, 0))))
        print('spécial %-18s ok' % name)
    lines.append("];")
    # v0.10 : image à partir de laquelle il peut enchaîner (Allow Interrupt du mod, ou fin de l'animation)
    lines.append("pub static FAF: &[(&str, f32)] = &[")
    for motion, f in FAFS:
        lines.append("    (\"%s\", %.1f)," % (motion, f))
    lines.append("];")
    # diagnostic : effets visuels (sous-actions GFX) des coups, pour remettre l'équivalent dans Ultimate
    from psa import header
    _, h, _ = header(m)
    idx = {}
    for i, n, mo in subs:
        if n and n not in idx:
            idx[n] = i
    for _, sub, _ in MAP + [('', 'AttackS4Start', 0), ('', 'AttackHi4Start', 0), ('', 'AttackLw4Start', 0), ('', 'SpecialN', 0)]:
        if sub not in idx:
            continue
        go = struct.unpack_from('>I', m, base + h['subGFX'] + 4 * idx[sub])[0]
        if not go:
            continue
        def walk(off, fr, depth):
            for code, params in events(m, base, off):
                if code == 0x00010100:
                    fr += pv(*params[0])
                    continue
                if code == 0x00020100:
                    fr = pv(*params[0])
                    continue
                if code in (0x00070100, 0x00090100) and params and params[0][0] == 2 and params[0][1] > 0 and depth < 4:
                    fr = walk(params[0][1], fr, depth + 1)  # sous-routine / saut : on suit
                    if code == 0x00090100:
                        return fr
                    continue
                ps = ",".join(str(round(pv(t, x), 2)) if t == 1 else ('%X' % (x & 0xffffffff) if t == 0 and abs(x) > 4096 else str(x)) for t, x in params)
                print("GFX %-14s f=%5.1f %08X [%s]" % (sub, fr, code, ps))
            return fr
        walk(go, 0.0, 0)
    open(out, 'w').write("\n".join(lines) + "\n")


if __name__ == '__main__':
    main()
