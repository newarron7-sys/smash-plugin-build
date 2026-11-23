"""Guilty Gear - Royal Court (Brawl, GameBanana 725841) sur la Destination Finale (stage/end).

usage : python3 rc_fd.py <STGROYALFIXED.pac> <réf Brawl FD 454060 .../stage/end> <numatb de Neonfield (SBF)> <Hashes_FullPath>
                         <sortie mod> [<end_00.lvd d'origine>]
  RC_HD=<dossier des textures *_x4.png>   (upscale_tex.py)

Même méthode que COT sur le Petit Champ (cot_sbf.py, validée en jeu), avec la structure de « Brawl Final Destination »
(454060, mod de la Destination Finale qui marche) :
- ring_set + ring_decorations_set : maillages de Royal Court (sans os, comme les maillages de 454060), numdlb de 454060
  comme modèle, nusktb / nuhlpb / xmb / nusrcmdlb de 454060 tels quels ;
- matériaux : ceux de COT (Neonfield « Material 1 » 8269 opaque, « Material 2 » 080100 sort pour la transparence et le
  ciel), seule la texture change ;
- textures : d'abord les textures d'origine du dossier écrasées (sauf cubemaps), le surplus en nouveaux fichiers dans
  ring_set (comme 454060 / Snowy SBF) ;
- paysages de la Destination Finale cachés : numdlb + nusrcmdlb de 454060 ; effet, motion, param et render de 454060
  tels quels (éclairage fixe, pas de cycle jour/nuit).
"""
import os, shutil, subprocess, sys
import numpy as np
import ssbh_data_py as S
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import brawl_stage  # noqa: E402

PAC, BFD, NEON_MATL, HASH, OUT = sys.argv[1:6]
LVD = sys.argv[6] if len(sys.argv) > 6 else None
A = S.mesh_data
BN = os.path.join(BFD, 'normal')
ST = os.path.join(OUT, 'stage', 'end', 'normal')
DST = os.path.join(ST, 'model')
if os.path.isdir(os.path.join(OUT, 'stage')):
    shutil.rmtree(os.path.join(OUT, 'stage'))

ECH = float(os.environ.get('RC_ECHELLE', '0.9'))
GRAND = 4000            # objets plus grands que ça (Brawl) : ciel / lointain -> matériau émissif
DOSSIERS = ['ring_set', 'ring_decorations_set']
PLACES = {d: [] for d in DOSSIERS}
for l in open(HASH):
    l = l.strip()
    for d in DOSSIERS:
        pre = 'stage/end/normal/model/%s/' % d
        if l.startswith(pre) and l.endswith('.nutexb') and '/' not in l[len(pre):] and 'cube' not in l:
            PLACES[d].append(l[len(pre):-7])

# 1) tout ce qui n'est pas un maillage : copié de 454060 tel quel
for racine, _, fichiers in os.walk(BN):
    rel = os.path.relpath(racine, BN)
    if rel.startswith('model'):
        continue
    for f in fichiers:
        os.makedirs(os.path.join(ST, rel), exist_ok=True)
        shutil.copy(os.path.join(racine, f), os.path.join(ST, rel, f))
for d in sorted(os.listdir(os.path.join(BN, 'model'))):
    src = os.path.join(BN, 'model', d)
    os.makedirs(os.path.join(DST, d), exist_ok=True)
    for f in os.listdir(src):
        if f.endswith('.nutexb') or f.endswith('.png') or f.endswith('.whl'):
            continue
        if d in DOSSIERS and f in ('model.numshb', 'model.numatb', 'model.numdlb', 'model.numshexb'):
            continue
        shutil.copy(os.path.join(src, f), os.path.join(DST, d, f))

TEX, models = brawl_stage.load(PAC)


def attr(name, data):
    a = A.AttributeData(name)
    a.data = np.asarray(data, dtype=np.float32)
    return a


def alpha(t):
    return int(np.asarray(TEX[t].convert('RGBA'))[:, :, 3].min()) < 200


libres = [(d, n) for d in DOSSIERS for n in sorted(PLACES[d])]
# atlas : textures non répétées (UV dans 0..1), opaques, 256x256, mises deux par deux côte à côte pour tenir dans les
# textures d'origine des dossiers (aucun nouveau fichier)
uvs = {}
for _, mn, ol in models:
    for o in ol:
        if mn == 'Royal' and o['tex'] in TEX:
            uvs.setdefault(o['tex'], []).append(o['uv'].reshape(-1, 2))
manque = max(0, len(uvs) - len(libres))
cand = [t for t in sorted(uvs) if TEX[t].size == (256, 256) and not alpha(t)
        and np.concatenate(uvs[t]).min() > -0.01 and np.concatenate(uvs[t]).max() < 1.01]
ATLAS = {}
for k in range(manque):
    t0, t1 = cand[2 * k], cand[2 * k + 1]
    nom = 'ATLAS%02d' % k
    TEX[nom] = Image.new('RGBA', (512, 256))
    TEX[nom].paste(TEX[t0].convert('RGBA'), (0, 0))
    TEX[nom].paste(TEX[t1].convert('RGBA'), (256, 0))
    hd = os.environ.get('RC_HD')
    if hd and os.path.exists(os.path.join(hd, t0 + '_x4.png')):
        a0, a1 = Image.open(os.path.join(hd, t0 + '_x4.png')), Image.open(os.path.join(hd, t1 + '_x4.png'))
        im = Image.new('RGBA', (a0.width * 2, a0.height))
        im.paste(a0.convert('RGBA'), (0, 0))
        im.paste(a1.convert('RGBA'), (a0.width, 0))
        im.save(os.path.join(hd, nom + '_x4.png'))
    ATLAS[t0], ATLAS[t1] = (nom, 0), (nom, 1)
