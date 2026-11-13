"""Zero (skin de _Silent_ sur Roy c00) : ajoute le canon (buster) de son modèle P+ au modèle du skin.

usage : python3 add_buster.py <modèle Zero P+ converti (build_model.py, KEEP_VIS=BusterOnM)> <model.nusktb de Roy>
                              <dossier modèle du skin (c00)> <sortie (dossier c00)>
- objets « Buster » du modèle P+ (visibilité BusterOnM dans P+) ;
- un os à lui, ZeroBusterN, enfant de HandR (main droite de Roy, même repère) : le canon y est collé. Le plugin
  le montre (échelle 1) seulement pendant les tirs, et rétrécit la main ; sinon échelle presque nulle ;
- placement : poses de repos en T des deux squelettes, même orientation -> le canon garde sa place par rapport
  à la main (déplacé de la main de Zero à celle de Roy, à la taille de Roy).
"""
import os, shutil, sys
import numpy as np
import ssbh_data_py as S

REX, SKEL, SIL, OUT = sys.argv[1:5]
os.makedirs(OUT, exist_ok=True)


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


rex_sk = S.skel_data.read_skel(os.path.join(REX, 'model.nusktb'))
roy_sk = S.skel_data.read_skel(SKEL)
WB, WR = worlds(rex_sk), worlds(roy_sk)
SCALE = WR['Hip'][1, 3] / WB['Hip'][1, 3]
hand_b = WB['HandR'][:3, 3] * SCALE
hand_r = WR['HandR'][:3, 3]
print('echelle %.3f ; main droite Zero %s -> Roy %s' % (SCALE, np.round(hand_b, 2), np.round(hand_r, 2)))
rb_ = WB['HandR'][:3, :3] / np.linalg.norm(WB['HandR'][:3, :3], axis=0)
rr_ = WR['HandR'][:3, :3] / np.linalg.norm(WR['HandR'][:3, :3], axis=0)
print('ecart d orientation des mains au repos : %.1f deg'
      % np.degrees(np.arccos(np.clip((np.trace(rb_.T @ rr_) - 1) / 2, -1, 1))))

# ------------------------------------------------------------------ squelette : Roy + ZeroBusterN
names = [b.name for b in roy_sk.bones]
# v0.10.1 : enfant de l'avant-bras (ArmR), posé à la place de la main : la main peut être rétrécie sans toucher
# au canon (le canon n'apparaissait pas : échelle main 0,01 x canon 100 pas fiable)
roy_sk.bones = [b for b in roy_sk.bones if b.name != 'ZeroBusterN']
_loc = np.linalg.inv(WR['ArmR']) @ WR['HandR']
roy_sk.bones.append(S.skel_data.BoneData('ZeroBusterN', np.ascontiguousarray(_loc.T, dtype=np.float32), names.index('ArmR')))
roy_sk.save(os.path.join(OUT, 'model.nusktb'))
print('squelette : %d os (+ ZeroBusterN sous ArmR, a la place de la main)' % len(roy_sk.bones))
_chk = worlds(roy_sk)['ZeroBusterN'][:3, 3]
print('ZeroBusterN au repos', np.round(_chk, 2), 'main', np.round(hand_r, 2))

