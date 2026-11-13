"""Zero (REX) : ses animations P+Ex (CHR0, son squelette Brawl) converties pour le squelette de ROY d'Ultimate,
pour le skin de _Silent_ sur Roy c00 (même méthode que les animations de Black Shadow, black-shadow/tools/anim).

Méthode :
- pose monde de chaque os Brawl à chaque image ;
- MIROIR gauche/droite : Zero tient son sabre de la main GAUCHE, Roy (et le sabre du skin) de la DROITE.
  On échange les os L/R et on reflète la pose (M -> F M F, F = diag(-1, 1, 1)) : Zero devient droitier ;
- ÉCHELLE : les positions Brawl sont mises à la taille de Roy (hauteur des hanches) ;
- monde Ultimate = monde Brawl (miroir, échelle) @ OFF, avec OFF = inv(repos Brawl) @ repos Ultimate pour chaque os
  associé (au repos, les deux sont en T, le sabre de Zero et l'épée de Roy pointent tous les deux vers l'avant) ;
- locale = inv(monde du parent) @ monde ; seuls Trans, Rot, Hip et Throw gardent leur translation animée,
  les autres gardent la longueur d'os de Roy ; les os de Roy sans équivalent gardent leur pose de repos.

usage : python3 retarget_roy.py <paquet extrait> <model.nusktb de Roy> <sortie> <groupe>
groupes : attaques (normaux, smashs, aériens)
"""
import json, math, os, re, sys
import numpy as np
import ssbh_data_py as S

HERE = os.path.dirname(os.path.abspath(__file__))
for p in ('../../black-shadow/tools', '../../black-shadow/tools/anim', '.'):
    sys.path.insert(0, os.path.join(HERE, p))
from arc import arc_entries  # noqa: E402
from brres import brres_files  # noqa: E402
import brawl_anim as R  # noqa: E402
from lz import decompress  # noqa: E402

PKG, SKEL, OUT, GROUP = sys.argv[1:5]
SILENT = sys.argv[5] if len(sys.argv) > 5 else None
A = S.anim_data
os.makedirs(os.path.join(OUT, 'fighter/roy/motion/body/c00'), exist_ok=True)

# ------------------------------------------------------------------ données
tmp = '/tmp/retarget_zero_model.pac'
open(tmp, 'wb').write(decompress(open(os.path.join(PKG, 'Costumes/PacFiles/0000/FitZero00.pac'), 'rb').read()))
BR = R.read_bones(tmp)
md = decompress(open(os.path.join(PKG, 'PacFiles/FitZeroMotion.pac'), 'rb').read())
ANIMS = {}
for e in arc_entries(md):
    if e[4][:4] == b'bres':
        for folder, items in brres_files(e[4]).items():
            if 'Chr' in folder:
                for n, o in items:
                    if n not in ANIMS:
                        try:
                            ANIMS[n] = R.read_chr0(e[4], o)
                        except Exception as ex:  # noqa: BLE001
                            print('illisible', n, ex)
LOW = {k.lower(): k for k in ANIMS}

sk = S.skel_data.read_skel(SKEL)
UB = sk.bones
UN = [b.name for b in UB]
PAR = [b.parent_index for b in UB]


def col(m):
    return np.array(m, dtype=float).T


L0 = [col(b.transform) for b in UB]
W0 = [None] * len(UB)
ORDER = []
_seen = set()


def visit(i):
    if i in _seen:
        return
    if PAR[i] is not None and PAR[i] >= 0:
        visit(PAR[i])
    _seen.add(i)
    ORDER.append(i)
    W0[i] = (W0[PAR[i]] @ L0[i]) if PAR[i] is not None and PAR[i] >= 0 else L0[i]


for i in range(len(UB)):
    visit(i)

FULL = {'Trans', 'XRot', 'Rot', 'Hip', 'Throw'}
CORE = {
    'Trans': 'TransN', 'XRot': 'XRotN', 'Rot': 'YRotN', 'Hip': 'HipN', 'Waist': 'WaistN', 'Bust': 'BustN', 'Neck': 'NeckN',
    'Head': 'HeadN', 'ClavicleL': 'LShoulderN', 'ShoulderL': 'LShoulderJ', 'ArmL': 'LArmJ', 'HandL': 'LHandN',
    'HaveL': 'LHaveN', 'ClavicleR': 'RShoulderN', 'ShoulderR': 'RShoulderJ', 'ArmR': 'RArmJ', 'HandR': 'RHandN',
    'HaveR': 'RHaveN', 'LegL': 'LLegJ', 'KneeL': 'LKneeJ', 'FootL': 'LFootJ', 'ToeL': 'LToeN', 'LegR': 'RLegJ',
    'KneeR': 'RKneeJ', 'FootR': 'RFootJ', 'ToeR': 'RToeN', 'Throw': 'ThrowN',
}
MAP = {u: b for u, b in CORE.items() if u in UN and b in BR}
# le sabre : chez Roy, l'épée suit l'os « HaveR » (Sword1 lui est collé) ; chez Zero, le sabre a son propre os
# (SwordN, enfant de la main) qui tourne dans la main pour certains coups (prise à l'envers du down air...).
# HaveR suit donc SwordN (au repos, les deux sont identiques à LHaveN / HaveR : même décalage)
if 'SwordN' in BR and 'HaveR' in UN:
    MAP['HaveR'] = 'SwordN'
print('os associés', len(MAP), '/', len(CORE), 'manquants', sorted(set(CORE) - set(MAP)))

# ------------------------------------------------------------------ miroir et échelle
F = np.diag([-1.0, 1.0, 1.0, 1.0])


def swap(n):
    """LArmJ <-> RArmJ (os gauche/droite de Brawl)"""
    if re.match(r'^[LR][A-Z0-9]', n):
        o = ('R' if n[0] == 'L' else 'L') + n[1:]
        if o in BR:
            return o
    return n


