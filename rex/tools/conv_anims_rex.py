"""REX (étape 3) : ses animations Brawl (CHR0 de FitXXXMotionEtc.pac) -> fichiers .nuanmb du perso de base d'Ultimate,
sur SON squelette (celui du modèle construit par build_model_rex.py). Pas de reciblage : chaque os reçoit son SRT
Brawl tel quel (Trans <- TransN, Rot <- YRotN, etc.) ; les os du perso de base ajoutés au squelette (sans équivalent)
gardent leur pose de repos.

Pourquoi TOUTES les animations : le modèle a son propre squelette ; une animation du perso de base qui reste (Mario)
tord le modèle (os orientés autrement) ou le fait disparaître. Chaque fichier d'animation du perso de base reçoit donc
une animation de REX : la sienne si elle existe, sinon la plus proche (liste ci-dessous), sinon l'attente.
Les animations partielles du jeu (paupières, l*share*) ne sont pas touchées.

usage : python3 conv_anims_rex.py <FitXXX00.pac> <FitXXXMotionEtc.pac> <sortie de build_model_rex.py> <liste des
        .nuanmb du perso de base> <dossier de sortie des .nuanmb>
"""
import json, math, os, re, sys
import numpy as np
import ssbh_data_py as S

HERE = os.path.dirname(os.path.abspath(__file__))
for p in ('../../black-shadow/tools', '../../black-shadow/tools/anim', '../../zero/tools', '.'):
    sys.path.insert(0, os.path.join(HERE, p))
from arc import arc_entries  # noqa: E402
from brres import brres_files  # noqa: E402
import brawl_anim  # noqa: E402
from gen_rex import plain  # noqa: E402

MODEL_PAC, MOTION_PAC, MOUT, LIST, DST = sys.argv[1:6]
os.makedirs(DST, exist_ok=True)
A = S.anim_data
info = json.load(open(os.path.join(MOUT, 'model.json')))
U2B = info['u2b']
sk = S.skel_data.read_skel(os.path.join(MOUT, 'model', 'model.nusktb'))
UB = sk.bones
BR = brawl_anim.read_bones(plain(MODEL_PAC))

ANIMS = {}


def walk(d):
    for e in arc_entries(d):
        if e[4][:4] == b'ARC\0':
            walk(e[4])
        elif e[4][:4] == b'bres':
            for folder, items in brres_files(e[4]).items():
                if 'Chr' in folder:
                    for n, o in items:
                        ANIMS.setdefault(n, (e[4], o))


walk(open(plain(MOTION_PAC), 'rb').read())
LOW = {k.lower(): k for k in ANIMS}
print('animations REX', len(ANIMS))

