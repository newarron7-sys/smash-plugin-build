"""Metal Sonic (REX) : ajoute Neo Metal Sonic (Final Smash de REX : FitMetalSonicFinal.pac, modèle « Neo_Metal_Sonic »
posé sur le Landmaster de Wolf) au modèle du skin, caché par défaut.

usage : python3 add_neo.py <FitMetalSonicFinal.pac> <FitMetalSonic00.pac> <dossier modèle du skin> <sortie> <dossier png>
- même méthode que le canon de Zero (zero/tools/add_buster.py) et la Black Bull (black-shadow/tools/fs/build_car.py) :
  géométrie lue dans le MDL0 (tous ses objets sont sur l'os TopN : Neo est figé dans REX, une seule pose), placée
  avec les matrices de repos, à l'échelle du skin (hauteur des hanches du skin / hanches de Metal dans REX) ;
- un os à lui, NeoN, enfant de Trans : le plugin peut le tourner (tonneau) sans toucher au reste ;
- objets « NeoMetal_VIS_O_OBJShape » : cachés par défaut, montrés par la piste de visibilité « NeoMetal » des
  animations du Final Smash (neo_vis.py) ;
- matériaux : copie d'un matériau opaque du skin, textures de Neo (png, converties en nutexb ensuite) ; PRM métallique
  (dans REX ses pièces brillent avec des textures de reflet).
"""
import json
import os
import struct
import sys

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

FINAL, METAL, SKIN, OUT, PNG = sys.argv[1:6]
os.makedirs(OUT, exist_ok=True)
os.makedirs(PNG, exist_ok=True)
NOM = 'NeoMetal_VIS_O_OBJShape'
OS_NEO = 'NeoN'

raw = decompress(open(FINAL, 'rb').read())
dd = [x for x in arc_entries(raw) if x[0] == 2][0][4]
mn, mo = brres_files(dd)['3DModels(NW4R)'][0]
print('modele', mn)
offs = struct.unpack_from('>11i', dd, mo + 0x10)
SEC = dict(zip(['defs', 'bones', 'pos', 'nrm', 'clr', 'uv', 'mats', 'shaders', 'objs', 'tex', 'pal'], offs))
M = mdl0.read_model(dd, mo)
bone_w = {b[1]: b[2] for b in M['bones']}

# ------------------------------------------------------------------ textures et matériaux de REX
TEX = {}
for ent in arc_entries(raw):
    if ent[4][:4] != b'bres':
        continue
    bf = brres_files(ent[4])
    pals = {n: tex0.palette(ent[4], o) for n, o in bf.get('Palettes(NW4R)', [])}
    for n, o in bf.get('Textures(NW4R)', []):
        if n not in TEX:
            try:
                TEX[n] = tex0.read_tex0(ent[4], o, pals.get(n)).convert('RGBA')
            except Exception as e:  # noqa: BLE001
                print('texture illisible', n, e)
print('TEXTURES', {n: TEX[n].size for n in TEX})


def mat_textures(moff):
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


MATTEX = {n: mat_textures(o) for n, o in dict_entries(dd, mo + SEC['mats'])}
for n, t in MATTEX.items():
    print('MATERIAU', n, t)


def couleur(mname):
    """texture de couleur : la première qui n'est pas un reflet (Ref2, Ref3, mstx_ref3)"""
    for t in MATTEX.get(mname, []):
        if not t.lower().startswith(('ref', 'mstx')):
            return t
    return None


# ------------------------------------------------------------------ échelle (Metal du skin / Metal de REX)
tmp = '/tmp/rex_metal_neo.pac'
open(tmp, 'wb').write(decompress(open(METAL, 'rb').read()))
BR = brawl_anim.read_bones(tmp)
hip_rex = float(BR['HipN'].world[1, 3])
sk = S.skel_data.read_skel(os.path.join(SKIN, 'model.nusktb'))


def worlds(sk):
    bs = sk.bones
    W = [None] * len(bs)

    def w(i):
        if W[i] is None:
            loc = np.array(bs[i].transform, dtype=float).T
            p = bs[i].parent_index
            W[i] = (w(p) @ loc) if p is not None and p >= 0 else loc
        return W[i]
    for i in range(len(bs)):
        w(i)
    return {b.name: W[i] for i, b in enumerate(bs)}