BW0 = R.world_pose(BR, None, 0)
hip_b = BW0['HipN'][1, 3]
hip_u = W0[UN.index('Hip')][1, 3]
SCALE = hip_u / hip_b if abs(hip_b) > 1e-3 else 1.0
print('hanches Brawl %.2f / Roy %.2f -> échelle %.3f' % (hip_b, hip_u, SCALE))


MIRROR = [True]


def mirrored_scaled(BW):
    """pose monde Brawl -> pose monde « droitière » à la taille de Roy, par nom d'os.
    v0.3 des animations : miroir seulement si Zero tient le sabre de la main GAUCHE dans cette animation
    (dans certains coups, comme le up tilt, il le passe dans sa main droite : pas de miroir alors)"""
    out = {}
    for n in BR:
        m = (F @ BW[swap(n)] @ F) if MIRROR[0] else BW[n]
        m = m.copy()
        m[:3, 3] *= SCALE
        out[n] = m
    return out


def sword_hand(an):
    """part des images où la poignée du sabre est plus près de la main gauche que de la droite"""
    left, n = 0, 0
    for k in range(0, max(1, an['frames']), 2):
        W = R.world_pose(BR, an, float(k))
        s_ = W['SwordN'][:3, 3]
        dl = np.linalg.norm(s_ - W['LHandN'][:3, 3])
        dr = np.linalg.norm(s_ - W['RHandN'][:3, 3])
        left += dl <= dr
        n += 1
    return left / max(n, 1)


OFFS = {}
for _m in (True, False):
    MIRROR[0] = _m
    _mb0 = mirrored_scaled(BW0)
    OFFS[_m] = {u: np.linalg.inv(_mb0[b]) @ W0[UN.index(u)] for u, b in MAP.items()}
MIRROR[0] = True
ROOTS = {'Trans', 'Rot'}
LIFT = [0.0]
LIFTZ = [0.0]
TILT = [0.0]
STRIP = [False, None]
MB0 = {}
for _m in (True, False):
    MIRROR[0] = _m
    MB0[_m] = mirrored_scaled(BW0)
MIRROR[0] = True
for _u in ('Trans', 'XRot', 'Rot', 'Hip', 'Throw'):
    if _u in MAP and _u in UN:
        _p = PAR[UN.index(_u)]
        print('repos %s (parent %s) : Roy y=%.2f, Zero %s y=%.2f' % (_u, UN[_p] if _p is not None and _p >= 0 else '-',
              W0[UN.index(_u)][1, 3], MAP[_u], MB0[False][MAP[_u]][1, 3]))


# ------------------------------------------------------------------ v0.6 : main fermée sur le sabre
def axis_rot(ax, ang):
    ax = ax / np.linalg.norm(ax)
    c, s_ = math.cos(ang), math.sin(ang)
    x, y, z = ax
    return np.array([[c + x * x * (1 - c), x * y * (1 - c) - z * s_, x * z * (1 - c) + y * s_],
                     [y * x * (1 - c) + z * s_, c + y * y * (1 - c), y * z * (1 - c) - x * s_],
                     [z * x * (1 - c) - y * s_, z * y * (1 - c) + x * s_, c + z * z * (1 - c)]])


def pos0(n):
    return W0[UN.index(n)][:3, 3]


CURL = {}   # 'L' / 'R' -> {indice d'os: rotation locale fermée}
for side in 'LR':
    need = ['Finger%s%d%d' % (side, f, j) for f in (1, 2, 3, 4) for j in (1, 2)] + ['Finger%s52' % side]
    if not all(n in UN for n in need):
        print('doigts absents', side)
        continue
    a = pos0('Finger%s11' % side) - pos0('Finger%s41' % side)            # en travers de la paume
    d = pos0('Finger%s22' % side) - pos0('Finger%s21' % side)            # le long des doigts
    nrm = np.cross(a, d)
    if np.dot(nrm, pos0('Finger%s52' % side) - pos0('Finger%s21' % side)) < 0:
        nrm = -nrm                                                        # côté paume = côté du pouce
    rots = {}
    for f in (1, 2, 3, 4):
        for j, deg in ((1, 70.0), (2, 80.0)):
            i = UN.index('Finger%s%d%d' % (side, f, j))
            tip = 'Finger%s%d%d' % (side, f, j + 1) if j == 1 else 'Finger%s%d3' % (side, f)
            best = None
            for sg in (1.0, -1.0):
                R3 = axis_rot(a, sg * math.radians(deg))
                if tip in UN:
                    v = pos0(tip) - pos0(UN[i])
                    gain = float(np.dot(R3 @ v - v, nrm))
                else:
                    gain = sg
                if best is None or gain > best[0]:
                    best = (gain, R3)
            Wr = W0[i][:3, :3]
            loc = L0[i].copy()
            # rotation autour de l'axe (monde, au repos) ramenée dans le repère local de l'os
            loc[:3, :3] = L0[i][:3, :3] @ (np.linalg.inv(Wr) @ best[1] @ Wr)
            rots[i] = loc
    CURL[side] = rots
print('doigts repliés', {k: len(v) for k, v in CURL.items()})


# l'épée de Roy : axe +Z de l'os Sword1 (enfant de HaveR, avec sa propre rotation de repos)
I_SW = UN.index('Sword1') if 'Sword1' in UN else None
I_HV = UN.index('HaveR')
I_SW2 = UN.index('Sword2') if 'Sword2' in UN else None
# v0.9 : lame du skin à la longueur de celle de Zero dans P+. Longueur de la lame de _Silent_ (maillage WeaponbladeM,
# porté par Sword2) mesurée le long de son axe principal dans le repère de Sword2
BLADE = {'axe': None, 'facteur': 1.0}
EDGE_LOCAL = None   # v0.22 : direction du côté arrondi de la lame (repère de Sword1)
P_LEN = 0.0
if 'SwordBladeN2' in BW0 and 'SwordBladeN' in BW0:
    P_LEN = float(np.linalg.norm(BW0['SwordBladeN2'][:3, 3] - BW0['SwordBladeN'][:3, 3])) * SCALE
