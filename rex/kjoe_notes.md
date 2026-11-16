# Knuckle Joe (REX) — notes de portage

## Choix (15/11)
- Case : la case « knucklejoe » existante du CSS (Kirby c100, 1 couleur) est remplacée, même nom (comme Metal Sonic) ;
  elle passe sur Ness (fighter_kind ness, costume c100 libre : Ness c00-07, freddy 88-95, ninten 120-127).
- Modèle : celui de REX (demandé : « tu peux pas prendre le model de REX ? »), méthode de Rock Howard / Zero
  (build_model_rex EXO=1 puis deexo : maillage rattaché directement aux os de Ness).
- Perso de base : Ness, car le module de REX (ft_kjoe.rel) est un clone de Ness (ftNess, PK Thunder, PK Fire, yoyo…),
  comme Master Chief sur Snake.
- REX : FitKJoe.pac, FitKJoe00-09.pac, FitKJoeMotionEtc.pac, FitKJoeFinal.pac, ft_kjoe.rel (branche chiffrée rex_kjoe).

## v1 (15/11)
- Modèle de REX (yeux EyesDefault en transparence, expressions retirées pour l'instant), 71 animations (groupe base),
  coups normaux de REX (generated.rs), effets PSI de Ness des coups vidés.
- Mod : construit en Ness c00 sur la CI puis reslotter.py c00 -> c100 ; 27 fichiers déclarés absents partagés avec
  Ness c00 ; ui_chara_db.prc du joueur : seule l'entrée knucklejoe change (fighter_kind et fighter_kind_corps ->
  ness, patch_kind.py), costume 100, 1 couleur.
- CI : kjoe.yml compile aussi sur la branche metalsonic (cache de la toolchain skyline ; sur la nouvelle branche
  l'installation restait bloquée 40 min).
- Pas encore : spéciaux, lancers de REX, effets, sons / voix, visage (expressions), Final Smash.

## Corrections (16/11)
- Traits noirs : queues du bandeau (Headband2, os BdnL/BdnS absents chez Ness) retirées (DROP_OBJ).
- Pieds dans le sol : retarget_exo SOL=1 (hanches remontées quand le pied de REX touche le sol).
- Visage noir : yeux posés sur la couleur de peau (FILL_MATS / FILL_RGB), opaques.
- Yeux : le masque de REX a déjà une moitié par œil (UV 0..1 en miroir dans la géométrie) : PAS de répétition en
  plus (v1e en avait mis une : 4 yeux, même erreur que Super Shadow) ; EYE_CLAMP=1 (bords bloqués).
- Taille : ECHELLE 0,6 puis 0,8 (photo à côté de Kirby).
- Téléportation (tilt bas) : animations qui avancent dans REX (tilt bas +28, attaque en courant +45, prise en
  courant +22) sur place (IN_PLACE_PLUS) + avancée par le plugin (table AVANCE, avance_rex.py, x ECHELLE).
  À vérifier à chaque nouveau coup : « avance » dans animations.txt (piège de Master Chief).
- Zone plus foncée autour des yeux (vidéo 16/11) : fond = couleur médiane du visage sous le masque (180,139,95, mesurée dans la texture aux UV du maillage Head) et même matière (PRM) que le visage.
- Double saut : effet PSI de Ness (bulle bleue) remplacé par le nuage normal (sys_jump_aerial).
- Smash avant : le coup de REX sur ThrowN devient un vrai projectile : PK Feu de Ness (generate_article, comme le f-smash de Primid avec la boule de feu de son perso de base), scripts ness_pkfire remplacés (9 %, boule bleue, pas de colonne). Attention : le PK Feu du side B actuel (celui de Ness, pas encore remplacé) prend aussi ces scripts.
