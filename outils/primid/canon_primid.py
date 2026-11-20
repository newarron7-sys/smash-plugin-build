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

v0.48 (vidéo du joueur : le canon « se téléporte et bugue ») : plus sur Throw placé par le plugin, mais tenu dans la
main comme ses autres armes (HaveL, copie miroir pr_canon_r sur HaveR, cf. armes_primid.py) : il suit l'animation de
REX (soulevé puis posé) sans rien placer à chaque image. Placement : à l'image IMG du mouvement final (accroupi, canon
pointé), poignée sur la main, bouche droit devant (+z du modèle), à plat ; ramené dans le repère de HaveL à cette image.

usage : python3 canon_primid.py <dossier « Dark Cannon Trophy »> <model/body/cXXX> <motion/body/cXXX/d04final.nuanmb>
"""
import math, os, subprocess, sys
import numpy as np
import ssbh_data_py as S
from PIL import Image

SRC, D, ANIM = sys.argv[1:4]
IMG = 60
M = S.mesh_data
NOM = 'pr_canon'
ECH = 1.15
OS = 'HaveL'
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
# (X devant, Y haut, Z côté) -> modèle : x = -Z, y = Y, z = X (devant du perso = +z)
Rt = np.array([[0, 0, -1], [0, 1, 0], [1, 0, 0]])
R = Rt @ Rz @ Ry

# ---- main gauche à l'image IMG (convention colonne : monde = parent @ local)
sk = S.skel_data.read_skel(os.path.join(D, 'model.nusktb'))
UN = [b.name for b in sk.bones]
IDX = {n: i for i, n in enumerate(UN)}
PAR = [b.parent_index if b.parent_index is not None and b.parent_index >= 0 else None for b in sk.bones]
L0 = [np.array(b.transform, dtype=float).T for b in sk.bones]


def mat(v):
    x, y, z, w = v.rotation
    r = np.array([[1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
                  [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
                  [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)]])
    m = np.eye(4)
    m[:3, :3] = r * np.array(v.scale)[None, :]
    m[:3, 3] = v.translation
    return m


def mondes(L):
    W = [None] * len(L)

    def w(i):
        if W[i] is None:
            W[i] = (w(PAR[i]) @ L[i]) if PAR[i] is not None else L[i]
        return W[i]
    for i in range(len(L)):
        w(i)
    return W


L = [m.copy() for m in L0]
an = S.anim_data.read_anim(ANIM)
for gr in an.groups:
    if gr.group_type.name != 'Transform':
        continue
    for nd in gr.nodes:
        if nd.name in IDX and nd.tracks:
            vals = nd.tracks[0].values
            L[IDX[nd.name]] = mat(vals[min(IMG, len(vals) - 1)])
L[IDX['Trans']] = np.eye(4)
WF = mondes(L)[IDX['HaveL']]
W0 = mondes(L0)
WL0, WR0 = W0[IDX['HaveL']], W0[IDX['HaveR']]
FX = np.diag([-1.0, 1.0, 1.0, 1.0])
main = WF[:3, 3]
print('main gauche à l\'image %d : %s' % (IMG, np.round(main, 2)))
# monde (image IMG) -> local HaveL ; local HaveL -> local HaveR (miroir X au repos, comme armes_primid.py)
VERS_L = np.linalg.inv(WF)
VERS_R = np.linalg.inv(WR0) @ FX @ WL0 @ VERS_L
COPIES = [(NOM, 'HaveL', VERS_L), (NOM + '_r', 'HaveR', VERS_R)]
print('axe du canon : %.1f° autour de Y, %.1f° vers le haut ; longueur %.1f' % (math.degrees(a), math.degrees(p), np.linalg.norm(d) * ECH))


def place(pts, A):
    m = (pts - prise) @ R.T * ECH + main                 # modèle, à l'image IMG
    h = np.c_[m, np.ones(len(m))] @ A.T
    return h[:, :3].astype(np.float32)


def tourne(vs, A):
    return ((vs @ R.T) @ A[:3, :3].T).astype(np.float32)


# ---- modèle de Primid
mesh = M.read_mesh(os.path.join(D, 'model.numshb'))
matl = S.matl_data.read_matl(os.path.join(D, 'model.numatb'))
modl = S.modl_data.read_modl(os.path.join(D, 'model.numdlb'))
mesh.objects = [o for o in mesh.objects if o.name not in (NOM, NOM + '_r')]
modl.entries = [x for x in modl.entries if x.mesh_object_name not in (NOM, NOM + '_r')]
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


for nom, os_, A in COPIES:
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
        pos = place(V[[c[0] for c in ks]], A)
        nor = tourne(N[[c[2] for c in ks]], A)
        nor /= np.linalg.norm(nor, axis=1, keepdims=True) + 1e-9
        ix = np.asarray(idx, np.uint32)
        if np.linalg.det(A[:3, :3]) < 0:
            ix = ix.reshape(-1, 3)[:, ::-1].reshape(-1).copy()       # miroir : ordre des triangles inversé
        uv = T[[c[1] for c in ks]].copy()
        uv[:, 1] = 1.0 - uv[:, 1]
        o = M.MeshObjectData(nom, k)
        o.positions = [att('Position0', pos)]
        o.normals = [att('Normal0', nor)]
        o.texture_coordinates = [att('map1', uv)]
        o.color_sets = [att('colorSet1', np.full((len(pos), 4), 0.5019608, np.float32))]
        o.vertex_indices = ix
        o.tangents = [att('Tangent0', M.calculate_tangents_vec4(pos, nor, uv.astype(np.float32), o.vertex_indices))]
        o.parent_bone_name = os_
        o.bone_influences = []
        mesh.objects.append(o)
        modl.entries.append(S.modl_data.ModlEntryData(nom, k, labels[MAT[g]]))
        k += 1

mesh.save(os.path.join(D, 'model.numshb'))
matl.save(os.path.join(D, 'model.numatb'))
modl.save(os.path.join(D, 'model.numdlb'))
S.meshex_data.MeshExData.from_mesh_objects(mesh.objects).save(os.path.join(D, 'model.numshexb'))
print('canon : %d objets, %d textures' % (k, len(textures)))
print('TEXTURES ' + ' '.join(t + '.nutexb' for t in textures))