if SILENT and I_SW2 is not None:
    try:
        _ms = S.mesh_data.read_mesh(os.path.join(SILENT, 'model.numshb'))
        _pts = []
        for _o in _ms.objects:
            if _o.name.startswith('WeaponbladeM') and not _o.name.startswith('WeaponbladeM_L'):
                _pts.append(np.array(_o.positions[0].data)[:, :3])
        if _pts:
            _p = np.vstack(_pts)
            _inv = np.linalg.inv(W0[I_SW2])
            _loc = (_p @ _inv[:3, :3].T) + _inv[:3, 3]          # sommets dans le repère de Sword2
            _ext = _loc.max(axis=0) - _loc.min(axis=0)
            _ax = int(np.argmax(_ext))
            _lo, _hi = float(_loc[:, _ax].min()), float(_loc[:, _ax].max())
            _len = _hi - _lo
            BLADE['axe'] = _ax
            BLADE['facteur'] = P_LEN / _len if _len > 0.5 and P_LEN > 0.5 else 1.0
            _s1 = (np.linalg.inv(W0[I_SW]) @ np.c_[_p, np.ones(len(_p))].T).T[:, 2] if I_SW is not None else None
            print('LAME skin : axe local %d de Sword2, de %.2f a %.2f (longueur %.2f) ; lame P+ %.2f -> facteur %.2f ; '
                  'parent de Sword2 %s ; lame le long de Sword1 z %.2f -> %.2f'
                  % (_ax, _lo, _hi, _len, P_LEN, BLADE['facteur'], UN[PAR[I_SW2]],
                     float(_s1.min()) if _s1 is not None else 0, float(_s1.max()) if _s1 is not None else 0))
            if I_SW is not None:
                # v0.22 : côté ARRONDI de la lame = tranchant (voir v0.22.1 pour le sens). Mesuré tranche par tranche dans le repère de Sword1
                # (z = le long de la lame), sur la partie haute (sans la garde)
                _l1 = (np.linalg.inv(W0[I_SW]) @ np.c_[_p, np.ones(len(_p))].T).T[:, :3]
                _z0, _z1 = float(_l1[:, 2].min()), float(_l1[:, 2].max())
                _cs = []
                for _k in range(12):
                    _a = _z0 + (_z1 - _z0) * (0.4 + 0.6 * _k / 12)
                    _b = _z0 + (_z1 - _z0) * (0.4 + 0.6 * (_k + 1) / 12)
                    _sl = _l1[(_l1[:, 2] >= _a) & (_l1[:, 2] <= _b)]
                    if len(_sl):
                        _cs.append(np.r_[(_sl[:, :2].min(0) + _sl[:, :2].max(0)) / 2, (_a + _b) / 2])
                if len(_cs) >= 4:
                    _cs = np.array(_cs)
                    _t = (_cs[:, 2] - _cs[0, 2]) / max(_cs[-1, 2] - _cs[0, 2], 1e-6)
                    _ch = _cs[0, :2][None] + _t[:, None] * (_cs[-1, :2] - _cs[0, :2])[None]
                    _bu = (_cs[1:-1, :2] - _ch[1:-1]).mean(0)
                    if np.linalg.norm(_bu) > 1e-3:
                        # v0.22.1 (vidéo en jeu) : le côté qui sort de la ligne droite est le bord plat qui file droit
                        # puis remonte d'un coup vers la pointe = le côté DROIT pour le propriétaire ; le côté ARRONDI
                        # (bord en arc régulier jusqu'à la pointe) est à l'opposé
                        # v0.23 : le côté qui sort de la ligne droite (bombé) EST le côté arrondi (image officielle de
                        # Zero envoyée par le propriétaire : côté bombé devant, pointe penchée vers l'arrière)
                        EDGE_LOCAL = np.array([_bu[0], _bu[1], 0.0]) / float(np.linalg.norm(_bu))
                print('TRANCHANT (cote arrondi) dans le repere de Sword1 :', EDGE_LOCAL,
                      'bombement %.3f' % (float(np.linalg.norm(_bu)) if len(_cs) >= 4 else 0.0))
    except Exception as _e:  # noqa: BLE001
        print('LAME skin illisible', _e)
SW_LOC = (np.linalg.inv(W0[I_HV]) @ W0[I_SW])[:3, :3] if I_SW is not None else np.eye(3)
TIP = next((n for n in ('SwordBladeN2', 'SwordBladeN1', 'SwordBladeN') if n in BR), None)
AIM = {'n': 0, 'max_deg': 0.0}


def rot_between(a, b):
    a = a / np.linalg.norm(a)
    b = b / np.linalg.norm(b)
    v = np.cross(a, b)
    c = float(np.dot(a, b))
    if c < -0.9999:
        # demi-tour : axe quelconque perpendiculaire
        ax = np.cross(a, [1, 0, 0])
        if np.linalg.norm(ax) < 1e-3:
            ax = np.cross(a, [0, 1, 0])
        ax /= np.linalg.norm(ax)
        return 2 * np.outer(ax, ax) - np.eye(3)
    vx = np.array([[0, -v[2], v[1]], [v[2], 0, -v[0]], [-v[1], v[0], 0]])
    return np.eye(3) + vx + vx @ vx / (1 + c)


