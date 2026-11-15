# Metal Sonic (REX) : notes du portage (usage perso)

Procédure suivie : rex/PROCEDURE.md (dépôt final-zoom, branche combo-wip), comme Master Chief, Super Sonic et Super Shadow.

## Ce que font les portages précédents (analyse du 13/11)
- Une branche à part par perso (Super Sonic : supersonic-vol) : main reste celui de Master Chief. -> Metal Sonic : branche metalsonic.
- Fichiers REX : rex.yml -> branche chiffrée. Metal Sonic : dossier pf/Fighter/metalsonic, module ft_metalsonic.rel
  (construit sur Wolf dans Brawl : blaster, illusion, Landmaster), 310 animations, 494 sous-actions.
- Skin GameBanana sur le squelette du perso de base, UNE seule couleur par case (Chief c140, Super Sonic c141,
  Super Shadow c140) : un costume du skin reslotté sur la case (reslotter.py), entrée de ui_chara_db.prc modifiée à
  partir du fichier ACTUEL du joueur, noms dans msg_name.
- Étape 1 (Super Sonic v1) : coups de REX (generated.rs) + effets du perso de base remplacés (scripts effect_ vides
  ou à la REX, poussière normale) + sa case. Étape 1 (Chief) : coups + armes + animations.
- Puis : effets / sons (banque REX -> nus3audio, noms en minuscules), spéciaux, prises / lancers, provocations,
  Final Smash, interface toutes tailles.
- Fichier d'effets par costume (ef_<perso>_cXXX.eff) mis dans la source AVANT reslotter.py.
- Avant chaque livraison : verif_config.py (textures du jeu absentes de la case), comparaison du zip avec la version
  précédente, un seul zip « (Moveset) <Perso> ».

## Choix pour Metal Sonic (demandés par le joueur)
- Skin : « Metal Sonic (Smash Ultimate Style) Beta Release » (GameBanana 475923), variante ProportionEdit, costume c00.
- Case : la case « msonic » déjà présente au CSS (ui_chara_msonic, Sonic, color_start_index 8) : costume c08, nom
  « Metal Sonic » déjà dans msg_name. Le CSS est plein : pas de nouvelle entrée. L'ancien Metal Sonic de cette case
  est remplacé (même nom).
- ui_chara_db.prc du joueur (11/11) : seul changement, color_num de msonic 8 -> 1 (une seule couleur).

## Écarts faits au début (corrigés)
- 8 couleurs (490 Mo, 1043 fichiers) au lieu d'une seule (62 Mo) ; d'abord à la place du vrai Sonic (c00-c07).
- Travail poussé sur main (déclenchait la compilation de Chief) : déplacé sur la branche metalsonic, main remis à v29.
- Effets et sons de Sonic encore joués sur ses coups (Super Sonic v1 les remplaçait déjà) : à faire (étape 1b).
- Fichier d'effets du skin retiré au lieu d'être mis en ef_sonic_c00.eff avant reslotter : à refaire.

## Demandes du joueur (13/11, vidéos de REX + ses essais)
- Ne plus livrer chara_7 (il en a déjà un).
- Course : flamme violette dans le dos (REX : effet n° 28 sur TopN) ; plus les nuages bleus de la course de Sonic.
- Arrière aérien « Jet Thrust » : feu qui sort du dos (REX : effet n° 73 sur BlasterM, image 6) qui le pousse vers
  l'avant (REX : pas d'avancée dans TransN : poussée faite par le plugin, POUSSEES dans lib.rs).
- Aérien neutre « Body Spark » : il gardait la boule de Sonic (expression / visibilité de Sonic) : étincelles
  électriques, corps normal forcé pendant tous les coups (effets::corps_normal).

## Corrections faites (v1b)
- effets.rs : scripts effect_ de Sonic vides ou remplacés (déplacements comme Super Sonic, coups), expression et son
  de l'aérien neutre vides ; flamme de course = sys_rocketbelt_jet violet sur « top » (0, 7, -2,5), tourné vers
  l'arrière (rotation x 1,5708 rad) ; back air = sys_flame + sys_rocketbelt_jet sur « top », poussée 1,6 / 0,4.
