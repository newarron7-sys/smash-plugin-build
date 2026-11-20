"""Primid v0.45 : le Dark Cannon (objet du mode Aventure de Brawl) pour son Final Smash.

REX n'a pas de modèle de canon dans les fichiers de Primid (il prend l'objet de Brawl) : modèle du trophée « Dark
Cannon » (The Models Resource, asset 286236, DolDarkcanon.obj + textures BoDCN_*), envoyé par le joueur.
Le trophée est posé en biais (axe poignée -> bouche : 36° autour de Y, 15° vers le haut) : remis à plat, bouche
devant ; origine = poignée (polygon18) ; échelle 1,15 (vidéo de REX : canon ~1,3 fois la taille de Primid).
Os : Throw (le plugin le place entre les mains de Primid). Repère de Throw (constaté en v0.41 sur les tirs) : le devant
du perso est son -z.
Parties sans texture dans l'export (mat6 à mat10 : métal à reflet dans Brawl) : couleurs de la vidéo de REX (dorures,
boule et voyants rouges), PRM métal.
Shader sans diffusion sous la surface (SFX_PBS_0100000008008269, cf. Cymul v0.42).

usage : python3 canon_primid.py <dossier « Dark Cannon Trophy »> <model/body/cXXX>
"""
import math, os, subprocess, sys
import numpy as np
import ssbh_data_py as S
from PIL import Image

SRC, D = sys.argv[1:3]
M = S.mesh_data
NOM = 'pr_canon'
ECH = 1.15
OS = 'Throw'
# couleurs des parties sans texture (vidéo de REX) et métal
SANS_TEX = {'mat6': ((200, 30, 30), 0.3), 'mat7': ((205, 160, 60), 1.0), 'mat8': ((190, 20, 25), 0.15),
            'mat9': ((140, 95, 35), 1.0), 'mat10': ((205, 160, 60), 1.0)}

# ---- lecture de l'OBJ
V, T, N = [], [], []
G, MAT = {}, {}
cur = None
for l in open(os.path.join(SRC, 'DolDarkcanon.obj')):
    if l.startswith('v '):
        V.append([float(x) for x in l.split()[1:4]])
    elif l.startswith('vt '):
        T.append([float(x) for x in l.split()[1:3]])
    elif l.startswith('vn '):
        N.append([float(x) for x in l.split()[1:4]])
    elif l.startswith('g '):
        cur = l.split()[1]
    elif l.startswith('usemtl'):
        MAT[cur] = l.split()[1]
    elif l.startswith('f '):
        G.setdefault(cur, []).append([tuple(int(a) - 1 for a in t.split('/')) for t in l.split()[1:]])
V, T, N = np.array(V), np.array(T), np.array(N)
TEX = {}
m = None
for l in open(os.path.join(SRC, 'DolDarkcanon.mtl')):
    l = l.strip()
    if l.startswith('newmtl'):
        m = l.split()[1]
    elif l.startswith('map_Kd'):
        TEX[m] = l.split()[1]


def centre(g):
    idx = np.unique([i[0] for f in G[g] for i in f])
    return V[idx].mean(0)


# ---- repère : poignée -> bouche
prise = centre('polygon18')
bouche = centre('polygon22')
d = bouche - prise
a = math.atan2(d[2], d[0])
p = math.atan2(d[1], math.hypot(d[0], d[2]))
Ry = np.array([[math.cos(a), 0, math.sin(a)], [0, 1, 0], [-math.sin(a), 0, math.cos(a)]])
Rz = np.array([[math.cos(p), math.sin(p), 0], [-math.sin(p), math.cos(p), 0], [0, 0, 1]])
# (X devant, Y haut, Z côté) -> Throw : x = Z, y = Y, z = -X
Rt = np.array([[0, 0, 1], [0, 1, 0], [-1, 0, 0]])
R = Rt @ Rz @ Ry
print('axe du canon : %.1f° autour de Y, %.1f° vers le haut ; longueur %.1f' % (math.degrees(a), math.degrees(p), np.linalg.norm(d) * ECH))


def place(pts):
    return ((pts - prise) @ R.T * ECH).astype(np.float32)


def tourne(vs):
    return (vs @ R.T).astype(np.float32)


# ---- modèle de Primid
mesh = M.read_mesh(os.path.join(D, 'model.numshb'))
matl = S.matl_data.read_matl(os.path.join(D, 'model.numatb'))
modl = S.modl_data.read_modl(os.path.join(D, 'model.numdlb'))
mesh.objects = [o for o in mesh.objects if o.name != NOM]
modl.entries = [x for x in modl.entries if x.mesh_object_name != NOM]
matl.entries = [e for e in matl.entries if not e.material_label.startswith('PrCanon_')]
modele = next(e for e in matl.entries if e.material_label.startswith('PrArme_WpnPrimSuperScope'))


