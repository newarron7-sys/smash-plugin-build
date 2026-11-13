"""Génère src/generated.rs d'un perso de REX (coups normaux, smashs, aériens) PENDANT la compilation.

Rien du mod n'est dans le dépôt : ce script lit les fichiers du build REX récupérés au moment de la compilation.
usage : python3 gen_rex.py <FitXXX.pac> <FitXXXMotionEtc.pac> <FitXXX00.pac (modèle)> <sortie.rs>
"""
import json
import os, sys, tempfile
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, '../../black-shadow/tools'))
sys.path.insert(0, os.path.join(HERE, '../../zero/tools'))
from psa import load, subactions  # noqa: E402
from gen import timeline, pv, BONES, ELEM, anim_lengths, bone_names  # noqa: E402
from lz import decompress  # noqa: E402
import gen as _gen  # noqa: E402
import psa as _psa  # noqa: E402

if os.environ.get('SUIVRE') == '1':
    # Super Shadow : certaines sous-actions ne font qu'un « goto » (0x00090100) ou un appel de sous-routine
    # (0x00070100, retour 0x00080000) vers le vrai script : on suit ces sauts (opt-in, Rock inchangé)
    def _events_suivis(m, base, off, maxn=400, prof=0, vus=None):
        vus = set() if vus is None else vus
        out = []
        for code, params in _psa.events(m, base, off, maxn):
            if code == 0x00080000:
                break
            if code in (0x00070100, 0x00090100) and params and params[0][0] == 2 and prof < 8:
                cible = params[0][1]
                if (cible, prof) not in vus and 0 < base + cible < len(m) - 8:
                    vus.add((cible, prof))
                    try:
                        out += _events_suivis(m, base, cible, maxn, prof + 1, vus)
                    except Exception:
                        pass  # cible externe (scripts communs de Brawl) : ignorée
                if code == 0x00090100:
                    break
                continue
            out.append((code, params))
        return out
    _gen.events = _events_suivis

# script Ultimate du perso de base <- sous-action Brawl (les absentes sont ignorées), type (0 normal, 1 smash, 2 aérien)
MAP = [
    ("game_attack11", "Attack11", 0), ("game_attack12", "Attack12", 0), ("game_attack13", "Attack13", 0), ("game_attackdash", "AttackDash", 0),
    ("game_attacks3hi", "AttackS3Hi", 0), ("game_attacks3", "AttackS3S", 0), ("game_attacks3lw", "AttackS3Lw", 0),
    ("game_attackhi3", "AttackHi3", 0), ("game_attacklw3", "AttackLw3", 0),
    ("game_attacks4hi", "AttackS4Hi", 1), ("game_attacks4", "AttackS4S", 1), ("game_attacks4lw", "AttackS4Lw", 1),
    ("game_attackhi4", "AttackHi4", 1), ("game_attacklw4", "AttackLw4", 1),
    ("game_attackairn", "AttackAirN", 2), ("game_attackairf", "AttackAirF", 2), ("game_attackairb", "AttackAirB", 2),
    ("game_attackairhi", "AttackAirHi", 2), ("game_attackairlw", "AttackAirLw", 2),
    # v0.11 : spéciaux de REX ajoutés à Terry (motion_patch.yaml) : Double Reppu Ken, attaque sautée du B bas
    ("game_specialairlwrisew", "SpecialN2", 0), ("game_specialairlwstart", "SpecialLwSpecialLoop", 0),
    # Super Shadow : B neutre (après le homing) : coup qui paralyse, puis poing, pied, pied qui éjecte
    ("game_specialsendstart", "SpecialSEnd", 0), ("game_specialnhit1", "SpecialNHit", 0), ("game_specialnhit2", "SpecialNHit2", 0), ("game_specialnhit3", "SpecialNHit3", 0), ("game_specialnhit4", "SpecialNHit4", 0),
]
# script -> (image où commencent ses coups, animation jouée) : l'attaque du B bas boucle en l'air dans REX
# (SpecialLwSpecialLoop) ; ici ses coups sont posés sur la descente de SpecialLwSpecialStart
PLACE = {"game_specialairlwstart": (16.0, "SpecialLwSpecialStart")}
# Master Chief : MAP_PLUS="script=SousAction@Debut1+Debut2:type,..." : scripts ajoutés / remplacés, avec les
# animations jouées avant (attaque en course : AttackDash puis Spartan / Slide ; Ground Pound : début + charge)
_MAP_PLUS = {}
for _it in os.environ.get('MAP_PLUS', '').split(','):
    if '=' in _it:
        _sc, _rest = _it.split('=')
        _sub, _, _rest2 = _rest.partition('@')
        _st, _, _k = _rest2.partition(':')
        _MAP_PLUS[_sc] = (_sub, _st, int(_k or 0))