# fichier du perso de base -> animation(s) REX (plusieurs = mises bout à bout)
SPECIAL = {
    # smashs : début + coup (Ultimate : un seul fichier), charge à part
    'c03attacks4s': ['AttackS4Start', 'AttackS4S'], 'c03attacks4hi': ['AttackS4Start', 'AttackS4S'],
    'c03attacks4lw': ['AttackS4Start', 'AttackS4S'], 'c03attacks4charge': ['AttackS4Hold'],
    'c04attackhi4': ['AttackHi4Start', 'AttackHi4'], 'c04attackhi4charge': ['AttackHi4Hold'],
    'c04attacklw4': ['AttackLw4Start', 'AttackLw4'], 'c04attacklw4charge': ['AttackLw4Hold'],
    # bord
    'g02cliffattack': ['CliffAttackQuick'], 'g02cliffclimb': ['CliffClimbQuick'], 'g02cliffescape': ['CliffEscapeQuick'],
    'g02cliffjump1': ['CliffJumpQuick1'], 'g02cliffjump2': ['CliffJumpQuick2'],
    # provocations, entrée, victoires
    'j01appealhir': ['AppealHi'], 'j01appealsr': ['AppealS'], 'j01appeallwr': ['AppealLw'],
    'j00entryr': ['EntryL'],
    # déplacements sans équivalent direct
    'a01walkbrakel': ['WalkBrake'], 'a01walkbraker': ['WalkBrake'], 'a02runbrakel': ['RunBrake'],
    'a02runbraker': ['RunBrake'], 'a02turndash': ['Dash'], 'a04runfalll': ['Fall'], 'a04runfallr': ['Fall'],
    'a04walkfalll': ['Fall'], 'a04walkfallr': ['Fall'], 'a03jumpb': ['JumpF'], 'a03jumpaerialf': ['JumpF'],
    'a03jumpaerialb': ['JumpF'], 'a04fallaerial': ['Fall'], 'a04fallaerialf': ['FallF'], 'a04fallaerialb': ['FallB'],
    'a04fallspecial': ['Fall'], 'a04fallspecialf': ['FallF'], 'a04fallspecialb': ['FallB'],
    'a04landingfallspecial': ['Landing'], 'a04damagefall': ['DamageFall'],
    'a05landinglight': ['Landing'], 'a05landingheavy': ['Landing'], 'a05squatwaititem': ['SquatWait'],
    'a00wait3': ['Wait1'], 'a00waititem': ['Wait1'],
    'c05landingairn': ['Landing'], 'c05landingairb': ['Landing'], 'c05landingairlw': ['Landing'],
    'c05landingairf': ['LandingAirF'], 'c05landingairhi': ['LandingAirHi'],
    'b00justshieldoff': ['GuardOff'], 'b02rebound': ['DamageN1'], 'e00catchpull': ['CatchWait'],
    # spéciaux (ceux de Mario ont ses noms ; le plugin pilote la suite)
    'd00specialn': ['SpecialN1'], 'd00specialairn': ['SpecialAirN'],
    'd01specials': ['SpecialS'], 'd01specialairs': ['SpecialAirS'],
    # haut B : dans REX, au sol comme en l'air, c'est la même sous-action (Rising Tackle, qui monte avec Trans)
    'd02specialhi': ['SpecialAirHi'], 'd02specialairhi': ['SpecialAirHi'],
    'd03speciallwstart': ['SpecialLwStart'], 'd03specialairlwstart': ['SpecialAirLwStart'],
    'd03speciallwhold': ['SpecialLwLoop'], 'd03specialairlwhold': ['SpecialLwLoop'],
    'd03speciallwlight': ['SpecialLwStop'], 'd03speciallwheavy': ['SpecialLwStop'],
    'd03specialairlwlight': ['SpecialLwStop'], 'd03specialairlwheavy': ['SpecialLwStop'],
    'd04final': ['Final'], 'd04finalair': ['Final'],
    # chope / lancers / projeté : variantes du jeu sans équivalent
    'e01thrownbigb': ['ThrownB'], 'e01thrownbigf': ['ThrownF'], 'e01thrownbighi': ['ThrownHi'], 'e01thrownbiglw': ['ThrownLw'],
    'e01throwngirlb': ['ThrownB'], 'e01throwngirlf': ['ThrownF'], 'e01throwngirlhi': ['ThrownHi'], 'e01throwngirllw': ['ThrownLw'],
    'e03capturecut': ['CaptureCut'],
    # dégâts, chutes
    'f01damageflymeteor': ['DamageFlyLw'], 'f01damageflyrollend': ['DamageFlyRoll'], 'f03downdamageu3': ['DownDamageU'],
    'f04downdamaged3': ['DownDamageD'], 'f03downboundu': ['DownDamageU'], 'f04downboundd': ['DownDamageD'], 'f04downspotd': ['DownSpotD'], 'f03downwaitu': ['DownWaitU'],
    'f05passivestandb': ['PassiveStandB'], 'f05passivestandf': ['PassiveStandF'],
    'f06furafurastartd': ['FuraFuraStartD'], 'f06furafurastartu': ['FuraFuraStartU'],
    'f07sleepend': ['FuraSleepEnd'], 'f07sleeploop': ['FuraSleepLoop'], 'f07sleepstart': ['FuraSleepStart'],
    'f08swallowed': ['Swallowed'], 'f09camerahit': ['DamageN1'],
    'g00ceildamage': ['DamageHi1'], 'g00walldamage': ['WallDamage'], 'g00stopceil': ['StopCeil'], 'g00stopwall': ['StopWall'],
    'g00pass': ['Pass'], 'g00missfoot': ['MissFoot'], 'g00ottotto': ['Ottotto'], 'g00ottottowait': ['OttottoWait'],
    'g03slipattack': ['SlipAttack'], 'g03slipdown': ['SlipDown'], 'g03slipescapeb': ['SlipEscapeB'],
    'g03slipescapef': ['SlipEscapeF'], 'g03slipstand': ['SlipStand'], 'g03slipwait': ['SlipWait'], 'g03slip': ['Slip'],
    # échelle
    'k00laddercatchairr': ['LadderCatchR'], 'k00laddercatchendr': ['LadderCatchEndR'], 'k00laddercatchr': ['LadderCatchR'],
    # objets
    'h00lightwalkeat': ['LightEat'], 'h00lightwalkget': ['LightGet'], 'h03heavywalk': ['HeavyWalk1'],
    'h04swing1': ['Swing1'], 'h04swing3': ['Swing3'], 'h04swing4': ['Swing4'], 'h04swing4charge': ['Swing4Hold'],
    'h04swingdash': ['SwingDash'], 'h06swing4bat': ['Swing4Bat'], 'h07itemscrew': ['ItemScrew'], 'h07itemscrewfall': ['ItemScrewFall'],
    'h10itemlegsjumpsquat': ['ItemLegsJumpSquat'], 'h10itemlegslanding': ['ItemLegsLanding'],
    'h10itemlegswalkb': ['ItemLegsDashB'], 'h10itemlegswalkf': ['ItemLegsDashF'],
    'h11itemshootloop': ['ItemShoot'], 'h11itemshootairloop': ['ItemShootAir'], 'h11itemshootupper': ['ItemShoot'],
    'h11itemshootloopupper': ['ItemShoot'],
    'h12itemscopeendupper': ['ItemScopeEnd'], 'h12itemscopefall': ['ItemScopeAirStart'], 'h12itemscopefireupper': ['ItemScopeFire'],
    'h12itemscoperapidupper': ['ItemScopeRapid'], 'h12itemscopestartupper': ['ItemScopeStart'],
    'h12itemscopewait': ['ItemScopeStart'], 'h12itemscopewaitupper': ['ItemScopeStart'],
    'h13itemassist': ['ItemAssist'], 'h14gekikarawait': ['Wait1'], 'h14gekikarawaitface': ['Wait1'], 'h15itemgrasspull': ['LightGet'],
    'h16itemgenesisairend': ['ItemLauncherAirFire'], 'h16itemgenesisairfire': ['ItemLauncherAirFire'],
    'h16itemgenesisairloop': ['ItemLauncherAirFire'], 'h16itemgenesisend': ['ItemLauncher'], 'h16itemgenesisfall': ['ItemLauncherFall'],
    'h16itemgenesisfire': ['ItemLauncherFire'], 'h16itemgenesisloop': ['ItemLauncher'], 'h16itemgenesisturn': ['ItemLauncher'],
    'h16itemgenesiswait': ['ItemLauncher'], 'h16itemgenesiswaitupper': ['ItemLauncher'],
    'h08itemdragoonget': ['ItemDragoonGet'], 'h08itemdragoonride': ['ItemDragoonRide'],
    # nage
    'i00swimrise': ['SwimRise'], 'i00swimupdamage': ['SwimUpDamage'], 'i01swimdrown': ['SwimDrown'], 'i01swimdrownout': ['SwimDrownOut'],
}
# animations partielles du jeu (paupières, expressions partagées) : pas touchées
KEEP = re.compile(r'^(a00defaulteyelid|l0)')


