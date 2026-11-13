"""Analyse du Zero de P+Ex/REX : sur quel perso de Brawl il est basé, ses sous-actions, ses spéciaux, ses animations.

usage : python3 analyse.py <dossier de travail (zip GameBanana déjà téléchargé)>
"""
import glob, os, re, sys, zipfile
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, '../../black-shadow/tools'))
from psa import load, sections, subactions, events  # noqa: E402
import gen  # noqa: E402

work = sys.argv[1]
z = zipfile.ZipFile(glob.glob(os.path.join(work, '*.zip'))[0])
pk = zipfile.ZipFile(z.open([n for n in z.namelist() if n.endswith('.fighterpackage')][0]))
pkg = os.path.join(work, 'pkg')
pk.extractall(pkg)

mod = open(os.path.join(pkg, 'Module/ft_zero.rel'), 'rb').read()
strs = sorted(set(s.decode() for s in re.findall(rb'[A-Za-z_][A-Za-z0-9_:<>\.]{4,}', mod)))
print('MODULE taille', len(mod), 'id', int.from_bytes(mod[0:4], 'big'))
print('MODULE chaines', len(strs))
for s in strs:
    if re.search(r'(?i)ft[A-Z_a-z]|fighter|marth|roy|ike|link|lucas|ness|mario|fox|falco|wolf|zelda|sheik|pit|meta|sonic|snake|samus|kirby|ganon|captain|lucario|zero|item|article|status', s):
        print('  ', s)

m = load(os.path.join(pkg, 'PacFiles/FitZero.pac'))
base, sec, ext = sections(m)
print('PSA sections', list(sec))
print('PSA externes', [e[1] for e in ext][:60])
base, subs = subactions(m)
lens = {}
try:
    lens = gen.anim_lengths(os.path.join(pkg, 'PacFiles/FitZeroMotion.pac'))
except Exception as e:  # noqa
    print('anim_lengths echec', e)
print('ANIMATIONS', len(lens))
print('SOUS-ACTIONS', len(subs))
for i, n, mo in subs:
    if not n:
        continue
    nev = len(events(m, base, mo)) if mo else 0
    tag = ' <== SPECIAL' if n.startswith('Special') or n.startswith('Final') else ''
    print('  %3d %-28s %4s images  %3d evenements%s' % (i, n, lens.get(n, '-'), nev, tag))
for f in sorted(os.listdir(os.path.join(pkg, 'PacFiles'))):
    print('PAC', f, os.path.getsize(os.path.join(pkg, 'PacFiles', f)))
