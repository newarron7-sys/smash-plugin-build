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
- Animations : groupe kjoe_base (toutes les animations de base de REX aux noms de Ness : double saut JumpAerialF/B, atterrissages LandingLight/Heavy, garde, roulades, dégâts, au sol, bord, nage, provocations, entrée/victoire) ; le groupe base de Rock prenait JumpF pour le double saut et Landing (absent) laissait la pose de Ness.

## Spéciaux (16/11) — speciaux.rs
- Décompilés dans FitKJoe.pac : B neutre (SpecialNStart / NHold / NFire, charge 0x1D aux paliers 15/30/60/90/180,
  article au tir image 7), côté B (SpecialAirSStart 15 + SpecialS 49 : ruée de 66, 12 % puis x1,25 = 15 %),
  haut B (SpecialHi 70 : monte de 30, 16 % à 8 puis 12/11 % jusqu'à 31, chute spéciale), bas B (SpecialLwStart :
  saut de 26 + 6 %, SpecialLw : plongée 0,5 / -5, 10 % 270° puis 88°, SpecialLwEnd, SpecialLwHit rebond -7/+8).
- Animations : groupe kjoe_speciaux (sur place), trajet par trajet_rex.py (table TRAJET), x ECHELLE.
- Projectile du B neutre : PK Feu de Ness redessiné (projectile.rs), dégâts 6 + 3 x niveau, plus gros avec la charge.
- Showcase des spéciaux (vidéo du 16/11) : B neutre = charge (corps qui clignote blanc-lilas, boule bleue au poing),
  coup de poing avec traits blancs et tir ; haut B = flash blanc, gerbe blanche au sol au départ, traînée bleu-blanc ;
  bas B = nuage de poussière au décollage et à l'atterrissage. Ajoutés dans speciaux.rs.
- Piège : retarget_exo.py applique les couches de visée de Master Chief (HAUT_SEUL / JAMBES_DE) par nom de fichier
  Ness (d00specialnhold…) : désactivées pour le groupe kjoe_speciaux (sinon KeyError SpecialNAimREDO, aucune anim).
- PIÈGE (v2, 16/11, « aucune animation ») : le config.json du reslot partageait (share-to-added / share-to-vanilla)
  111 fichiers que le mod fournit (spéciaux, dégâts, garde, bord, esquives, nage, provocations, victoire) : ARCropolis
  chargeait ceux de Ness. Lancer rex/tools/fix_partages.py sur le mod à CHAQUE livraison, avant verif_config.
