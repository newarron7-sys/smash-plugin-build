# Metal Sonic (REX) : spéciaux (données de FitMetalSonic.pac, construit sur Wolf)

Extraction du 14/11 (gen_rex.py, MAP_PLUS sur les sous-actions des spéciaux ; décompilation complète des
événements Main / GFX / SFX / Other gardée hors dépôt). À compléter avec les vidéos du showcase REX (noms des coups),
comme pour Master Chief, avant d'écrire les statuts.

| Sous-action | Durée | Coups (image : dégâts / angle / élément / os) |
|---|---|---|
| SpecialN | 57 | 0 : 14 % / 361° / élec / bust (x4), 10 % ; effacé |
| SpecialNShot (3 variantes) | 50 | tirs du réacteur (BlasterM) toutes les 2 images de 2,9 à 12,9 (élec, 0 % : touche par le projectile) ; variantes selon la charge |
| SpecialSStart | 16 | 20 et 24 : 15 % / 361° normal (head) + 15 % / 35° élec (top) |
| SpecialS / SpecialSshort | 14 / 10 | 0 : 11 % / 361° / normal / waist (ruée longue / courte) |
| SpecialSEnd, SpecialAirS, SpecialAirSEnd | 41 / - / 41 | pas de coup (fin de ruée) |
| SpecialHiHold, SpecialHiHoldAir | 10 / 15 | pas de coup (charge) |
| SpecialHi | 22 | 0 : 9 % / 35° / feu / top ; 1 : 5 % / 64° / feu / hip ; 4 : 3 % / 70° / hip |
| SpecialHiFall, SpecialHiLanding | 20 / 20 | chute, atterrissage |
| SpecialHiBound | 28 | 1 : 12 % / 361° (bust, head) ; effacé à 8 (rebond sur mur / sol) |
| SpecialLwStart (+ Air) | 12 | 4 : coups de 1 % (angles 0 / 10 / 100 / 250 : aspiration autour de lui) |
| SpecialLwLoop | 10 | 1 : 1 % / 135° / élec (x3) ; 24 : 3 % / 135° / élec |
| SpecialLwHit | 2 | réaction quand il est touché |
| SpecialLwEnd (+ Air) | 18 | 0 : 7 % / 72° / élec ; 2 : 2 % / 80° / élec ; 18 : 3 % (footl) ; 20 : 4 % / 80° / feu |
| FinalStart / FinalEnd | 120 / 25 | Final Smash |

## Vidéo du showcase REX (14/11, noms affichés)
- B neutre « Ring Spark Field » : appui (« no charge ») = décharge électrique rose tout de suite ; B tenu (« full
  charge ») = il se charge en changeant de couleur (rose, orange), au sol comme en l'air, puis grosse décharge.
- side B « V.Maximum Overdrive » : départ puis ruée vers l'avant dans un anneau doré / rose, traînée dorée, éclat
  rose à l'impact ; version courte.
- haut B « Jet Boost » (« movement option ») : explosion de feu et fumée au départ, il file en diagonale ou tout droit,
  traînée de fumée.
- bas B « Black Shield » : losange de cristal transparent autour de lui (éclat en croix à l'apparition) ; touché
  pendant le bouclier : riposte, l'adversaire prend 12 % et part ; « horizontal momentum during dashing ».

## Version 2 (14/11) : choix faits dans speciaux.rs
- Animations : groupe metal_speciaux (retarget_exo.py), sur place. Longueurs CHR0 réelles : SpecialHiFall = 50 (pas
  20 comme la sous-action) : special_hi = 0-10 charge, 10-32 envol, 32-82 chute ; special_s_hold = 0-16 départ,
  16-30 ruée, 30-71 fin.
- B neutre : charge 60 images max, appui < 8 images = décharge sans charge ; décharge 10 % + 8 % x charge (élec,
  361°), taille 9 + 6 x charge, à l'image 6 de special_n_spin.
- side B : ruée 3,6 de vitesse ; coups : tête 15 % élec 35° (4 images), corps 11 % 361° ; chute spéciale en l'air.
- haut B : direction du stick à l'image 10 (vers le haut sans stick, jamais vers le bas), vitesse 3,2 ; explosion
  9 % feu, puis 5 % feu et 3 % ; chute spéciale ; une fois par saut.
- bas B : bouclier des images 4 à 34 (pas de recul, 0 dégât), riposte 12 % élec 45° si touché (StopModule::is_damage) ;
  bouclier = sys_shield teinté cristal ; élan horizontal gardé (0,92 par image).
