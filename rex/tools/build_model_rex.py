"""REX : modèle Brawl (MDL0 d'un FitXXX00.pac, compressé ou non) -> modèle Ultimate (numshb, numdlb, numatb, nusktb,
textures png). Adapté de l'outil de Zero (zero/tools/build_model.py), sans ses réglages propres (yeux, lame, buster).

usage : python3 build_model_rex.py <FitXXX00.pac> <modèle Ultimate de référence du perso de base (dossier c00)> <sortie> [préfixe]
- squelette : celui de Zero (63 os), os principaux renommés comme dans Ultimate (Trans, Rot, Hip, HandR, HaveR…),
  TopN retiré (Trans devient la racine) ; ses animations Brawl iront dessus sans reciblage ;
- sommets : liés à un os -> placés avec la matrice de repos de l'os ; pondérés (NodeMix) -> déjà dans l'espace
  du modèle ; 4 os au plus par sommet (comme Ultimate) ;
- un objet Ultimate par objet Brawl, un matériau Ultimate (PBS opaque) par matériau Brawl, sa texture en Texture0.
"""
import json, os, struct, sys
import numpy as np
import ssbh_data_py as S
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
for p in ('../../black-shadow/tools', '../../black-shadow/tools/fs', '../../black-shadow/tools/anim', '../../zero/tools', '.'):
    sys.path.insert(0, os.path.join(HERE, p))
from arc import arc_entries  # noqa: E402
from brres import brres_files, dict_entries  # noqa: E402
import mdl0  # noqa: E402
import tex0  # noqa: E402
import brawl_anim  # noqa: E402
from lz import decompress  # noqa: E402

SRC, REF, OUT = sys.argv[1:4]
PFX = sys.argv[4] if len(sys.argv) > 4 else 'rex'
MDL = os.path.join(OUT, 'model')
PNG = os.path.join(OUT, 'png')
os.makedirs(MDL, exist_ok=True)
os.makedirs(PNG, exist_ok=True)

raw = decompress(open(SRC, 'rb').read())
tmp = '/tmp/rex_model_%s.pac' % PFX
open(tmp, 'wb').write(raw)
dd = [x for x in arc_entries(raw) if x[0] == 2][0][4]
files = brres_files(dd)
mn, mo = files['3DModels(NW4R)'][0]
offs = struct.unpack_from('>11i', dd, mo + 0x10)
SEC = dict(zip(['defs', 'bones', 'pos', 'nrm', 'clr', 'uv', 'mats', 'shaders', 'objs', 'tex', 'pal'], offs))

# ------------------------------------------------------------------ os
BR = brawl_anim.read_bones(tmp)                     # nom -> Bone (SRT de repos, parent, matrice monde lue)
by_index = {b.index: b for b in BR.values()}
nid = {}                                            # n° de matrice -> nom d'os
for name, bo in dict_entries(dd, mo + SEC['bones']):
    nid[struct.unpack_from('>i', dd, bo + 0x10)[0]] = name
CORE = {
    'Trans': 'TransN', 'Rot': 'YRotN', 'Hip': 'HipN', 'Waist': 'WaistN', 'Bust': 'BustN', 'Neck': 'NeckN',
    'Head': 'HeadN', 'ClavicleL': 'LShoulderN', 'ShoulderL': 'LShoulderJ', 'ArmL': 'LArmJ', 'HandL': 'LHandN',
    'HaveL': 'LHaveN', 'ClavicleR': 'RShoulderN', 'ShoulderR': 'RShoulderJ', 'ArmR': 'RArmJ', 'HandR': 'RHandN',
    'HaveR': 'RHaveN', 'LegL': 'LLegJ', 'KneeL': 'LKneeJ', 'FootL': 'LFootJ', 'ToeL': 'LToeN', 'LegR': 'RLegJ',
    'KneeR': 'RKneeJ', 'FootR': 'RFootJ', 'ToeR': 'RToeN', 'Throw': 'ThrowN', 'XRot': 'XRotN',
}
for s in 'LR':
    for n, b in ((1, '1st'), (2, '2nd'), (3, '3rd'), (4, '4th'), (5, 'Thumb')):
        CORE['Finger%s%d1' % (s, n)] = '%s%sNa' % (s, b)
        CORE['Finger%s%d2' % (s, n)] = '%s%sNb' % (s, b)
        # v0.7 : 3e phalange (Terry en a une) : sinon le bout des doigts de Rock reste droit, poings à moitié ouverts
        CORE['Finger%s%d3' % (s, n)] = '%s%sNc' % (s, b)
REN = {v: k for k, v in CORE.items()}
UNAME = {n: REN.get(n, n) for n in BR}              # nom Brawl -> nom Ultimate

order = []                                          # parents avant enfants, TopN retiré


for _n, _b in BR.items():
    if _b.parent == 'TopN' and _n != 'TransN':
        _b.parent = 'TransN'           # os de visibilité (EyeYellowM…) : sous Trans, Trans reste la seule racine


def visit(n):
    if n in order or n == 'TopN':
        return
    p = BR[n].parent
    if p and p != 'TopN':
        visit(p)
    order.append(n)


