"""Analyse d'un perso de Super Smash Bros REX (sans rien publier du mod) : fichiers, perso de base dans Brawl
(module .rel, squelette du modèle), sous-actions, spéciaux, animations.

usage : python3 analyse_rex.py <dossier pf/Fighter/<nom>> <module ft_<nom>.rel ou -> <nom affiché>
"""
import os, re, sys, glob, struct, collections
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, '../../black-shadow/tools'))
from arc import arc_entries  # noqa: E402
from psa import load, sections, subactions, events  # noqa: E402
import gen  # noqa: E402

d, rel, name = sys.argv[1], sys.argv[2], sys.argv[3]
files = sorted(os.listdir(d))
print('=== FICHIERS de', name, '(%d)' % len(files))
for f in files:
    print('  %10d  %s' % (os.path.getsize(os.path.join(d, f)), f))


def pick(pat):
    c = [f for f in files if re.fullmatch(pat, f, re.I)]
    return os.path.join(d, c[0]) if c else None


# ------------------------------------------------------------------ module de code
BASES = ['mario', 'donkey', 'link', 'samus', 'yoshi', 'kirby', 'fox', 'pikachu', 'luigi', 'captain', 'ness', 'koopa',
         'peach', 'zelda', 'sheik', 'popo', 'nana', 'marth', 'gamewatch', 'falco', 'ganon', 'wario', 'metaknight',
         'pit', 'szerosuit', 'pikmin', 'lucas', 'diddy', 'poke', 'pokelizardon', 'pokezenigame', 'pokefushigisou',
         'dedede', 'lucario', 'ike', 'robot', 'purin', 'toonlink', 'wolf', 'snake', 'sonic', 'mewtwo', 'roy', 'knuckles']
if rel != '-' and os.path.exists(rel):
    m = open(rel, 'rb').read()
    print('=== MODULE', os.path.basename(rel), 'taille', len(m), 'id', int.from_bytes(m[0:4], 'big'))
    strs = sorted(set(s.decode() for s in re.findall(rb'[A-Za-z_][A-Za-z0-9_:<>\.]{3,}', m)))
    print('chaines', len(strs))
    cnt = collections.Counter()
    for s in strs:
        for b in BASES:
            if re.search(r'(?i)(ft|Ft)_?%s\b|ft%s[A-Z]|%s' % (b, b.capitalize(), b.capitalize()), s):
                cnt[b] += 1
    print('PERSOS DE BRAWL CITES DANS LE MODULE', cnt.most_common(8))
    for s in strs:
        if re.search(r'(?i)^ft|fighter|status|article|weapon|::', s):
            print('  ', s)

# ------------------------------------------------------------------ squelette du modèle (costume 00)
mdl = pick(r'Fit%s00\.pac' % re.escape(name)) or next((os.path.join(d, f) for f in files if re.search(r'00\.pac$', f, re.I) and 'motion' not in f.lower()), None)
if mdl:
    try:
        bones = gen.bone_names(mdl)
        print('=== SQUELETTE', os.path.basename(mdl), len(bones), 'os')
        print('  ', ' '.join(bones[i] for i in sorted(bones)))
    except Exception as e:  # noqa
        print('squelette echec', e)

# ------------------------------------------------------------------ moveset (PSA)
mv = pick(r'Fit%s\.pac' % re.escape(name)) or next((os.path.join(d, f) for f in files if re.fullmatch(r'Fit[A-Za-z0-9]+\.pac', f)), None)
mot = pick(r'Fit%sMotionEtc\.pac' % re.escape(name)) or pick(r'Fit%sMotion\.pac' % re.escape(name)) \
    or next((os.path.join(d, f) for f in files if 'motion' in f.lower()), None)
lens = {}
if mot:
    try:
        lens = gen.anim_lengths(mot)
    except Exception as e:  # noqa
        print('animations echec', e)
print('=== ANIMATIONS', len(lens), '(%s)' % (os.path.basename(mot) if mot else 'aucun fichier'))
if mv:
    try:
        m = load(mv)
        base, sec, ext = sections(m)
        print('=== MOVESET', os.path.basename(mv), 'sections', list(sec))
        print('externes', [e[1] for e in ext][:40])
        base, subs = subactions(m)
        print('SOUS-ACTIONS', len(subs))
        for i, n, mo in subs:
            if not n:
                continue
            nev = len(events(m, base, mo)) if mo else 0
            tag = ' <== SPECIAL' if n.startswith(('Special', 'Final')) else ''
            print('  %3d %-28s %4s images  %3d evenements%s' % (i, n, lens.get(n, '-'), nev, tag))
    except Exception as e:  # noqa
        print('moveset echec', e)
# animations sans sous-action (propres au perso)
if lens:
    print('=== LISTE DES ANIMATIONS')
    print('  ', ' '.join('%s(%d)' % (k, v) for k, v in sorted(lens.items())))