def aim_blade(wt, MB):
    """v0.2 des animations : l'épée de Roy vise le long de la vraie lame de Zero (poignée SwordN -> bout de la
    lame), car Zero tourne parfois les os de la lame eux-mêmes (pas seulement l'os du sabre)"""
    if TIP is None or 'SwordN' not in MB:
        return wt
    d = MB[TIP][:3, 3] - MB['SwordN'][:3, 3]
    if np.linalg.norm(d) < 1.0:
        return wt  # lame rentrée : on garde l'orientation de l'os du sabre
    cur = (wt[:3, :3] @ SW_LOC)[:, 2]
    if np.linalg.norm(cur) < 1e-6:
        return wt
    rf = rot_between(cur, d)
    ang = math.degrees(math.acos(max(-1.0, min(1.0, float(np.dot(cur / np.linalg.norm(cur), d / np.linalg.norm(d)))))))
    AIM['n'] += 1
    AIM['max_deg'] = max(AIM['max_deg'], ang)
    out = wt.copy()
    out[:3, :3] = rf @ wt[:3, :3]
    return out


def safe_inv(m):
    """v0.13 : certaines animations (victoires) mettent un os à l'échelle 0 : pseudo-inverse dans ce cas"""
    try:
        return np.linalg.inv(m)
    except np.linalg.LinAlgError:
        return np.linalg.pinv(m)


def follow_hand(L, W, MB):
    """v0.5 des animations : plus de miroir. Le sabre de Roy (os HaveR) suit le sabre de Zero (SwordN) dans la
    main qui le tient (la GAUCHE la plupart du temps, comme dans P+), à la même place par rapport à cette main"""
    sw = MB['SwordN']
    dl = np.linalg.norm(sw[:3, 3] - MB['LHandN'][:3, 3])
    dr = np.linalg.norm(sw[:3, 3] - MB['RHandN'][:3, 3])
    hb, hu = ('LHandN', 'HandL') if dl <= dr else ('RHandN', 'HandR')
    if hu not in MAP:
        return
    HAND['gauche' if hu == 'HandL' else 'droite'] += 1
    # v0.6 : sabre placé par rapport à la main de Zero (Brawl) puis reporté sur la main de Roy. Les décalages de
    # repos (OFF) ne servent que pour l'ORIENTATION : leurs translations viennent des bras en T de longueurs
    # différentes et faisaient flotter le sabre loin de la main (v0.5)
    # poignée = point de prise de la main de Roy (HaveL / HaveR, à leur place de repos dans la main) ;
    # orientation = celle du sabre de Zero, puis la lame est visée le long de la sienne (aim_blade)
    if hu == 'HandL' and 'HaveL' in UN:
        grip = W[UN.index('HaveL')][:3, 3]
    else:
        grip = (W[UN.index('HandR')] @ L0[I_HV])[:3, 3]
    rot = OFFS[MIRROR[0]]['HaveR'].copy()
    rot[:3, 3] = 0.0
    wt = sw @ rot
    wt[:3, 3] = grip
    wt = aim_blade(wt, MB)
    # v0.22 (demandé : c'est le côté droit de la lame qui touchait, il faut le côté arrondi) : le sabre tourne sur
    # lui-même (autour de sa lame, sa direction ne change pas) d'un angle choisi pour CHAQUE animation (ROLL), pour
    # que le côté arrondi passe devant dans le mouvement de la lame. 1er passage : on note la lame image par image
    r3_ = wt[:3, :3] @ SW_LOC
    a_ = r3_[:, 2]
    n_ = float(np.linalg.norm(a_))
    if n_ > 1e-6:
        a_ = a_ / n_
        if EDGE_LOCAL is not None:
            e0_ = r3_ @ EDGE_LOCAL
            e0_ = e0_ - a_ * float(np.dot(e0_, a_))
            ne_ = float(np.linalg.norm(e0_))
            hip_ = W[UN.index('Hip')][:3, 3] if 'Hip' in UN and W[UN.index('Hip')] is not None else np.zeros(3)
            if ne_ > 1e-6:
                ROLLREC.append((a_.copy(), e0_ / ne_, wt[:3, 3] + a_ * 9.0 - hip_))
        # v0.23 (image officielle de Zero) : côté arrondi tourné du même côté que l'avant-bras (coude -> poignet),
        # comme quand on tient un sabre : le tranchant regarde devant le poing. Calculé à CHAQUE image ; si la lame
        # est presque dans l'axe de l'avant-bras, on garde l'angle de l'image précédente
        arm_ = ('ArmL' if hu == 'HandL' else 'ArmR')
        if EDGE_LOCAL is not None and arm_ in UN:
            fa_ = W[UN.index(hu)][:3, 3] - W[UN.index(arm_)][:3, 3]
            fp_ = fa_ - a_ * float(np.dot(fa_, a_))
            e0_ = r3_ @ EDGE_LOCAL
            e0_ = e0_ - a_ * float(np.dot(e0_, a_))
            if np.linalg.norm(fp_) > 0.25 * np.linalg.norm(fa_) and np.linalg.norm(e0_) > 1e-6:
                ROLL[0] = math.atan2(float(np.dot(np.cross(a_, e0_), fp_)), float(np.dot(e0_, fp_)))
        if ROLL[0]:
            wt[:3, :3] = axis_rot(a_, ROLL[0]) @ wt[:3, :3]
    L[I_HV] = safe_inv(W[PAR[I_HV]]) @ wt
    W[I_HV] = wt
    GRIP.append(float(np.linalg.norm(wt[:3, 3] - W[UN.index(hu)][:3, 3])))
    # v0.8 : la lame de Zero est rentrée (os SwordBladeN2 à l'échelle 0) hors des coups de sabre : même échelle sur
    # l'os Sword2 de Roy, qui porte le maillage de la lame du skin (WeaponbladeM) ; la poignée (Sword1) reste
    if I_SW2 is not None and TIP in MB:
        sc_ = float(np.linalg.norm(MB[TIP][:3, :3], axis=0).mean())
        BL.append(min(max(sc_, 0.0), 1.0))
    # la main qui tient le sabre se ferme dessus (doigts de Roy au repos = main ouverte)
    for i, loc in CURL.get('L' if hu == 'HandL' else 'R', {}).items():
        L[i] = loc