def nutexb(png, nom, fmt):
    subprocess.run(['ultimate_tex_cli', png, os.path.join(D, nom + '.nutexb'), '--format', fmt], check=True, capture_output=True)


tmp = os.path.join(D, '_canon_tmp.png')
Image.new('RGBA', (8, 8), (128, 128, 255, 255)).save(tmp)
nutexb(tmp, 'canon_nor', 'BC7RgbaUnorm')
Image.new('RGBA', (8, 8), (0, 115, 255, 128)).save(tmp)          # métal 0, rugosité 0,45, AO 1, spéculaire 0,5
nutexb(tmp, 'canon_prm', 'BC7RgbaUnorm')
Image.new('RGBA', (8, 8), (255, 77, 255, 128)).save(tmp)         # métal 1, rugosité 0,3
nutexb(tmp, 'canon_metal_prm', 'BC7RgbaUnorm')
textures = ['canon_nor', 'canon_prm', 'canon_metal_prm']
labels = {}
for mt in sorted(set(MAT.values())):
    if mt in TEX:
        col = 'canon_' + TEX[mt][:-4].lower().replace('bodcn_', '') + '_col'
        im = Image.open(os.path.join(SRC, TEX[mt])).convert('RGBA')
        im.save(tmp)
        prm = 'canon_prm'
    else:
        c, metal = SANS_TEX[mt]
        col = 'canon_%s_col' % mt
        Image.new('RGBA', (8, 8), c + (255,)).save(tmp)
        prm = 'canon_metal_prm' if metal > 0.5 else 'canon_prm'
    nutexb(tmp, col, 'BC7RgbaUnormSrgb')
    textures.append(col)
    e = S.matl_data.read_matl(os.path.join(D, 'model.numatb')).entries[matl.entries.index(modele)]
    e.material_label = 'PrCanon_' + mt
    e.shader_label = 'SFX_PBS_0100000008008269_opaque'
    e.vectors = [v for v in e.vectors if v.param_id.name not in ('CustomVector11', 'CustomVector30')]
    tx = e.textures
    for t in tx:
        if t.param_id.name == 'Texture0':
            t.data = col
        elif t.param_id.name == 'Texture4':
            t.data = 'canon_nor'
        elif t.param_id.name == 'Texture6':
            t.data = prm
    e.textures = tx
    rs = e.rasterizer_states
    for r in rs:
        dd = r.data
        dd.cull_mode = S.matl_data.CullMode.Disabled
        r.data = dd
    e.rasterizer_states = rs
    matl.entries.append(e)
    labels[mt] = e.material_label
os.remove(tmp)


def att(n, dta):
    x = M.AttributeData(n)
    x.data = np.asarray(dta, np.float32)
    return x


k = 0
for g in sorted(G, key=lambda s: int(s.replace('polygon', ''))):
    cles, idx = {}, []
    for f in G[g]:
        for j in range(1, len(f) - 1):
            for c in (f[0], f[j], f[j + 1]):
                if c not in cles:
                    cles[c] = len(cles)
                idx.append(cles[c])
    ks = list(cles)
    pos = place(V[[c[0] for c in ks]])
    nor = tourne(N[[c[2] for c in ks]])
    nor /= np.linalg.norm(nor, axis=1, keepdims=True) + 1e-9
    uv = T[[c[1] for c in ks]].copy()
    uv[:, 1] = 1.0 - uv[:, 1]
    o = M.MeshObjectData(NOM, k)
    o.positions = [att('Position0', pos)]
    o.normals = [att('Normal0', nor)]
    o.texture_coordinates = [att('map1', uv)]
    o.color_sets = [att('colorSet1', np.full((len(pos), 4), 0.5019608, np.float32))]
    o.vertex_indices = np.asarray(idx, np.uint32)
    o.tangents = [att('Tangent0', M.calculate_tangents_vec4(pos, nor, uv.astype(np.float32), o.vertex_indices))]
    o.parent_bone_name = OS
    o.bone_influences = []
    mesh.objects.append(o)
    modl.entries.append(S.modl_data.ModlEntryData(NOM, k, labels[MAT[g]]))
    k += 1

mesh.save(os.path.join(D, 'model.numshb'))
matl.save(os.path.join(D, 'model.numatb'))
modl.save(os.path.join(D, 'model.numdlb'))
S.meshex_data.MeshExData.from_mesh_objects(mesh.objects).save(os.path.join(D, 'model.numshexb'))
print('canon : %d objets, %d textures' % (k, len(textures)))
print('TEXTURES ' + ' '.join(t + '.nutexb' for t in textures))