visit('TransN')


for b in sorted(BR.values(), key=lambda b: b.index):
    visit(b.name)
WORLD = {}
for n in order:
    b = BR[n]
    loc = brawl_anim.srt_matrix(b.s, b.r, b.t)
    WORLD[n] = (WORLD[b.parent] @ loc) if (b.parent and b.parent != 'TopN') else loc
sk = S.skel_data.SkelData()
sk.major_version, sk.minor_version = 1, 0
bones = []
for n in order:
    b = BR[n]
    par = b.parent if (b.parent and b.parent != 'TopN') else None
    loc = brawl_anim.srt_matrix(b.s, b.r, b.t)
    bd = S.skel_data.BoneData(UNAME[n], np.ascontiguousarray(loc.T, dtype=np.float32), order.index(par) if par else None)
    bones.append(bd)
# os de Roy absents chez Zero (boîtes de dégâts, effets du jeu) : ajoutés en bout de chaîne, sans effet sur le modèle
ALIAS = [('ClavicleC', 'BustN'), ('LegC', 'HipN'), ('Sword1', 'SwordBladeN'), ('Sword2', 'SwordBladeN2'),
         ('Scabbard', 'HipN'), ('Hilt', 'HipN'), ('Face', 'HeadN')]
for an, par in ALIAS:
    if par in order and an not in [b.name for b in bones]:
        bones.append(S.skel_data.BoneData(an, np.eye(4, dtype=np.float32), order.index(par)))
# v0.5 : tous les os de Roy présents (squelette « exo ») : les os de Roy absents chez Zero sont ajoutés à la suite,
# avec leur position de repos de Roy ; leur nuhlpb (os d'aide) peut alors être repris tel quel.
# (GameBanana : un modèle invisible au départ jusqu'à une action = os d'origine ou nuhlpb manquants)
try:
    roy_sk = S.skel_data.read_skel(os.path.join(REF, 'model.nusktb'))
    have = {b.name.lower(): i for i, b in enumerate(bones)}
    added = 0
    for rb in roy_sk.bones:
        if rb.name.lower() in have:
            continue
        par = roy_sk.bones[rb.parent_index].name.lower() if rb.parent_index is not None else None
        pi = have.get(par, 0) if par else 0
        bones.append(S.skel_data.BoneData(rb.name, np.ascontiguousarray(np.array(rb.transform, dtype=np.float32)), pi))
        have[rb.name.lower()] = len(bones) - 1
        added += 1
    print('os du perso de base ajoutes :', added, '; total', len(bones))
except Exception as e:  # noqa: BLE001
    print('os du perso de base non ajoutes :', e)
sk.bones = bones
sk.save(os.path.join(MDL, 'model.nusktb'))
print('SQUELETTE', len(bones), 'os ; racine', bones[0].name)
err = max(np.abs(WORLD[n] - BR[n].world).max() for n in order)
print('ecart matrices recalculees / lues : %.4f' % err)