HAND = {'gauche': 0, 'droite': 0}
# v0.22 : angle du sabre autour de sa lame pour l'animation en cours (radians) et relevé de la lame (1er passage)
ROLL = [0.0]
ROLLREC = []
ROLL_DEFAUT = 0.0   # v0.22.1 : sans vrai coup de lame (ou un coup d'estoc, lame qui avance dans son axe) : sabre retourné par rapport à la v0.21.2, comme la majorité des coups


def best_roll():
    """angle autour de la lame qui met le côté arrondi devant : moyenne, pondérée par la vitesse, du sens du
    mouvement de la pointe (par rapport au bassin) sur les images où la lame est sortie"""
    ext = BL if len(BL) == len(ROLLREC) else [1.0] * len(ROLLREC)
    A = B = tot = 0.0
    for k in range(1, len(ROLLREC)):
        if min(ext[k], ext[k - 1]) < 0.5:
            continue
        a, e0, p = ROLLREC[k]
        v = p - ROLLREC[k - 1][2]
        v = v - a * float(np.dot(v, a))           # seule compte la vitesse de travers (pas le long de la lame)
        sp = float(np.linalg.norm(v))
        A += sp * float(np.dot(v, e0))
        B += sp * float(np.dot(v, np.cross(a, e0)))
        tot += sp
    if tot < 20.0 or math.hypot(A, B) < 1e-6:
        return ROLL_DEFAUT, tot
    return math.atan2(B, A), tot
YS = {}
SC = {}
GRIP = []
BL = []          # échelle brute de la lame de Zero, image par image


def convert_frames(names):
    locals_ = []
    for nm in names:
        # v0.10 : (nom, début, fin) = seulement ces images de l'animation de Zero
        nm, k0, k1 = (nm if isinstance(nm, tuple) else (nm, 0, None))
        an = ANIMS[nm]
        for k in range(k0, min(k1 if k1 is not None else 10 ** 6, max(1, an['frames']))):
            MB = mirrored_scaled(R.world_pose(BR, an, float(k)))
            if STRIP[0] and 'XRotN' in MB:
                # v0.9 : lancer haut : il monte vraiment (le plugin déplace le personnage) -> l'animation ne garde
                # pas cette montée (sinon il monterait deux fois)
                if STRIP[1] is None:
                    STRIP[1] = MB['XRotN'][:3, 3].copy()
                d_ = MB['XRotN'][:3, 3] - STRIP[1]
                d_[0] = 0.0
                for n_ in MB:
                    MB[n_] = MB[n_].copy()
                    MB[n_][:3, 3] -= d_
            W = [None] * len(UB)
            L = [None] * len(UB)
            for i in ORDER:
                u = UN[i]
                pw = W[PAR[i]] if PAR[i] is not None and PAR[i] >= 0 else np.eye(4)
                if u in MAP:
                    wt = MB[MAP[u]] @ OFFS[MIRROR[0]][u]
                    if u in ROOTS:
                        # v0.6 : Trans / Rot gardent leur place de Roy (+ le déplacement animé de Zero). Le bouclier
                        # d'Ultimate est centré sur Rot : posé au sol comme le YRotN de Zero, il était à ses pieds
                        b_ = MAP[u]
                        wt[:3, 3] = W0[i][:3, 3] + (MB[b_][:3, 3] - MB0[MIRROR[0]][b_][:3, 3])
                    if u == 'Hip':
                        # v0.16 : ses pieds ne touchaient pas tout à fait le sol : tout le corps un peu plus bas
                        wt[1, 3] -= 0.5
                    if u == 'Throw':
                        # v0.7 : pendant les lancers, l'adversaire (animations « projeté » d'Ultimate) pend sous l'os
                        # throw : posé à la hauteur de P+, il passait sous le sol -> os levé
                        wt[1, 3] += LIFT[0]
                        wt[2, 3] += LIFTZ[0]
                        if TILT[0]:
                            # v0.12 : lancer bas : adversaire plus à plat (moins les jambes en l'air) ; rotation
                            # autour de l'axe de côté, à la même hauteur (pivot = os throw)
                            wt[:3, :3] = axis_rot(np.array([1.0, 0.0, 0.0]), math.radians(TILT[0])) @ wt[:3, :3]
                    if u == 'HaveR':
                        wt = aim_blade(wt, MB)
                    loc = safe_inv(pw) @ wt
                    if u not in FULL:
                        loc[:3, 3] = L0[i][:3, 3]
                else:
                    loc = L0[i].copy()
                L[i] = loc
                W[i] = pw @ loc
            _bw = R.world_pose(BR, an, float(k))
            for _b in ('SwordN', 'SwordBladeN2', 'SwordSheathedM'):
                if _b in _bw:
                    SC.setdefault(_b, []).append(float(np.linalg.norm(_bw[_b][:3, :3], axis=0).mean()))
            if TIP and 'SwordN' in _bw:
                SC.setdefault('lame', []).append(float(np.linalg.norm(_bw[TIP][:3, 3] - _bw['SwordN'][:3, 3])))
            for _b in ('XRotN', 'YRotN', 'HipN', 'ThrowN'):
                if _b in MB:
                    YS.setdefault(_b, []).append(MB[_b][1, 3])
            if 'HaveR' in MAP and 'SwordN' in MB:
                follow_hand(L, W, MB)
            locals_.append(L)
    return locals_


