"""Convertit les animations Brawl de Black Shadow (CHR0) en animations Ultimate (.nuanmb) sur son squelette.

usage : python3 conv_anim.py <dossier du mod (recup.py)> <dossier BlackShadow des assets> <sortie>

Méthode (celle de l'ancien conv.py, reconstruite) :
- pose monde de chaque os Brawl à chaque image ;
- monde Ultimate = monde Brawl @ OFF, avec OFF = inv(repos Brawl) @ repos Ultimate pour chaque os associé ;
- locale = inv(monde du parent) @ monde ; seuls Trans, Rot, Hip et Throw gardent leur translation animée,
  les autres gardent la longueur d'os d'Ultimate ;
- os rangés par ordre alphabétique (sinon le jeu ne trouve pas Trans).
Le script vérifie aussi sa conversion sur des animations déjà validées en jeu (v0.6.3) et dessine une planche.
"""
import json
import math
import os
import sys

import numpy as np
import ssbh_data_py as S

sys.path.insert(0, os.path.dirname(__file__))
import brawl_anim as R  # noqa: E402

A = S.anim_data
MOD, ASSETS, OUT = sys.argv[1:4]
SKEL = os.path.join(ASSETS, 'fighter/ganon/model/body/c00/model.nusktb')
REF = os.path.join(ASSETS, 'fighter/ganon/motion/body/c00')

FULL = {'Trans', 'Rot', 'Hip', 'Throw'}
SKIP_PREFIX = ('FaceN', 'Mouth', 'Jaw', 'H_')
# pas de quart de tour : dans Ultimate aussi, il avance le long de +Z dans l'animation (le quart de tour de la
# v0.4.1 était faux et a été retiré en v0.5.1 ; vérifié : les animations validées en jeu n'en ont pas)
YAW = np.eye(4)

# os Ultimate <- os Brawl (le reste est associé par position)
CORE = {
    'Trans': 'TransN', 'Rot': 'YRotN', 'Hip': 'HipN', 'Waist': 'WaistN', 'Bust': 'BustN', 'Neck': 'NeckN',
    'Head': 'HeadN', 'ClavicleL': 'LShoulderN', 'ShoulderL': 'LShoulderJ', 'ArmL': 'LArmJ', 'HandL': 'LHandN',
    'HaveL': 'LHaveN', 'ClavicleR': 'RShoulderN', 'ShoulderR': 'RShoulderJ', 'ArmR': 'RArmJ', 'HandR': 'RHandN',
    'HaveR': 'RHaveN', 'LegL': 'LLegJ', 'KneeL': 'LKneeJ', 'FootL': 'LFootJ', 'ToeL': 'LToeN', 'LegR': 'RLegJ',
    'KneeR': 'RKneeJ', 'FootR': 'RFootJ', 'ToeR': 'RToeN', 'Throw': 'ThrowN',
}
# doigts : 1er à 4e doigt puis le pouce (5), chacun en deux phalanges (a, b)
for _s in 'LR':
    for _n, _b in ((1, '1st'), (2, '2nd'), (3, '3rd'), (4, '4th'), (5, 'Thumb')):
        CORE['Finger%s%d1' % (_s, _n)] = '%s%sNa' % (_s, _b)
        CORE['Finger%s%d2' % (_s, _n)] = '%s%sNb' % (_s, _b)

# fichier Ultimate <- animation Brawl (même nom que le fichier de Ganondorf qu'il remplace)
JOBS = [
    ('e00catch.nuanmb', 'Catch'), ('e00catchdash.nuanmb', 'CatchDash'), ('e00catchturn.nuanmb', 'CatchTurn'),
    ('e00catchwait.nuanmb', 'CatchWait'), ('e00catchattack.nuanmb', 'CatchAttack'), ('e00catchcut.nuanmb', 'CatchCut'),
    ('e01throwf.nuanmb', 'ThrowF'), ('e01throwb.nuanmb', 'ThrowB'), ('e01throwhi.nuanmb', 'ThrowHi'),
    ('e01throwlw.nuanmb', 'ThrowLw'),
    # provocations (Ultimate n'a que la version « r » pour Ganondorf)
    ('j01appealhir.nuanmb', 'AppealHiR|AppealHiL|AppealHi'), ('j01appealsr.nuanmb', 'AppealSR|AppealSL|AppealS'),
    ('j01appeallwr.nuanmb', 'AppealLwR|AppealLwL|AppealLw'),
    # écran de fin : victoires (pose puis boucle) et défaite (applaudit)
    ('j02win1.nuanmb', 'Win1'), ('j02win1wait.nuanmb', 'Win1Wait|Win1'), ('j02win2.nuanmb', 'Win2'),
    ('j02win2wait.nuanmb', 'Win2Wait|Win2'), ('j02win3.nuanmb', 'Win3'), ('j02win3wait.nuanmb', 'Win3Wait|Win3'),
    ('j02lose.nuanmb', 'Lose'),
]
# contrôle : animations déjà converties et validées en jeu
CHECKS = [('c00attack11.nuanmb', 'Attack11'), ('c01attacks3s.nuanmb', 'AttackS3S'), ('c05attackairn.nuanmb', 'AttackAirN'),
          ('a00wait1.nuanmb', 'Wait1')]