# ------------------------------------------------------------------ objets du canon
rex_mesh = S.mesh_data.read_mesh(os.path.join(REX, 'model.numshb'))
rex_modl = S.modl_data.read_modl(os.path.join(REX, 'model.numdlb'))
mat_of = {(e.mesh_object_name, e.mesh_object_subindex): e.material_label for e in rex_modl.entries}
bus = [o for o in rex_mesh.objects if 'Buster' in o.name]
print('objets du canon', [(o.name, len(o.vertex_indices) // 3, mat_of.get((o.name, o.subindex))) for o in bus])
new_objs, new_entries = [], []
lo, hi = np.full(3, 1e9), np.full(3, -1e9)
for o in bus:
    p = np.array(o.positions[0].data, dtype=float)[:, :3]
    p = p * SCALE - hand_b + hand_r
    lo, hi = np.minimum(lo, p.min(0)), np.maximum(hi, p.max(0))
    o.positions[0].data = np.c_[p, np.ones(len(p))].astype(np.float32) if np.array(o.positions[0].data).shape[1] == 4 else p.astype(np.float32)
    nv = len(p)
    o.bone_influences = [S.mesh_data.BoneInfluence('ZeroBusterN', [S.mesh_data.VertexWeight(i, 1.0) for i in range(nv)])]
    lab = mat_of.get((o.name, o.subindex))
    # v0.12.1 : un seul nom, « _VIS_O_OBJ » : caché par défaut, montré par la piste de visibilité « ZeroBuster »
    # des animations du B neutre et du lancer arrière (et par le plugin)
    o.name = 'ZeroBuster_VIS_O_OBJShape'
    o.subindex = len(new_objs)
    new_objs.append(o)
    new_entries.append(S.modl_data.ModlEntryData(o.name, o.subindex, lab))
print('canon chez Roy : de %s a %s (taille %s)' % (np.round(lo, 2), np.round(hi, 2), np.round(hi - lo, 2)))

# ------------------------------------------------------------------ fusion avec le modèle du skin
mesh = S.mesh_data.read_mesh(os.path.join(SIL, 'model.numshb'))
mesh.objects = [o for o in mesh.objects if not o.name.startswith('ZeroBuster_')] + new_objs
mesh.save(os.path.join(OUT, 'model.numshb'))
modl = S.modl_data.read_modl(os.path.join(SIL, 'model.numdlb'))
modl.entries = [e for e in modl.entries if not e.mesh_object_name.startswith('ZeroBuster_')] + new_entries
modl.save(os.path.join(OUT, 'model.numdlb'))
try:
    S.meshex_data.MeshExData.from_mesh_objects(mesh.objects).save(os.path.join(OUT, 'model.numshexb'))
except Exception as e:  # noqa: BLE001
    print('numshexb non genere :', e)
matl = S.matl_data.read_matl(os.path.join(SIL, 'model.numatb'))
rex_matl = S.matl_data.read_matl(os.path.join(REX, 'model.numatb'))
want = {e.material_label for e in new_entries}
have = {e.material_label for e in matl.entries}
tex = set()
for e in rex_matl.entries:
    if e.material_label in want and e.material_label not in have:
        matl.entries.append(e)
        for t in e.textures:
            if not t.data.startswith('#') and not t.data.startswith('/'):
                tex.add(t.data)
matl.save(os.path.join(OUT, 'model.numatb'))
print('materiaux ajoutes', sorted(want - have), 'textures', sorted(tex))
copied = []
for t in sorted(tex):
    src = os.path.join(REX, t + '.nutexb')
    if os.path.exists(src):
        shutil.copy(src, os.path.join(OUT, t + '.nutexb'))
        copied.append(t)
    else:
        print('TEXTURE ABSENTE', t)
open(os.path.join(OUT, 'nouvelles_textures.txt'), 'w').write('\n'.join(copied) + '\n')

# ------------------------------------------------------------------ planche de contrôle (2 vues)
from PIL import Image, ImageDraw  # noqa: E402
allp = []
for o in mesh.objects:
    p = np.array(o.positions[0].data, dtype=float)[:, :3]
    allp.append((p, o.name.startswith('ZeroBuster_')))
im = Image.new('RGB', (900, 520), (24, 22, 32))
d = ImageDraw.Draw(im)
for vi, (ax0, ax1, ox) in enumerate(((2, 1, 20), (0, 1, 470))):
    for p, isb in allp:
        sc = 20.0
        for q in p[:: (1 if isb else 4)]:
            x = ox + 210 + q[ax0] * sc
            y = 500 - q[ax1] * sc
            d.point((x, y), fill=(90, 255, 120) if isb else (150, 140, 160))
d.text((20, 6), 'cote (z = avant)', fill=(220, 220, 220))
d.text((470, 6), 'face (x)', fill=(220, 220, 220))
im.save(os.path.join(OUT, 'planche_canon.png'))
print('ok')
