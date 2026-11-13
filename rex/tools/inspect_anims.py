"""Diagnostic : contenu de fichiers .nuanmb (groupes, os / maillages / matériaux animés, images).
usage : python3 inspect_anims.py <dossier> [motif]"""
import os, sys
import ssbh_data_py as S
D = sys.argv[1]
PAT = sys.argv[2] if len(sys.argv) > 2 else ''
for r, _, fs in os.walk(D):
    for f in sorted(fs):
        if not f.endswith('.nuanmb') or PAT not in f:
            continue
        p = os.path.join(r, f)
        try:
            a = S.anim_data.read_anim(p)
        except Exception as e:  # noqa: BLE001
            print(p, 'illisible', e)
            continue
        print('==', os.path.relpath(p, D), 'images', a.final_frame_index + 1)
        for g in a.groups:
            names = [nd.name for nd in g.nodes]
            print('   ', g.group_type, len(names), ' '.join(names[:80]))
