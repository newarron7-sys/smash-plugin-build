"""Zero (REX) : animations Brawl (CHR0 de FitZeroMotion.pac) -> fichiers .nuanmb de Roy, sur le squelette de Zero.

Pas de reciblage : le squelette Ultimate de Zero est le sien (mêmes os, mêmes repos), chaque os reçoit son SRT
Brawl tel quel (Trans <- TransN, Rot <- YRotN, etc.). Les objets cachés par défaut (buster, yeux jaunes, sabre
dans le dos) le sont avec une piste de visibilité dans chaque animation.

usage : python3 conv_anims.py <paquet extrait> <sortie de build_model.py> <dossier des .nuanmb> <groupe>
groupes : base (déplacements, attaques, prises, bord, provocations, victoire), puis degats, objets, divers
"""
import json, math, os, re, sys
import numpy as np
import ssbh_data_py as S

HERE = os.path.dirname(os.path.abspath(__file__))
for p in ('../../black-shadow/tools', '../../black-shadow/tools/anim', '.'):
    sys.path.insert(0, os.path.join(HERE, p))
from arc import arc_entries  # noqa: E402
from brres import brres_files  # noqa: E402
import brawl_anim  # noqa: E402
from lz import decompress  # noqa: E402

PKG, MOUT, DST, GROUP = sys.argv[1:5]
os.makedirs(DST, exist_ok=True)
A = S.anim_data
info = json.load(open(os.path.join(MOUT, 'model.json')))
U2B = info['u2b']
sk = S.skel_data.read_skel(os.path.join(MOUT, 'model', 'model.nusktb'))
UB = sk.bones
tmp = '/tmp/zero_anim_model.pac'
open(tmp, 'wb').write(decompress(open(os.path.join(PKG, 'Costumes/PacFiles/0000/FitZero00.pac'), 'rb').read()))
BR = brawl_anim.read_bones(tmp)

md = decompress(open(os.path.join(PKG, 'PacFiles/FitZeroMotion.pac'), 'rb').read())
ANIMS = {}
for e in arc_entries(md):
    if e[4][:4] == b'bres':
        for folder, items in brres_files(e[4]).items():
            if 'Chr' in folder:
                for n, o in items:
                    ANIMS.setdefault(n, (e[4], o))
LOW = {k.lower(): k for k in ANIMS}

# fichier de Roy -> animation(s) de Brawl (plusieurs = mises bout à bout)
SPECIAL = {
    'c03attacks4s': ['AttackS4Start', 'AttackS4S'], 'c03attacks4charge': ['AttackS4Hold'],
    'c04attackhi4': ['AttackHi4Start', 'AttackHi4'], 'c04attackhi4charge': ['AttackHi4Hold'],
    'c04attacklw4': ['AttackLw4Start', 'AttackLw4'], 'c04attacklw4charge': ['AttackLw4Hold'],
    'g02cliffattack': ['CliffAttackQuick'], 'g02cliffclimb': ['CliffClimbQuick'],
    'g02cliffescape': ['CliffEscapeQuick'], 'g02cliffjump1': ['CliffJumpQuick1'], 'g02cliffjump2': ['CliffJumpQuick2'],
    'j01appealhil': ['AppealHiL'], 'j01appealhir': ['AppealHiR'], 'j01appealsl': ['AppealS'], 'j01appealsr': ['AppealS'],
    'j01appeallwl': ['AppealLw'], 'j01appeallwr': ['AppealLw'],
    # v0.6 : sans équivalent direct dans Brawl -> le plus proche (sinon l'animation de Roy le tordait au sol)
    'a05landinglight': ['LandingHeavy'], 'a02turndash': ['Dash'], 'a04runfalll': ['Fall'], 'a04runfallr': ['Fall'],
    'a04walkfalll': ['Fall'], 'a04walkfallr': ['Fall'], 'b00justshieldoff': ['GuardOff'], 'e00catchpull': ['CatchWait'],
}
GROUPS = {
    'base': r'^(a0[1-6]|a00wait|b0|c0|e00|e01throw[bfhl]|g0[12]|j00|j01|j02)',
    'degats': r'^(f0|e01thrown|e0[23])',
    'divers': r'^(g00|g03|i0|k0)',
    'objets': r'^(h|a00item)',
}
roy = open(os.path.join(HERE, 'roy_motions.txt')).read().split()
jobs = []
for fn in roy:
    base = fn[:-len('.nuanmb')]
    if not re.match(GROUPS[GROUP], base):
        continue
    if base in SPECIAL:
        src = [s for s in SPECIAL[base] if s in ANIMS]
    else:
        nm = base[3:]
        cands = [nm] + ([nm[:-1]] if nm[-1:] in ('l', 'r') else [])
        hit = next((LOW[c] for c in cands if c in LOW), None)
        src = [hit] if hit else []
    if src:
        jobs.append((fn, src))
    else:
        print('SANS', fn)