def fallback(base):
    """animation REX de repli quand il n'y a rien de mieux"""
    if base.startswith('f0'):
        return 'DamageFlyN' if 'fly' in base else 'DamageN1'
    if base.startswith(('e02', 'e03')):
        return 'CaptureWaitHi'
    if base.startswith('i0'):
        return 'Swim'
    if base.startswith('k0'):
        return 'LadderWait'
    if 'air' in base or 'fall' in base or 'jump' in base:
        return 'Fall'
    return 'Wait1'


names = [l.strip() for l in open(LIST) if l.strip().endswith('.nuanmb')]
jobs, missing = [], []
for fn in names:
    base = fn[:-len('.nuanmb')]
    if KEEP.match(base):
        continue
    if base in SPECIAL:
        src = [s for s in SPECIAL[base] if s in ANIMS]
    else:
        nm = base[3:]
        cands = [nm] + ([nm[:-1]] if nm[-1:] in ('l', 'r') else [])
        hit = next((LOW[c] for c in cands if c in LOW), None)
        src = [hit] if hit else []
    if not src:
        fb = fallback(base)
        src = [fb] if fb in ANIMS else ['Wait1']
        missing.append('%s->%s' % (base, src[0]))
    jobs.append((fn, src))
print('remplacements de repli :', len(missing), ' '.join(missing))


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


CACHE = {}


def frames_of(src):
    out = []
    for name in src:
        if name not in CACHE:
            d, o = ANIMS[name]
            CACHE[name] = brawl_anim.read_chr0(d, o)
        an = CACHE[name]
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
        if n == 1:
            vals = vals * 2
        tr = A.TrackData('Transform')
        tr.values = vals
        nd = A.NodeData(b.name)
        nd.tracks = [tr]
        nodes.append(nd)
    g = A.GroupData(A.GroupType.Transform)
    g.nodes = nodes
    a = A.AnimData()
    a.major_version, a.minor_version = 2, 0
    a.final_frame_index = float(max(n - 1, 1))
    a.groups = [g]
    a.save(os.path.join(DST, fn))
    report[fn] = {'rex': src, 'images': n}
print('ANIMATIONS CONVERTIES', len(report))
for fn in ('a00wait1.nuanmb', 'c00attack11.nuanmb', 'd02specialhi.nuanmb', 'j01appealhir.nuanmb'):
    if fn in report:
        print('  ', fn, report[fn])
json.dump(report, open(os.path.join(DST, '..', 'anims_rex.json'), 'w'), indent=1)