# ------------------------------------------------------------------ v0.5 : squelette « exo » (EXO=1)
# Méthode des skins aux proportions différentes (Magic Exo Skel Maker du plugin Blender smash-ultimate-blender) :
# - le squelette du perso de base (Mario) est gardé TEL QUEL (noms, parents, poses de repos) : le jeu le retrouve
#   comme pour n'importe quel skin ;
# - les os de REX sont ajoutés à part, préfixés « H_Exo_ » (même hiérarchie que dans REX), la racine (HipN) sous Hip ;
#   un os de REX associé à un os de Mario (Hip, Waist, Bust, Head, épaules, bras, mains, doigts, jambes, pieds) prend
#   l'ORIENTATION de repos de l'os de Mario, à SA position à lui (ses proportions sont gardées) ;
# - model.nuhlpb : celui de Mario + une contrainte « copie la rotation » par paire (os de Mario -> os H_Exo_), mêmes
#   réglages que le plugin Blender (type 1, axes 1/1/1, quaternions neutres, -180..180) : les animations de Mario
#   font bouger ses os à lui ;
# - le maillage de Rock est lié aux os H_Exo_ (sommets inchangés : au repos, c'est exactement lui).
EXO = os.environ.get('EXO') == '1'
UNAME_INF = dict(UNAME)
DEFORM = {}                                          # os REX -> déformation de mise en pose (EXO)
EXO_INFO = {}
if EXO:
    base_sk = S.skel_data.read_skel(os.path.join(REF, 'model.nusktb'))
    bones = list(base_sk.bones)
    BN = [b.name for b in bones]
    BW = [None] * len(bones)

    def bw(i):
        if BW[i] is None:
            l_ = np.array(bones[i].transform, dtype=float).T
            p_ = bones[i].parent_index
            BW[i] = (bw(p_) @ l_) if p_ is not None and p_ >= 0 else l_
        return BW[i]
    for i in range(len(bones)):
        bw(i)
    SMW = {BN[i]: BW[i] for i in range(len(bones))}
    PAIR = {}
    for un_, bn_ in CORE.items():
        if un_ in SMW and un_ not in ('Trans', 'Rot', 'XRot', 'Throw') and bn_ in BR:
            PAIR[bn_] = un_
    # os de REX sous HipN (le maillage n'utilise que ceux-là)
    under = []
    for n in order:
        a_ = n
        while a_ and a_ != 'HipN':
            a_ = BR[a_].parent if BR[a_].parent and BR[a_].parent != 'TopN' else None
        if a_ == 'HipN':
            under.append(n)
    # v0.5 : MISE EN POSE avant de lier (comme on le fait dans Blender avant le Magic Exo Skel Maker) : Rock est en
    # « A » (bras à 45°, jambes écartées), Mario en « T » (bras à l'horizontale, jambes droites). Chaque segment de
    # Rock est tourné (autour de son articulation, longueurs gardées) pour pointer comme celui de Mario ; le maillage
    # suit (mélange des poids). Sans ça, les animations de Mario tordraient ses bras et ses jambes de 45°.
    ALIGN = [('HipN', 'WaistN', 'Hip', 'Waist'), ('WaistN', 'BustN', 'Waist', 'Bust'), ('BustN', 'NeckN', 'Bust', 'Neck'),
             ('NeckN', 'HeadN', 'Neck', 'Head')]
    for sd_ in 'LR':
        ALIGN += [('%sShoulderN' % sd_, '%sShoulderJ' % sd_, 'Clavicle' + sd_, 'Shoulder' + sd_),
                  ('%sShoulderJ' % sd_, '%sArmJ' % sd_, 'Shoulder' + sd_, 'Arm' + sd_),
                  ('%sArmJ' % sd_, '%sHandN' % sd_, 'Arm' + sd_, 'Hand' + sd_),
                  ('%sLegJ' % sd_, '%sKneeJ' % sd_, 'Leg' + sd_, 'Knee' + sd_),
                  ('%sKneeJ' % sd_, '%sFootJ' % sd_, 'Knee' + sd_, 'Foot' + sd_),
                  ('%sFootJ' % sd_, '%sToeN' % sd_, 'Foot' + sd_, 'Toe' + sd_)]
        for k_, f_ in ((1, '1st'), (2, '2nd'), (3, '3rd'), (4, '4th'), (5, 'Thumb')):
            ALIGN.append(('%s%sNa' % (sd_, f_), '%s%sNb' % (sd_, f_), 'Finger%s%d1' % (sd_, k_), 'Finger%s%d2' % (sd_, k_)))
            ALIGN.append(('%s%sNb' % (sd_, f_), '%s%sNc' % (sd_, f_), 'Finger%s%d2' % (sd_, k_), 'Finger%s%d3' % (sd_, k_)))
    ALN = {a: (b, c, d) for a, b, c, d in ALIGN if a in BR and b in BR and c in SMW and d in SMW}

    def _rotfromto(u, v):
        u = u / np.linalg.norm(u)
        v = v / np.linalg.norm(v)
        c_ = float(np.dot(u, v))
        ax = np.cross(u, v)
        s_ = float(np.linalg.norm(ax))
        if s_ < 1e-8:
            return np.eye(3)
        ax = ax / s_
        K = np.array([[0, -ax[2], ax[1]], [ax[2], 0, -ax[0]], [-ax[1], ax[0], 0]])
        ang = np.arctan2(s_, c_)
        return np.eye(3) + np.sin(ang) * K + (1 - np.cos(ang)) * (K @ K)
    NW = {}
    for n in under:
        if n == 'HipN':
            base_ = WORLD[n].copy()
        else:
            pn_ = BR[n].parent
            base_ = NW[pn_] @ np.linalg.inv(WORLD[pn_]) @ WORLD[n]
        if n in ALN:
            ch_, s1_, s2_ = ALN[n]
            ch_pos = (base_ @ np.linalg.inv(WORLD[n]) @ WORLD[ch_])[:3, 3]
            cur = ch_pos - base_[:3, 3]
            tgt = SMW[s2_][:3, 3] - SMW[s1_][:3, 3]
            if np.linalg.norm(cur) > 1e-4 and np.linalg.norm(tgt) > 1e-4:
                R4 = np.eye(4)
                R4[:3, :3] = _rotfromto(cur, tgt)
                T1, T2 = np.eye(4), np.eye(4)
                T1[:3, 3] = base_[:3, 3]
                T2[:3, 3] = -base_[:3, 3]
                base_ = T1 @ R4 @ T2 @ base_
        NW[n] = base_
    DEFORM.update({n: NW[n] @ np.linalg.inv(WORLD[n]) for n in under})
    _hd = NW['HeadN'][:3, 3] if 'HeadN' in NW else None
    print('MISE EN POSE (A -> T de Mario) : %d segments orientes ; tete a %s' % (len(ALN), np.round(_hd, 1) if _hd is not None else '?'))
    EXW = {}
    for n in under:
        Wr = NW[n].copy()
        if n in PAIR:
            R_ = SMW[PAIR[n]][:3, :3]
            R_ = R_ / np.linalg.norm(R_, axis=0)
            Wn = np.eye(4)
            Wn[:3, :3] = R_
            Wn[:3, 3] = Wr[:3, 3]
        else:
            Wn = np.eye(4)
            Wn[:3, :3] = Wr[:3, :3] / np.where(np.linalg.norm(Wr[:3, :3], axis=0) > 1e-6, np.linalg.norm(Wr[:3, :3], axis=0), 1.0)
            Wn[:3, 3] = Wr[:3, 3]
        EXW[n] = Wn
        par = BR[n].parent if n != 'HipN' else None
        if n == 'HipN':
            pi, pw = BN.index('Hip'), SMW['Hip']
        else:
            pi, pw = BN.index('H_Exo_' + par), EXW[par]
        loc_ = np.linalg.inv(pw) @ Wn
        bones.append(S.skel_data.BoneData('H_Exo_' + n, np.ascontiguousarray(loc_.T, dtype=np.float32), pi))
        BN.append('H_Exo_' + n)
    sk = S.skel_data.SkelData()
    sk.major_version, sk.minor_version = base_sk.major_version, base_sk.minor_version
    sk.bones = bones
    sk.save(os.path.join(MDL, 'model.nusktb'))
    for n in BR:
        UNAME_INF[n] = 'H_Exo_' + n if n in under else {'TransN': 'Trans', 'XRotN': 'Rot', 'YRotN': 'Rot'}.get(n, 'Hip')
    EXO_INFO = {'paires': PAIR, 'exo': {'H_Exo_' + n: n for n in under},
                # v0.5 : pour l'étape 3 (animations de REX vers le squelette de Mario) : pose de repos mise en « T »
                # (NW) et pose de repos de REX (WORLD) de chaque os de Rock
                'nw': {n: NW[n].tolist() for n in under}, 'rest': {n: WORLD[n].tolist() for n in under}}
    print('SQUELETTE EXO : %d os de Mario + %d os H_Exo_ (%d associes a un os de Mario)' % (len(base_sk.bones), len(under), sum(1 for n in under if n in PAIR)))