# objets cachés par défaut (visibilité)
VIS = info.get('visibilite', {})
HIDDEN = set()   # v0.5 : objets à cacher retirés du modèle, plus de pistes de visibilité


def quat(m):
    r = m[:3, :3]
    sc = np.linalg.norm(r, axis=0)
    r = r / np.where(sc > 1e-9, sc, 1)
    t = np.trace(r)
    if t > 0:
        s = math.sqrt(t + 1.0) * 2
        q = [(r[2, 1] - r[1, 2]) / s, (r[0, 2] - r[2, 0]) / s, (r[1, 0] - r[0, 1]) / s, 0.25 * s]
    elif r[0, 0] > r[1, 1] and r[0, 0] > r[2, 2]:
        s = math.sqrt(1.0 + r[0, 0] - r[1, 1] - r[2, 2]) * 2
        q = [0.25 * s, (r[0, 1] + r[1, 0]) / s, (r[0, 2] + r[2, 0]) / s, (r[2, 1] - r[1, 2]) / s]
    elif r[1, 1] > r[2, 2]:
        s = math.sqrt(1.0 + r[1, 1] - r[0, 0] - r[2, 2]) * 2
        q = [(r[0, 1] + r[1, 0]) / s, 0.25 * s, (r[1, 2] + r[2, 1]) / s, (r[0, 2] - r[2, 0]) / s]
    else:
        s = math.sqrt(1.0 + r[2, 2] - r[0, 0] - r[1, 1]) * 2
        q = [(r[0, 2] + r[2, 0]) / s, (r[1, 2] + r[2, 1]) / s, 0.25 * s, (r[1, 0] - r[0, 1]) / s]
    return np.array(q), sc


def frames_of(src):
    out = []
    for name in src:
        d, o = ANIMS[name]
        an = brawl_anim.read_chr0(d, o)
        for f in range(max(1, an['frames'])):
            loc = []
            for b in UB:
                bn = U2B.get(b.name)
                if bn is None or bn not in BR:
                    loc.append(np.array(b.transform, dtype=float).T)
                    continue
                bb = BR[bn]
                ba = an['bones'].get(bn)
                s, r, t = ba.srt(f, bb) if ba is not None else (bb.s, bb.r, bb.t)
                loc.append(brawl_anim.srt_matrix(s, r, t))
            out.append(loc)
    return out


report = {}
for fn, src in jobs:
    fr = frames_of(src)
    n = len(fr)
    nodes = []
    for i, b in sorted(enumerate(UB), key=lambda x: x[1].name):
        vals = []
        prev = None
        for k in range(n):
            m = fr[k][i]
            q, sc = quat(m)
            if prev is not None and np.dot(prev, q) < 0:
                q = -q
            prev = q
            vals.append(A.Transform([float(x) for x in sc], [float(x) for x in q], [float(x) for x in m[:3, 3]]))
        tr = A.TrackData('Transform')
        tr.values = vals
        nd = A.NodeData(b.name)
        nd.tracks = [tr]
        nodes.append(nd)
    g = A.GroupData(A.GroupType.Transform)
    g.nodes = nodes
    groups = [g]
    if HIDDEN:
        # v0.4 : seulement les objets cachés (False) ; avec des pistes True pour tous, Zero devenait invisible en jeu
        vn = []
        for o in sorted(HIDDEN):
            t = A.TrackData('Visibility')
            t.values = [False] * n
            nd = A.NodeData(o)
            nd.tracks = [t]
            vn.append(nd)
        gv = A.GroupData(A.GroupType.Visibility)
        gv.nodes = vn
        groups.append(gv)
    a = A.AnimData()
    a.major_version, a.minor_version = 2, 0
    a.final_frame_index = float(max(n - 1, 1))
    if n == 1:
        for nd in nodes:
            nd.tracks[0].values = nd.tracks[0].values * 2
        for gg in groups[1:]:
            for nd in gg.nodes:
                nd.tracks[0].values = nd.tracks[0].values * 2
    a.groups = groups
    a.save(os.path.join(DST, fn))
    report[fn] = {'brawl': src, 'images': n}
    print('%-28s <- %-30s %4d images' % (fn, '+'.join(src), n))
# model.nuanmb (dossier du modèle) : vide comme celui de Roy (une piste de visibilité dedans le cachait au départ)
if GROUP == 'base':
    a = A.AnimData()
    a.major_version, a.minor_version = 2, 0
    a.final_frame_index = 1.0
    a.groups = []
    a.save(os.path.join(MOUT, 'model', 'model.nuanmb'))
    print('model.nuanmb vide (comme Roy)')
json.dump(report, open(os.path.join(DST, '..', 'anims_%s.json' % GROUP), 'w'), indent=1)
print('ANIMATIONS CONVERTIES', len(report), 'groupe', GROUP)
