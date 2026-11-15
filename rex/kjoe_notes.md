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