def smooth_throw(locs, base):
    # v0.19 : l'adversaire « se téléportait » pendant les lancers (sauts de l'os throw d'une image à l'autre) :
    # les sauts de plus de 2,5 unités sont étalés sur 8 images, puis la trajectoire est lissée (moyenne sur 5 images)
    i_ = UN.index('Throw')
    n_ = len(locs)
    P_ = [locs[k][i_][:3, 3].copy() for k in range(n_)]
    before = max([float(np.linalg.norm(P_[k] - P_[k - 1])) for k in range(1, n_)] or [0.0])
    for k in range(1, n_):
        if np.linalg.norm(P_[k] - P_[k - 1]) > 2.5:
            a_, b_ = max(0, k - 4), min(n_ - 1, k + 4)
            for j in range(a_ + 1, b_):
                t_ = (j - a_) / (b_ - a_)
                P_[j] = P_[a_] * (1 - t_) + P_[b_] * t_
    Q_ = []
    for k in range(n_):
        lo, hi = max(0, k - 2), min(n_, k + 3)
        Q_.append(sum(P_[lo:hi]) / (hi - lo))
    for k in range(n_):
        locs[k][i_][:3, 3] = Q_[k]
    after = max([float(np.linalg.norm(Q_[k] - Q_[k - 1])) for k in range(1, n_)] or [0.0])
    print('LISSAGE %s : saut max %.1f -> %.1f' % (base, before, after))


def quat_from(m):
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


def buster_frames(base, n):
    """v0.12.1 : images où le canon est sorti (et la main droite rétrécie) : charge et tirs du B neutre, fin du
    lancer arrière (tir de buster)"""
    if base in ('d00specialnstart', 'd00specialnloop', 'd00specialairnstart', 'd00specialairnloop'):
        return [True] * n
    if base in ('d00specialnend', 'd00specialairnend'):
        return [k < 151 for k in range(n)]      # 151 = début du Tenkuuha (sabre)
    if base == 'e01throwb':
        return [k >= 28 for k in range(n)]
    if base == 'e01throwf':
        # v0.20 : lancer avant : il tire au buster (5 tirs de P+, images 13 à 30) -> canon de 10 à 44
        return [10 <= k < 45 for k in range(n)]
    return [False] * n


def build_anim(locals_, vis=None):
    n = len(locals_)
    keep = [i for i, u in enumerate(UN) if u in MAP or any(i in c for c in CURL.values()) or i == I_SW2]
    nodes = []
    for i in sorted(keep, key=lambda i: UN[i]):
        vals, prev = [], None
        for k in range(n):
            m = locals_[k][i]
            sc = np.linalg.norm(m[:3, :3], axis=0)
            q = quat_from(m[:3, :3] / np.where(sc == 0, 1, sc))
            if prev is not None and np.dot(prev, q) < 0:
                q = -q
            if prev is None and q[3] < 0:
                q = -q
            prev = q
            vals.append(A.Transform([float(x) for x in sc], [float(x) for x in q], [float(x) for x in m[:3, 3]]))
        if n == 1:
            vals = vals * 2
        tr = A.TrackData('Transform')
        tr.values = vals
        try:
            # l'échelle d'un os passe à ses enfants (main rétrécie -> doigts et sabre aussi)
            tr.scale_options.inherit_scale = True
            tr.scale_options.compensate_scale = False
        except Exception:  # noqa: BLE001
            pass
        nd = A.NodeData(UN[i])
        nd.tracks = [tr]
        nodes.append(nd)
    g = A.GroupData(A.GroupType.Transform)
    g.nodes = nodes
    a = A.AnimData()
    a.major_version, a.minor_version = 2, 0
    a.final_frame_index = float(max(n - 1, 1))
    a.groups = [g]
    if vis is not None:
        # piste de visibilité du canon (maillages ZeroBuster_VIS_O_OBJ du modèle)
        tv = A.TrackData('Visibility')
        tv.values = [bool(v) for v in vis] if n > 1 else [bool(vis[0])] * 2
        nv = A.NodeData('ZeroBuster')
        nv.tracks = [tv]
        gv = A.GroupData(A.GroupType.Visibility)
        gv.nodes = [nv]
        a.groups = [g, gv]
    return a