WS = worlds(sk)
hip_skin = float(WS['Hip'][1, 3])
ECH = hip_skin / hip_rex
print('hanches : skin %.2f, REX %.2f -> echelle %.3f' % (hip_skin, hip_rex, ECH))

# ------------------------------------------------------------------ géométrie (par matériau)
A = S.mesh_data


def attr(name, data):
    a = A.AttributeData(name)
    a.data = np.asarray(data, dtype=np.float32)
    return a


par_mat = {}
for i, (on, ob) in enumerate(M['objs']):
    mi = M['obj_mat'].get(i)
    mname = M['mat_names'][mi] if mi is not None else 'defaut'
    P = M['arrays']['pos'][ob['ids'][0]][0]
    N = M['arrays']['nrm'][ob['ids'][1]][0] if ob['ids'][1] >= 0 else None
    U = M['arrays']['uv'][ob['ids'][4]][0] if ob['ids'][4] >= 0 else None
    g = par_mat.setdefault(mname, {'p': [], 'n': [], 'uv': []})
    for t in ob['tris']:
        for v in (t[0], t[2], t[1]):                # sens des triangles : Brawl -> Ultimate
            b = v.get('bone', ob['node'])
            w = bone_w.get(b, np.eye(4)) if b is not None and b >= 0 else np.eye(4)
            g['p'].append((w @ np.r_[P[v['pos']], 1])[:3])
            nn = (w[:3, :3] @ N[v['nrm']][:3]) if N is not None and 'nrm' in v else np.array([0, 1.0, 0])
            g['n'].append(nn / (np.linalg.norm(nn) or 1))
            g['uv'].append(U[v['uv0']] if U is not None and 'uv0' in v else np.zeros(2))

allp = np.concatenate([np.array(g['p']) for g in par_mat.values()]) * ECH
lo, hi = allp.min(0), allp.max(0)
print('Neo chez le skin : de %s a %s (taille %s)' % (np.round(lo, 2), np.round(hi, 2), np.round(hi - lo, 2)))

# ------------------------------------------------------------------ squelette : skin + NeoN sous Trans
names = [b.name for b in sk.bones]
sk.bones = [b for b in sk.bones if b.name != OS_NEO]
sk.bones.append(S.skel_data.BoneData(OS_NEO, np.ascontiguousarray(np.eye(4, dtype=np.float32)), names.index('Trans')))
sk.save(os.path.join(OUT, 'model.nusktb'))
print('squelette : %d os (+ %s sous Trans)' % (len(sk.bones), OS_NEO))

# ------------------------------------------------------------------ objets
matl = S.matl_data.read_matl(os.path.join(SKIN, 'model.numatb'))
labs = [e.material_label for e in matl.entries]
gabarit = next((l for l in labs if l.startswith('def_')), labs[0])
gab_alpha = next((l for l in labs if l.startswith('alp_')), gabarit)
print('materiaux modeles', gabarit, gab_alpha)


def copie(label):
    m = S.matl_data.read_matl(os.path.join(SKIN, 'model.numatb'))
    return next(e for e in m.entries if e.material_label == label)


def up2(im):
    if max(im.size) >= 512:
        return im
    return im.resize((im.width * 2, im.height * 2), Image.LANCZOS)