- Fichier d'effets du skin remis en ef_sonic_c00.eff avant reslotter -> ef_sonic_c08.eff déclaré.
  -> RETIRÉ (14/11) : plantage au chargement du combat (rapport Atmosphère 2168-0002, Data Abort, pile :
  smashline load_fighter_effects -> nn::util::BinaryBlockHeader::GetBlockSize). Le ef_sonic.eff de ce skin ne se
  charge pas comme fichier d'effets par costume : ne plus le livrer (la version sans lui, 12/11 16:51, ne plantait pas).

## Corrections v1c (14/11, photo + vidéos du joueur)
- Jet du dos : collé à « top », il partait vers le bas (la rotation de req_follow n'a pas joué) et sa fumée était
  violette. Maintenant : sys_rocketbelt_burner (flammes) violet, créé avec EffectModule::req puis replacé à chaque
  image en coordonnées monde entre bust et waist, 1,8 derrière, tourné vers l'arrière (z = -lr x 1,5708 rad), comme
  les jets du down air de Chief. Même jet pour le back air (images 6 à 20) + gerbe sys_flame sur bust.
- Sauts : il se mettait en boule (boule de Sonic) : corps normal forcé dans tous les statuts communs sauf spéciaux et
  Final Smash ; expressions des sauts de Sonic vides.

## v3 (15/11) : prises, lancers, provocations, voix et bruitages
- Demandé : pummel, lancers SAUF le lancer arrière (il reste celui de Sonic), provocations, sa voix et ses sons.
- Animations : groupe metal_prises (retarget_exo.py), sur place ; prise en courant (20,7) et lancer bas (13) : avancée
  de REX faite par le plugin (prises.rs, tables d'avance_rex.py).
- prises.rs : zones de prise, coup pendant la prise (2 % élec, direct), lancers avant (3 % + 4 %), haut (4 % + 3 %
  feu), bas (6 %) aux images de REX ; coup sur l'adversaire tenu quelques images avant le lâcher.
- Sons : voir rex/metalsonic_sons.md.
- Livraison : les animations déjà livrées sont gardées à l'octet près (la CI les régénère avec quelques octets de
  différence d'arrondi d'une machine à l'autre) : seuls config.json, plugin.nro et les nouveaux fichiers changent.

## VALIDÉ PAR LE JOUEUR / CORRECTIONS (ne pas casser)
- (rien de validé en jeu pour l'instant)

## v3b (15/11) — lancers corrigés, ondes de choc, explosions
- Animations de REX relues image par image (pied, ThrowN, TransN) :
  - lancer avant : le pied touche à 18-19. Le coup tenu à 14 (pied replié) donnait dégâts + gel trop tôt → retiré,
    3 % ajoutés au lancer (7 %), lâché à 19 ;
  - lancer haut : coup tenu retiré (7 %), lâché à 23 avec explosion (sys_bomb_a / b) au-dessus de lui ;
  - lancer bas : l'adversaire touche le sol à 26 (sommet du saut à 22, où était l'onde) → lâché à 26, onde au sol à
    11 devant lui.
- Tous les lancers : sys_crown_collision + sys_crown au sol + QUAKE (M, M, L).
- Arrière aérien : explosion sys_bomb_a derrière lui à l'image 6 ; B neutre : sys_bomb_a / b et QUAKE selon la charge.
- Effets des coups normaux (coups.rs, demandé avec le showcase) : traits de griffe blanc-bleu (sys_attack_arc*,
  sys_attack_line) aux images et positions des effets de REX + traînée image par image sur la griffe / le pied ;
  smashs : éclat de charge, tourbillon au sol du smash bas, griffes en tourbillon du smash haut.
- Showcase n° 2 (attaque en courant, smashs) : deux traits croisés en courant ; bulle brillante au début de la charge
  des smashs ; smash haut : nuage de poussière au bond et bulle bleue transparente (sys_shield teinté, comme l'effet
  n° 04 de REX) pendant qu'il tourne en l'air ; smash bas : bulle bleue au ras du sol, vent, poussière gauche / droite.