# ------------------------------------------------------------------ poids (NodeTree / NodeMix)
INF = {}                                            # n° de matrice -> [(nom d'os, poids)]
for m_id, n in nid.items():
    INF[m_id] = [(n, 1.0)]
for dn, do in dict_entries(dd, mo + SEC['defs']):
    if dn != 'NodeMix':
        continue
    p = do
    while dd[p] != 1:
        c = dd[p]
        if c == 3:
            mid, cnt = struct.unpack_from('>HB', dd, p + 1)
            ws = []
            for i in range(cnt):
                ni, w = struct.unpack_from('>Hf', dd, p + 4 + 6 * i)
                ws.append((nid.get(ni, 'TransN'), w))
            INF[mid] = ws
            p += 4 + 6 * cnt
        elif c == 5:
            mid, bi = struct.unpack_from('>HH', dd, p + 1)
            INF.setdefault(mid, [(by_index[bi].name if bi in by_index else 'TransN', 1.0)])
            p += 5
        else:
            break
single = {m for m, ws in INF.items() if len(ws) == 1 and m in nid}

# ------------------------------------------------------------------ textures
TEXN = [n for n, _ in dict_entries(dd, mo + SEC['tex'])] if SEC['tex'] else []
TEX = {}
for ent in arc_entries(raw):
    if ent[4][:4] != b'bres':
        continue
    bf = brres_files(ent[4])
    pals = {n: tex0.palette(ent[4], o) for n, o in bf.get('Palettes(NW4R)', [])}
    for n, o in bf.get('Textures(NW4R)', []):
        if n in TEX:
            continue
        try:
            TEX[n] = tex0.read_tex0(ent[4], o, pals.get(n)).convert('RGBA')
        except Exception as e:  # noqa: BLE001
            print('texture illisible', n, e)
print('TEXTURES', {n: TEX[n].size for n in TEX})


def mat_textures(moff):
    """références de textures d'un matériau : on cherche les entrées dont le nom pointe sur une texture connue"""
    size = struct.unpack_from('>i', dd, moff)[0]
    found = []
    for e in range(moff, moff + max(0, min(size, 0x1000)) - 4, 4):
        so = struct.unpack_from('>i', dd, e)[0]
        if 0 < so < 0x200000 and e + so < len(dd):
            try:
                s = dd[e + so:dd.index(b'\0', e + so)].decode('latin1')
            except ValueError:
                continue
            if s in TEX and s not in found:
                found.append(s)
    return found


MATS = [(n, o) for n, o in dict_entries(dd, mo + SEC['mats'])]
MATTEX = {n: mat_textures(o) for n, o in MATS}
for n, t in MATTEX.items():
    print('MATERIAU', n, t)

# ------------------------------------------------------------------ maillages
M = mdl0.read_model(dd, mo)
DRAWS = []                                          # (opaque/transparent, matériau, objet, os de visibilité)
for dn in ('DrawOpa', 'DrawXlu'):
    for n_, do in dict_entries(dd, mo + SEC['defs']):
        if n_ != dn:
            continue
        p = do
        while dd[p] == 4:
            mi, oi, bi = struct.unpack_from('>HHH', dd, p + 1)
            DRAWS.append((dn == 'DrawXlu', mi, oi, by_index[bi].name if bi in by_index else None))
            p += 8
