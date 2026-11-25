"""Unreal Introduction Map (Brawl, GameBanana 692367) en variante Stage Alts de Smashville (stage/animal_village,
forme normal_sNN), v1. Le Champ de bataille utilise ses modèles au démarrage (plates-formes) ; Smashville a un décor
fixe où cacher les ensembles par un numdlb à 0 entrée est validé (Sleepless Lamp).

usage : python3 ue_sv.py <STGBATTLEFIELD (DOLPHIN).pac> <réf complète .../stage/animal_village (387162)>
                         <liste des fichiers du jeu stage/animal_village/normal/ (Hashes_FullPath)>
                         <numatb modèle (bg2_set de 392643)> <xvillage00.lvd HDR (normal)> <sortie mod>

Méthode de Sleepless Lamp (lamp_sv.py) dans un dossier de variante (générateur de CSK) :
- décor de la carte Unreal dans stc_bg_set (numshb / numdlb / numatb / numshexb de 387162 comme base ; squelette du jeu
  partagé), maillages regroupés par texture, sans os, matériau « Bottom » ; textures sous de nouveaux noms ;
- tous les autres ensembles cachés par le numdlb à 0 entrée de 387162 (comme Lamp) ;
- collisions de la carte (3 niveaux de sol, rebords aux deux bouts, bloc solide) dans le lvd de Smashville d'HDR ;
  plate-forme mobile enterrée (comme Lamp) ;
- config.json : share_to_vanilla de chaque fichier du jeu de la forme normal non fourni ; image de l'écran des terrains
  stage_1/2/4 fournies, 0 et 3 partagées.
"""
import os, shutil, subprocess, sys
import numpy as np
import ssbh_data_py as S

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import brawl_stage  # noqa: E402

PAC, REF, LISTE, MATL_MODELE, LVD, OUT = sys.argv[1:7]
A = S.mesh_data
FORME = os.environ.get('UE_FORME', 'normal_s01')
RM = os.path.join(REF, 'normal', 'model')
ST = os.path.join(OUT, 'stage', 'animal_village', FORME)
if os.path.isdir(os.path.join(OUT, 'stage')):
    shutil.rmtree(os.path.join(OUT, 'stage'))
ECH = float(os.environ.get('UE_ECHELLE', '1.0'))
PFX = 'ue_'
DOS = 'stc_bg_set'
# v6 : sol et murs (objets à texture d'ombre) dans le modèle du terrain principal, qui reçoit les ombres des persos
DOS_SOL = os.environ.get('UE_DOS_SOL', 'stc_mainstage_set')

TEX, models = brawl_stage.load(PAC)
SANS = {'EffCommonShield', 'Dummy', 'MShadow'}
SANS_TEX = set()
LUM = float(os.environ.get('UE_LUM', '1.0'))
SAT = float(os.environ.get('UE_SAT', '1.0'))


# v2 : textures éclaircies (UE_LUM) pour retrouver les couleurs des images de référence du joueur (gris ~200, bleu
# ~ (83, 152, 208) au soleil) ; ciel et nuages gardés tels quels
SAUF = set(x for x in os.environ.get('UE_LUM_SAUF', '').split(',') if x)


def ajuste(im, t=None):
    """im : tableau RGBA flottant 0-255 -> saturation (SAT) et luminosité (LUM)"""
    if t in SAUF:
        return im
    rgb = im[:, :, :3]
    g = (rgb * np.array([0.299, 0.587, 0.114])).sum(axis=2, keepdims=True)
    im[:, :, :3] = np.clip((g + (rgb - g) * SAT) * LUM, 0, 255)
    return im


# v2 : faux soleil cuit dans colorSet1 (0,502 = neutre) : dessus clair, côtés plus sombres
SOLEIL = np.array([float(v) for v in os.environ.get('UE_SOLEIL', '-0.3,1,0.35').split(',')])
SOLEIL /= np.linalg.norm(SOLEIL)
OMBRE = float(os.environ.get('UE_OMBRE', '1.0'))      # 1 = pas d'ombrage ; 0,55 = côtés à l'ombre à 55 %


# v5 : quadrillage refait en HD façon Unreal (la texture Brawl 512 a 5 carreaux et des bords noirs épais -> gros traits
# baveux en jeu) : panneaux = carreaux Brawl (5 par répétition), séparés par un trait fin, chacun en 4 x 4 petits carreaux
# à lignes très légères ; même gris moyen que l'original
GRILLE = os.environ.get('UE_GRILLE', '1') == '1'


def grille(base, n=2048, panneaux=5, sous=4):
    x = np.arange(n)
    def lignes(pas, larg):
        d = np.minimum(x % pas, pas - (x % pas))
        return d < larg / 2.0
    pas_p = n / panneaux
    maj = lignes(pas_p, 10)
    mi = lignes(pas_p / sous, 4)
    f = np.ones((n, n), dtype=np.float32)
    f[mi[None, :] | mi[:, None]] = 0.8
    f[maj[None, :] | maj[:, None]] = 0.5
    im = np.zeros((n, n, 4), dtype=np.float32)
    im[:, :, :3] = f[:, :, None] * np.asarray(base, dtype=np.float32)[None, None, :3]
    im[:, :, 3] = 255
    return im


def ombrage(n):
    """n : normales (N, 3) Ultimate (y en haut) -> facteur 0..1 (dessus = 1)"""
    d = np.clip(n @ SOLEIL, 0, None) / SOLEIL[1]
    return np.clip(OMBRE + (1 - OMBRE) * d, 0, 1)


garde = []
for _, mn, ol in models:
    if mn in SANS:
        print('ignore (modèle)', mn)
        continue
    for o in ol:
        if not o['tex'] or o['tex'] not in TEX or o['tex'] in SANS_TEX:
            print('ignore', mn, o['name'], o['tex'])
            continue
        garde.append((mn, o))


# v7 (demande du joueur) : murs de devant retirés là où ils cachent les persos (partie gauche x < -140, partie basse à
# droite 111,85 < x < 216 : faces devant le plan de jeu (z > 8) qui dépassent du sol) ; murs invisibles aux deux bouts
# (x -213,48 et 216) au lieu des rebords ; sol de droite arrêté à 216
COUPE = os.environ.get('UE_COUPE', '1') == '1'
X_DROITE = float(os.environ.get('UE_X_DROITE', '240.9'))  # v9 : face du grand bloc blanc de devant (demande du joueur)
# v9 : mur invisible de gauche au vrai bout de la dalle (x -392,45 au lieu de -213,48), sol prolongé jusque-là
X_GAUCHE = float(os.environ.get('UE_X_GAUCHE', '-392.45'))
TOIT = None
ZONE_TOIT = None                  # v11 : (x0, x1, y0, y1, z0, z1) zone couverte par le toit (ombre peinte)
X_BLEU, X_BLEU1, Y_BLEU = -140.0, -110.87, 70.49
OMBRE_TOIT = float(os.environ.get('UE_OMBRE_TOIT', '0.0'))   # v12 : ombre portée par le texel sombre (uv1), plus de multiplicateur


UV_SOMBRE = None


def zone_toit(P):
    """P : (N, 3) coordonnées Brawl -> masque des sommets sous le toit"""
    if ZONE_TOIT is None:
        return np.zeros(len(P), dtype=bool)
    x0, x1, y0, y1, z0, z1 = ZONE_TOIT
    return (P[:, 0] >= x0 - 0.01) & (P[:, 0] <= x1 + 0.01) & (P[:, 1] >= y0) & (P[:, 1] <= y1 + 0.01) & (P[:, 2] >= z0 - 0.01) & (P[:, 2] <= z1 + 0.01)