GRAB_SUBS = ['Catch', 'CatchDash', 'CatchTurn', 'CatchAttack', 'ThrowF', 'ThrowB', 'ThrowHi', 'ThrowLw']
START = {"AttackS4S": "AttackS4Start", "AttackS4Hi": "AttackS4Start", "AttackS4Lw": "AttackS4Start",
         "AttackHi4": "AttackHi4Start", "AttackLw4": "AttackLw4Start"}
for _sc, (_sub, _st, _k) in _MAP_PLUS.items():
    MAP = [x for x in MAP if x[0] != _sc] + [(_sc, _sub, _k)]
    if _st:
        START[_sub] = _st


# Super Shadow : repères des os pour les décalages de hitbox (hit_keys.py), opt-in
HIT_KEYS = json.load(open(os.environ['HIT_KEYS'])) if os.environ.get('HIT_KEYS') else {}
# Master Chief : coups de ses armes posés sur ThrowN (le bout du canon, animé : il vise en haut, en bas, balaie) avec
# un décalage le long du canon, parfois variable (tir instantané : distance trouvée par un rayon dans son module de
# code). CANON=<portée> : ces coups passent dans le repère de TopN avec la pose de ThrowN à leur image (animation de
# REX) ; un décalage variable devient une capsule du canon jusqu'à la portée (le tir traverse l'écran comme dans REX)
CANON = {}
for _kv in os.environ.get('CANON', '').split(','):
    if ':' in _kv:
        CANON[_kv.split(':')[0]] = float(_kv.split(':')[1])
_POSE = {}


def canon_pose(model, motion, anim, f):
    """matrice de ThrowN dans le repère de TopN à l'image f de l'animation REX `anim`"""
    import numpy as np
    sys.path.insert(0, os.path.join(HERE, '../../black-shadow/tools/anim'))
    import brawl_anim as BA
    from gen import arc_entries, brres_files
    if not _POSE:
        _POSE['B'] = BA.read_bones(plain(model))
        an = {}

        def walk(d):
            for e in arc_entries(d):
                if e[4][:4] == b'ARC\0':
                    walk(e[4])
                elif e[4][:4] == b'bres':
                    for fo, items in brres_files(e[4]).items():
                        if 'Chr' in fo:
                            for n, o in items:
                                an.setdefault(n, (e[4], o))
        walk(open(plain(motion), 'rb').read())
        _POSE['A'] = an
    if anim not in _POSE['A']:
        return None
    W = BA.world_pose(_POSE['B'], BA.read_chr0(*_POSE['A'][anim]), f)
    return np.linalg.inv(W['TopN']) @ W['ThrowN']


def extra_bone(bn):
    """os de Brawl absents de la table : doigts (L1stNa, R2ndNc, LThumbNb…) -> main, os des yeux / du visage -> tête"""
    import re as _re
    if _re.match(r'^[LR](1st|2nd|3rd|4th|Thumb)N', bn):
        return 'handl' if bn[0] == 'L' else 'handr'
    if _re.search(r'(?i)eye|face|mouth|jaw', bn):
        return 'head'
    if bn == 'SphereN':  # Super Shadow : boule de la toupie -> centre du corps
        return 'hip'
    return None


def plain(path):
    """fichier .pac éventuellement compressé (LZ77 0x10 / 0x11) -> chemin d'un fichier ARC normal"""
    d = open(path, 'rb').read()
    if d[:4] == b'ARC\0':
        return path
    dd = decompress(d)
    t = tempfile.NamedTemporaryFile(delete=False, suffix='.pac')
    t.write(dd)
    t.close()
    print('decompresse', os.path.basename(path), len(d), '->', len(dd))
    return t.name