A = S.mesh_data


def attr(name, data):
    a = A.AttributeData(name)
    a.data = np.asarray(data, dtype=np.float32)
    return a


GEO = {}


def geometry(oi):
    if oi in GEO:
        return GEO[oi]
    on, ob = M['objs'][oi]
    P = M['arrays']['pos'][ob['ids'][0]][0]
    N = M['arrays']['nrm'][ob['ids'][1]][0] if ob['ids'][1] >= 0 else None
    U = M['arrays']['uv'][ob['ids'][4]][0] if ob['ids'][4] >= 0 else None
    pos, nrm, uv, infl = [], [], [], []
    for t in ob['tris']:
        for v in (t[0], t[2], t[1]):                # sens des triangles : Brawl -> Ultimate
            m_id = v.get('bone', ob['node'])
            ws = INF.get(m_id, [(nid.get(ob['node'], 'TransN'), 1.0)])
            pp = np.r_[P[v['pos']], 1.0]
            nn = N[v['nrm']][:3] if N is not None and 'nrm' in v else np.array([0, 1.0, 0])
            if m_id in single or ob['node'] >= 0 and 'pmi' not in v:
                W = BR[ws[0][0]].world
                pp = W @ pp
                nn = W[:3, :3] @ nn
            if DEFORM:
                # v0.5 : mise en pose du maillage (mélange des déformations de ses os)
                ws_ = sorted(ws, key=lambda x: -x[1])[:4]
                t_ = sum(w for _, w in ws_) or 1
                pp2 = np.zeros(4)
                nn2 = np.zeros(3)
                for b_, w_ in ws_:
                    D_ = DEFORM.get(b_)
                    if D_ is None:
                        pp2 += (w_ / t_) * pp
                        nn2 += (w_ / t_) * nn
                    else:
                        pp2 += (w_ / t_) * (D_ @ pp)
                        nn2 += (w_ / t_) * (D_[:3, :3] @ nn)
                pp, nn = pp2, nn2
            pos.append(pp[:3])
            nrm.append(nn / (np.linalg.norm(nn) or 1))
            uv.append(U[v['uv0']] if U is not None and 'uv0' in v else np.zeros(2))
            ws = sorted(ws, key=lambda x: -x[1])[:4]
            s_ = sum(w for _, w in ws) or 1
            infl.append([(UNAME_INF.get(b, 'Trans'), w / s_) for b, w in ws])
    GEO[oi] = (on, np.array(pos), np.array(nrm), np.array(uv), infl)
    return GEO[oi]


def kind_of(mname, transparent):
    if transparent:
        return 'alpha'
    return 'opaque'