print('atlas', len(ATLAS) // 2, 'pour', manque, 'textures en trop')
objs, texs, nouv = [], {}, 0
liste = [o for _, mn, ol in models for o in ol if mn == 'Royal' and o['tex'] in TEX]
for o in liste:
    pos = o['pos'][:, [0, 2, 1]].reshape(-1, 3).astype(np.float64)
    nrm = o['nrm'][:, [0, 2, 1]].reshape(-1, 3)
    uv = o['uv'][:, [0, 2, 1]].reshape(-1, 2).copy()
    if o['tex'] in ATLAS:
        nom, moitie = ATLAS[o['tex']]
        uv[:, 0] = moitie * 0.5 + 0.004 + np.clip(uv[:, 0], 0, 1) * 0.492
        o = dict(o, tex=nom)
    grand = np.ptp(pos, 0).max() > GRAND
    key = np.round(np.c_[pos, nrm * 100, uv * 4096], 3)
    uk, inv = np.unique(key, axis=0, return_inverse=True)
    first = np.zeros(len(uk), dtype=np.int64)
    first[inv[::-1]] = np.arange(len(inv))[::-1]
    pos, nrm, uv = pos[first] * ECH, nrm[first], uv[first]
    nrm = nrm / np.maximum(np.linalg.norm(nrm, axis=1, keepdims=True), 1e-6)
    idx = inv.astype(np.uint32).ravel()
    nv = len(pos)
    if o['tex'] not in texs:
        if libres:
            texs[o['tex']] = libres.pop(0)
        else:
            nouv += 1
            texs[o['tex']] = ('ring_set', 'rc_tex_%02d' % nouv)
    dos, cible = texs[o['tex']]
    genre = 'a' if alpha(o['tex']) else ('e' if grand else 'o')
    mo = A.MeshObjectData('rc_' + o['name'], 0)
    mo.positions = [attr('Position0', pos)]
    mo.normals = [attr('Normal0', np.c_[nrm, np.zeros(nv)])]
    try:
        tg = A.calculate_tangents_vec4(pos.astype(np.float32), nrm.astype(np.float32), uv.astype(np.float32), idx)
    except Exception:  # noqa: BLE001
        tg = np.tile([1.0, 0.0, 0.0, 1.0], (nv, 1))
    mo.tangents = [attr('Tangent0', tg)]
    mo.texture_coordinates = [attr('map1', uv), attr('uvSet', np.zeros((nv, 2))), attr('uvSet1', np.zeros((nv, 2)))]
    mo.color_sets = [attr('colorSet1', np.full((nv, 4), 0.502))]
    mo.vertex_indices = idx
    objs.append((dos, mo, 'rc_%s_%s' % (genre, cible), o['tex'], genre))

REFMAT = {'o': ('Material 1', 'Texture0'), 'a': ('Material 2', 'Texture5'), 'e': ('Material 2', 'Texture5')}
for dos in DOSSIERS:
    sel = [x for x in objs if x[0] == dos]
    mesh = A.read_mesh(os.path.join(BN, 'model', dos, 'model.numshb'))
    modl = S.modl_data.read_modl(os.path.join(BN, 'model', dos, 'model.numdlb'))
    matl = S.matl_data.read_matl(NEON_MATL)
    mesh.objects = [x[1] for x in sel]
    mesh.save(os.path.join(DST, dos, 'model.numshb'))
    modl.entries = [S.modl_data.ModlEntryData(x[1].name, x[1].subindex, x[2]) for x in sel]
    modl.save(os.path.join(DST, dos, 'model.numdlb'))
    S.meshex_data.MeshExData.from_mesh_objects(mesh.objects).save(os.path.join(DST, dos, 'model.numshexb'))
    entries, vus, faites = [], set(), set()
    for _, _, lab, t, genre in sel:
        if lab in vus:
            continue
        vus.add(lab)
        nom, slot = REFMAT[genre]
        e = next(x for x in S.matl_data.read_matl(NEON_MATL).entries if x.material_label == nom)
        e.material_label = lab
        cible = texs[t][1]
        for x in e.textures:
            if x.param_id.name == slot:
                x.data = cible
        entries.append(e)
        if cible in faites:
            continue
        faites.add(cible)
        hd = os.path.join(os.environ.get('RC_HD', ''), t + '_x4.png')
        src = Image.open(hd) if os.environ.get('RC_HD') and os.path.exists(hd) else TEX[t]
        im = np.asarray(src.convert('RGBA')).astype(np.float32)
        if genre == 'a':
            im[:, :, :3] *= im[:, :, 3:] / 255.0
        png = os.path.join(OUT, '_tmp.png')
        Image.fromarray(np.clip(im + 0.5, 0, 255).astype(np.uint8), 'RGBA').save(png)
        subprocess.run(['ultimate_tex_cli', png, os.path.join(DST, dos, cible + '.nutexb'), '--format',
                        'BC7RgbaUnormSrgb'], check=True, capture_output=True)
        os.remove(png)
    matl.entries = entries
    matl.save(os.path.join(DST, dos, 'model.numatb'))
    print('DOSSIER', dos, len(sel), 'objets', len(vus), 'materiaux', len(faites), 'textures')
print('objets', len(objs), 'textures', len(texs), 'dont nouvelles', nouv)
