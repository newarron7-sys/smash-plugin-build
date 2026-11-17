"""Primid v0.37 : Mite, Boobas et Cymul dans le modèle du J.E.T. (pump, comme Botron).

Le joueur (v0.36) : les invocations collées à l'os Throw de Primid buguaient au demi-tour et aux prises (le jeu tourne
le modèle de Primid et prend Throw pour tenir l'adversaire). Le J.E.T. est un objet à part : elles y sont cuites
(mêmes poses que invocs_primid.py), sur l'os racine Trans du J.E.T. (immobile dans ses animations). Le plugin place
l'objet et le tourne (rot Y 0 ou 180) : un seul exemplaire par pose (pr_<nom><pose>).
Repère du J.E.T. tourné de 0 : x = droite de l'écran (devant de l'invocation), y haut, z = plan.
Retire aussi les anciennes poses du modèle de Primid (body/c00) et leurs textures.

usage : python3 invocs_pump.py <model/pump/c00> <model/body/c00> <config.json>
"""
import json, os, shutil, subprocess, sys
import numpy as np
import ssbh_data_py as S

D, BODY, CONF = sys.argv[1:4]
M = S.mesh_data
mesh = M.read_mesh(os.path.join(D, 'model.numshb'))
matl = S.matl_data.read_matl(os.path.join(D, 'model.numatb'))
modl = S.modl_data.read_modl(os.path.join(D, 'model.numdlb'))
skel = S.skel_data.read_skel(os.path.join(D, 'model.nusktb'))
OS = 'Trans'
ob = next(b for b in skel.bones if b.name == OS)
INV = np.linalg.inv(np.asarray(skel.calculate_world_transform(ob), np.float32))
CS = np.array([0.5019608] * 4, np.float32)
for o in mesh.objects:
    if o.color_sets:
        CS = np.array(o.color_sets[0].data[0], dtype=np.float32)
        break


def att(n, d):
    a = M.AttributeData(n)
    a.data = np.asarray(d, np.float32)
    return a