# ------------------------------------------------------------------ fichiers de Roy <- animations de Zero
SPECIAL = {
    'c03attacks4s': ['AttackS4Start', 'AttackS4S'], 'c03attacks4charge': ['AttackS4Hold'],
    'c04attackhi4': ['AttackHi4Start', 'AttackHi4'], 'c04attackhi4charge': ['AttackHi4Hold'],
    'c04attacklw4': ['AttackLw4Start', 'AttackLw4'], 'c04attacklw4charge': ['AttackLw4Hold'],
    # v0.10 : B neutre. Roy n'a qu'un fichier pour ses 4 fins (niveaux 1-3 et maximum) : les 4 animations de Zero
    # bout à bout (tirs N1, N2, N3 puis coup de sabre N4) ; le plugin saute au bon morceau selon le niveau.
    # Charge : Zero lève son buster (début de N2), puis garde la pose
    'd00specialnend': ['SpecialN1', 'SpecialN2', 'SpecialN3', 'SpecialN4'],
    # v0.13 : jabs 1-2-3 de P+ bout à bout (Roy n'a qu'un jab ; le plugin enchaîne les morceaux)
    'c00attack11': ['Attack11', 'Attack12', 'Attack13'],
    'd00specialairnend': ['SpecialAirN1', 'SpecialAirN2', 'SpecialAirN3', 'SpecialAirN4'],
    'd00specialnstart': [('SpecialN2', 0, 16)], 'd00specialnloop': [('SpecialN2', 15, 17)],
    'd00specialairnstart': [('SpecialAirN2', 0, 16)], 'd00specialairnloop': [('SpecialAirN2', 15, 17)],
    # v0.11 : bas B (Raikousen). Ruée horizontale au sol / en l'air, plongée en l'air (sur le « contre réussi »
    # en l'air de Roy, inutilisé), onde de choc à l'atterrissage (sur le « contre réussi » au sol)
    'd03speciallw': ['SpecialLw'], 'd03specialairlw': ['SpecialLwAir'],
    'd03specialairlwhit': ['SpecialAirLw'], 'd03speciallwhit': ['SpecialLwLanding'],
    # v0.15 : smash chargés de P+ (maintenus = Shoenzan, Rakuhouha, smash avant chargé) sur des mouvements du côté B
    # de Roy inutilisés (son côté B est remplacé) ; le plugin y passe quand le smash a été chargé
    'd01specials3hi': ['AttackHi4Charged'], 'd01specials3lw': ['AttackLw4Charged'], 'd01specials3s': ['AttackS4SCharged'],
    # v0.15 : côté B aérien (Air Hienkyaku) : départ + ruée aérienne bout à bout, et son coup
    'd01specialairs1': ['SpecialAirSStart', 'SpecialAirSEnd'], 'd01specialairs2hi': ['SpecialAirSAttack'],
    # v0.18 : Final Smash de P+ (sol, air)
    'd04final': ['Final'], 'd04finalair': ['FinalAir'],
}
# v0.16 : esquives (sur place, roulades, aérienne) : nombre d'images visé (durées d'Ultimate)
RESAMPLE = {'b02escapen': 28, 'b02escapef': 32, 'b02escapeb': 32, 'b02escapeair': 49}
GROUPS = {
    'attaques': r'^c0',
    # v0.4 des animations (demandé) : attente, sauts, marche, course, bouclier
    'base': r'^(a00wait[123]|a01walk(slow|middle|fast|brakel|braker)|a02(dash|run|runbrakel|runbraker|turnrun)|'
            r'a03jump(squat|f|b|aerialf|aerialb)|b00guard(on|off)?|b02escape(n|f|b|air)|'
            # v0.18 : demi-tour, chutes, atterrissages, accroupi, pas, bouclier qui encaisse
            r'a01turn|a02turnrunbrake|a04(fall|fallf|fallb|fallaerial|fallaerialf|fallaerialb|fallspecial|fallspecialf|'
            r'fallspecialb|landingfallspecial|damagefall)|a05(landingheavy|squat|squatrv|squatwait|squatwaititem)|'
            r'a06(stepairpose|stepback|stepfall|stepjump|steppose)|b01guarddamage)$',
    # v0.5 : chopes (sol, course, demi-tour, tirer, tenir, coup, lâcher) et lancers
    'prises': r'^(e00catch(attack|cut|dash|pull|turn|wait)?|e01throw(b|f|hi|lw))$',
    'speciaux': r'^(d00special(air)?n(end|start|loop)|d03special(air)?lw(hit)?|d01specials3(hi|lw|s)|d01specialairs(1|2hi)|d04final(air)?)$',
    # v0.13 : provocations, victoires, défaite
    'fin': r'^(j01appeal(hi|s|lw)(l|r)|j02(win[123](wait)?|lose)|j00entry(l|r))$',
}
roy = open(os.path.join(HERE, 'roy_motions.txt')).read().split()
report = {}
for fn in roy:
    base = fn[:-len('.nuanmb')]
    if not re.match(GROUPS[GROUP], base):
        continue
    if base in SPECIAL:
        src = [s for s in SPECIAL[base] if (s[0] if isinstance(s, tuple) else s) in ANIMS]
    else:
        nm = base[3:]
        cands = [nm] + ([nm[:-1]] if nm[-1:] in ('l', 'r') else [])
        hit = next((LOW[c] for c in cands if c in LOW), None)
        src = [hit] if hit else []
    if not src:
        print('SANS', fn)
        continue
    srcn = [n_[0] if isinstance(n_, tuple) else n_ for n_ in src]
    frac = min(sword_hand(ANIMS[n_]) for n_ in srcn) if 'SwordN' in BR else 1.0
    # v0.5 : jamais de miroir (le miroir montrait son dos dans les coups et le faisait se retourner à la fin) ;
    # Zero garde sa pose de P+ et le sabre passe dans sa main gauche (follow_hand)
    MIRROR[0] = False
    # lancer bas : il le frappe au sol -> moins haut (0 : sous le sol, 10 : trop haut)
    # v0.20.2 : lancer avant remis comme avant (10) : à 2 il passait sous le sol ; seul le début était faux (lissé)
    LIFT[0] = 1.5 if base == 'e01throwlw' else (10.0 if base.startswith('e01throw') else 0.0)
    # lancer bas : adversaire devant lui, sous ses poings (v0.9)
    LIFTZ[0] = 15.0 if base == 'e01throwlw' else 0.0
    # v0.15.1 : plus à plat, pieds moins hauts
    TILT[0] = -70.0 if base == 'e01throwlw' else 0.0
    # la ruée du bas B est faite par le plugin (vitesse) : l'animation ne garde pas son déplacement
    STRIP[0], STRIP[1] = (base in ('d03speciallw', 'd03specialairlw', 'd03specialairlwhit', 'd01specialairs1', 'd01specialairs2hi')), None
    BL.clear()
    HAND['gauche'] = HAND['droite'] = 0
    YS.clear()
    SC.clear()
    GRIP.clear()
    ROLLREC.clear()
    ROLL[0] = 0.0
    loc = convert_frames(src)
    if False and EDGE_LOCAL is not None and ROLLREC:   # v0.23 : plus de 2e passage (angle calculé image par image)
        # v0.22 : 2e passage avec le sabre tourné pour que le côté arrondi frappe
        ROLL[0], tot_ = best_roll()
        print('TRANCHANT %-22s sabre tourne de %4.0f deg (mouvement de lame %.0f)' % (base, math.degrees(ROLL[0]), tot_))
        BL.clear()
        HAND['gauche'] = HAND['droite'] = 0
        YS.clear()
        SC.clear()
        GRIP.clear()
        ROLLREC.clear()
        STRIP[1] = None
        loc = convert_frames(src)
    if base.startswith('e01throw') and 'Throw' in UN:
        smooth_throw(loc, base)
    if base in ('a01turn',):
        # v0.20 : demi-tour : dans Ultimate, le personnage est retourné dès le début et l'animation part « de dos » ;
        # celle de P+ le fait pivoter elle-même -> il était retourné deux fois puis revenait d'un coup : on la fait
        # pivoter de 180° autour de la verticale (racine)
        R_ = np.eye(4)
        R_[0, 0] = R_[2, 2] = -1.0
        roots_ = [i_ for i_ in range(len(UN)) if PAR[i_] is None or PAR[i_] < 0]
        for k_ in range(len(loc)):
            for i_ in roots_:
                loc[k_][i_] = R_ @ loc[k_][i_]
        print('DEMI-TOUR %s : racines %s pivotées de 180°' % (base, [UN[i_] for i_ in roots_]))
    if base in ('a00wait1', 'a00wait2', 'a00wait3') and CURL:
        # v0.24 (demandé) : pendant l'attente, il ferme les DEUX poings (ses mains P+ n'ont pas de doigts : celles
        # du skin restaient ouvertes, la main tendue). Même poing que la main qui tient le sabre
        for k_ in range(len(loc)):
            for side_ in ('L', 'R'):
                for i_, c_ in CURL.get(side_, {}).items():
                    loc[k_][i_] = c_
        print('POINGS FERMES %s : %s' % (base, {k: len(v) for k, v in CURL.items()}))
    if I_SW2 is not None and len(BL) == len(loc):
        # v0.9 : lame à la taille de P+ ; elle reste sortie 10 images de plus après le coup, puis rentre en 5 images
        ax = BLADE['axe'] if BLADE['axe'] is not None else 2
        for k_ in range(len(loc)):
            se = 0.0
            for j_ in range(0, 16):
                if k_ - j_ < 0:
                    break
                w_ = 1.0 if j_ <= 10 else (16 - j_) / 6.0
                se = max(se, BL[k_ - j_] * w_)
            Sm = np.eye(3) * min(1.0, se * 4.0)
            Sm[ax, ax] = max(se * BLADE['facteur'], 0.01)
            if base == 'd04finalair':
                # v0.20.3 : Final Smash aérien : sabre géant (5 fois plus long, un peu plus épais) des images 18 à 98
                g_ = 1.0 + 4.0 * max(0.0, min(1.0, (k_ - 18) / 6.0, (98 - k_) / 6.0))
                Sm[ax, ax] *= g_
                for a_ in range(3):
                    if a_ != ax:
                        Sm[a_, a_] *= 1.0 + (g_ - 1.0) * 0.15
            loc2 = L0[I_SW2].copy()
            loc2[:3, :3] = L0[I_SW2][:3, :3] @ Sm
            loc[k_][I_SW2] = loc2
    if len(src) == 1 and not isinstance(src[0], tuple) and ANIMS[src[0]]['loop'] and len(loc) > 2:
        # boucle (attente, marche, course, bouclier) : la dernière image rejoint la première
        loc = loc + [loc[0]]
    bf = buster_frames(base, len(loc))
    if 'HandR' in UN and any(bf):
        ih = UN.index('HandR')
        for k_, on_ in enumerate(bf):
            if on_:
                m_ = loc[k_][ih].copy()
                m_[:3, :3] = m_[:3, :3] * 0.01
                loc[k_][ih] = m_
    if base in RESAMPLE and len(loc) > 1:
        # v0.16 : esquives de P+ ramenées à la durée de celles de Roy (sinon elles dureraient plus longtemps)
        T_ = RESAMPLE[base]
        idx_ = [round(i_ * (len(loc) - 1) / (T_ - 1)) for i_ in range(T_)]
        loc = [loc[i_] for i_ in idx_]
        bf = [bf[i_] for i_ in idx_]
    build_anim(loc, bf).save(os.path.join(OUT, 'fighter/roy/motion/body/c00', fn))
    if any(bf):
        print('    CANON images %d a %d (main droite retrecie)' % (bf.index(True), len(bf) - 1 - bf[::-1].index(True)))
    report[fn] = {'zero': [str(x) for x in src], 'images': len(loc)}
    if len(srcn) > 1:
        _o, _segs = 0, []
        for n_ in src:
            _n = ANIMS[n_]['frames'] if not isinstance(n_, tuple) else (min(n_[2], ANIMS[n_[0]]['frames']) - n_[1])
            _segs.append((n_ if isinstance(n_, str) else n_[0], _o, _n))
            _o += _n
        print('    MORCEAUX', _segs)
    print('%-26s <- %-30s %4d images, sabre main gauche %3.0f %% (images : %s), correction max de la lame %.0f°'
          % (fn, '+'.join(srcn), len(loc), 100 * frac, HAND, AIM['max_deg']))
    if SC:
        print('    sabre (echelle/longueur min/moy/max) ' + ', '.join('%s %.2f/%.2f/%.2f' % (k, min(v), sum(v) / len(v), max(v)) for k, v in SC.items()))
    if GRIP:
        print('    sabre-main : moy %.1f max %.1f' % (sum(GRIP) / len(GRIP), max(GRIP)))
    print('    hauteur (min/moy/max) ' + ', '.join('%s %.1f/%.1f/%.1f' % (k, min(v), sum(v) / len(v), max(v)) for k, v in YS.items()))
    AIM['max_deg'] = 0.0
json.dump({'echelle': SCALE, 'os': MAP, 'fichiers': report}, open(os.path.join(OUT, 'retarget_%s.json' % GROUP), 'w'),
          indent=1, ensure_ascii=False)
print('ANIMATIONS', len(report))