def uv_sombre():
    """v12 : texel uni et sombre de la carte d'ombres (ombre pleine sous le toit, au lieu de l'éclairage cuit d'origine
    qui laissait des zones claires) : bloc uniforme proche du centile UE_OMBRE_PCT des texels utilisés par le décor"""
    global UV_SOMBRE
    if UV_SOMBRE is None:
        bk_ = np.asarray(TEX['BakeMap'].convert('RGB')).astype(np.float32)
        h_, w_ = bk_.shape[:2]
        lum_ = bk_.mean(2)
        U_ = np.concatenate([o['uv1'].reshape(-1, 2) for _, o in garde if o.get('uv1') is not None])
        rr_ = np.clip((U_[:, 1] % 1.0) * h_, 0, h_ - 1).astype(int)
        cc_ = np.clip((U_[:, 0] % 1.0) * w_, 0, w_ - 1).astype(int)
        cible_ = float(np.percentile(lum_[rr_, cc_], float(os.environ.get('UE_OMBRE_PCT', '15'))))
        best_, buv_ = 1e9, (0.5, 0.5)
        pas_ = max(1, min(h_, w_) // 256)
        for y_ in range(3, h_ - 3, pas_):
            for x_ in range(3, w_ - 3, pas_):
                blk_ = lum_[y_ - 3:y_ + 4, x_ - 3:x_ + 4]
                sc_ = abs(float(blk_.mean()) - cible_) + float(blk_.std()) * 2
                if sc_ < best_:
                    best_, buv_ = sc_, ((x_ + 0.5) / w_, (y_ + 0.5) / h_)
        UV_SOMBRE = buv_
        print('ombre pleine sous le toit : texel %s (luminance visée %.1f, écart %.2f)' % (tuple(round(v, 4) for v in buv_), cible_, best_))
    return UV_SOMBRE


def fac_toit(P):
    """P : (N, 3) coordonnées Brawl -> facteur d'ombre sous le toit (1 ailleurs) : plus sombre au fond qu'à l'ouverture"""
    f = np.ones(len(P))
    if ZONE_TOIT is None or OMBRE_TOIT <= 0:
        return f
    x0, x1, y0, y1, z0, z1 = ZONE_TOIT
    m = (P[:, 0] >= x0 - 0.01) & (P[:, 0] <= x1 + 0.01) & (P[:, 1] >= y0) & (P[:, 1] <= y1 + 0.01) & (P[:, 2] >= z0 - 0.01) & (P[:, 2] <= z1 + 0.01)
    d = np.clip((z1 - P[:, 2]) / max(z1 - z0, 1e-6), 0, 1)
    # v12 : ombre pleine et uniforme sous le toit (demande du joueur), au lieu d'un dégradé
    f[m] = 1 - OMBRE_TOIT
    return f
ZONES_COUPE = [(-1e9, -140.0), (111.85, X_DROITE)]
SOL_B = [(-213.48, 11.44), (-110.73, 11.44), (-87.29, 0.0), (111.85, 0.0), (159.06, -23.60), (180.64, -23.77),
         (182.61, -23.01), (216.80, -23.01), (218.89, -23.82), (266.65, -23.82)]


def _sol_b(x):
    return np.interp(x, [p_[0] for p_ in SOL_B], [p_[1] for p_ in SOL_B])


def _coupe_x(att, xc):
    """att : liste de tableaux (3, k) d'un triangle (pos en premier) -> liste de triangles coupés par x = xc"""
    d = att[0][:, 0] - xc
    if (d >= -1e-6).all() or (d <= 1e-6).all():
        return [att]
    out = []
    for sgn in (1, -1):
        poly = []
        for i in range(3):
            j = (i + 1) % 3
            da, db = d[i] * sgn, d[j] * sgn
            if da >= 0:
                poly.append([a_[i] for a_ in att])
            if (da >= 0) != (db >= 0):
                t = da / (da - db)
                poly.append([a_[i] + (a_[j] - a_[i]) * t for a_ in att])
        for k in range(1, len(poly) - 1):
            out.append([np.array([poly[0][m], poly[k][m], poly[k + 1][m]]) for m in range(len(att))])
    return out


def _coupe_z(att, zc):
    """comme _coupe_x mais par le plan z = zc"""
    d = att[0][:, 2] - zc
    if (d >= -1e-6).all() or (d <= 1e-6).all():
        return [att]
    out = []
    for sgn in (1, -1):
        poly = []
        for i in range(3):
            j = (i + 1) % 3
            da, db = d[i] * sgn, d[j] * sgn
            if da >= 0:
                poly.append([a_[i] for a_ in att])
            if (da >= 0) != (db >= 0):
                t = da / (da - db)
                poly.append([a_[i] + (a_[j] - a_[i]) * t for a_ in att])
        for k in range(1, len(poly) - 1):
            out.append([np.array([poly[0][m], poly[k][m], poly[k + 1][m]]) for m in range(len(att))])
    return out


def _coupe_sol(att):
    """garde la partie du triangle sous le sol de combat (y <= sol(x)) ; sol linéaire dans le morceau"""
    d = att[0][:, 1] - _sol_b(att[0][:, 0])
    if (d <= 1e-6).all():
        return [att]
    if (d >= -1e-6).all():
        return []
    poly = []
    for i in range(3):
        j = (i + 1) % 3
        da, db = -d[i], -d[j]
        if da >= 0:
            poly.append([a_[i] for a_ in att])
        if (da >= 0) != (db >= 0):
            t = da / (da - db)
            poly.append([a_[i] + (a_[j] - a_[i]) * t for a_ in att])
    return [[np.array([poly[0][m], poly[k][m], poly[k + 1][m]]) for m in range(len(att))] for k in range(1, len(poly) - 1)]


if COUPE:
    # v8 : les murs de devant sont arasés à la hauteur du sol (au lieu d'être retirés) : leur face reste jusqu'au sol et
    # bouche le trou noir qui apparaissait au bord du sol (demande du joueur)
    retires = 0
    EMPRISE = []
    for _, o in garde:
        if o['tex'] not in ('Blue', 'Grid'):
            continue
        cles = [k for k in ('pos', 'uv', 'nrm', 'uv1') if o.get(k) is not None]
        neufs = {k: [] for k in cles}
        for i in range(len(o['pos'])):
            morceaux = [[o[k][i].astype(np.float64) for k in cles]]
            for xc in (-140.0, 111.85, 159.06, 180.64, 182.61, 216.8, 218.89, X_DROITE):
                morceaux = [q for m_ in morceaux for q in _coupe_x(m_, xc)]
            # v12 : sol de la partie gauche découpé au mur du fond (z -64,7) pour l'ombre pleine sous le toit
            morceaux = [q for m_ in morceaux for q in (_coupe_z(m_, -64.7) if m_[0][:, 0].mean() < -140.0 else [m_])]
            for m_ in morceaux:
                P = m_[0]
                cx = P[:, 0].mean()
                dedans = any(lo <= cx <= hi for lo, hi in ZONES_COUPE)
                if dedans and P[:, 2].min() >= 8.0 and (P[:, 1] - _sol_b(P[:, 0])).max() > 1.0:
                    retires += 1
                    garde_m = _coupe_sol(m_)
                    EMPRISE.append((P[:, 0].min(), P[:, 0].max(), P[:, 2].min(), P[:, 2].max()))
                else:
                    garde_m = [m_]
                for g_ in garde_m:
                    for k, v in zip(cles, g_):
                        neufs[k].append(v)
        for k in cles:
            o[k] = np.array(neufs[k])
    print('murs de devant arasés au niveau du sol :', retires, 'morceaux de triangles')

# v9 : mur sous la dalle de gauche (il laissait un grand trou) : face avant (z de la dalle) et face du bout gauche,
# du dessous de la dalle jusqu'à y -250 ; quadrillage à la même échelle, éclairage cuit pris sur la face avant de la dalle
if COUPE and os.environ.get('UE_MUR_BAS', '1') == '1':
    og = next(o for _, o in garde if o['tex'] == 'Grid')
    Tg, Ug, U1g = og['pos'], og['uv'], og.get('uv1')
    dens = float(np.median(np.linalg.norm(Ug[:, 1] - Ug[:, 0], axis=1) / (np.linalg.norm(Tg[:, 1] - Tg[:, 0], axis=1) + 1e-9)))
    ng = np.cross(Tg[:, 1] - Tg[:, 0], Tg[:, 2] - Tg[:, 0])
    ng /= np.linalg.norm(ng, axis=1, keepdims=True) + 1e-9
    cg = Tg.mean(1)
    fa = (np.abs(ng[:, 2]) > 0.9) & (cg[:, 0] < -216) & (cg[:, 1] < 15)
    zf = float(Tg[fa][:, :, 2].max()) if fa.any() else 76.6
    yh = float(Tg[fa][:, :, 1].min()) if fa.any() else -0.2
    zb = float(Tg[fa][:, :, 2].min()) if fa.any() else -64.1
    u1 = U1g[fa].reshape(-1, 2).mean(0) if (U1g is not None and fa.any()) else np.array([0.5, 0.5])
    XA, XB, YBAS = X_GAUCHE, float(os.environ.get('UE_MUR_BAS_X', '-140.0')), -250.0

    def rect(p0, p1, p2, p3, nrm, uvf):
        pos = np.array([[p0, p1, p2], [p0, p2, p3]], dtype=np.float64)
        uv = np.array([[uvf(p) for p in t] for t in pos])
        return pos, uv, np.tile(np.array(nrm, dtype=np.float64), (2, 3, 1))
    faces = [rect((XA, YBAS, zf), (XB, YBAS, zf), (XB, yh, zf), (XA, yh, zf), (0, 0, 1), lambda p: (p[0] * dens, -p[1] * dens)),
             rect((XA, YBAS, zb), (XA, YBAS, zf), (XA, yh, zf), (XA, yh, zb), (-1, 0, 0), lambda p: (p[2] * dens, -p[1] * dens))]
    for pos, uv, nrm in faces:
        og['pos'] = np.concatenate([og['pos'], pos])
        og['uv'] = np.concatenate([og['uv'], uv])
        og['nrm'] = np.concatenate([og['nrm'], nrm])
        if og.get('uv1') is not None:
            og['uv1'] = np.concatenate([og['uv1'], np.tile(u1, (2, 3, 1))])
    # v11 : bloc bleu (arche) à droite du toit : bord gauche, bord droit, dessus (pour le toit collé et sa collision)
    _vb = np.concatenate([o['pos'].reshape(-1, 3) for _, o in garde if o['tex'] == 'Blue'])
    _sel = _vb[(_vb[:, 1] > 50) & (_vb[:, 1] < 72) & (_vb[:, 0] > -139.9) & (_vb[:, 0] < -100)]
    if len(_sel):
        X_BLEU, X_BLEU1, Y_BLEU = float(_sel[:, 0].min()), float(_sel[:, 0].max()), float(_sel[:, 1].max())
    print('bloc bleu : x %.2f..%.2f, dessus y %.2f (%d sommets)' % (X_BLEU, X_BLEU1, Y_BLEU, len(_sel)))
    # v10 : mur plein au bout gauche et toit sur la partie gauche (demande du joueur), hauteur du haut du mur du fond
    if os.environ.get('UE_TOIT', '1') == '1':
        YT0, YT1, ZT0, ZT1 = 70.1, 75.0, -64.7, zf
        # v11 : toit prolongé jusqu'au bloc bleu (il restait un espace entre le bout du toit et le bloc, demande du joueur)
        XT1 = float(os.environ.get('UE_TOIT_X', str(max(-140.0, X_BLEU + 0.3))))
        # v12 : le dessus du bloc bleu est arrondi (bord gauche à x -134,4 en haut) -> le toit va jusque-là
        _top = _vb[(_vb[:, 1] > 65) & (_vb[:, 1] < 76) & (_vb[:, 0] > -139.9) & (_vb[:, 0] < -100)]
        if len(_top) and 'UE_TOIT_X' not in os.environ:
            XT1 = max(XT1, float(_top[:, 0].min()) + 0.4)
        lot = [rect((XA, YT1, ZT0), (XT1, YT1, ZT0), (XT1, YT1, ZT1), (XA, YT1, ZT1), (0, 1, 0), lambda p: (p[0] * dens, p[2] * dens)),
               rect((XA, YT0, ZT0), (XA, YT0, ZT1), (XT1, YT0, ZT1), (XT1, YT0, ZT0), (0, -1, 0), lambda p: (p[0] * dens, p[2] * dens)),
               rect((XA, YT0, ZT1), (XT1, YT0, ZT1), (XT1, YT1, ZT1), (XA, YT1, ZT1), (0, 0, 1), lambda p: (p[0] * dens, -p[1] * dens)),
               rect((XA, 11.44, ZT0), (XA, 11.44, ZT1), (XA, YT1, ZT1), (XA, YT1, ZT0), (1, 0, 0), lambda p: (p[2] * dens, -p[1] * dens))]
        for pos, uv, nrm in lot:
            og['pos'] = np.concatenate([og['pos'], pos])
            og['uv'] = np.concatenate([og['uv'], uv])
            og['nrm'] = np.concatenate([og['nrm'], nrm])
            if og.get('uv1') is not None:
                og['uv1'] = np.concatenate([og['uv1'], np.tile(u1, (2, 3, 1))])
        TOIT = [(XT1, YT0), (XA, YT0), (XA, YT1), (XT1, YT1), (XT1, YT0)]
        ZONE_TOIT = (XA, -140.0, 11.0, YT0, ZT0, zf)
        # v11 : pilier quadrillé entre x -140 et le bloc bleu (devant) : on bouche du haut du pilier jusqu'au toit
        _vg = og['pos']
        _cg = _vg.mean(1)
        _pm = (_cg[:, 0] > -140.5) & (_cg[:, 0] < XT1 + 0.5) & (_cg[:, 2] >= 8.0)
        if _pm.any() and XT1 > -140.0:
            ptop = float(_vg[_pm][:, :, 1].max())
            pz0, pz1 = float(_vg[_pm][:, :, 2].min()), float(_vg[_pm][:, :, 2].max())
            if ptop < YT0 - 0.05:
                bouche = [rect((-140.0, ptop, pz1), (XT1, ptop, pz1), (XT1, YT0, pz1), (-140.0, YT0, pz1), (0, 0, 1), lambda p: (p[0] * dens, -p[1] * dens)),
                          rect((-140.0, ptop, pz0), (-140.0, ptop, pz1), (-140.0, YT0, pz1), (-140.0, YT0, pz0), (-1, 0, 0), lambda p: (p[2] * dens, -p[1] * dens)),
                          rect((-140.0, ptop, pz0), (XT1, ptop, pz0), (XT1, YT0, pz0), (-140.0, YT0, pz0), (0, 0, -1), lambda p: (p[0] * dens, -p[1] * dens))]
                for pos, uv, nrm in bouche:
                    og['pos'] = np.concatenate([og['pos'], pos])
                    og['uv'] = np.concatenate([og['uv'], uv])
                    og['nrm'] = np.concatenate([og['nrm'], nrm])
                    if og.get('uv1') is not None:
                        og['uv1'] = np.concatenate([og['uv1'], np.tile(u1, (2, 3, 1))])
            print('pilier : haut y %.2f, z %.1f..%.1f -> bouché jusqu\'au toit (y %.1f)' % (ptop, pz0, pz1, YT0))
        else:
            print('pilier : non trouvé (x -140..%.1f)' % XT1)
    # v12 : joint entre le pilier et la face avant du bloc bleu (le bloc avance jusqu'à z ~100, le toit et le pilier
    # s'arrêtent à z 88,4 : on voyait un trou noir) : colonne du sol au toit, affleurant la face avant du bloc
    if os.environ.get('UE_JOINT', '1') == '1' and TOIT is not None:
        _fr = _vb[(_vb[:, 0] > -139.9) & (_vb[:, 0] < -100) & (_vb[:, 1] > 40) & (_vb[:, 1] < 60)]
        ZJ = float(_fr[:, 2].max()) if len(_fr) else 100.1
        XJ0, XJ1, YJ0 = -140.0, X_BLEU + 0.1, -5.0
        if ZJ > zf + 0.5 and XJ1 > XJ0:
            lot = [rect((XJ0, YJ0, ZJ), (XJ1, YJ0, ZJ), (XJ1, YT1, ZJ), (XJ0, YT1, ZJ), (0, 0, 1), lambda p: (p[0] * dens, -p[1] * dens)),
                   rect((XJ0, YJ0, zf - 0.5), (XJ0, YJ0, ZJ), (XJ0, YT1, ZJ), (XJ0, YT1, zf - 0.5), (-1, 0, 0), lambda p: (p[2] * dens, -p[1] * dens)),
                   rect((XJ0, YT1, zf - 0.5), (XJ0, YT1, ZJ), (XJ1, YT1, ZJ), (XJ1, YT1, zf - 0.5), (0, 1, 0), lambda p: (p[0] * dens, p[2] * dens))]
            for pos, uv, nrm in lot:
                og['pos'] = np.concatenate([og['pos'], pos])
                og['uv'] = np.concatenate([og['uv'], uv])
                og['nrm'] = np.concatenate([og['nrm'], nrm])
                if og.get('uv1') is not None:
                    og['uv1'] = np.concatenate([og['uv1'], np.tile(u1, (2, 3, 1))])
            print('joint pilier / bloc bleu : x %.2f..%.2f, y %.1f..%.1f, z %.1f..%.2f' % (XJ0, XJ1, YJ0, YT1, zf, ZJ))
    # v12 : couloir prolongé loin à gauche (au lieu de la grande face plate de la v11, demande du joueur) : sol de la dalle,
    # face avant + mur dessous, mur du fond, toit (dessus, dessous, face avant), de XL jusqu'au mur du bout gauche
    if os.environ.get('UE_COULOIR', '1') == '1' and TOIT is not None:
        XL = XA - float(os.environ.get('UE_COULOIR_L', '3000'))
        YS_ = 11.44
        lot = [rect((XL, YS_, ZT0), (XL, YS_, zf), (XA, YS_, zf), (XA, YS_, ZT0), (0, 1, 0), lambda p: (p[0] * dens, p[2] * dens)),
               rect((XL, YBAS, zf), (XA, YBAS, zf), (XA, YS_, zf), (XL, YS_, zf), (0, 0, 1), lambda p: (p[0] * dens, -p[1] * dens)),
               rect((XL, YS_, ZT0), (XA, YS_, ZT0), (XA, YT0, ZT0), (XL, YT0, ZT0), (0, 0, 1), lambda p: (p[0] * dens, -p[1] * dens)),
               rect((XL, YT1, ZT0), (XA, YT1, ZT0), (XA, YT1, zf), (XL, YT1, zf), (0, 1, 0), lambda p: (p[0] * dens, p[2] * dens)),
               rect((XL, YT0, ZT0), (XL, YT0, zf), (XA, YT0, zf), (XA, YT0, ZT0), (0, -1, 0), lambda p: (p[0] * dens, p[2] * dens)),
               rect((XL, YT0, zf), (XA, YT0, zf), (XA, YT1, zf), (XL, YT1, zf), (0, 0, 1), lambda p: (p[0] * dens, -p[1] * dens))]
        for pos, uv, nrm in lot:
            og['pos'] = np.concatenate([og['pos'], pos])
            og['uv'] = np.concatenate([og['uv'], uv])
            og['nrm'] = np.concatenate([og['nrm'], nrm])
            if og.get('uv1') is not None:
                og['uv1'] = np.concatenate([og['uv1'], np.tile(u1, (2, 3, 1))])
        ZONE_TOIT = (XL, ZONE_TOIT[1], ZONE_TOIT[2], ZONE_TOIT[3], ZONE_TOIT[4], ZONE_TOIT[5])
        print('couloir prolongé à gauche : x %.0f..%.1f (sol y %.2f, toit %.1f..%.1f, z %.1f..%.1f)' % (XL, XA, YS_, YT0, YT1, ZT0, zf))
    # v12 : relevé du bloc bleu et du pilier près du bout du toit (pour les coller)
    if os.environ.get('UE_RELEVE', '1') == '1':
        for nom_, tx_ in (('Blue', 'Blue'), ('Grid', 'Grid')):
            V_ = np.concatenate([o['pos'].reshape(-1, 3) for _, o in garde if o['tex'] == tx_])
            V_ = V_[(V_[:, 0] > -175) & (V_[:, 0] < -95) & (V_[:, 1] > -5)]
            print('relevé', nom_, len(V_), 'sommets')
            for y0_ in range(-5, 90, 10):
                m_ = (V_[:, 1] >= y0_) & (V_[:, 1] < y0_ + 10)
                if m_.any():
                    W_ = V_[m_]
                    print('   y %3d..%3d : x %.2f..%.2f  z %.2f..%.2f' % (y0_, y0_ + 10, W_[:, 0].min(), W_[:, 0].max(), W_[:, 2].min(), W_[:, 2].max()))
            T_ = np.concatenate([o['pos'] for _, o in garde if o['tex'] == tx_])
            n_ = np.cross(T_[:, 1] - T_[:, 0], T_[:, 2] - T_[:, 0])
            n_ /= np.linalg.norm(n_, axis=1, keepdims=True) + 1e-9
            c_ = T_.mean(1)
            sel_ = (c_[:, 0] > -175) & (c_[:, 0] < -95) & (c_[:, 1] > -5)
            for i_ in np.where(sel_)[0][:400]:
                if abs(n_[i_, 0]) > 0.9 or abs(n_[i_, 2]) > 0.9 or abs(n_[i_, 1]) > 0.9:
                    P_ = T_[i_]
                    print('   face %s n(%.1f,%.1f,%.1f) x %.2f..%.2f y %.2f..%.2f z %.2f..%.2f' % (nom_, n_[i_, 0], n_[i_, 1], n_[i_, 2], P_[:, 0].min(), P_[:, 0].max(), P_[:, 1].min(), P_[:, 1].max(), P_[:, 2].min(), P_[:, 2].max()))
        print('toit y %.1f..%.1f x %.1f..%.1f z %.1f..%.1f + mur plein au bout gauche' % (YT0, YT1, XA, XT1, ZT0, ZT1))
    # v9 : dessus des murs arasés (le creux entre le bord du sol et le mur arasé laissait voir un trou) : bandes
    # horizontales juste sous le sol (y - 0,3, cachées là où le sol existe), sur l'emprise des morceaux retirés
    for lo_, hi_ in ZONES_COUPE:
        em = [e for e in EMPRISE if lo_ <= (e[0] + e[1]) / 2 <= hi_]
        if not em:
            continue
        xa_, xb_ = max(lo_, min(e[0] for e in em)), min(hi_, max(e[1] for e in em))
        za_, zb_ = min(e[2] for e in em), max(e[3] for e in em)
        xs_ = sorted(set([xa_, xb_] + [p_[0] for p_ in SOL_B if xa_ < p_[0] < xb_]))
        for x0_, x1_ in zip(xs_, xs_[1:]):
            y0_, y1_ = float(_sol_b(x0_)) - 0.3, float(_sol_b(x1_)) - 0.3
            pos = np.array([[(x0_, y0_, za_), (x1_, y1_, za_), (x1_, y1_, zb_)], [(x0_, y0_, za_), (x1_, y1_, zb_), (x0_, y0_, zb_)]])
            uv = np.array([[(p_[0] * dens, p_[2] * dens) for p_ in t] for t in pos])
            og['pos'] = np.concatenate([og['pos'], pos])
            og['uv'] = np.concatenate([og['uv'], uv])
            og['nrm'] = np.concatenate([og['nrm'], np.tile(np.array([0.0, 1.0, 0.0]), (2, 3, 1))])
            if og.get('uv1') is not None:
                og['uv1'] = np.concatenate([og['uv1'], np.tile(u1, (2, 3, 1))])
        print('dessus des murs arasés : x %.1f..%.1f z %.1f..%.1f' % (xa_, xb_, za_, zb_))
    print('mur sous la dalle de gauche : x %.1f..%.1f, z %.1f, y %.1f..%.1f (densité uv %.4f)' % (XA, XB, zf, YBAS, yh, dens))

def surface(x, z=0.0):
    """hauteur la plus haute du décor sous y +15 à la verticale de (x, z) (Brawl)"""
    best = None
    for _, o in garde:
        P = o['pos']
        a, b, c = P[:, 0], P[:, 1], P[:, 2]
        v0, v1 = c - a, b - a
        d = v0[:, 0] * v1[:, 2] - v1[:, 0] * v0[:, 2]
        ok = np.abs(d) > 1e-9
        px, pz = x - a[:, 0], z - a[:, 2]
        with np.errstate(divide='ignore', invalid='ignore'):
            u = (px * v1[:, 2] - v1[:, 0] * pz) / d
            v = (v0[:, 0] * pz - px * v0[:, 2]) / d
        m = ok & (u >= -1e-6) & (v >= -1e-6) & (u + v <= 1 + 1e-6)
        y = a[m, 1] + u[m] * v0[m, 1] + v[m] * v1[m, 1]
        y = y[y < 15]
        if len(y) and (best is None or y.max() > best):
            best = float(y.max())
    return best


# sol de la carte (Brawl, plans 8, 7, 0 ... 6) de gauche à droite
SOL_HAUT = [(-213.48, 11.44), (-110.73, 11.44), (-87.29, 0.0), (111.85, 0.0), (159.06, -23.60), (180.64, -23.77),
            (182.61, -23.01), (216.80, -23.01), (218.89, -23.82), (266.65, -23.82)]
BLOC = [(-110.87, 52.08), (-214.10, 52.08), (-216.79, 70.49), (-110.87, 70.49), (-110.87, 52.08)]   # plans 9-11


def sol_coll(x):
    return float(np.interp(x, [p[0] for p in SOL_HAUT], [p[1] for p in SOL_HAUT]))


ecarts = []
for x in (-180, -50, 0, 60, 240):
    h = surface(x)
    if h is not None:
        ecarts.append(h - sol_coll(x))
print('décor - collisions (Brawl) à z 0 :', [round(e, 2) for e in ecarts])
vals = [e for e in ecarts if abs(e) < 10]
Y0 = float(os.environ.get('UE_Y0', str(round(float(np.median(vals)), 3) if vals else 0.0)))
print('Y0 =', Y0)


def attr(name, data):
    a = A.AttributeData(name)
    a.data = np.asarray(data, dtype=np.float32)
    return a


def alpha(t):
    return int(np.asarray(TEX[t].convert('RGBA'))[:, :, 3].min()) < 200


def tname(t):
    return (PFX + t).lower().replace(' ', '_')


tm = S.matl_data.read_matl(MATL_MODELE)
TI = next(i for i, e in enumerate(tm.entries) if e.material_label == 'Bottom')

# v3 : carte d'ombres d'origine (BakeMap de la carte Brawl, 2e texture + 2e jeu d'UV de Blue / Grid) dans un matériau
# du jeu à texture d'ombre : Texture0 (couleur, map1) + Texture9 (éclairage cuit, bake1), modèle pris dans
# stc_bg_set de 387162 (même dossier que le décor)
CUIT = os.environ.get('UE_CUIT', '1') == '1'
GAB_CUIT = None
if CUIT:
    ATTR_CUIT = None
    DOS_GAB = DOS
    prefs = ('SFX_PBS_0000000008088280', 'SFX_PBS_0000000000088280', 'SFX_PBS_0000000008080280', 'SFX_PBS_0000000000080280')
    for dg in (DOS_SOL, DOS):          # matériau modèle : d'abord le terrain principal, sinon le fond (v5, validé)
        _rm = S.matl_data.read_matl(os.path.join(RM, dg, 'model.numatb'))
        _dl = S.modl_data.read_modl(os.path.join(RM, dg, 'model.numdlb'))
        _sh = A.read_mesh(os.path.join(RM, dg, 'model.numshb'))
        _obj = {(o.name, o.subindex): o for o in _sh.objects}
        for pref, opq in [(p_, True) for p_ in prefs] + [(p_, False) for p_ in prefs]:
            for e in _rm.entries:
                if not (e.shader_label.startswith(pref) and sorted(t.param_id.name for t in e.textures) == ['Texture0', 'Texture9']):
                    continue
                if opq and not e.shader_label.endswith('_opaque'):
                    continue
                utilis = [(en.mesh_object_name, en.mesh_object_subindex) for en in _dl.entries if en.material_label == e.material_label]
                o = next((_obj[k] for k in utilis if k in _obj), None)
                if o is not None:
                    GAB_CUIT, DOS_GAB = e, dg
                    ATTR_CUIT = ([x.name for x in o.texture_coordinates], [x.name for x in o.color_sets])
                    break
            if GAB_CUIT is not None:
                break
        if GAB_CUIT is not None:
            break
    # v11 : SFX_PBS_0000000000088280 ne reçoit pas l'ombre des persos ; son jumeau 0000000008088280 (mêmes attributs et
    # paramètres d'après la base de shaders ssbh_wgpu) la reçoit -> on change seulement le nom du shader
    if GAB_CUIT is not None and os.environ.get('UE_SHADER_OMBRE', '1') == '1' and GAB_CUIT.shader_label.startswith('SFX_PBS_0000000000088280'):
        GAB_CUIT.shader_label = GAB_CUIT.shader_label.replace('SFX_PBS_0000000000088280', 'SFX_PBS_0000000008088280')
    print('matériau à texture d\'ombre :', DOS_GAB, GAB_CUIT.shader_label if GAB_CUIT else None, GAB_CUIT.material_label if GAB_CUIT else None, ATTR_CUIT)

# regroupement par texture
par_tex = {}
for mn, o in garde:
    par_tex.setdefault(o['tex'], []).append(o)

# autres ensembles cachés par le numdlb à 0 entrée de 387162 (méthode de Sleepless Lamp, validée sur Smashville)
vide = os.path.join(RM, 'dys_guest_asami_set', 'model.numdlb')
assert len(S.modl_data.read_modl(vide).entries) == 0
ens = set(d for d in os.listdir(RM) if os.path.isdir(os.path.join(RM, d)))
for l in open(LISTE):
    p = l.strip()
    if p.startswith('stage/animal_village/normal/model/'):
        ens.add(p.split('/')[4])
caches = sorted(ens - ({DOS, DOS_SOL} if GAB_CUIT is not None else {DOS}))
for d_ in caches:
    os.makedirs(os.path.join(ST, 'model', d_), exist_ok=True)
    shutil.copy(vide, os.path.join(ST, 'model', d_, 'model.numdlb'))
print('cachés (0 entrée) :', len(caches), ' '.join(caches))
BN = os.path.join(REF, 'normal')
# v2 : toujours le jour (Smashville suit l'horloge de la console) : fichiers render/ d'un mod « toujours le jour »
# (Islandville 382440 : la même série d'éclairage copiée sur matin / jour / soir / nuit, comme Siege's Smashville)
JOUR = os.environ.get('UE_JOUR', '')
if JOUR and os.path.isdir(JOUR):
    shutil.copytree(JOUR, os.path.join(ST, 'render'))
    print('render : jour permanent', sorted(os.listdir(os.path.join(ST, 'render'))))
D = os.path.join(ST, 'model', DOS)
D_SOL = os.path.join(ST, 'model', DOS_SOL)
os.makedirs(D_SOL, exist_ok=True)
os.makedirs(D, exist_ok=True)
objs, labs, entries, nouveaux = [], [], [], []
for k, (t, ol) in enumerate(sorted(par_tex.items())):
    pos = np.concatenate([o['pos'][:, [0, 2, 1]] for o in ol]).reshape(-1, 3).astype(np.float64)
    nrm = np.concatenate([o['nrm'][:, [0, 2, 1]] for o in ol]).reshape(-1, 3)
    uv = np.concatenate([(o['uv'][:, [0, 2, 1]] if o['uv'] is not None else np.zeros(o['pos'].shape[:2] + (2,)))
                         for o in ol]).reshape(-1, 2)
    cuit = GAB_CUIT is not None and ATTR_CUIT is not None and all(o.get('uv1') is not None and 'BakeMap' in o.get('texs', []) for o in ol)
    uv1 = np.concatenate([(o['uv1'][:, [0, 2, 1]] if o.get('uv1') is not None else np.zeros(o['pos'].shape[:2] + (2,)))
                          for o in ol]).reshape(-1, 2)
    key = np.round(np.c_[pos, nrm * 100, uv * 4096, uv1 * 4096], 3)
    uk, inv = np.unique(key, axis=0, return_inverse=True)
    first = np.zeros(len(uk), dtype=np.int64)
    first[inv.ravel()[::-1]] = np.arange(len(inv.ravel()))[::-1]
    pos, nrm, uv, uv1 = pos[first], nrm[first], uv[first], uv1[first]
    ftoit = fac_toit(pos)
    mtoit = zone_toit(pos)
    if cuit and mtoit.any() and os.environ.get('UE_OMBRE_PLEINE', '1') == '1':
        uv1 = uv1.copy()
        uv1[mtoit] = uv_sombre()
        print('  ombre pleine : %d sommets sous le toit' % int(mtoit.sum()))
    pos[:, 1] -= Y0
    pos *= ECH
    nrm = nrm / np.maximum(np.linalg.norm(nrm, axis=1, keepdims=True), 1e-6)
    idx = inv.astype(np.uint32).ravel()
    nv = len(pos)
    mo = A.MeshObjectData('ue_%02d' % k, 0)
    mo.positions = [attr('Position0', pos)]
    mo.normals = [attr('Normal0', np.c_[nrm, np.zeros(nv)])]
    try:
        tg = A.calculate_tangents_vec4(pos, nrm, uv, idx)
    except Exception:  # noqa: BLE001
        up = np.where(np.abs(nrm[:, 1:2]) > 0.9, np.array([[1, 0, 0]]), np.array([[0, 1, 0]]))
        tg = np.c_[np.cross(up, nrm), np.ones(nv)]
    mo.tangents = [attr('Tangent0', tg)]
    if cuit:
        mo.texture_coordinates = [attr(n_, uv1 if n_ == 'bake1' else uv) for n_ in ATTR_CUIT[0]]
        _cs = []
        for n_ in ATTR_CUIT[1]:
            c_ = np.full((nv, 4), 0.502)
            if n_ == 'colorSet1':
                c_[:, :3] *= ftoit[:, None]
            _cs.append(attr(n_, c_))
        mo.color_sets = _cs
        print('  ombre sous le toit :', int((ftoit < 1).sum()), 'sommets assombris')
    else:
        mo.texture_coordinates = [attr('map1', uv), attr('bake1', uv), attr('uvSet', uv)]
        c1 = np.full((nv, 4), 0.502)
        c1[:, :3] *= ombrage(nrm)[:, None]
        mo.color_sets = [attr('colorSet1', c1), attr('colorSet4', np.full((nv, 4), 0.502))]
    mo.vertex_indices = idx
    a = alpha(t)
    lab = 'ue_%s%s%s' % (t, '_a' if a else '', '_cuit' if cuit else '')
    if cuit:
        e = next(x for x in S.matl_data.read_matl(os.path.join(RM, DOS_GAB, 'model.numatb')).entries
                 if x.material_label == GAB_CUIT.material_label)
    else:
        e = S.matl_data.read_matl(MATL_MODELE).entries[TI]
    e.material_label = lab
    for x in e.textures:
        if x.param_id.name in ('Texture5', 'Texture0'):
            x.data = tname(t)
        elif x.param_id.name == 'Texture9':
            x.data = tname('BakeMap')
    for x in e.rasterizer_states:
        d_ = x.data
        d_.cull_mode = S.matl_data.CullMode.Disabled
        x.data = d_
    if a:
        for x in e.blend_states:
            d_ = x.data
            d_.alpha_sample_to_coverage = True
            x.data = d_
    entries.append(e)
    objs.append(mo)
    labs.append(lab)
    print('%-8s %-12s %6d tri %6d sommets %s' % (mo.name, t, len(idx) // 3, nv, lab))
    from PIL import Image
    png = os.path.join(OUT, '_tmp.png')
    im = np.asarray(TEX[t].convert('RGBA')).astype(np.float32)
    hd = os.path.join(os.environ.get('UE_HD', ''), t + '_x4.png')
    if os.environ.get('UE_HD') and os.path.exists(hd):
        im = np.asarray(Image.open(hd).convert('RGBA')).astype(np.float32)
    if GRILLE and t == 'Grid':
        im = grille(np.median(np.asarray(TEX[t].convert('RGBA')).reshape(-1, 4).astype(np.float32), axis=0))
        print('quadrillage refait', im.shape)
    im = ajuste(im, t) if not cuit else im
    if a:
        im[:, :, :3] *= im[:, :, 3:] / 255.0
    Image.fromarray(np.clip(im + 0.5, 0, 255).astype(np.uint8), 'RGBA').save(png)
    cible = os.path.join(D_SOL if cuit else D, tname(t) + '.nutexb')
    subprocess.run(['ultimate_tex_cli', png, cible, '--format', 'BC7RgbaUnormSrgb'], check=True, capture_output=True)
    os.remove(png)

if any(l.endswith('_cuit') for l in labs):
    from PIL import Image
    png = os.path.join(OUT, '_tmp.png')
    # v4 : en jeu, beaucoup trop lumineux (soleil de Smashville + éclairage cuit + textures déjà éclaircies) ->
    # textures d'origine pour les objets à texture d'ombre et carte d'ombres assombrie (UE_CUIT_LUM)
    bk = np.asarray(TEX['BakeMap'].convert('RGBA')).astype(np.float32)
    bk[:, :, :3] *= float(os.environ.get('UE_CUIT_LUM', '1.0'))
    Image.fromarray(np.clip(bk + 0.5, 0, 255).astype(np.uint8), 'RGBA').save(png)
    subprocess.run(['ultimate_tex_cli', png, os.path.join(D_SOL, tname('BakeMap') + '.nutexb'), '--format', 'BC7RgbaUnormSrgb'],
                   check=True, capture_output=True)
    os.remove(png)
    print('texture d\'ombre :', tname('BakeMap'), TEX['BakeMap'].size, [l for l in labs if l.endswith('_cuit')])

# scène three.js (aperçus / image de l'écran des terrains), même décor que le mod
if os.environ.get('UE_SCENE'):
    import json as _j
    from PIL import Image
    SC = os.environ['UE_SCENE']
    os.makedirs(os.path.join(SC, 'tex'), exist_ok=True)
    noms, scene = {}, []
    for i, t in enumerate(sorted(par_tex)):
        noms[t] = 't%02d' % i
        hd = os.path.join(os.environ.get('UE_HD', ''), t + '_x4.png')
        im_ = np.asarray((Image.open(hd) if os.environ.get('UE_HD') and os.path.exists(hd) else TEX[t]).convert('RGBA')).astype(np.float32)
        if GRILLE and t == 'Grid':
            im_ = grille(np.median(np.asarray(TEX[t].convert('RGBA')).reshape(-1, 4).astype(np.float32), axis=0))
        Image.fromarray(np.clip(ajuste(im_, t) + 0.5, 0, 255).astype(np.uint8), 'RGBA').save(os.path.join(SC, 'tex', noms[t] + '.png'))
    TEX['BakeMap'].convert('RGBA').save(os.path.join(SC, 'tex', 'bake.png'))
    for t, ol in par_tex.items():
        for o in ol:
            p = o['pos'][:, [0, 2, 1]].astype(np.float64)
            p[:, :, 1] -= Y0
            p *= ECH
            u = o['uv'][:, [0, 2, 1]] if o['uv'] is not None else np.zeros(p.shape[:2] + (2,))
            el = {'p': np.round(p.reshape(-1), 4).tolist(), 'uv': np.round(u.reshape(-1), 5).tolist(), 't': noms[t], 'a': bool(alpha(t))}
            if GAB_CUIT is not None and o.get('uv1') is not None and 'BakeMap' in o.get('texs', []):
                el['b'] = 'bake'
                u1_ = o['uv1'][:, [0, 2, 1]].reshape(-1, 2).copy()
                mt_ = zone_toit(o['pos'][:, [0, 2, 1]].reshape(-1, 3).astype(np.float64))
                if mt_.any() and os.environ.get('UE_OMBRE_PLEINE', '1') == '1':
                    u1_[mt_] = uv_sombre()
                el['u1'] = np.round(u1_.reshape(-1), 5).tolist()
                ft_ = fac_toit(o['pos'][:, [0, 2, 1]].reshape(-1, 3).astype(np.float64))
                if (ft_ < 1).any():
                    el['c'] = np.round(np.repeat(ft_[:, None], 3, axis=1).reshape(-1), 3).tolist()
            else:
                nn = o['nrm'][:, [0, 2, 1]].reshape(-1, 3)
                nn = nn / np.maximum(np.linalg.norm(nn, axis=1, keepdims=True), 1e-6)
                el['c'] = np.round(np.repeat(ombrage(nn)[:, None], 3, axis=1).reshape(-1), 3).tolist()
            scene.append(el)
    _j.dump(scene, open(os.path.join(SC, 'scene.json'), 'w'))
    print('scène', len(scene), 'objets')

for dos, dd, garde_l in ((DOS, D, lambda l: not l.endswith('_cuit')), (DOS_SOL, D_SOL, lambda l: l.endswith('_cuit'))):
    sel = [i for i, l in enumerate(labs) if garde_l(l)]
    if not sel:
        continue
    matl = S.matl_data.read_matl(MATL_MODELE)
    matl.entries = [entries[i] for i in sel]
    matl.save(os.path.join(dd, 'model.numatb'))
    mesh = A.read_mesh(os.path.join(BN, 'model', dos, 'model.numshb'))
    mesh.objects = [objs[i] for i in sel]
    mesh.save(os.path.join(dd, 'model.numshb'))
    modl = S.modl_data.read_modl(os.path.join(BN, 'model', dos, 'model.numdlb'))
    modl.entries = [S.modl_data.ModlEntryData(objs[i].name, objs[i].subindex, labs[i]) for i in sel]
    modl.save(os.path.join(dd, 'model.numdlb'))
    mx = S.meshex_data.MeshExData.from_mesh_objects([objs[i] for i in sel])
    if os.environ.get('UE_PROJETTE', '1') == '1':
        try:
            for g_ in mx.mesh_object_groups:
                for fl_ in g_.entry_flags:
                    fl_.cast_shadow = True
                    fl_.draw_model = True
            print('  projette une ombre :', dos, [(g_.mesh_object_name, [(f_.draw_model, f_.cast_shadow) for f_ in g_.entry_flags]) for g_ in mx.mesh_object_groups])
        except Exception as ex_:  # noqa: BLE001
            print('  drapeaux numshexb non modifiés :', ex_, [a_ for a_ in dir(mx) if not a_.startswith('_')])
    mx.save(os.path.join(dd, 'model.numshexb'))
    print('DOSSIER', dos, len(sel), 'objets', sum(len(objs[i].vertex_indices) // 3 for i in sel), 'triangles', [labs[i] for i in sel])

# collisions : sol de la carte, rebords aux deux bouts, bloc solide ; lvd de Smashville d'HDR
import yaml, copy
P_ = os.path.join(ST, 'param')
os.makedirs(P_, exist_ok=True)
tmp = os.path.join(OUT, '_lvd.yml')
subprocess.run(['yamlvd', LVD, tmp], check=True, capture_output=True)
d = yaml.safe_load(open(tmp))
YB = -60.0
YH = 200.0
haut = [(x * ECH, y * ECH) for x, y in SOL_HAUT]
if COUPE:
    haut = [(x, y) for x, y in haut if x < X_DROITE * ECH] + [(X_DROITE * ECH, float(_sol_b(X_DROITE)) * ECH)]
    haut[0] = (X_GAUCHE * ECH, haut[0][1])
(x0, y0), (x1, y1) = haut[0], haut[-1]
if COUPE:
    # murs invisibles aux deux bouts (sol + murs jusqu'à YH, comme Lamp), plus de rebords
    pts = [(x1 + 10, YB), (x0 - 10, YB), (x0 - 10, YH), (x0, YH)] + haut + [(x1, YH), (x1 + 10, YH), (x1 + 10, YB)]
else:
    haut = [haut[0], (x0 + 5, y0)] + haut[1:-1] + [(x1 - 5, y1), haut[-1]]
    pts = [(x1, YB), (x0, YB)] + haut + [(x1, YB)]
sol = next(c for c in d['collisions'] if c['entry']['name'] == 'COL_00_Floor01')
m_rien = copy.deepcopy(sol['materials'][0])
for k in m_rien['line_flags']:
    m_rien['line_flags'][k] = False
m_l, m_r = copy.deepcopy(m_rien), copy.deepcopy(m_rien)
m_l['line_flags']['hang_l'] = True
m_r['line_flags']['hang_r'] = True


def poly(c, pts, mats):
    c['vertices'] = [{'x': float(x), 'y': float(y)} for x, y in pts]
    nrm = []
    for (a0, b0), (a1, b1) in zip(pts, pts[1:]):
        dx, dy = a1 - a0, b1 - b0
        n = (dx * dx + dy * dy) ** 0.5
        nrm.append({'x': float(-dy / n), 'y': float(dx / n)})
    c['normals'] = nrm
    c['materials'] = mats
    c['cliffs'] = []


n_l = len(pts) - 1
i_l = -1 if COUPE else 2                    # segment (x0, y0) -> (x0 + 5, y0)
i_r = -1 if COUPE else n_l - 2              # segment (x1 - 5, y1) -> (x1, y1)
poly(sol, pts, [m_l if i == i_l else m_r if i == i_r else copy.deepcopy(m_rien) for i in range(n_l)])
bloc = copy.deepcopy(sol)
bloc['entry']['name'] = 'COL_00_Block01'
bloc['entry']['subname'] = 'Block01'
bp = [(x * ECH, y * ECH) for x, y in BLOC]
poly(bloc, bp, [copy.deepcopy(m_rien) for _ in range(len(bp) - 1)])
autres = [c for c in d['collisions'] if c is not sol]
for c in autres:              # plate-forme mobile : petit segment enterré (comme Lamp ; la retirer peut faire planter)
    c['vertices'] = [{'x': -5.0, 'y': -45.0}, {'x': 5.0, 'y': -45.0}]
    c['normals'] = [{'x': 0.0, 'y': 1.0}]
    print('enterrée :', c['entry']['name'])
d['collisions'] = ([sol] if (COUPE and os.environ.get('UE_BLOC', '0') != '1') else [sol, bloc]) + autres  # v9 : bloc retiré (plafond invisible)
if COUPE and os.environ.get('UE_BLOC_BLEU', '1') == '1':   # v11 : collision remise sur le bloc bleu seulement (demande du joueur)
    bleu = copy.deepcopy(sol)
    bleu['entry']['name'] = 'COL_00_Block02'
    bleu['entry']['subname'] = 'Block02'
    xb0 = max(X_BLEU, TOIT[0][0] if TOIT is not None else X_BLEU)
    bb = [(X_BLEU1 * ECH, 52.08 * ECH), (xb0 * ECH, 52.08 * ECH), (xb0 * ECH, Y_BLEU * ECH), (X_BLEU1 * ECH, Y_BLEU * ECH), (X_BLEU1 * ECH, 52.08 * ECH)]
    poly(bleu, bb, [copy.deepcopy(m_rien) for _ in range(len(bb) - 1)])
    d['collisions'].insert(1, bleu)
    print('collision du bloc bleu', bb)
if TOIT is not None:          # v10 : toit = bloc solide (plafond) au-dessus de la partie gauche
    toit = copy.deepcopy(sol)
    toit['entry']['name'] = 'COL_00_Roof01'
    toit['entry']['subname'] = 'Roof01'
    tp = [(x * ECH, y * ECH) for x, y in TOIT]
    poly(toit, tp, [copy.deepcopy(m_rien) for _ in range(len(tp) - 1)])
    d['collisions'].insert(1, toit)
    print('collision du toit', tp)
BZ = {'left': x0 - 130.0, 'right': x1 + 130.0, 'top': 260.0, 'bottom': -170.0}
CAM = {'left': x0 - 90.0, 'right': x1 + 90.0, 'top': 190.0, 'bottom': -110.0}
for k, v in BZ.items():
    d['blast_zone'][0][k] = v
for k, v in CAM.items():
    d['camera_boundary'][0][k] = v
xs = [-60, 60, -20, 20, -80, 100, -40, 40]
for s_, x in zip(d['spawns'], xs):
    s_['pos']['x'], s_['pos']['y'] = float(x), sol_coll(x) * ECH + 0.1
for r_, x in zip(d['respawns'], xs):
    r_['pos']['x'], r_['pos']['y'] = float(x), 100.0
print('sol x %.1f..%.1f (rebords lignes %d et %d), bloc ; caméra %s ; zones de K.O. %s' % (x0, x1, i_l, i_r, CAM, BZ))
yaml.safe_dump(d, open(tmp, 'w'), sort_keys=False)
subprocess.run(['yamlvd', tmp, os.path.join(P_, 'xvillage00.lvd')], check=True, capture_output=True)
os.remove(tmp)

import json
# config.json (Stage Alts) : chaque fichier du jeu de stage/animal_village/normal non fourni est partagé vers la variante
fournis = set()
for racine, _, fichiers in os.walk(ST):
    for f in fichiers:
        fournis.add(os.path.relpath(os.path.join(racine, f), OUT).replace(os.sep, '/'))
partage = {}
for l in open(LISTE):
    p = l.strip()
    if not p.startswith('stage/animal_village/normal/'):
        continue
    v = p.replace('stage/animal_village/normal/', 'stage/animal_village/%s/' % FORME, 1)
    if v not in fournis:
        partage[p] = [v]
UI_FOURNIS = [int(x) for x in os.environ.get('UE_UI_FOURNIS', '1,2,4').split(',') if x]
for n in range(5):
    if n not in UI_FOURNIS:
        partage['ui/replace/stage/stage_%d/stage_%d_animal_village.bntx' % (n, n)] = \
            ['ui/replace/stage/stage_%d/stage_%d_animal_village_%s.bntx' % (n, n, FORME.split('_')[-1])]
json.dump({'share_to_vanilla': partage}, open(os.path.join(OUT, 'config.json'), 'w'), indent=2)
print('fournis', len(fournis), '; partagés avec le jeu', len(partage))
