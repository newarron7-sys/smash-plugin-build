"""Diagnostic d'un dossier de modèle de stage : attributs (NaN / inf / normales nulles), indices, matériaux, textures.
usage : python3 diag_modele.py <dossier du modèle> [dossier de référence d'un mod qui marche]"""
import glob, os, struct, sys
import numpy as np
import ssbh_data_py as S

def diag(D):
    print('=====', D)
    m = S.mesh_data.read_mesh(os.path.join(D, 'model.numshb'))
    print('numshb version', getattr(m, 'major_version', '?'), getattr(m, 'minor_version', '?'), len(m.objects), 'objets')
    for o in m.objects:
        att = []
        for grp in ('positions', 'normals', 'tangents', 'texture_coordinates', 'color_sets', 'binormals'):
            for a in getattr(o, grp):
                d = np.asarray(a.data, dtype=np.float64)
                bad = int((~np.isfinite(d)).sum())
                extra = ''
                if grp == 'normals':
                    extra = ' nulles=%d' % int((np.linalg.norm(d[:, :3], axis=1) < 1e-4).sum())
                att.append('%s%s[%d]%s%s' % (a.name, d.shape[1:], len(d), ' NaN/inf=%d' % bad if bad else '', extra))
        idx = np.asarray(o.vertex_indices)
        nv = len(np.asarray(o.positions[0].data))
        print(' %-10s sub%d os=%r inf=%d idx=%d max=%d nv=%d %s' % (o.name, o.subindex, o.parent_bone_name, len(o.bone_influences), len(idx),
              int(idx.max()) if len(idx) else -1, nv, ' '.join(att)))
        if len(idx) and idx.max() >= nv:
            print('   !!! indice hors limites')
        if len(idx) % 3:
            print('   !!! nombre d\'indices non multiple de 3')
    dl = S.modl_data.read_modl(os.path.join(D, 'model.numdlb'))
    print('numdlb', dl.model_name, dl.skeleton_file_name, dl.material_file_names, dl.animation_file_name, dl.mesh_file_name, len(dl.entries))
    ma = S.matl_data.read_matl(os.path.join(D, 'model.numatb'))
    labs = {e.material_label for e in ma.entries}
    for e in dl.entries:
        if e.material_label not in labs:
            print('   !!! matériau absent', e.material_label)
    noms = {(o.name, o.subindex) for o in m.objects}
    for e in dl.entries:
        if (e.mesh_object_name, e.mesh_object_subindex) not in noms:
            print('   !!! objet absent', e.mesh_object_name, e.mesh_object_subindex)
    for e in ma.entries:
        print(' mat %-22s %s tex=%s samp=%d vec=%d flt=%d bool=%d bl=%d ra=%d' % (e.material_label, e.shader_label,
              [(x.param_id.name, x.data) for x in e.textures], len(e.samplers), len(e.vectors), len(e.floats), len(e.booleans),
              len(e.blend_states), len(e.rasterizer_states)))
        for x in e.textures:
            t = x.data
            if not t.startswith('/') and not os.path.exists(os.path.join(D, t + '.nutexb')):
                print('   texture non fournie dans le dossier :', t)
    for f in sorted(glob.glob(os.path.join(D, '*.nutexb'))):
        b = open(f, 'rb').read()
        ft = b[-0x70:]
        name = ft[4:68].split(b'\0')[0]
        w, h, d_ = struct.unpack_from('<III', ft, 0x44)
        fmt = ft[0x50]
        mips = struct.unpack_from('<I', ft, 0x58)[0]
        print(' tex %-28s %dx%d fmt=%02x mips=%d nom=%r taille=%d' % (os.path.basename(f), w, h, fmt, mips, name, len(b)))

for D in sys.argv[1:]:
    try:
        diag(D)
    except Exception as ex:  # noqa: BLE001
        print('ERREUR', D, ex)
