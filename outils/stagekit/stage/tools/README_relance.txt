# relance 07:02:11

Unreal Introduction Map v2 (24/11) : la v1 sur la Destination Finale plantait au chargement (lvd du Champ de bataille
sur la DF, pointeur nul TaskWorker2 cross2+0x2835f9c). v2 = variante Stage Alts normal_s02 du Champ de bataille
(touche R ; normal + normal_s01/L déjà chez le joueur) : ue_bf.py + stage-uebf.yml (sortie_uebf). config.json =
share_to_vanilla de chaque fichier de stage/battlefield/normal (Hashes_FullPath) non fourni ; UI stage_1/2/4
_battlefield_s02.bntx fournies (nom interne stage_N_BattleField), 0 et 3 partagées. Nouvelles textures sans
new-dir-files (comme les variantes d'HDR, 78 textures nouvelles dans leurs dossiers _s01/_s02).
Unreal (suite) : v4 (Champ de bataille) a replanté à cross2+0x3601e30 (comme Suzaku) ; v5 = dessous des plates-formes
d'origine. ATTENTION : HDR du joueur = 0.49.11 (Champ de bataille normal_s01 seulement, aucune variante de Smashville) ;
les fichiers HDR utilisés pour le Champ de bataille venaient de la 0.50.10. Unreal sur Smashville = normal_s01 (L) :
ue_sv.py + stage-uesv.yml (sortie_uesv), méthode de Sleepless Lamp (0 entrée pour 62 ensembles), forme libre calculée
d'après la 0.49.11.
Unreal Smashville v6 (25/11) : sol / murs à carte d'ombres (BakeMap Brawl, 2e UV) dans stc_mainstage_set, matériau
ashiba_jimenkusa04 (Texture0 + Texture9, bake1) de 387162 ; v3 chargée ; textures d'origine (pas d'éclaircissement),
BakeMap x0,9, quadrillage refait 2048 ; jour permanent = render d'Islandville 382440.
Gathering of Old Blood v1 : gob_sv.py + stage-gobsv.yml (sortie_gobsv), Smashville normal_s02 (R), méthode de Lamp ;
matériaux additifs Brawl (BP 0x41 src SRCALPHA dst DSTALPHA) -> Bottom One/One prémultiplié, passe _sort.
