"""Calcule où placer la traînée de chaque coup : position de la hitbox (os + décalage du mod) à l'image où elle sort,
et direction du mouvement de cet os, à partir des animations de Black Shadow (.nuanmb validées en jeu).

usage : python3 trails.py <dossier du mod> <dossier BlackShadow des assets>
Écrit sur la sortie standard une ligne TRAIL par coup : image, position (x, y, z) dans le repère de l'os top,
vitesse (x, y, z) et l'angle à donner à la traînée.
"""
import math
import os
import sys

import numpy as np
import ssbh_data_py as S

HERE = os.path.dirname(__file__)
sys.path.insert(0, os.path.join(HERE, '..'))
import gen  # noqa: E402
from psa import load, subactions  # noqa: E402

MOD, ASSETS = sys.argv[1:3]
MOT = os.path.join(ASSETS, 'fighter/ganon/motion/body/c00')
sk = S.skel_data.read_skel(os.path.join(ASSETS, 'fighter/ganon/model/body/c00/model.nusktb'))
UN = [b.name for b in sk.bones]
PAR = [b.parent_index for b in sk.bones]
L0 = [np.array(b.transform, dtype=float).T for b in sk.bones]


def mat_from(sc, q, tr):
    x, y, z, w = q
    r = np.array([[1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
                  [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
                  [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)]])
    m = np.eye(4)
    m[:3, :3] = r @ np.diag(sc)
    m[:3, 3] = tr
    return m


def worlds(anim_file):
    a = S.anim_data.read_anim(os.path.join(MOT, anim_file))
    tracks = {}
    for g in a.groups:
        for nd in g.nodes:
            if nd.tracks and hasattr(nd.tracks[0].values[0], 'rotation'):
                tracks[nd.name] = nd.tracks[0].values
    n = int(a.final_frame_index) + 1
    out = []
    for k in range(n):
        W = [None] * len(UN)

        def w(i):
            if W[i] is None:
                p = PAR[i]
                pw = w(p) if p is not None and p >= 0 else np.eye(4)
                if UN[i] in tracks:
                    v = tracks[UN[i]][min(k, len(tracks[UN[i]]) - 1)]
                    W[i] = pw @ mat_from(v.scale, v.rotation, v.translation)
                else:
                    W[i] = pw @ L0[i]
            return W[i]

        for i in range(len(UN)):
            w(i)
        # repère de l'os top = Trans sans sa rotation (le personnage regarde vers +z)
        t = W[UN.index('Trans')][:3, 3].copy()
        for i in range(len(UN)):
            W[i] = W[i].copy()
            W[i][:3, 3] -= t
        out.append(W)
    return out


bones = gen.bone_names(os.path.join(MOD, 'FitBlackShadow00.pac'))
m = load(os.path.join(MOD, 'FitBlackShadow.pac'))
base, subs = subactions(m)
byname = {}
for i, n, mo in subs:
    if n and mo and n not in byname:
        byname[n] = mo
lens = gen.anim_lengths(os.path.join(MOD, 'FitBlackShadowMotionEtc.pac'))

JOBS = [('attack_air_n', 'AttackAirN', 'c05attackairn.nuanmb', 0.0), ('attack_air_f', 'AttackAirF', 'c05attackairf.nuanmb', 0.0),
        ('attack_air_b', 'AttackAirB', 'c05attackairb.nuanmb', 0.0), ('attack_air_hi', 'AttackAirHi', 'c05attackairhi.nuanmb', 0.0),
        ('attack_air_lw', 'AttackAirLw', 'c05attackairlw.nuanmb', 0.0),
        ('attack_s4_s', 'AttackS4S', 'c03attacks4s.nuanmb', float(lens.get('AttackS4Start', 0))),
        ('attack_hi4', 'AttackHi4', 'c04attackhi4.nuanmb', float(lens.get('AttackHi4Start', 0))),
        ('attack_lw4', 'AttackLw4', 'c04attacklw4.nuanmb', float(lens.get('AttackLw4Start', 0)))]
lower = {u.lower(): u for u in UN}
RES = {}
for motion, sub, fn, start in JOBS:
    if sub not in byname or not os.path.exists(os.path.join(MOT, fn)):
        print('TRAIL absent', motion)
        continue
    tl = gen.timeline(m, base, byname[sub])
    hits = [(f, p) for f, k, p in tl if k == 'hit']
    if not hits:
        continue
    f0, p = hits[0]
    v = [gen.pv(t, x) for t, x in p]
    raw0 = p[0][1] & 0xffffffff
    ub = gen.BONES.get(bones.get(raw0 >> 16, ''), 'top')
    off = np.array([v[6], v[7], v[8], 1.0])  # Brawl Z, Y, X -> Ultimate x, y, z
    W = worlds(fn)
    fr = int(round(f0 + start))
    bn = lower.get(ub)

    def hitpos(k):
        k = max(0, min(len(W) - 1, k))
        if bn is None:
            return off[:3]
        return (W[k][UN.index(bn)] @ off)[:3]

    pos = hitpos(fr)
    vel = hitpos(fr + 1) - hitpos(fr - 1)
    # traînée en ligne : son axe propre est +z (vers l'avant) ; Rx(t) envoie +z sur (0, -sin t, cos t)
    rx = math.degrees(math.atan2(-vel[1], vel[2])) if np.linalg.norm(vel[1:]) > 1e-3 else -90.0
    RES[motion] = (W, fr)
    print('TRAIL %-14s image %5.1f os %-10s pos %s vit %s rx %.0f' % (motion, fr, bn, np.round(pos, 2).tolist(),
                                                                   np.round(vel, 2).tolist(), rx))


# ------------------------------------------------------------------ planche : pose à l'image du coup + traînée du .nro
if len(sys.argv) > 3:
    sys.path.insert(0, HERE)
    import render  # noqa: E402
    from PIL import Image, ImageDraw  # noqa: E402

    # (coup, titre, légende, départ y, départ z, rx) : mêmes valeurs que trail() dans src/lib.rs
    PANELS = [('attack_air_f', 'Forward air', 'devant, horizontale, du poing vers l\'avant', 9.0, 3.0, 5.0),
              ('attack_air_n', 'Neutral air', 'devant, au genou, vers l\'avant', 5.4, 3.0, -7.0),
              ('attack_air_b', 'Back air', 'part vers l\'arrière', 7.5, 3.0, -160.0),
              ('attack_hi4', 'Smash haut', 'monte devant lui', 6.0, 12.0, -94.0)]
    W0 = [None] * len(UN)

    def w0(i):
        if W0[i] is None:
            p = PAR[i]
            W0[i] = (w0(p) if p is not None and p >= 0 else np.eye(4)) @ L0[i]
        return W0[i]

    for i in range(len(UN)):
        w0(i)
    objs = render.load_mesh(ASSETS, UN)
    W0inv = [np.linalg.inv(x) for x in W0]
    ti, tr = UN.index('Throw'), UN.index('Trans')
    C = render.CELL = 420
    sc = C / 34.0
    img = Image.new('RGB', (2 * C + 30, 2 * (C + 70) + 60), render.BG)
    d = ImageDraw.Draw(img)
    d.text((10, 12), 'Traînées de Black Shadow : à quoi ça doit ressembler (vu de côté, il regarde à droite)',
           font=render.font(20), fill=render.TXT)
    for k, (mo, title, sub, y0, z0, rx) in enumerate(PANELS):
        x, y = 10 + (k % 2) * (C + 10), 50 + (k // 2) * (C + 70)
        d.text((x, y), title, font=render.font(20), fill=render.TXT)
        d.text((x, y + 26), sub, font=render.font(15, False), fill=render.TXT)
        if mo not in RES:
            continue
        Wk, fr = RES[mo]
        cell = render.draw_pose(objs, Wk[max(0, min(len(Wk) - 1, fr))], W0inv, ti, tr, ring=False)
        cd = ImageDraw.Draw(cell, 'RGBA')
        t = math.radians(rx)
        dz, dy = math.cos(t), -math.sin(t)
        L = 14.0
        for j in range(24):  # traînée : fine et transparente au départ, épaisse et vive au bout
            a0, a1 = j / 24, (j + 1) / 24
            p0 = (C / 2 + (z0 + dz * L * a0) * sc, C - 18 - (y0 + dy * L * a0) * sc)
            p1 = (C / 2 + (z0 + dz * L * a1) * sc, C - 18 - (y0 + dy * L * a1) * sc)
            cd.line([p0, p1], fill=(170, 80, 255, int(60 + 180 * a1)), width=int(3 + 12 * a1))
        img.paste(cell, (x, y + 50))
    img.save(sys.argv[3])
    print('planche', sys.argv[3])