def col(m):
    return np.array(m, dtype=float).T


def quat_from(m3):
    m = m3
    t = m[0, 0] + m[1, 1] + m[2, 2]
    if t > 0:
        s = math.sqrt(t + 1.0) * 2
        w, x, y, z = 0.25 * s, (m[2, 1] - m[1, 2]) / s, (m[0, 2] - m[2, 0]) / s, (m[1, 0] - m[0, 1]) / s
    elif m[0, 0] > m[1, 1] and m[0, 0] > m[2, 2]:
        s = math.sqrt(1.0 + m[0, 0] - m[1, 1] - m[2, 2]) * 2
        w, x, y, z = (m[2, 1] - m[1, 2]) / s, 0.25 * s, (m[0, 1] + m[1, 0]) / s, (m[0, 2] + m[2, 0]) / s
    elif m[1, 1] > m[2, 2]:
        s = math.sqrt(1.0 + m[1, 1] - m[0, 0] - m[2, 2]) * 2
        w, x, y, z = (m[0, 2] - m[2, 0]) / s, (m[0, 1] + m[1, 0]) / s, 0.25 * s, (m[1, 2] + m[2, 1]) / s
    else:
        s = math.sqrt(1.0 + m[2, 2] - m[0, 0] - m[1, 1]) * 2
        w, x, y, z = (m[1, 0] - m[0, 1]) / s, (m[0, 2] + m[2, 0]) / s, (m[1, 2] + m[2, 1]) / s, 0.25 * s
    q = np.array([x, y, z, w])
    return q / np.linalg.norm(q)