def mat(v):
    x, y, z, w = v.rotation
    r = np.array([[1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
                  [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
                  [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)]])
    m = np.eye(4)
    m[:3, :3] = r * np.array(v.scale)[None, :]
    m[:3, 3] = v.translation
    return m


def tr(v, plr, s):
    """repère de l'invocation (x devant, y haut, z plan) -> modèle de Primid ; s = -1 : demi-tour autour de Y"""
    v = np.asarray(v, np.float32)
    return np.stack([v[:, 0] * s, v[:, 1], v[:, 2] * s], 1).astype(np.float32)


class Invoc:
    def __init__(self, c):
        self.c = c
        mo = c['modele']
        md = os.path.join(mo, 'model') if os.path.isdir(os.path.join(mo, 'model')) else mo
        self.md = md
        self.mesh = M.read_mesh(os.path.join(md, 'model.numshb'))
        self.mat = S.matl_data.read_matl(os.path.join(md, 'model.numatb'))
        self.mod = S.modl_data.read_modl(os.path.join(md, 'model.numdlb'))
        sk = S.skel_data.read_skel(os.path.join(md, 'model.nusktb'))
        self.an = S.anim_data.read_anim(c['anim'])
        self.UN = [b.name for b in sk.bones]
        self.IDX = {n: i for i, n in enumerate(self.UN)}
        self.PAR = [b.parent_index for b in sk.bones]
        self.L0 = [np.array(b.transform, float).T for b in sk.bones]
        self.INV0 = [np.linalg.inv(m) for m in self.mondes(self.L0)]

    def mondes(self, L):
        W = [None] * len(L)

        def w(i):
            if W[i] is None:
                W[i] = (w(self.PAR[i]) @ L[i]) if self.PAR[i] is not None else L[i]
            return W[i]
        for i in range(len(L)):
            w(i)
        return W

    def pose(self, f):
        L = [m.copy() for m in self.L0]
        for g in self.an.groups:
            if g.group_type.name != 'Transform':
                continue
            for nd in g.nodes:
                if nd.name in self.IDX and nd.tracks:
                    vals = nd.tracks[0].values
                    L[self.IDX[nd.name]] = mat(vals[min(f, len(vals) - 1)])
        L[self.IDX['Trans']] = np.eye(4)
        W = self.mondes(L)
        return [W[i] @ self.INV0[i] for i in range(len(L))]

    def peau(self, o, P):
        p = np.asarray(o.positions[0].data, float)[:, :3]
        n = np.asarray(o.normals[0].data, float)[:, :3]
        t = np.asarray(o.tangents[0].data, float) if o.tangents else np.c_[np.tile([1.0, 0, 0], (len(p), 1)), np.ones(len(p))]
        ph = np.c_[p, np.ones(len(p))]
        op, on, ot, ws = np.zeros_like(p), np.zeros_like(n), np.zeros((len(p), 3)), np.zeros(len(p))
        if o.bone_influences:
            for inf in o.bone_influences:
                Mb = P[self.IDX[inf.bone_name]]
                for vw in inf.vertex_weights:
                    i, w = vw.vertex_index, vw.vertex_weight
                    op[i] += w * (Mb @ ph[i])[:3]
                    on[i] += w * (Mb[:3, :3] @ n[i])
                    ot[i] += w * (Mb[:3, :3] @ t[i, :3])
                    ws[i] += w
        elif o.parent_bone_name in self.IDX:
            Mb = P[self.IDX[o.parent_bone_name]] @ self.mondes(self.L0)[self.IDX[o.parent_bone_name]]
            op[:] = (ph @ Mb.T)[:, :3]
            on[:] = n @ Mb[:3, :3].T
            ot[:] = t[:, :3] @ Mb[:3, :3].T
            ws[:] = 1
        m = ws > 0
        op[~m], on[~m], ot[~m] = p[~m], n[~m], t[~m, :3]
        on /= np.maximum(np.linalg.norm(on, axis=1, keepdims=True), 1e-6)
        ot /= np.maximum(np.linalg.norm(ot, axis=1, keepdims=True), 1e-6)
        k = float(self.c.get('echelle', 1.0))
        re = lambda a: np.stack([a[:, 2], a[:, 1], -a[:, 0]], 1).astype(np.float32)
        tw = t[:, 3:4] if t.shape[1] > 3 else np.ones((len(p), 1))
        return re(op) * k, re(on), np.c_[re(ot), tw].astype(np.float32)


confs = json.load(open(CONF))
noms = [c['nom'] for c in confs]
objs = [o for o in mesh.objects if not any(o.name.startswith('pr_' + n) for n in noms)]
ents = [x for x in modl.entries if not any(x.mesh_object_name.startswith('pr_' + n) for n in noms)]
mats = [e for e in matl.entries if not any(e.material_label.startswith('Pr%s_' % n.capitalize()) for n in noms)]
textures = []
for c in confs:
    iv = Invoc(c)
    nom = c['nom']
    # matériaux (visibles des deux côtés)
    lab_de = {}
    for e in iv.mat.entries:
        lab = 'Pr%s_%s' % (nom.capitalize(), e.material_label)
        lab_de[e.material_label] = lab
        e.material_label = lab
        rs = e.rasterizer_states
        for p in rs:
            d = p.data
            d.cull_mode = S.matl_data.CullMode.Disabled
            p.data = d
        e.rasterizer_states = rs
        mats.append(e)
    mat_de = {x.mesh_object_name + '/%d' % x.mesh_object_subindex: x.material_label for x in iv.mod.entries}
    # textures : .nutexb du dossier du modèle, sinon png/ converties
    nut = [f for f in os.listdir(iv.md) if f.endswith('.nutexb')]
    if nut:
        for f in nut:
            shutil.copy(os.path.join(iv.md, f), os.path.join(D, f))
            textures.append(f)
    else:
        pd = os.path.join(c['modele'], 'png')
        for f in sorted(os.listdir(pd)):
            if not f.endswith('.png'):
                continue
            fmt = 'BC7RgbaUnormSrgb' if f.endswith('_col.png') else 'BC7RgbaUnorm'
            out = os.path.join(D, f[:-4] + '.nutexb')
            subprocess.run(['ultimate_tex_cli', os.path.join(pd, f), out, '--format', fmt], check=True, capture_output=True)
            textures.append(f[:-4] + '.nutexb')
    ex = {'': (1, 1)}
    nv = 0
    for kp, fp in enumerate(c['poses']):
        P = iv.pose(fp)
        cuits = [iv.peau(src, P) for src in iv.mesh.objects]
        for l, (plr, s) in ex.items():
            nomo = 'pr_%s%d%s' % (nom, kp, l)
            for k, src in enumerate(iv.mesh.objects):
                pp, nn, tt = cuits[k]
                o = M.MeshObjectData(nomo, k)
                o.positions = [att('Position0', M.transform_points(tr(pp, plr, s), INV))]
                o.normals = [att('Normal0', M.transform_vectors(tr(nn, plr, s), INV))]
                t3 = np.asarray(M.transform_vectors(tr(tt[:, :3], plr, s), INV), np.float32)
                o.tangents = [att('Tangent0', np.concatenate([t3, tt[:, 3:4]], 1))]
                o.texture_coordinates = [att(u.name, u.data) for u in src.texture_coordinates]
                if src.color_sets:
                    o.color_sets = [att(cc.name, cc.data) for cc in src.color_sets]
                elif CS is not None:
                    o.color_sets = [att('colorSet1', np.tile(CS, (len(pp), 1)))]
                o.vertex_indices = np.asarray(src.vertex_indices, np.uint32)
                o.parent_bone_name = OS
                o.bone_influences = []
                o.sort_bias = 0
                objs.append(o)
                nv += len(pp)
                lab = mat_de.get('%s/%d' % (src.name, src.subindex)) or iv.mod.entries[k].material_label
                ents.append(S.modl_data.ModlEntryData(nomo, k, lab_de[lab]))
    print('%s : %d poses x %d exemplaires, %d sommets' % (nom, len(c['poses']), len(ex), nv))
mesh.objects = objs
modl.entries = ents
matl.entries = mats
mesh.save(os.path.join(D, 'model.numshb'))
matl.save(os.path.join(D, 'model.numatb'))
modl.save(os.path.join(D, 'model.numdlb'))
S.meshex_data.MeshExData.from_mesh_objects(mesh.objects).save(os.path.join(D, 'model.numshexb'))
# anciennes poses dans le modèle de Primid
bm = M.read_mesh(os.path.join(BODY, 'model.numshb'))
bmat = S.matl_data.read_matl(os.path.join(BODY, 'model.numatb'))
bmod = S.modl_data.read_modl(os.path.join(BODY, 'model.numdlb'))
av = len(bm.objects)
bm.objects = [o for o in bm.objects if not any(o.name.startswith('pr_' + n) for n in noms)]
bmod.entries = [x for x in bmod.entries if not any(x.mesh_object_name.startswith('pr_' + n) for n in noms)]
bmat.entries = [e for e in bmat.entries if not any(e.material_label.startswith('Pr%s_' % n.capitalize()) for n in noms)]
bm.save(os.path.join(BODY, 'model.numshb'))
bmat.save(os.path.join(BODY, 'model.numatb'))
bmod.save(os.path.join(BODY, 'model.numdlb'))
S.meshex_data.MeshExData.from_mesh_objects(bm.objects).save(os.path.join(BODY, 'model.numshexb'))
utiles = set()
for e in bmat.entries:
    for t in e.textures:
        utiles.add(t.data.lower())
for f in sorted(set(textures)):
    p = os.path.join(BODY, f)
    if os.path.exists(p) and f[:-7].lower() not in utiles:
        os.remove(p)
print('corps de Primid : %d -> %d objets' % (av, len(bm.objects)))
print('textures :', len(set(textures)))