Image.new('RGBA', (8, 8), (128, 128, 255, 255)).save(os.path.join(PNG, 'neo_nor.png'))
new_objs, new_entries, new_mats, textures = [], [], [], ['neo_nor']
for k, (mname, g) in enumerate(sorted(par_mat.items())):
    p = np.array(g['p']) * ECH
    nv = len(p)
    if nv == 0:
        continue
    nrm, uv = np.array(g['n']), np.array(g['uv'])
    o = A.MeshObjectData(NOM, len(new_objs))
    o.positions = [attr('Position0', p)]
    o.normals = [attr('Normal0', np.c_[nrm, np.zeros(nv)])]
    idx = np.arange(nv, dtype=np.uint32)
    try:
        tg = A.calculate_tangents_vec4(p, nrm, uv, idx)
    except Exception:  # noqa: BLE001
        up = np.where(np.abs(nrm[:, 1:2]) > 0.9, np.array([[1, 0, 0]]), np.array([[0, 1, 0]]))
        tg = np.c_[np.cross(up, nrm), np.ones(nv)]
    o.tangents = [attr('Tangent0', tg)]
    o.texture_coordinates = [attr('map1', uv)]
    o.color_sets = [attr('colorSet1', np.full((nv, 4), 0.5))]
    o.vertex_indices = idx
    o.bone_influences = [A.BoneInfluence(OS_NEO, [A.VertexWeight(int(i), 1.0) for i in range(nv)])]
    t = couleur(mname)
    col_im = TEX[t] if t else Image.new('RGBA', (8, 8), (200, 200, 210, 255))
    transp = np.asarray(col_im)[..., 3].min() < 200
    lab = 'neo_' + mname.lower()
    col = 'neo_%s_col' % (t or 'blanc').lower().replace('.', '_').replace(' ', '_')
    up2(col_im).save(os.path.join(PNG, col + '.png'))
    # yeux : rouges lumineux (peu métalliques) ; le reste : métal brillant
    oeil = 'eye' in (t or '').lower() or '106' in mname
    prm = 'neo_prm_oeil' if oeil else 'neo_prm_metal'
    Image.new('RGBA', (8, 8), (0, 90, 255, 70) if oeil else (190, 70, 255, 140)).save(os.path.join(PNG, prm + '.png'))
    for x in (col, prm):
        if x not in textures:
            textures.append(x)
    e = copie(gab_alpha if transp else gabarit)
    e.material_label = lab
    for tx in e.textures:
        n = tx.param_id.name
        if n == 'Texture0':
            tx.data = col
        elif n == 'Texture4':
            tx.data = 'neo_nor'
        elif n == 'Texture6':
            tx.data = prm
    if lab not in [m.material_label for m in new_mats]:
        new_mats.append(e)
    new_objs.append(o)
    new_entries.append(S.modl_data.ModlEntryData(o.name, o.subindex, lab))
    print('OBJET', o.subindex, mname, nv // 3, 'triangles', 'texture', t, 'transparent' if transp else 'opaque', lab)

# ------------------------------------------------------------------ fusion avec le modèle du skin
mesh = S.mesh_data.read_mesh(os.path.join(SKIN, 'model.numshb'))
mesh.objects = [o for o in mesh.objects if not o.name.startswith('NeoMetal_')] + new_objs
mesh.save(os.path.join(OUT, 'model.numshb'))
modl = S.modl_data.read_modl(os.path.join(SKIN, 'model.numdlb'))
modl.entries = [e for e in modl.entries if not e.mesh_object_name.startswith('NeoMetal_')] + new_entries
modl.save(os.path.join(OUT, 'model.numdlb'))
try:
    S.meshex_data.MeshExData.from_mesh_objects(mesh.objects).save(os.path.join(OUT, 'model.numshexb'))
except Exception as e:  # noqa: BLE001
    print('numshexb non genere :', e)
matl.entries = [e for e in matl.entries if not e.material_label.startswith('neo_')] + new_mats
matl.save(os.path.join(OUT, 'model.numatb'))
json.dump({'echelle': ECH, 'boite': [lo.tolist(), hi.tolist()], 'textures': textures,
           'objets_metal': [o.name for o in mesh.objects if not o.name.startswith('NeoMetal_')]},
          open(os.path.join(OUT, 'neo.json'), 'w'), indent=1)

# ------------------------------------------------------------------ planche de contrôle (côté / face)
from PIL import ImageDraw  # noqa: E402
im = Image.new('RGB', (900, 520), (24, 22, 32))
d = ImageDraw.Draw(im)
for ob in mesh.objects:
    pp = np.array(ob.positions[0].data, dtype=float)[:, :3]
    isn = ob.name.startswith('NeoMetal_')
    for ax0, ox in ((2, 20), (0, 470)):
        for q in pp[:: (2 if isn else 6)]:
            d.point((ox + 210 + q[ax0] * 8, 500 - q[1] * 8), fill=(90, 255, 120) if isn else (150, 140, 160))
d.text((20, 6), 'cote (z = avant)', fill=(220, 220, 220))
d.text((470, 6), 'face (x)', fill=(220, 220, 220))
im.save(os.path.join(OUT, 'planche_neo.png'))
print('ok :', len(new_objs), 'objets Neo,', len(new_mats), 'materiaux,', len(textures), 'textures')