def main():
    moveset, motion, model, out = sys.argv[1:5]
    bones = bone_names(plain(model))
    print('squelette :', len(bones), 'os ;', ' '.join(bones[i] for i in sorted(bones)))
    lens = anim_lengths(plain(motion))
    if not lens:
        # Super Sonic : son Motion.pac contient la brres directement (pas d'ARC imbriqué comme Super Shadow) : sans ça
        # toutes les durées valaient 0 (coups des smashs posés sans le décalage de leur « Start » : trop tôt)
        from gen import arc_entries, brres_files, chr0_info
        dd_ = open(plain(motion), 'rb').read()
        for _t, _i, _s, _r, data in arc_entries(dd_):
            if data[:4] == b'bres':
                for folder, items in brres_files(data).items():
                    if 'Chr' in folder:
                        for n_, o_ in items:
                            lens[n_] = chr0_info(data, o_)[1]
        print('durées des animations (brres directe) :', len(lens))
    m = load(plain(moveset))
    base, subs = subactions(m)
    byname = {}
    for i, n, mo in subs:
        if n and mo and n not in byname and any(k == 'hit' for _, k, _ in timeline(m, base, mo)):
            byname[n] = mo
    # toutes les sous-actions nommées (les chopes n'ont pas de coups)
    byname_all = {}
    for i, n, mo in subs:
        if n and mo and n not in byname_all:
            byname_all[n] = mo
    unmapped = set()
    lines = ["// généré pendant la compilation à partir du build REX (ne pas publier)", "pub static MOVES: &[Mv] = &["]
    for script, sub, kind in MAP:
        mo = byname.get(sub)
        if not mo and sub in ('AttackS4Hi', 'AttackS4Lw') and byname.get('AttackS4S'):
            # v0.4 : pas de smash avant incliné chez lui : le même coup que le smash avant droit (même animation)
            print('absent :', sub, '-> AttackS4S')
            sub = 'AttackS4S'
            mo = byname.get(sub)
        if not mo:
            print('absent ou sans coup :', sub)
            continue
        tl = timeline(m, base, mo)
        start = float(sum(lens.get(_a, 0) for _a in START[sub].split('+'))) if sub in START else 0.0
        total = float(lens.get(sub, 0)) + start
        if script in PLACE:
            # v0.11 : coups d'une sous-action posés sur l'animation d'une autre (descente de l'attaque sautée)
            start, total = PLACE[script][0], float(lens.get(PLACE[script][1], 0))
        evs = []
        for f, k, p in tl:
            f += start
            if k == 'clear':
                evs.append("Ev { f: %.1f, clear: true, ..E }" % f)
            elif k == 'hit':
                v = [pv(t, x) for t, x in p]
                raw0 = p[0][1] & 0xffffffff
                bn = bones.get(raw0 >> 16, '')
                bone = BONES.get(bn) or extra_bone(bn)
                if bone is None:
                    unmapped.add(bn)
                    bone = 'top'
                hid = raw0 & 0xffff
                kbg = p[3][1] & 0xffff
                wdsk = (p[3][1] >> 16) & 0xffff
                bkb = p[4][1] & 0xffff
                attr = ELEM.get((p[12][1] & 0xffffffff) & 0x1f, 'collision_attr_normal')
                bz, by, bx = v[6], v[7], v[8]
                if bn in HIT_KEYS:
                    # Super Shadow : décalage dans le repère de l'os de Brawl (X, Y, Z = v[8], v[7], v[6]) -> repère de
                    # l'os d'Ultimate associé (hit_keys.py) ; sinon les coups des membres partaient de côté
                    _R = HIT_KEYS[bn]['R']
                    _v = (v[8], v[7], v[6])
                    bz, by, bx = (sum(_R[k][j] * _v[j] for j in range(3)) for k in range(3))
                    bone = HIT_KEYS[bn]['u']
                dmg = v[1] if 0 <= v[1] < 200 else 0.0
                cap = ''
                if CANON and bn == 'ThrowN':
                    M = canon_pose(model, motion, sub, f - start)
                    if M is not None:
                        var = [p[k][0] == 5 for k in (6, 7, 8)]
                        o_ = [0.0 if var[k] else v[6 + k] for k in range(3)]
                        a_ = M[:3, :3] @ o_ + M[:3, 3]
                        bone, bz, by, bx = 'top', a_[0], a_[1], a_[2]
                        if any(var):
                            d_ = M[:3, :3] @ [1.0 if var[k] else 0.0 for k in range(3)]
                            L = CANON.get(sub, CANON.get('*', 120.0))
                            cap = ', cap: true, x2: %.3f, y2: %.3f, z2: %.3f' % tuple(a_ + d_ * L)
                evs.append("Ev { f: %.1f, id: %d, bone: \"%s\", dmg: %.2f, angle: %d, kbg: %d, fkb: %d, bkb: %d, size: %.3f, x: %.3f, y: %.3f, z: %.3f, hitlag: %.3f, sdi: %.3f, trip: %.3f, attr: \"%s\"%s, ..E }"
                           % (f, min(hid, 7), bone, dmg, int(v[2]) & 0xffff, kbg, wdsk, bkb, v[5], bz, by, bx,
                              v[10] if v[10] > 0 else 1.0, v[11] if v[11] > 0 else 1.0, v[9] / 100.0, attr, cap))
        if os.environ.get('RATE') == '1':
            # Super Shadow : « Frame Speed Modifier » de REX (PSA 04070100) : l'animation ET le script vont plus vite
            # (f tilt x1,5, fair x2,7…) ; ignoré jusqu'ici, ses coups étaient trop lents. Une nouvelle sous-action
            # repart à x1 : après le début d'un smash (sous-action « Start »), vitesse remise à 1 si besoin.
            def _fsm(mo_, off_):
                _f, out_ = 0.0, []
                for _c, _p in _gen.events(m, base, mo_):
                    if _c == 0x00010100:
                        _f += pv(*_p[0])
                    elif _c == 0x00020100:
                        _f = pv(*_p[0])
                    elif _c == 0x04070100 and _p:
                        out_.append((_f + off_, pv(*_p[0])))
                return out_
            _r = []
            if sub in START and (byname.get(START[sub]) or byname_all.get(START[sub])):
                _r += _fsm(byname.get(START[sub]) or byname_all.get(START[sub]), 0.0)
            _main = _fsm(mo, start)
            if _r and start > 0 and not any(abs(f_ - start) < 0.01 for f_, _ in _main):
                _r.append((start, 1.0))
            _r += _main
            evs += ["Ev { f: %.1f, rate: %.3f, ..E }" % (f_, v_) for f_, v_ in _r if v_ > 0]
            evs.sort(key=lambda e_: float(e_.split('f: ')[1].split(',')[0]))
        if os.environ.get('BODY') == '1':
            # Super Shadow : corps invincible / intangible pendant le coup (PSA 06050100 : 0 normal, 1 invincible,
            # 2 intangible), oublié jusqu'ici ; images comptées comme timeline() (attente 00010100, image 00020100)
            _f = 0.0
            for _c, _p in _gen.events(m, base, mo):
                if _c == 0x00010100:
                    _f += pv(*_p[0])
                elif _c == 0x00020100:
                    _f = pv(*_p[0])
                elif _c == 0x06050100 and _p:
                    evs.append("Ev { f: %.1f, body: %d, ..E }" % (_f + start, int(_p[0][1])))
            evs.sort(key=lambda e_: float(e_.split('f: ')[1].split(',')[0]))
        # Super Shadow (demandé) : hitbox agrandies pour certains coups (HIT_SCALE='game_attacklw3=1.3,...')
        _hs = dict(kv.split('=') for kv in os.environ.get('HIT_SCALE', '').split(',') if '=' in kv)
        if script in _hs:
            import re as _re2
            evs = [_re2.sub(r'size: ([0-9.]+)', lambda m_: 'size: %.3f' % (float(m_.group(1)) * float(_hs[script])), e) for e in evs]
        # v0.10 (demandé) : smashs aussi forts que ceux du perso de base (SMASH_MAX='game_attacks4=18,...') : les dégâts
        # de REX sont mis à l'échelle pour que le coup le plus fort atteigne ce maximum
        _mx = dict(kv.split('=') for kv in os.environ.get('SMASH_MAX', '').split(',') if '=' in kv)
        if kind == 1 and script in _mx:
            import re as _re
            _d = [float(x) for x in _re.findall(r'dmg: ([0-9.]+)', ' '.join(evs))]
            if _d and 0 < max(_d) < float(_mx[script]):  # seulement plus fort, jamais moins
                _k = float(_mx[script]) / max(_d)
                evs = [_re.sub(r'dmg: ([0-9.]+)', lambda m_: 'dmg: %.2f' % (float(m_.group(1)) * _k), e) for e in evs]
                print('%s : degats x%.2f (max %.1f -> %s)' % (script, _k, max(_d), _mx[script]))
        lines.append("    Mv { script: \"%s\", total: %.1f, hold: %.1f, kind: %d, evs: &[\n        %s,\n    ] },"
                     % (script, total, start, kind, ",\n        ".join(evs)))
        print("%-18s <- %-12s %5.1f images, %d evenements" % (script, sub, total, len(evs)))
    lines.append("];")
    # v0.18 : prises de REX (chopes, coup pendant la prise, lancers) : même générateur que Black Shadow
    import gen as _G
    lines.append("pub static SUBS: &[Sub] = &[")
    for name in GRAB_SUBS:
        mo = byname_all.get(name)
        if not mo:
            print('prise absente :', name)
            continue
        lines.append(_G.sub_lines(name, mo, m, base, bones, float(lens.get(name, 0))))
    lines.append("];")
    if unmapped:
        print('OS SANS CORRESPONDANCE (coups posés sur top) :', sorted(unmapped))
    open(out, 'w').write("\n".join(lines) + "\n")


if __name__ == '__main__':
    main()