def mat_from(sc, q, tr):
    x, y, z, w = q
    r = np.array([[1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
                  [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
                  [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)]])
    m = np.eye(4)
    m[:3, :3] = r @ np.diag(sc)
    m[:3, 3] = tr
    return m


def decompose(m):
    sc = np.linalg.norm(m[:3, :3], axis=0)
    r = m[:3, :3] / np.where(sc == 0, 1, sc)
    return sc, quat_from(r), m[:3, 3].copy()


# ------------------------------------------------------------------ données

bones = R.read_bones(os.path.join(MOD, 'FitBlackShadow00.pac'))
anims = R.read_anims(os.path.join(MOD, 'FitBlackShadowMotionEtc.pac'))
sk = S.skel_data.read_skel(SKEL)
UB = sk.bones
UN = [b.name for b in UB]
PAR = [b.parent_index for b in UB]
L0 = [col(b.transform) for b in UB]
W0 = [col(sk.calculate_world_transform(b)) for b in UB]
BW0 = R.world_pose(bones, None, 0)

report = {'brawl_bones': {}, 'ultimate_bones': {}, 'map': {}, 'checks': {}, 'jobs': {}, 'decode': {}}
for n, b in bones.items():
    file_w = b.world
    calc_w = BW0[n]
    report['brawl_bones'][n] = {'parent': b.parent, 'pos': [round(float(x), 2) for x in (YAW @ calc_w)[:3, 3]],
                                'err_vs_file': round(float(np.abs(file_w - calc_w).max()), 4)}
for i, n in enumerate(UN):
    report['ultimate_bones'][n] = {'parent': UN[PAR[i]] if PAR[i] is not None and PAR[i] >= 0 else None,
                                   'pos': [round(float(x), 2) for x in W0[i][:3, 3]]}

# association des os : noyau par nom, le reste par position.
# Au repos, les deux squelettes sont identiques SANS le quart de tour (le quart de tour ne sert qu'à l'animation).
MAP = {}
bpos = {n: BW0[n][:3, 3] for n in bones}
used = set()
for u, b in CORE.items():
    if u in UN and b in bones:
        MAP[u] = b
        used.add(b)
for i, u in enumerate(UN):
    if u in MAP or u.startswith(SKIP_PREFIX):
        continue
    best, bd = None, 1e9
    for b, p in bpos.items():
        if b in used:
            continue
        dist = float(np.linalg.norm(p - W0[i][:3, 3]))
        if dist < bd:
            best, bd = b, dist
    if best is not None and bd < 0.6:
        MAP[u] = best
        used.add(best)
for u, b in MAP.items():
    i = UN.index(u)
    report['map'][u] = {'brawl': b, 'dist': round(float(np.linalg.norm(bpos[b] - W0[i][:3, 3])), 3)}
OFF = {u: np.linalg.inv(BW0[b]) @ W0[UN.index(u)] for u, b in MAP.items()}

ORDER = []  # parents avant enfants
seen = set()


def visit(i):
    if i in seen:
        return
    if PAR[i] is not None and PAR[i] >= 0:
        visit(PAR[i])
    seen.add(i)
    ORDER.append(i)


for i in range(len(UB)):
    visit(i)


def convert_frames(names):
    """mondes et locales Ultimate pour chaque image des animations Brawl mises bout à bout"""
    worlds, locals_ = [], []
    for nm in names:
        an = anims[nm]
        for k in range(max(1, an['frames'])):
            BW = R.world_pose(bones, an, float(k))
            W = [None] * len(UB)
            L = [None] * len(UB)
            for i in ORDER:
                u = UN[i]
                pw = W[PAR[i]] if PAR[i] is not None and PAR[i] >= 0 else np.eye(4)
                if u in MAP:
                    wt = YAW @ BW[MAP[u]] @ OFF[u]
                    loc = np.linalg.inv(pw) @ wt
                    if u not in FULL:
                        loc[:3, 3] = L0[i][:3, 3]
                else:
                    loc = L0[i].copy()
                L[i] = loc
                W[i] = pw @ loc
            worlds.append(W)
            locals_.append(L)
    return worlds, locals_


def build_anim(locals_, loop, ref_names=None):
    n = len(locals_)
    prev = [None] * len(UB)
    vals = [[] for _ in UB]
    for k in range(n):
        for i in range(len(UB)):
            sc, q, tr = decompose(locals_[k][i])
            if prev[i] is not None and np.dot(prev[i], q) < 0:
                q = -q
            if prev[i] is None and q[3] < 0:
                q = -q
            prev[i] = q
            vals[i].append(A.Transform([float(x) for x in sc], [float(x) for x in q], [float(x) for x in tr]))
    if loop and n > 2:
        for i in range(len(UB)):
            v0, vl = vals[i][0], vals[i][-1]
            q = np.array(v0.rotation)
            if np.dot(q, np.array(vl.rotation)) < 0:
                q = -q
            vals[i].append(A.Transform(list(v0.scale), [float(x) for x in q], list(v0.translation)))
        n += 1
    nodes = []
    keep = set(ref_names) if ref_names else None
    for i, b in sorted(enumerate(UB), key=lambda x: x[1].name):
        if keep is not None:
            if b.name not in keep:
                continue
        elif b.name.startswith(SKIP_PREFIX):
            continue
        tr = A.TrackData('Transform')
        tr.values = vals[i]
        nd = A.NodeData(b.name)
        nd.tracks = [tr]
        nodes.append(nd)
    g = A.GroupData(A.GroupType.Transform)
    g.nodes = nodes
    a = A.AnimData()
    a.major_version, a.minor_version = 2, 0
    a.final_frame_index = float(n - 1)
    a.groups = [g]
    return a


def ref_worlds(path):
    """mondes Ultimate d'une animation .nuanmb existante, image par image"""
    a = A.read_anim(path)
    tracks = {}
    for g in a.groups:
        if 'Transform' in str(g.group_type) or getattr(g.group_type, 'name', '') == 'Transform':
            for nd in g.nodes:
                tracks[nd.name] = nd.tracks[0].values
    n = int(a.final_frame_index) + 1
    out = []
    for k in range(n):
        W = [None] * len(UB)
        for i in ORDER:
            u = UN[i]
            pw = W[PAR[i]] if PAR[i] is not None and PAR[i] >= 0 else np.eye(4)
            if u in tracks:
                v = tracks[u][min(k, len(tracks[u]) - 1)]
                loc = mat_from(v.scale, v.rotation, v.translation)
            else:
                loc = L0[i]
            W[i] = pw @ loc
        out.append(W)
    return out, sorted(tracks), n


# ------------------------------------------------------------------ contrôle sur des animations validées

ref_node_names = None
for fn, nm in CHECKS:
    p = os.path.join(REF, fn)
    if not os.path.exists(p) or nm not in anims:
        report['checks'][fn] = 'absent'
        continue
    rw, rnames, rn = ref_worlds(p)
    ref_node_names = ref_node_names or rnames
    mw, _ = convert_frames([nm])
    n = min(len(mw), rn)
    errs = {}
    for i, u in enumerate(UN):
        if u.startswith(SKIP_PREFIX):
            continue
        e = max(float(np.linalg.norm(mw[k][i][:3, 3] - rw[k][i][:3, 3])) for k in range(n))
        errs[u] = round(e, 3)
    worst = sorted(errs.items(), key=lambda x: -x[1])[:8]
    report['checks'][fn] = {'frames_ref': rn, 'frames_mine': len(mw), 'max_err': worst[0][1] if worst else None,
                            'worst': worst, 'all': errs}
    print('CONTROLE %-22s images ref %3d / moi %3d  ecart max %.3f %s' % (fn, rn, len(mw), worst[0][1] if worst else -1, worst[:4]))
report['ref_nodes'] = ref_node_names

# diagnostic détaillé sur la première animation de contrôle
p = os.path.join(REF, CHECKS[0][0])
if os.path.exists(p):
    rw, _, rn = ref_worlds(p)
    mw, ml = convert_frames([CHECKS[0][1]])
    ra = A.read_anim(p)
    rt = {}
    for g in ra.groups:
        for nd in g.nodes:
            rt[nd.name] = nd.tracks[0].values
    for k in (0, 10):
        for u in ('Trans', 'Rot', 'Hip', 'Head', 'HandL', 'FootL', 'Throw'):
            i = UN.index(u)
            v = rt.get(u, [None])[min(k, len(rt.get(u, [0])) - 1)] if u in rt else None
            sc, q, tr = decompose(ml[k][i])
            print('DIAG f%d %-6s ref %s moi %s | ref loc t=%s q=%s | moi loc t=%s q=%s' % (
                k, u, np.round(rw[k][i][:3, 3], 2), np.round(mw[k][i][:3, 3], 2),
                np.round(v.translation, 2) if v else None, np.round(v.rotation, 3) if v else None,
                np.round(tr, 2), np.round(q, 3)))
    an = anims[CHECKS[0][1]]
    for bn in ('TransN', 'XRotN', 'YRotN', 'HipN', 'TopN'):
        ba = an['bones'].get(bn)
        if ba is not None:
            print('DIAG brawl %-6s code %08X srt0 %s srt10 %s' % (bn, ba.code, [np.round(x, 2) for x in ba.srt(0, bones[bn])],
                                                                 [np.round(x, 2) for x in ba.srt(10, bones[bn])]))
        else:
            print('DIAG brawl %-6s absent de l\'animation' % bn)

# ------------------------------------------------------------------ conversion
os.makedirs(os.path.join(OUT, 'BlackShadow/fighter/ganon/motion/body/c00'), exist_ok=True)
POSES = {}
# v0.16 : toutes les autres animations de Ganondorf qui ont leur équivalent dans le mod (même nom, sans le
# préfixe « a01 » etc. ; les versions l/r d'Ultimate reprennent l'animation unique de Brawl) — sauf celles déjà
# converties et validées (bs-assets) et celles des listes ci-dessus
_done = set(os.listdir(os.path.join(ASSETS, 'fighter/ganon/motion/body/c00'))) | {j[0] for j in JOBS}
_low = {k.lower(): k for k in anims}
for _fn in open(os.path.join(os.path.dirname(__file__), 'ganon_motions.txt')).read().split():
    if _fn in _done or _fn[0] in 'cd':
        continue
    _base = _fn[3:-7]
    _cands = [_base] + ([_base[:-1]] if _base[-1:] in ('l', 'r') else [])
    _hit = next((_low[c] for c in _cands if c in _low), None)
    if _hit:
        JOBS.append((_fn, _hit))
        print('AUTO', _fn, '<-', _hit)
    else:
        print('SANS', _fn)
print('provocations du mod :', sorted(k for k in anims if 'appeal' in k.lower()))
print('victoire du mod :', sorted(k for k in anims if 'win' in k.lower() or 'lose' in k.lower()))
for fn, nm in JOBS:
    nm = next((n for n in nm.split('|') if n in anims), nm.split('|')[0])
    if nm not in anims:
        print('absent', nm)
        report['jobs'][fn] = 'absent'
        continue
    an = anims[nm]
    report['decode'][nm] = {str(k): v for k, v in an['stats'].items()}
    w, loc = convert_frames([nm])
    a = build_anim(loc, an['loop'], ref_node_names)
    a.save(os.path.join(OUT, 'BlackShadow/fighter/ganon/motion/body/c00', fn))
    POSES[nm] = w
    report['jobs'][fn] = {'brawl': nm, 'frames': len(loc), 'loop': an['loop']}
    print('%-22s <- %-12s %3d images%s' % (fn, nm, len(loc), ' (boucle)' if an['loop'] else ''))

json.dump(report, open(os.path.join(OUT, 'rapport.json'), 'w'), indent=1, ensure_ascii=False)

# ------------------------------------------------------------------ planche
import render  # noqa: E402

render.sheet(ASSETS, sk, UN, W0, POSES, os.path.join(OUT, 'planche_lancers.png'))
print('planche ok')
