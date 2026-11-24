# relance 07:02:11

Unreal Introduction Map v2 (24/11) : la v1 sur la Destination Finale plantait au chargement (lvd du Champ de bataille
sur la DF, pointeur nul TaskWorker2 cross2+0x2835f9c). v2 = variante Stage Alts normal_s02 du Champ de bataille
(touche R ; normal + normal_s01/L déjà chez le joueur) : ue_bf.py + stage-uebf.yml (sortie_uebf). config.json =
share_to_vanilla de chaque fichier de stage/battlefield/normal (Hashes_FullPath) non fourni ; UI stage_1/2/4
_battlefield_s02.bntx fournies (nom interne stage_N_BattleField), 0 et 3 partagées. Nouvelles textures sans
new-dir-files (comme les variantes d'HDR, 78 textures nouvelles dans leurs dossiers _s01/_s02).