objects, labels = [], []
vis_of, kinds = {}, {}
EYE_TILES = {}
used_mats = {}
names_used = {}
for transparent, mi, oi, vb in DRAWS:
    on, pos, nrm, uv, infl = geometry(oi)
    nv = len(pos)
    if nv == 0:
        continue
    mname = M['mat_names'][mi]
    if vb in os.environ.get('DROP_VIS', '').split(',') and vb:
        # v0.5 : buster, yeux jaunes, sabre dans le dos retirés pour l'instant (pas de pistes de visibilité : avec,
        # Zero restait invisible en jeu) ; ils reviendront avec ses spéciaux, pilotés par le plugin
        print('OBJET RETIRE', on, mname, vb)
        continue
    oname = on if on not in names_used else '%s_%s' % (on, (vb or 'b').replace('M', '').lower())
    names_used[oname] = 1
    o = A.MeshObjectData(oname, 0)
    o.positions = [attr('Position0', pos)]
    o.normals = [attr('Normal0', np.c_[nrm, np.zeros(nv)])]
    idx = np.arange(nv, dtype=np.uint32)
    try:
        tg = A.calculate_tangents_vec4(pos, nrm, uv, idx)
    except Exception:  # noqa: BLE001
        up = np.where(np.abs(nrm[:, 1:2]) > 0.9, np.array([[1, 0, 0]]), np.array([[0, 1, 0]]))
        tg = np.c_[np.cross(up, nrm), np.ones(nv)]
    o.tangents = [attr('Tangent0', tg)]
    if os.environ.get('EYE_ATLAS') == '1' and mname.startswith('Eye') and len(uv):
        # Super Shadow : les UV de l'œil couvrent plusieurs « cases » de texture (Brawl ne répète pas la texture,
        # Ultimate si -> iris en double). Grande texture couvrant toutes ces cases, iris seulement dans la case
        # d'origine (0..1), le reste blanc ; UV recalées dessus : un seul iris, à sa place de Brawl
        lo = np.floor(uv.min(0)).astype(int)
        n = np.maximum(np.ceil(uv.max(0)).astype(int) - lo, 1)
        EYE_TILES[mname] = (lo, n)
        uv = (uv - lo) / n
        print('YEUX cases', mname, 'debut', lo.tolist(), 'nombre', n.tolist())
    elif os.environ.get('EYE_MIX') == '1' and os.environ.get('EYE_CLAMP') != '1' and mname.startswith('Eye') and len(uv):
        # Super Shadow : les UV des yeux couvrent ~2x2 (Brawl les recadre par matrice de texture) ; en Ultimate la
        # texture se répétait -> plusieurs yeux. Recadrées sur 0..1 : un seul iris par œil
        lo, hi = uv.min(0), uv.max(0)
        uv = (uv - lo) / np.where(hi - lo > 1e-6, hi - lo, 1)
    o.texture_coordinates = [attr('map1', uv)]
    o.color_sets = [attr('colorSet1', np.full((nv, 4), 0.5))]
    o.vertex_indices = idx
    per_bone = {}
    for vi, ws in enumerate(infl):
        for b, w in ws:
            per_bone.setdefault(b, []).append(A.VertexWeight(vi, float(w)))
    o.bone_influences = [A.BoneInfluence(b, l) for b, l in per_bone.items()]
    used_mats[mname] = MATTEX.get(mname, [])
    kinds[mname] = kind_of(mname, transparent)
    vis_of[oname] = UNAME.get(vb, vb)
    objects.append(o)
    labels.append(mname)
    print('OBJET %-24s %5d triangles  materiau %-30s %-7s textures %s visibilite %s' % (oname, nv // 3, mname, kinds[mname], used_mats[mname], vis_of[oname]))

ref_mesh = S.mesh_data.read_mesh(os.path.join(REF, 'model.numshb'))
ref_mesh.objects = objects
ref_mesh.save(os.path.join(MDL, 'model.numshb'))
modl = S.modl_data.read_modl(os.path.join(REF, 'model.numdlb'))
modl.model_name = 'model'
modl.skeleton_file_name = 'model.nusktb'
modl.material_file_names = ['model.numatb']
modl.mesh_file_name = 'model.numshb'
modl.entries = [S.modl_data.ModlEntryData(o.name, o.subindex, lab) for o, lab in zip(objects, labels)]
modl.save(os.path.join(MDL, 'model.numdlb'))
try:
    S.meshex_data.MeshExData.from_mesh_objects(objects).save(os.path.join(MDL, 'model.numshexb'))
except Exception as e:  # noqa: BLE001
    print('numshexb non genere :', e)

# os d'aide de Roy (nuhlpb) : ils visent des os de Roy absents ici -> fichier vide ; xmb / nusrcmdlb repris
import shutil  # noqa: E402
try:
    shutil.copy(os.path.join(REF, 'model.nuhlpb'), os.path.join(MDL, 'model.nuhlpb'))
    print('nuhlpb du perso de base repris')
    if EXO:
        hl = S.hlpb_data.read_hlpb(os.path.join(REF, 'model.nuhlpb'))
        cons = list(hl.orient_constraints)
        _par = {b.name: (sk.bones[b.parent_index].name if b.parent_index is not None else None) for b in sk.bones}
        k_ = 0
        for bn_, un_ in sorted(EXO_INFO['paires'].items()):
            if _par.get(un_) is None:
                continue
            cons.append(S.hlpb_data.OrientConstraintData('nuHelperBoneRotateInterp%d' % (3000 + k_), _par[un_], _par[un_], un_,
                                                         'H_Exo_' + bn_, 1, [1.0, 1.0, 1.0], [0.0, 0.0, 0.0, 1.0],
                                                         [0.0, 0.0, 0.0, 1.0], [-180.0, -180.0, -180.0], [180.0, 180.0, 180.0]))
            k_ += 1
        hl.orient_constraints = cons
        hl.save(os.path.join(MDL, 'model.nuhlpb'))
        print('nuhlpb EXO : %d contraintes de Mario + %d copies de rotation vers les os H_Exo_' % (len(cons) - k_, k_))
except Exception as e:  # noqa: BLE001
    print('nuhlpb non ecrit :', e)
for f in ('model.xmb', 'model.nusrcmdlb'):
    if os.path.exists(os.path.join(REF, f)):
        shutil.copy(os.path.join(REF, f), os.path.join(MDL, f))

# ------------------------------------------------------------------ matériaux + textures
# v0.3 : rendu façon Ultimate (Smush Material Research) : couleurs d'origine (pas éclaircies : leçon de Black Shadow),
# textures 2x, relief (NOR) tiré de la peinture, PRM par matière (armure brillante, combinaison satinée, cheveux,
# peau), lame du Z-Saber et lumières (orbes, cristal, bottes) en émission (shader des yeux lumineux de Roy).
from scipy import ndimage  # noqa: E402
ref_matl = os.path.join(REF, 'model.numatb')
_labs = [e.material_label for e in S.matl_data.read_matl(ref_matl).entries]
_opa = next((l for l in _labs if l.startswith('def_')), None) or next((l for l in _labs if l.startswith('skin_')), _labs[0])
_alp = next((l for l in _labs if l.startswith('alp_')), _opa)
TEMPLATE = {'opaque': _opa, 'alpha': _alp, 'lame': _alp, 'emit': _opa}
print('MATERIAUX MODELES', TEMPLATE)


def template(kind):
    m = S.matl_data.read_matl(ref_matl)
    for e in m.entries:
        if e.material_label == TEMPLATE[kind]:
            return e
    return m.entries[0]


texfiles = {}


def save_png(name, im):
    if name not in texfiles:
        im.save(os.path.join(PNG, name + '.png'))
        texfiles[name] = 1
    return name


def arr(im):
    return np.asarray(im.convert('RGBA'), dtype=float) / 255.0


def img(a):
    return Image.fromarray((np.clip(a, 0, 1) * 255 + 0.5).astype(np.uint8), 'RGBA')


def up2(im):
    from PIL import ImageFilter
    if max(im.size) >= 1024:
        return im
    im = im.resize((im.width * 2, im.height * 2), Image.LANCZOS)
    return im.filter(ImageFilter.UnsharpMask(radius=2, percent=50, threshold=2))


def lum(c):
    return c[..., 0] * 0.299 + c[..., 1] * 0.587 + c[..., 2] * 0.114


def nor_from(col, strength=1.2, blur=1.6):
    """relief tiré de la peinture (clair = bosse) ; NOR d'Ultimate : RG normale, B 1, A cavité"""
    h = ndimage.gaussian_filter(lum(col), blur)
    dx, dy = ndimage.sobel(h, axis=1), ndimage.sobel(h, axis=0)
    n = np.dstack([-dx * strength, dy * strength, np.ones_like(h)])
    n /= np.linalg.norm(n, axis=-1, keepdims=True)
    cav = np.clip(1 + (h - ndimage.gaussian_filter(h, 6)) * 2.5, 0.6, 1.0)
    return np.dstack([n[..., 0] * 0.5 + 0.5, n[..., 1] * 0.5 + 0.5, np.ones_like(h), cav])


def ao_from(col, lo=0.8):
    a = ndimage.gaussian_filter(lum(col), 6)
    a = (a - a.min()) / max(a.max() - a.min(), 1e-4)
    return lo + (1 - lo) * a


# matière -> (métal, rugosité, reflet) ; reflet = alpha du PRM (x0,2 dans le shader, 0,16 par défaut)
def matter(mname):
    if 'Hair' in mname:
        return 0.0, 0.42, 0.35
    if 'Face' in mname or 'Eye' in mname:
        return 0.0, 0.6, 0.22
    if 'Gray_Ref' in mname:
        return 0.55, 0.3, 0.5           # pièces grises métalliques (bras, casque)
    if '_Ref' in mname:
        return 0.0, 0.28, 0.55          # armure rouge / blanche : laquée, très brillante
    if 'Gray' in mname:
        return 0.0, 0.5, 0.3            # combinaison noire / grise : satinée
    return 0.0, 0.45, 0.35


def eye_texture(texs):
    """yeux : blanc, iris puis ombre superposés (Brawl les mélange avec trois jeux d'UV)"""
    base = TEX[texs[-1]].copy()
    for t in (texs[1], texs[0]):
        layer = TEX[t].resize(base.size)
        base.alpha_composite(layer)
    return base


def blade_texture(t):
    """lame du Z-Saber : la texture de Brawl est une forme en alpha, sa couleur verte venait du matériau"""
    a = np.asarray(t, dtype=float) / 255
    k = np.clip(a[..., 3] * 0.6 + a[..., :3].mean(-1) * 0.8, 0, 1)
    rgb = np.stack([0.25 + 0.75 * k ** 3, 0.85 + 0.15 * k, 0.35 + 0.65 * k ** 3], -1)
    out = np.dstack([rgb, np.clip(0.35 + 0.65 * a[..., 3], 0, 1)])
    return Image.fromarray((out * 255).astype(np.uint8), 'RGBA')


def is_emit(mname):
    return 'Emission' in mname or 'Gem' in mname


entries = []
for mname, texs in used_mats.items():
    kind = kinds[mname]
    if kind == 'opaque' and is_emit(mname):
        kind = kinds[mname] = 'emit'
    e = template(kind)
    e.material_label = mname
    if not texs:
        col = save_png(PFX + '_white_col', Image.new('RGBA', (8, 8), (200, 200, 200, 255)))
    elif os.environ.get('EYE_MIX') == '1' and mname.startswith('Eye') and len(texs) >= 2:
        # Super Shadow : yeux de Brawl = blanc + iris + reflet superposés -> une seule texture
        base = Image.new('RGBA', TEX[texs[1]].size, (255, 255, 255, 255))
        for t in (texs[1], texs[0]):
            base.alpha_composite(TEX[t].resize(base.size))
        if mname in EYE_TILES:
            (lu, lv), (nu, nv_) = EYE_TILES[mname]
            W, H = base.size
            grand = Image.new('RGBA', (W * nu, H * nv_), (255, 255, 255, 255))
            grand.alpha_composite(base, (-lu * W, -lv * H))
            base = grand.resize((512, 512), Image.LANCZOS)
        k = float(os.environ.get('EYE_SCALE', '1'))
        if k < 1:
            # iris à sa taille de Brawl (les UV de l'œil y couvrent ~2x la texture) : centré, entouré de blanc
            W, H = base.size
            grand = Image.new('RGBA', (W * 2, H * 2), (255, 255, 255, 255))
            n = (int(W * 2 * k), int(H * 2 * k))
            grand.alpha_composite(base.resize(n, Image.LANCZOS), ((W * 2 - n[0]) // 2, (H * 2 - n[1]) // 2))
            base = grand
        col = save_png(PFX + '_' + mname.lower() + '_col', up2(base))
    elif kind == 'lame':
        col = save_png(PFX + '_%s_col' % texs[0].lower(), blade_texture(TEX[texs[0]]))
    elif os.environ.get('ENV_TINT') == '1' and 'Mask' in texs[0] and len(texs) >= 2:
        # Super Shadow : dans Brawl le masque (blanc + mèches) est multiplié par la sphère d'environnement dorée ;
        # sans elle le corps sortait blanc et éblouissant -> masque x couleur moyenne de l'environnement
        a = np.asarray(TEX[texs[0]].convert('RGBA'), dtype=float)
        env = np.asarray(TEX[texs[1]].convert('RGB'), dtype=float).reshape(-1, 3).mean(0) / 255
        a[..., :3] *= env * float(os.environ.get('ENV_GAIN', '0.9'))
        col = save_png(PFX + '_' + texs[0].lower().replace(' ', '_') + '_col', up2(Image.fromarray(a.clip(0, 255).astype(np.uint8), 'RGBA')))
    else:
        col = save_png(PFX + '_' + texs[0].lower().replace(' ', '_') + '_col', up2(TEX[texs[0]]))
    c = arr(Image.open(os.path.join(PNG, col + '.png')))
    nor = save_png(col.replace('_col', '_nor'), img(nor_from(c))) if kind in ('opaque', 'alpha') and c.shape[0] > 16 else PFX + '_nor'
    if nor == PFX + '_nor':
        save_png(PFX + '_nor', Image.new('RGBA', (8, 8), (128, 128, 255, 255)))
    met, rough, spec = matter(mname)
    if kind in ('emit', 'lame'):
        prm = PFX + '_prm'                  # le shader d'émission n'a pas de PRM
        save_png(PFX + '_prm', Image.new('RGBA', (8, 8), (0, 165, 255, 80)))
    else:
        # PRM partagé par texture + matière, en 256 au plus (place sur la carte SD : l'occlusion est floue de toute façon)
        one = np.ones(c.shape[:2])
        prm_a = np.dstack([met * one, rough * one, ao_from(c) if c.shape[0] > 16 else one, spec * one])
        pim = img(prm_a)
        if max(pim.size) > 256:
            pim = pim.resize((min(256, pim.width), min(256, pim.height)), Image.LANCZOS)
        prm = save_png('%s_m%02d_r%02d_s%02d_prm' % (col.replace('_col', ''), met * 100, rough * 100, spec * 100), pim)
    for t in e.textures:
        n = t.param_id.name
        if n == 'Texture0':
            t.data = col
        elif n == 'Texture4':
            t.data = nor
        elif n == 'Texture6':
            t.data = prm
        elif n in ('Texture5', 'Texture14'):
            t.data = col                      # émission : la couleur elle-même
    if kind in ('lame', 'emit'):
        for v in e.vectors:
            if v.param_id.name == 'CustomVector3':
                v.data = [1.6, 1.6, 1.6, 1.0] if kind == 'lame' else [1.15, 1.15, 1.15, 1.0]
    if os.environ.get('EYE_CLAMP') == '1' and mname.startswith('Eye'):
        # Super Shadow : comme dans Brawl, UV d'origine (matrice identité) et texture non répétée : l'iris n'occupe
        # que la partie de l'œil côté nez, à sa vraie taille, le reste prend le bord blanc de la texture
        try:
            for smp in e.samplers:
                sd = smp.data
                sd.wraps = sd.wrapt = sd.wrapr = S.matl_data.WrapMode.ClampToEdge
                smp.data = sd
            print('YEUX sans repetition', mname)
        except Exception as ex:  # noqa: BLE001
            print('YEUX : echec du mode de bord', mname, ex)
    entries.append(e)
    print('MATERIAU ULTIMATE', mname, kind, col, nor, prm, (met, rough, spec))
matl = S.matl_data.read_matl(ref_matl)
matl.entries = entries
matl.save(os.path.join(MDL, 'model.numatb'))
json.dump({'materiaux': used_mats, 'types': kinds, 'textures': sorted(texfiles), 'os': [b.name for b in bones],
           'u2b': {UNAME[n]: n for n in order}, 'visibilite': vis_of, 'exo': EXO_INFO},
          open(os.path.join(OUT, 'model.json'), 'w'), indent=1, ensure_ascii=False)
print('MODELE ok :', len(objects), 'objets,', len(entries), 'materiaux,', len(texfiles), 'textures')
