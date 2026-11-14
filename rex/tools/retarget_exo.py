"""REX (étape 3, squelette « exo ») : ses animations Brawl (CHR0 de FitXXXMotionEtc.pac) -> fichiers .nuanmb du perso
de base (Mario), pour le modèle construit par build_model_rex.py avec EXO=1.

Le squelette de Mario reste tel quel. Le maillage de Rock est lié aux os H_Exo_ ; ceux qui sont associés à un os de
Mario copient sa rotation monde (contraintes du nuhlpb). Pour retrouver la pose de REX :
- os de Rock n (pose Brawl monde BRW_n(t), repos Brawl WORLD_n, repos mis en T NW_n, repos exo EXW_n) : il doit
  être en E_n(t) = BRW_n(t) . NW_n^-1 . EXW_n (le maillage a été lié en NW, avec EXW comme repos) ;
- os de Mario associé u : rotation monde = celle de E_n(t) (la contrainte la recopie sur l'os H_Exo_), translation
  locale de Mario gardée (ses proportions à lui) ;
- Hip : placé pour que la racine H_Exo_HipN (fille fixe de Hip) tombe en E_HipN(t) ;
- Trans : le déplacement de TransN (le jeu s'en sert pour faire avancer / monter le perso) ;
- os H_Exo_ : leur transformation locale complète (ceux qui ne sont pas associés, cheveux, veste, visage, bouts des
  doigts, n'ont que ça ; pour les autres, la rotation est la même que celle de la contrainte) ;
- les autres os de Mario gardent leur pose de repos.

usage : python3 retarget_exo.py <FitXXX00.pac> <FitXXXMotionEtc.pac> <sortie de build_model_rex.py> <dossier de
        sortie des .nuanmb> [groupe : speciaux | base]
"""
import json, math, os, sys
import numpy as np
import ssbh_data_py as S

HERE = os.path.dirname(os.path.abspath(__file__))
for p in ('../../black-shadow/tools', '../../black-shadow/tools/anim', '../../zero/tools', '.'):
    sys.path.insert(0, os.path.join(HERE, p))
from arc import arc_entries  # noqa: E402
from brres import brres_files  # noqa: E402
import brawl_anim  # noqa: E402
from gen_rex import plain  # noqa: E402

MODEL_PAC, MOTION_PAC, MOUT, DST = sys.argv[1:5]
GROUPE = sys.argv[5] if len(sys.argv) > 5 else 'speciaux'
os.makedirs(DST, exist_ok=True)
A = S.anim_data

# fichier de Mario -> animation(s) de REX (plusieurs = mises bout à bout)
GROUPES = {
    # étape 3, partie 2 : spéciaux et provocations (actions 0x112 à 0x116 et provocations de REX)
    'speciaux': {
        # B neutre : Reppuken (sous-actions 462 au sol, 463 en l'air)
        'd00specialn': ['SpecialN1'], 'd00specialairn': ['SpecialAirN'],
        # B côté : Hard Edge (464 / 465)
        'd01specials': ['SpecialS'], 'd01specialairs': ['SpecialAirS'],
        # B haut : Rising Tackle (467, la même au sol et en l'air dans REX ; elle monte avec TransN)
        'd02specialhi': ['SpecialAirHi'], 'd02specialairhi': ['SpecialAirHi'],
        # B bas : Crack Counter (468 / 469 début, 474 attente, 475 fin) sur les 3 temps du J.E.T. de Mario
        'd03speciallwstart': ['SpecialLwStart'], 'd03specialairlwstart': ['SpecialAirLwStart'],
        'd03speciallwhold': ['SpecialLwLoop'], 'd03specialairlwhold': ['SpecialLwLoop'],
        'd03speciallwlight': ['SpecialLwStop'], 'd03speciallwheavy': ['SpecialLwStop'],
        'd03specialairlwlight': ['SpecialLwStop'], 'd03specialairlwheavy': ['SpecialLwStop'],
        # Final Smash (476 / 477 : la même animation « Final » au sol et en l'air)
        'd04final': ['Final'], 'd04finalair': ['Final'],
        # provocations (Mario n'a que les fichiers « r », utilisés des deux côtés)
        'j01appealhir': ['AppealHi'], 'j01appealsr': ['AppealS'], 'j01appeallwr': ['AppealLw'],
    },
    # étape 3, partie 1 : attente, déplacements, sauts et coups normaux (REX n'a pas toutes les variantes d'Ultimate :
    # la plus proche est reprise). Marche, course, attaque rapide et smashs gardent l'avancée de TransN (le jeu règle
    # la vitesse de lecture de la marche / course dessus, et fait avancer l'attaque rapide et les smashs avec).
    # étape 3, partie 1 (v0.7, sur Terry) : attente, déplacements, sauts et coups normaux. REX n'a pas toutes les
    # variantes d'Ultimate : la plus proche est reprise ; « -Nom » = jouée à l'envers (Terry marche à reculons quand
    # l'adversaire est derrière lui : la marche de REX à l'envers, son déplacement part alors en arrière).
    # Marche, course, attaque rapide et smashs gardent l'avancée de TransN (le jeu règle la vitesse de la marche /
    # course dessus, et fait avancer l'attaque rapide et les smashs avec).
    # v0.9 (sur Terry) : ses spéciaux de REX sur les mécaniques de Terry (coups au plus proche : Power Wave = Reppuken,
    # Burn Knuckle = Hard Edge, Crack Shoot = OverHeadKick, Rising Tackle = Rising Tackle, Power Dunk = le coup de pied
    # sauté de son B bas, super-coups = Shining Knuckle / Raging Storm). Les animations en plusieurs temps sont découpées.
    # v0.13 (demandé) : haut B = animations de Terry ; côté B arrière : son coup de pied retourné de REX
    'speciaux_terry': {
        'd00specialn': ['SpecialN1'], 'd00specialairn': ['SpecialAirN'],
        'd01specialsfstart': ['SpecialS@0:10'], 'd01specialsf': ['SpecialS@10:20'], 'd01specialsfend': ['SpecialS@20:44'],
        'd01specialairsfstart': ['SpecialAirS@0:10'], 'd01specialairsfend': ['SpecialAirS@10:44'],
        # v0.14 : la montée du saut (images 4 à 10 de REX) passe dans la partie « attaque » : c'est elle que la Crack
        # Shoot de Terry utilise pour le faire décoller ; avant, il ne montait pas, retombait tout de suite (fumée
        # d'atterrissage) et revenait en arrière
        'd01specialsbstart': ['OverHeadKick@0:4'], 'd01specialsbs': ['OverHeadKick@4:30'],
        'd01specialsbw': ['OverHeadKick@4:30'], 'd01specialsbend': ['OverHeadKick@30:45'],
        'd01specialairsbstart': ['OverHeadKick@0:4'], 'd01specialairsbend': ['OverHeadKick@30:45'],
        # v0.11 : B bas au sol = sa course de REX (le plugin enchaîne les animations ajoutées « rock_lw_… »)
        'd03speciallwstart': ['SpecialLwStart'], 'd03specialairlwstart': ['SpecialLwSpecialStart'],
        'd03specialairlwrisew': ['SpecialN2'],  # v0.18 : Double Reppu Ken sur un mouvement de Terry inutilisé (le motion_patch.yaml n'existe pas dans ARCropolis)
        'd03speciallwend': ['SpecialLwSpecialEnd'],
        'j01appealhir': ['AppealHi'], 'j01appealsr': ['AppealS'], 'j01appeallwr': ['AppealLw'],
        # v0.17 : Power Geyser (super_special) = Raging Storm en entier ; Buster Wolf (super_special2_start) =
        # Shining Knuckle en entier (le plugin joue tout dans ce statut, sans la prise de Terry)
        'd04superspecial1': ['RagingStorm'], 'd04superspecial2start': ['ShiningKnuckle'],
        'd04superspecial2': ['ShiningKnuckle@66:101'],
        # v0.19 : son Final de REX en entier dans final_start (pas de cinématique, comme dans REX)
        'd04finalstart': ['Final'], 'd04finalhit': ['Final@115:121'], 'd04finalend': ['Final@115:121'],
        'd04finalairstart': ['Final'], 'd04finalairhit': ['Final@115:121'], 'd04finalairend': ['Final@115:121'],
    },
    # Super Shadow sur Sonic, étape 4 partie 1 : attente, déplacements, sauts, chutes, atterrissages, accroupi
    # seulement (demi-tours et coups gardent les animations de Sonic). REX n'a qu'une attente : Wait1 pour les trois.
    'chief_base': {
        # Master Chief (REX) sur Snake : mêmes animations de base que Super Shadow / Super Sonic (procédure)
        'a02turnrun': ['TurnRun'], 'a02turnrunbrake': ['TurnRunBrake'],
        'a00wait1': ['Wait1'], 'a00wait2': ['Wait1'], 'a00wait3': ['Wait1'], 'a00waititem': ['WaitItem'],
        'a01walkslow': ['WalkSlow'], 'a01walkmiddle': ['WalkMiddle'], 'a01walkfast': ['WalkFast'],
        'a01walkbrakel': ['WalkBrake'], 'a01walkbraker': ['WalkBrake'],
        'a02dash': ['Dash'], 'a02run': ['Run'], 'a02runbrakel': ['RunBrake'], 'a02runbraker': ['RunBrake'],
        'a03jumpsquat': ['JumpSquat'], 'a03jumpf': ['JumpF'], 'a03jumpb': ['JumpB'],
        'a03jumpaerialf': ['JumpAerialF'], 'a03jumpaerialb': ['JumpAerialB'],
        'a04fall': ['Fall'], 'a04fallf': ['FallF'], 'a04fallb': ['FallB'],
        'a04fallaerial': ['Fall'], 'a04fallaerialf': ['FallF'], 'a04fallaerialb': ['FallB'],
        'a04fallspecial': ['FallSpecial'], 'a04fallspecialf': ['FallSpecialF'], 'a04fallspecialb': ['FallSpecialB'],
        'a04runfalll': ['Fall'], 'a04runfallr': ['Fall'], 'a04walkfalll': ['Fall'], 'a04walkfallr': ['Fall'],
        'a05landinglight': ['LandingHeavy'], 'a05landingheavy': ['LandingHeavy'], 'a04landingfallspecial': ['LandingFallSpecial'],
        'a05squat': ['Squat'], 'a05squatwait': ['SquatWait'], 'a05squatrv': ['SquatRv'], 'a05squatwaititem': ['SquatWaitItem'],
        'a05squatf': ['SquatF'], 'a05squatb': ['SquatB'],
    },    'chief_coups': {
        'c00attack11': ['Attack11'], 'c00attack12': ['Attack12'], 'c00attack13': ['Attack13'],
        # attaque en course : début commun (AttackDash) puis « Spartan Charge » ; « Slide Kick » (en pleine course) sur
        # l'animation d'objet swing_dash, que le plugin joue à la place quand il courait
        'c00attackdash': ['AttackDash', 'AttackDashSpartan'], 'h04swingdash': ['AttackDash', 'AttackDashSlide'],
        'c01attacks3s': ['AttackS3S'], 'c01attacks3s2': ['AttackS3S'],   # 2e coup du tilt de Snake : une autre rafale
        'c02attackhi3': ['AttackHi3'], 'c02attacklw3': ['AttackLw3'],
        'c03attacks4s': ['AttackS4Start', 'AttackS4S'], 'c03attacks4charge': ['AttackS4Hold'],
        'c04attackhi4': ['AttackHi4Start', 'AttackHi4'], 'c04attackhi4charge': ['AttackHi4Hold'],
        'c04attacklw4': ['AttackLw4Start', 'AttackLw4'], 'c04attacklw4charge': ['AttackLw4Hold'],
        'c05attackairn': ['AttackAirN'], 'c05attackairf': ['AttackAirF'], 'c05attackairb': ['AttackAirB'],
        # « Ground Pound » : départ, charge (tant que A est tenu, piloté par le plugin) puis plongée
        'c05attackairlw': ['AttackAirLwStart', 'AttackAirLwCharge', 'AttackAirLwGo'],
        'c05attackairhi': ['AttackAirHi'],
        'c05landingairn': ['LandingAirN'], 'c05landingairf': ['LandingAirF'], 'c05landingairb': ['LandingAirB'],
        'c05landingairhi': ['LandingAirHi'], 'c05landingairlw': ['LandingAirLw'],
    },
    'shadow_base': {
        # demandé : son demi-tour en course de REX (retourné d'un demi-tour pour Ultimate, voir ROT180) et son freinage
        'a02turnrun': ['TurnRun'], 'a02turnrunbrake': ['TurnRunBrake'],
        'a00wait1': ['Wait1'], 'a00wait2': ['Wait1'], 'a00wait3': ['Wait1'], 'a00waititem': ['WaitItem'],
        'a01walkslow': ['WalkSlow'], 'a01walkmiddle': ['WalkMiddle'], 'a01walkfast': ['WalkFast'],
        'a01walkbrakel': ['WalkBrake'], 'a01walkbraker': ['WalkBrake'],
        'a02dash': ['Dash'], 'a02run': ['Run'], 'a02runbrakel': ['RunBrake'], 'a02runbraker': ['RunBrake'],
        'a03jumpsquat': ['JumpSquat'], 'a03jumpf': ['JumpF'], 'a03jumpb': ['JumpB'],
        'a03jumpaerialf': ['JumpAerialF'], 'a03jumpaerialb': ['JumpAerialB'],
        'a04fall': ['Fall'], 'a04fallf': ['FallF'], 'a04fallb': ['FallB'],
        'a04fallaerial': ['FallAerial'], 'a04fallaerialf': ['FallAerialF'], 'a04fallaerialb': ['FallAerialB'],
        'a04fallspecial': ['FallSpecial'], 'a04fallspecialf': ['FallSpecialF'], 'a04fallspecialb': ['FallSpecialB'],
        'a04runfalll': ['Fall'], 'a04runfallr': ['Fall'], 'a04walkfalll': ['Fall'], 'a04walkfallr': ['Fall'],
        'a05landinglight': ['LandingLight'], 'a05landingheavy': ['LandingHeavy'], 'a04landingfallspecial': ['LandingFallSpecial'],
        'a05squat': ['Squat'], 'a05squatwait': ['SquatWait'], 'a05squatrv': ['SquatRv'], 'a05squatwaititem': ['SquatWaitItem'],
    },
    # Super Shadow sur Sonic, étape 4 partie 2 : ses coups normaux de REX (jab, attaque en courant, tilts, smashs avec
    # leur charge, aériens et leurs atterrissages). Le smash avant n'a qu'une version : la même pour les 3 directions.
    'shadow_coups': {
        'c00attack11': ['Attack11'], 'c00attack12': ['Attack12'], 'c00attack13': ['Attack13'],
        'c00attackdash': ['AttackDash'],
        'c01attacks3': ['AttackS3S'], 'c01attacks3s': ['AttackS3S'], 'c01attacks3hi': ['AttackS3S'], 'c01attacks3lw': ['AttackS3S'],
        'c02attackhi3': ['AttackHi3'], 'c02attacklw3': ['AttackLw3'],
        'c03attacks4s': ['AttackS4Start', 'AttackS4S'], 'c03attacks4hi': ['AttackS4Start', 'AttackS4S'],
        'c03attacks4lw': ['AttackS4Start', 'AttackS4S'], 'c03attacks4charge': ['AttackS4Hold'],
        'c04attackhi4': ['AttackHi4Start', 'AttackHi4'], 'c04attackhi4charge': ['AttackHi4Hold'],
        'c04attacklw4': ['AttackLw4Start', 'AttackLw4'], 'c04attacklw4charge': ['AttackLw4Hold'],
        'c05attackairn': ['AttackAirN'], 'c05attackairf': ['AttackAirF'], 'c05attackairb': ['AttackAirB'],
        'c05attackairhi': ['AttackAirHi'], 'c05attackairlw': ['AttackAirLw'],
        'c05landingairn': ['LandingAirN'], 'c05landingairf': ['LandingAirF'], 'c05landingairb': ['LandingAirB'],
        'c05landingairhi': ['LandingAirHi'], 'c05landingairlw': ['LandingAirLw'],
    },
    # Super Shadow, spéciaux : B neutre de REX (homing puis combo : poing, pied, pied qui éjecte, pose de fin), sur
    # de nouvelles animations ajoutées à Sonic (motion_patch.yaml lu par le plugin)
    # Super Shadow, spéciaux : B neutre de REX après le homing (poing, pied, pied qui éjecte, pose de fin) mis bout à
    # bout dans special_n_hit de Sonic (de nouvelles animations restaient en T-pose)
    'shadow_speciaux': {
        'd00specialnhit': ['SpecialNHit', 'SpecialNHit2', 'SpecialNHit3', 'SpecialNHit4', 'SpecialNHit5'],
        # side B de REX (Dash Boost, vidéo) : il fonce comme dans sa course (Run au sol, AirBoost en l'air, action
        # 0x113 de REX ; SpecialSDashLw de REX est la boule de Sonic) puis freine debout (SpecialSEndStart)
        # demandé : la fin (freinage) dans le même fichier que la course, juste après (le plugin boucle sur la course
        # puis saute à la fin) : au sol SpecialSEndStart à partir de l'image 3 (avant : encore la boule), en l'air
        # SpecialAirSEndStart + SpecialAirSEnd
        'd01specialsdashlw': ['Run', 'SpecialSEndStart@3:30'],
        'd01specialsdashhi': ['AirBoost', 'SpecialAirSEndStart@2:13', 'SpecialAirSEnd'],
        'd01specialsspin': ['SpecialSSpin'], 'd01specialsturn': ['SpecialSTurn'],
        'd01specialsendstart': ['SpecialSEndStart'],
        # haut B (Chaos Control) : concentration puis téléportation
        'd02specialhi': ['SpecialHiStart', 'SpecialHi'],
        # projection vers le haut de REX (il élève l'adversaire, se téléporte autour, sphères, explosion)
        'e01throwhi': ['ThrowHi'],
        # demandé : ses autres lancers et son coup pendant la prise (REX)
        'e01throwf': ['ThrowF'], 'e01throwb': ['ThrowB'], 'e01throwlw': ['ThrowLw'],
        'e00catchattack': ['CatchAttack'],
        # demandé : ses chopes et l'attente pendant la prise
        'e00catch': ['Catch'], 'e00catchdash': ['CatchDash'], 'e00catchturn': ['CatchTurn'],
        'e00catchwait': ['CatchWait'], 'e00catchcut': ['CatchCut'],
        # demandé : ses provocations
        'j01appealhil': ['AppealHiL'], 'j01appealhir': ['AppealHiR'], 'j01appealsl': ['AppealS'],
        'j01appealsr': ['AppealS'], 'j01appeallwl': ['AppealLw'], 'j01appeallwr': ['AppealLw'],
        # demandé : ses esquives (l'esquive aérienne sert aussi pour la directionnelle)
        'b02escapen': ['EscapeN'], 'b02escapef': ['EscapeF'], 'b02escapeb': ['EscapeB'], 'b02escapeair': ['EscapeAir'],
    },
    # Super Sonic, spéciaux de REX (vidéo du joueur + FitSuperSonic.pac) :
    # B neutre (Homing Attack) : départ, boule du homing, à l'arrivée 1 des 4 poses au hasard (action 0x11C : NHit 1 à 4)
    # et le Bound Jump (bas + B pendant le homing : SpecialNBounce) rangés bout à bout dans special_n_hit
    'chief_speciaux': {
        # side B « Plasma Grenade » (lancer), up B « Gravity Lift » (pose / lancer de l'ascenseur, « pas prêt » si
        # déjà posé), down B « Fusion Coil » (pose de la bonbonne) ; fichiers des spéciaux de Snake (statuts remplacés)
        'd01specialsstart': ['Garnade toss'], 'd01specialairsstart': ['Air Garnade toss'],
        'd02specialhistart': ['SpecialHi'], 'd02specialairhistart': ['SpecialAirHi'],
        'd02specialairhicut': ['NotReadyAir'], 'd02specialairhihang': ['NotReady'],
        'd03speciallwstart': ['SpecialLw'], 'd03specialairlwstart': ['SpecialAirLw'],
        'a04fallspecial': ['FallSpecial'], 'a04fallspecialf': ['FallSpecialF'], 'a04fallspecialb': ['FallSpecialB'],
        'a04landingfallspecial': ['LandingFallSpecial'],
    },
    # Master Chief, B neutre (REX : action 0x112 + surcharge 0x10D, sous-actions SpecialN*) sur la grenade de Snake :
    # B tapé = magnum (throw_m), B tenu = posture fusil d'assaut sur les statuts « tenir la grenade » de Snake (marche,
    # saut, atterrissage = jambes h10itemlegs*, haut du corps = mouvement partiel special_n_hold). Les autres gestes du
    # haut du corps (tir, crosse, recharge, clic à vide) et les animations sans équivalent sont mis dans des fichiers
    # que Chief n'utilise plus (lancers de la grenade, B bas de Snake) : le plugin les joue
    'chief_neutre': {
        'd00specialnstart': ['SpecialN'], 'd00specialairnstart': ['SpecialAirN'],
        'd00specialnhold': ['SpecialNGunShoot'], 'd00specialairnhold': ['SpecialNGunShoot'],  # v15 : tir en boucle (B tenu = rafale)
        'd00specialnthrowm': ['SpecialNTapPistol'], 'd00specialairnthrowm': ['SpecialAirNTapPistol'],
        'd00specialnthrowhi': ['SpiecalNSmack'], 'd00specialnthrowlw': ['AssaultReload'],
        'd00specialairnthrowhi': ['SpecialNGunEmpty'], 'd00specialairnthrowlw': ['SpecialNGunShoot'],
        'd00specialnclose': ['SpecialNEnd'], 'd03specialairlwblast': ['SpecialAirNEnd'],
        'd03speciallwsquatstart': ['SpecialNTapPistolReload'], 'd03specialairlwwall': ['SpecialNTapPistolReload'],
        'h10itemlegswait': ['SpecialNLegWait'], 'h10itemlegsjumpsquat': ['SpecialNJumpSquat'],
        'h10itemlegslanding': ['SpecialNLand'],
        'h10itemlegsslowf': ['ItemLegsSlowF'], 'h10itemlegsmiddlef': ['ItemLegsMiddleF'], 'h10itemlegsfastf': ['ItemLegsFastF'],
        'h10itemlegsslowb': ['ItemLegsSlowB'], 'h10itemlegsmiddleb': ['ItemLegsMiddleB'], 'h10itemlegsfastb': ['ItemLegsFastB'],
        'h10itemlegswalkf': ['ItemLegsMiddleF'], 'h10itemlegswalkb': ['ItemLegsMiddleB'],
        'h10itemlegsdashf': ['ItemLegsDashF'], 'h10itemlegsdashb': ['ItemLegsDashB'],
        'h10itemlegsbrakef': ['ItemLegsBrakeF'], 'h10itemlegsbrakeb': ['ItemLegsBrakeB'],
        'd00specialnlegsdashf': ['ItemLegsDashF'], 'd00specialnlegsdashb': ['ItemLegsDashB'],
        # v20 : debout en posture, le jeu ne met pas de jambes (v18 / v19 : jambes droites, serrées) : tir avec les
        # jambes de SpecialNLegWait, joué par le plugin quand il est immobile
        'd03speciallwground': ['SpecialNGunShoot'], 'd03speciallwsquatground': ['AssaultReload'],
        'd03speciallwsquatblast': ['SpiecalNSmack'], 'd03speciallwenemy': ['SpecialNGunEmpty'],
    },
    # Master Chief : provocations, prises et coup pendant la prise (REX). Snake n'a que les fichiers « R » des
    # provocations ; le coup pendant la prise de REX joue OutOfAmmoCatchAttack (sous-action 112)
    'chief_prises': {
        'j01appealhir': ['AppealHi'], 'j01appealsr': ['AppealS'], 'j01appeallwr': ['AppealLw'],
        'e00catch': ['Catch'], 'e00catchdash': ['CatchDash'], 'e00catchturn': ['CatchTurn'],
        'e00catchwait': ['CatchWait'], 'e00catchattack': ['OutOfAmmoCatchAttack'], 'e00catchcut': ['CatchCut'],
        'e00catchpull': ['CatchWait@0:10'],
    },
    # Final Smash de Master Chief (REX : sous-action FinalSmashplaceholder, sans animation propre ; lance-roquettes de
    # FitChiefFinal.pac sur l'épaule) : pose du lance-roquettes de REX (ItemLauncher : visée de l'horizontale, image 75,
    # vers le bas, image 150 : la cible de REX est 45 devant et 24 sous lui)
    'chief_final': {
        # v28 (vidéo REX « Grenade Launcher ») : debout 20 images (aura), envol en arrière (pose en l'air du
        # lance-roquettes, 54 images), tir (12 images), retombée (72 images) : le plugin fait le vol (finale.rs)
        'd04finalstart': ['ItemLauncher@75:95'] + ['ItemLauncherFall'] * 6 + ['ItemLauncherAirFire'] + ['ItemLauncherFall'] * 8,
        'd04finalairstart': ['ItemLauncher@75:95'] + ['ItemLauncherFall'] * 6 + ['ItemLauncherAirFire'] + ['ItemLauncherFall'] * 8,
        # v29 : mouvements de Snake « final_start_r » / « final_start_l » (son statut Final choisit selon lr) : les deux
        'd04finalstartl': ['ItemLauncher@75:95'] + ['ItemLauncherFall'] * 6 + ['ItemLauncherAirFire'] + ['ItemLauncherFall'] * 8,
        'd04finalairstartl': ['ItemLauncher@75:95'] + ['ItemLauncherFall'] * 6 + ['ItemLauncherAirFire'] + ['ItemLauncherFall'] * 8,
    },
    # lancers de Master Chief (REX : ThrowF / ThrowB / ThrowHi / ThrowLw), sur place : le plugin le déplace (lancers.rs)
    'chief_lancers': {
        'e01throwf': ['ThrowF'], 'e01throwb': ['ThrowB'], 'e01throwhi': ['ThrowHi'], 'e01throwlw': ['ThrowLw'],
    },
    'supersonic_speciaux': {
        'd00specialnstart': ['SpecialNStart'], 'd00specialairnstart': ['SpecialAirNStart'],
        'd00specialnspin': ['SpecialNSpin'],
        'd00specialnhit': ['SpecialNHit', 'SpecialNHit2', 'SpecialNHit3', 'SpecialNHit4', 'SpecialNBounce'],
        'd00specialnrebound': ['SpecialNRebound'], 'd00specialnlanding': ['SpecialNLanding'],
        'd00specialncancel': ['SpecialNCancel'],
        # side B (Dash Boost, comme Super Shadow) : course de REX puis freinage, dans des fichiers que le side B de Sonic
        # n'utilise plus (special_s_hold au sol, special_air_s_end_loop en l'air) : special_s_dash_lw / hi servent aussi
        # au bas B de Sonic (Spin Charge), qui garde la boule de REX
        'd01specialshold': ['Run', 'SpecialSEndStart@3:30'],
        # demandé (vidéo) : la fin restait figée (SpecialAirSEndStart presque immobile après l'image 6) : coupée
        'd01specialairsendloop': ['AirBoost', 'SpecialAirSEndStart@2:6', 'SpecialAirSEnd'],
        'd01specialsdashlw': ['SpecialSDashLw'], 'd01specialsdashhi': ['SpecialSDashHi'],
        'd01specialsspin': ['SpecialSSpin'], 'd01specialsturn': ['SpecialSTurn'],
        # haut B (Flight) : vol libre (UpBMove)
        # demandé : debout quand il monte (Fall de REX, vidéo du Flight), couché quand il file sur le côté (UpBMove)
        'd02specialhi': ['Fall', 'UpBMove'],
        # bas B (Spin Charge)
        'd03speciallwstart': ['SpecialLwStart'], 'd03speciallwhold': ['SpecialLwHold'], 'd03speciallwend': ['SpecialLwEnd'],
        'd03specialairlwstart': ['SpecialAirLwStart'], 'd03specialairlwhold': ['SpecialAirLwHold'],
        'd03specialairlwend': ['SpecialAirLwEnd'],
        # prise, coup pendant la prise, lancers de REX (Wild Rush, Point Blank Blast Off, Interception, Battering Ram)
        'e00catch': ['Catch'], 'e00catchdash': ['CatchDash'], 'e00catchturn': ['CatchTurn'],
        'e00catchwait': ['CatchWait'], 'e00catchcut': ['CatchCut'], 'e00catchattack': ['CatchAttack'],
        'e01throwf': ['ThrowF'], 'e01throwb': ['ThrowB'], 'e01throwhi': ['ThrowHi'], 'e01throwlw': ['ThrowLw'],
        # attaques au sol (Spinning Leg Sweep)
        'f03downattacku': ['DownAttackU'], 'f04downattackd': ['DownAttackD'],
    },
    # Super Sonic, demandé : provocations, bouclier et toutes les autres animations de REX qui manquaient
    'supersonic_reste': {
        'j01appealhil': ['AppealHiL'],
        'j01appealhir': ['AppealHiR'],
        'j01appealsl': ['AppealSL'],
        'j01appealsr': ['AppealSR'],
        'j01appeallwl': ['AppealLw'],
        'j01appeallwr': ['AppealLw'],
        'b00guardon': ['GuardOn'],
        'b00guard': ['Guard'],
        'b00guardoff': ['GuardOff'],
        'b00justshieldoff': ['GuardOff'],
        'b01guarddamage': ['GuardDamage'],
    },
    # Metal Sonic (REX, sur Sonic c08) : spéciaux d'après la vidéo du showcase REX (14/11) ; tout sur place, le
    # déplacement est fait par le plugin. Parties mises bout à bout (positions dans rex/metalsonic/src/speciaux.rs)
    'metal_speciaux': {
        # B neutre « Ring Spark Field » : charge (SpecialNShot) puis décharge (SpecialN)
        'd00specialnstart': ['SpecialNShot'], 'd00specialairnstart': ['SpecialNShot'],
        'd00specialnspin': ['SpecialN'],
        # side B « V.Maximum Overdrive » : départ (16), ruée (14), fin au sol / en l'air (41)
        'd01specialshold': ['SpecialSStart', 'SpecialS', 'SpecialSEnd'],
        'd01specialairsendloop': ['SpecialSStart', 'SpecialS', 'SpecialAirSEnd'],
        # haut B « Jet Boost » : charge (10), envol (22), chute (20)
        'd02specialhi': ['SpecialHiHold', 'SpecialHi', 'SpecialHiFall'],
        # bas B « Black Shield » : départ, maintien, riposte
        'd03speciallwstart': ['SpecialLwStart'], 'd03speciallwhold': ['SpecialLwLoop'], 'd03speciallwend': ['SpecialLwEnd'],
        'd03specialairlwstart': ['SpecialAirLwStart'], 'd03specialairlwhold': ['SpecialLwLoop'],
        'd03specialairlwend': ['SpecialAirLwEnd'],
    },
    'base': {
        # attente
        'a00wait1': ['Wait1'], 'a00wait2': ['Wait2'], 'a00wait3': ['Wait1'], 'a00waititem': ['Wait1'],
        # déplacements
        'a01walkslow': ['WalkSlow'], 'a01walkmiddle': ['WalkMiddle'], 'a01walkfast': ['WalkFast'],
                'a01walkbrakel': ['WalkBrake'], 'a01walkbraker': ['WalkBrake'],
        'a02dash': ['Dash'],         'a02run': ['Run'], 'a02runbrakel': ['RunBrake'], 'a02runbraker': ['RunBrake'],
        
        'a05squat': ['Squat'], 'a05squatwait': ['SquatWait'], 'a05squatrv': ['SquatRv'], 'a05squatwaititem': ['SquatWait'],
        # sauts, chutes, atterrissages (REX : pas de saut arrière ni de 2e saut à part -> JumpF)
        'a03jumpsquat': ['JumpSquat'], 'a03jumpf': ['JumpF'], 'a03jumpb': ['JumpF'],
        'a03jumpaerialf': ['JumpF'], 'a03jumpaerialb': ['JumpF'],
        'a04fall': ['Fall'], 'a04fallf': ['FallF'], 'a04fallb': ['FallB'],
        'a04fallaerial': ['Fall'], 'a04fallaerialf': ['FallF'], 'a04fallaerialb': ['FallB'],
        'a04fallspecial': ['Fall'], 'a04fallspecialf': ['FallF'], 'a04fallspecialb': ['FallB'],
        'a04landingfallspecial': ['SpecialLwSpecialEnd'],  # v0.16 : fin de son B bas (REX)
        # v0.18 : ses prises de REX (chopes, attente, coup pendant la prise, lâcher, lancers)
        'e00catch': ['Catch'], 'e00catchdash': ['CatchDash'], 'e00catchturn': ['CatchTurn'], 'e00catchwait': ['CatchWait'],
        'e00catchattack': ['CatchAttack'], 'e00catchcut': ['CatchCut'],
        'e01throwf': ['ThrowF'], 'e01throwb': ['ThrowB'], 'e01throwhi': ['ThrowHi'], 'e01throwlw': ['ThrowLw'],
        'a04runfalll': ['Fall'], 'a04runfallr': ['Fall'], 'a04walkfalll': ['Fall'], 'a04walkfallr': ['Fall'],
        'a05landinglight': ['Landing'], 'a05landingheavy': ['Landing'],
        'a06stepjump': ['StepJump'], 'a06steppose': ['StepPose'], 'a06stepairpose': ['StepAirPose'],
        'a06stepback': ['StepBack'], 'a06stepfall': ['Fall'],
        # coups normaux (smashs : début + coup dans un seul fichier, charge à part, comme dans Ultimate)
        'c00attack11': ['Attack11'], 'c00attack12': ['Attack12'], 'c00attack13': ['Attack13'],
        'c00attackdash': ['AttackDash'],
        'c01attacks3': ['AttackS3S'], 'c01attacks3hi': ['AttackS3Hi'], 'c01attacks3s': ['AttackS3S'],
        'c01attacks3lw': ['AttackS3Lw'],
        'c02attackhi3': ['AttackHi3'], 'c02attacklw3': ['AttackLw3'],
        'c03attacks4s': ['AttackS4Start', 'AttackS4S'], 'c03attacks4hi': ['AttackS4Start', 'AttackS4S'],
        'c03attacks4lw': ['AttackS4Start', 'AttackS4S'], 'c03attacks4charge': ['AttackS4Hold'],
        'c04attackhi4': ['AttackHi4Start', 'AttackHi4'], 'c04attackhi4charge': ['AttackHi4Hold'],
        'c04attacklw4': ['AttackLw4Start', 'AttackLw4'], 'c04attacklw4charge': ['AttackLw4Hold'],
        'c05attackairn': ['AttackAirN'], 'c05attackairf': ['AttackAirF'], 'c05attackairb': ['AttackAirB'],
        'c05attackairhi': ['AttackAirHi'], 'c05attackairlw': ['AttackAirLw'],
        'c05landingairn': ['Landing'], 'c05landingairb': ['Landing'], 'c05landingairlw': ['Landing'],
        'c05landingairf': ['LandingAirF'], 'c05landingairhi': ['LandingAirHi'],
    },
}
JOBS = GROUPES[GROUPE]
# v0.7 : 6e argument optionnel = liste des animations du perso de base : les autres noms sont ignorés (un fichier
# inconnu du jeu ne serait pas chargé)
if len(sys.argv) > 6:
    _ok = set(l.strip() for l in open(sys.argv[6]) if l.strip())
    _ok |= set(k for k in JOBS if 'rock' in k)        # v0.11 : animations ajoutées (motion_patch.yaml)
    _hors = sorted(k for k in JOBS if k not in _ok)
    if _hors:
        print('absents chez le perso de base (ignores) :', ' '.join(_hors))
    JOBS = {k: v for k, v in JOBS.items() if k in _ok}

info = json.load(open(os.path.join(MOUT, 'model.json')))
EX = info['exo']
PAIR = EX['paires']                                   # os REX -> os de Mario
M2B = {u: n for n, u in PAIR.items()}
UNDER = list(EX['nw'].keys())                         # os de Rock sous HipN, parents avant enfants
NW = {n: np.array(m, dtype=float) for n, m in EX['nw'].items()}
sk = S.skel_data.read_skel(os.path.join(MOUT, 'model', 'model.nusktb'))
UB = sk.bones
UN = [b.name for b in UB]
IDX = {n: i for i, n in enumerate(UN)}
PAR = [b.parent_index if b.parent_index is not None and b.parent_index >= 0 else None for b in UB]
L0 = [np.array(b.transform, dtype=float).T for b in UB]


def worlds(L):
    W = [None] * len(UB)

    def w(i):
        if W[i] is None:
            W[i] = (w(PAR[i]) @ L[i]) if PAR[i] is not None else L[i]
        return W[i]
    for i in range(len(UB)):
        w(i)
    return W


W0 = worlds(L0)
EXW = {n: W0[IDX['H_Exo_' + n]] for n in UNDER}

def inv_s(m):
    """inverse ; pseudo-inverse si la matrice n'est pas inversible (os à l'échelle 0 dans REX)"""
    try:
        return np.linalg.inv(m)
    except np.linalg.LinAlgError:
        return np.linalg.pinv(m)

KEY = {n: inv_s(NW[n]) @ EXW[n] for n in UNDER}
L_ROOT = L0[IDX['H_Exo_HipN']]
INV_L_ROOT = inv_s(L_ROOT)
BR = brawl_anim.read_bones(plain(MODEL_PAC))
# pistes écrites : tous les os sauf les os d'aide de Mario (H_ sans Exo : calculés par le jeu, comme d'habitude)
TRACKED = [i for i, n in enumerate(UN) if not (n.startswith('H_') and not n.startswith('H_Exo_'))]
# v0.8 (méthode de Zero) : le modèle est un skin lié directement aux os du perso de base : NO_EXO=1 n'écrit que ses os
if os.environ.get('NO_EXO') == '1':
    TRACKED = [i for i in TRACKED if not UN[i].startswith('H_Exo_')]

# Chief, B neutre : animations du HAUT du corps jouées en mouvement partiel par la posture (tenir la grenade de
# Snake) : un mouvement partiel applique toutes les pistes du fichier ; avec les jambes dedans, elles écrasaient
# celles de la marche (vidéos v13 à v15 : jambes figées en tirant). Seulement « Waist » et ce qui est dessous
HAUT_SEUL = {'d00specialnhold', 'd00specialairnhold', 'd00specialnthrowhi', 'd00specialnthrowlw',
             'd00specialairnthrowhi', 'd00specialairnthrowlw'}


# REX joue ces gestes avec, par-dessus, la visée (2e couche : SpecialNAimREDO, image = angle visé, 94 = droit devant)
# qui donne le torse (BodyN, BustN, NeckN) et l'axe du tir (ThrowN, ShootBone). Vidéo v16 : recharge « bizarre » et
# torse tordu = le torse propre de ces gestes, que REX ne montre pas. Torse pris dans la visée, image 94 (le plugin
# ajoute l'angle visé par-dessus)
AIM_OS = ('BodyN', 'BustN', 'NeckN', 'ThrowN', 'ShootBone')
AIM_IMAGE = 94.0


class _Fixe:
    def __init__(self, ba):
        self.ba = ba

    def srt(self, f, bone):
        return self.ba.srt(AIM_IMAGE, bone)


JAMBES_DE = {k: 'SpecialNLegWait' for k in ('d03speciallwground', 'd03speciallwsquatground', 'd03speciallwsquatblast', 'd03speciallwenemy')}


class _Boucle:
    def __init__(self, ba, nf):
        self.ba, self.nf = ba, max(1, nf)

    def srt(self, f, bone):
        return self.ba.srt(f % self.nf, bone)


def jambes(an, src):
    if src not in CACHE:
        d_, o_ = ANIMS[src]
        CACHE[src] = brawl_anim.read_chr0(d_, o_)
    lg = CACHE[src]
    out = dict(an)
    out['bones'] = dict(an['bones'])
    for b in lg['bones']:
        if any(w in b for w in ('Hip', 'Leg', 'Knee', 'Foot', 'Toe')):
            out['bones'][b] = _Boucle(lg['bones'][b], lg['frames'])
    return out


def torse_vise(an):
    if 'SpecialNAimREDO' not in CACHE:
        d_, o_ = ANIMS['SpecialNAimREDO']
        CACHE['SpecialNAimREDO'] = brawl_anim.read_chr0(d_, o_)
    aim = CACHE['SpecialNAimREDO']
    out = dict(an)
    out['bones'] = dict(an['bones'])
    for b in AIM_OS:
        if b in aim['bones']:
            out['bones'][b] = _Fixe(aim['bones'][b])
    return out


def sous_waist(i):
    while i is not None:
        if UN[i] == 'Waist':
            return True
        i = PAR[i]
    return False


ANIMS = {}


def walk(d):
    for e in arc_entries(d):
        if e[4][:4] == b'ARC\0':
            walk(e[4])
        elif e[4][:4] == b'bres':
            for folder, items in brres_files(e[4]).items():
                if 'Chr' in folder:
                    for n, o in items:
                        ANIMS.setdefault(n, (e[4], o))


walk(open(plain(MOTION_PAC), 'rb').read())
print('animations REX', len(ANIMS))


def orth(R):
    U, _, Vt = np.linalg.svd(R)
    m = U @ Vt
    if np.linalg.det(m) < 0:
        U[:, -1] *= -1
        m = U @ Vt
    return m


def locals_at(an, f):
    """transformations locales de tous les os du squelette (Mario + H_Exo_) à l'image f de l'animation REX"""
    BW = brawl_anim.world_pose(BR, an, f)
    if GROUPE == 'speciaux_terry' and CUR_FN not in IN_PLACE:
        # v0.12 : les mécaniques de Terry (Rising Tackle, Crack Shoot…) se déplacent avec Trans : on le garde, mais
        # chaque fichier repart de 0 (comme les animations du jeu) au lieu de l'endroit où commence son morceau
        _t = np.eye(4)
        _t[:3, 3] = -SLICE0
        BW = {k: _t @ v for k, v in BW.items()}
    elif GROUPE == 'speciaux_terry' or CUR_FN in IN_PLACE:
        # v0.10 : spéciaux sur place (le déplacement est celui des mécaniques de Terry) : sinon, sur les morceaux qui
        # bouclent, le déplacement de TransN revient en arrière à chaque tour (« téléportation »)
        _t = np.eye(4)
        _t[:3, 3] = -BW['TransN'][:3, 3]
        if CUR_FN in KEEP_Y:
            _t[1, 3] = 0.0     # saut gardé dans la pose (le plugin ne le fait pas monter), l'avancée reste au plugin
        BW = {k: _t @ v for k, v in BW.items()}
    if CUR_FN in ROT180:
        # demi-tour : dans Brawl l'animation le fait pivoter de face vers dos (le jeu le retourne à la fin) ; dans
        # Ultimate il est retourné dès le début : toute la pose tourne de 180° autour de la verticale (elle part donc
        # « de dos » et finit de face). Appliqué au corps (Hip), pas à Trans (le jeu ignore sa rotation : échec de Zero)
        _r = np.diag([-1.0, 1.0, -1.0, 1.0])
        BW = {k: _r @ v for k, v in BW.items()}
    E = {n: BW[n] @ KEY[n] for n in UNDER}
    L = [None] * len(UB)
    W = [None] * len(UB)

    def world(i):
        if W[i] is None:
            W[i] = (world(PAR[i]) @ loc(i)) if PAR[i] is not None else loc(i)
        return W[i]

    def loc(i):
        if L[i] is not None:
            return L[i]
        n = UN[i]
        if n == 'Trans':
            m = np.eye(4)
            m[:3, 3] = BW['TransN'][:3, 3]
            if CUR_FN in KEEP_Y:
                m[1, 3] = 0.0
        elif n == 'Hip':
            m = inv_s(world(PAR[i])) @ (E['HipN'] @ INV_L_ROOT)
        elif n == 'Throw' and 'ThrowN' in BW:
            # v0.20 : l'os « throw » (où l'adversaire est tenu) suit ThrowN de REX : sinon il restait à sa place de
            # repos (au niveau des pieds) et l'adversaire tenu était dans le sol
            wt = W0[i].copy()
            # v0.22 : tourné d'un demi-tour (comme dans Ultimate) : l'adversaire tenu fait face à Rock, pas dos
            # (v26 avait retiré ce demi-tour pour Chief : ERREUR, le lancer bas le couchait sur le ventre au lieu du dos
            # (v25, validé) et la prise de dos venait d'ailleurs (coup pendant la prise « indirect », prises.rs) : remis)
            wt[:3, :3] = wt[:3, :3] @ np.diag([-1.0, 1.0, -1.0])
            if CUR_FN == 'e01throwb' and GROUPE not in ('shadow_speciaux', 'supersonic_speciaux', 'chief_lancers') and 8 <= f <= 46:
                # v0.25 (demandé, comme dans REX) : l'adversaire tenu vrille sur lui-même pendant qu'il est balancé en
                # arrière, AVANT le coup de poing (2 tours, images 8 à 46) : l'os throw tourne, l'adversaire avec
                a = math.radians(720.0 * (f - 8) / 38.0)
                c, s_ = math.cos(a), math.sin(a)
                wt[:3, :3] = wt[:3, :3] @ np.array([[1, 0, 0], [0, c, -s_], [0, s_, c]])
            if GROUPE == 'chief_lancers':
                # v25 (vidéo : au lancer bas l'adversaire restait debout, au lancer arrière il n'était pas tenu par les
                # pieds) : dans Ultimate l'adversaire tenu prend l'orientation de l'os throw ; il tourne donc comme
                # ThrowN de REX depuis la 1re image du lancer (couché au lancer bas, tête en bas au lancer arrière)
                if CUR_FN not in THROW0:
                    THROW0[CUR_FN] = orth(brawl_anim.world_pose(BR, an, 0)['ThrowN'][:3, :3])
                D = orth(BW['ThrowN'][:3, :3]) @ THROW0[CUR_FN].T
                wt[:3, :3] = D @ wt[:3, :3]
            wt[:3, 3] = BW['ThrowN'][:3, 3]
            m = inv_s(world(PAR[i])) @ wt
        elif n == 'Sphere' and 'SphereN' in BW:
            # Super Sonic : la boule de Sonic (maillage body_sphere, lié à l'os « sphere ») suit SphereN de REX ; sinon
            # elle restait au sol pendant le smash haut alors que REX la fait monter (YRotN / SphereN à ~23 de haut)
            wt = W0[i].copy()
            wt[:3, 3] = BW['SphereN'][:3, 3]
            m = inv_s(world(PAR[i])) @ wt
        elif n in M2B and M2B[n] in E:
            Rw = orth(E[M2B[n]][:3, :3])
            Rp = orth(world(PAR[i])[:3, :3])
            m = np.eye(4)
            m[:3, :3] = Rp.T @ Rw
            m[:3, 3] = L0[i][:3, 3]
            if os.environ.get('SCALE') == '1':
                # Super Shadow : REX écrase certains os (jambe repliée à 7 % dans le saut) : l'échelle locale de l'os
                # de Brawl est reportée sur l'axe de l'os d'Ultimate le plus proche (sinon la jambe restait tendue)
                b_ = M2B[n]
                pb = BR[b_].parent if b_ in BR else None
                if pb in BW and b_ in BW and np.allclose(np.linalg.norm(BW[pb][:3, :3], axis=0), 1, atol=0.02):
                    sl = np.linalg.norm((inv_s(BW[pb]) @ BW[b_])[:3, :3], axis=0)
                    if np.abs(sl - 1).max() > 0.02:
                        su = np.ones(3)
                        for k_ in range(3):
                            if abs(sl[k_] - 1) > 0.02:
                                d_ = BW[b_][:3, k_] / (np.linalg.norm(BW[b_][:3, k_]) or 1)
                                su[int(np.argmax(np.abs(Rw.T @ d_)))] = sl[k_]
                        m[:3, :3] = m[:3, :3] @ np.diag(su)
        elif n.startswith('H_Exo_'):
            b = n[6:]
            pw = world(IDX['Hip']) if b == 'HipN' else E[BR[b].parent]
            # REX met parfois un os à l'échelle 0 (Super Shadow caché pendant ses téléportations, projection vers le
            # haut) : matrice non inversible -> pseudo-inverse (inv_s)
            m = inv_s(pw) @ E[b]
        else:
            m = L0[i]
        L[i] = m
        return m

    for i in range(len(UB)):
        world(i)
    return L


def quat(m):
    r = m[:3, :3]
    sc = np.linalg.norm(r, axis=0)
    r = r / np.where(sc > 1e-9, sc, 1)
    t = np.trace(r)
    if t > 0:
        s = math.sqrt(t + 1.0) * 2
        q = [(r[2, 1] - r[1, 2]) / s, (r[0, 2] - r[2, 0]) / s, (r[1, 0] - r[0, 1]) / s, 0.25 * s]
    elif r[0, 0] > r[1, 1] and r[0, 0] > r[2, 2]:
        s = math.sqrt(1.0 + r[0, 0] - r[1, 1] - r[2, 2]) * 2
        q = [0.25 * s, (r[0, 1] + r[1, 0]) / s, (r[0, 2] + r[2, 0]) / s, (r[2, 1] - r[1, 2]) / s]
    elif r[1, 1] > r[2, 2]:
        s = math.sqrt(1.0 + r[1, 1] - r[0, 0] - r[2, 2]) * 2
        q = [(r[0, 1] + r[1, 0]) / s, 0.25 * s, (r[1, 2] + r[2, 1]) / s, (r[0, 2] - r[2, 0]) / s]
    else:
        s = math.sqrt(1.0 + r[2, 2] - r[0, 0] - r[1, 1]) * 2
        q = [(r[0, 2] + r[2, 0]) / s, (r[1, 2] + r[2, 1]) / s, 0.25 * s, (r[1, 0] - r[0, 1]) / s]
    q = np.array(q)
    return q / np.linalg.norm(q), sc


def check(L, E_ref):
    """contrôle : ce que le jeu calcule (os de Mario animés, puis os H_Exo_ dont la rotation est recopiée par le
    nuhlpb) doit retomber sur la pose voulue ; renvoie l'écart maximal de position des os de Rock"""
    W = worlds(L)
    Wf = {}
    err = 0.0
    for b in UNDER:
        i = IDX['H_Exo_' + b]
        m = (W[IDX['Hip']] if b == 'HipN' else Wf[BR[b].parent]) @ L[i]
        if b in PAIR:
            m = m.copy()
            m[:3, :3] = W[IDX[PAIR[b]]][:3, :3]
        Wf[b] = m
        err = max(err, float(np.abs(m[:3, 3] - E_ref[b][:3, 3]).max()))
    return err


CACHE = {}
THROW0 = {}
report = {}
KEEP_Y = {'d03specialairlwstart', 'e01throwhi'}  # Super Shadow : lancer haut, la montée reste dans la pose (comme Zero)
# v1c (vidéo) : Ultimate retourne le perso à la FIN du demi-tour en course, comme Brawl : l'animation de REX telle quelle
# (pivotée de 180°, il se retournait d'un coup au début puis finissait de dos)
ROT180 = set()
# v0.14 : animations de son B bas (le plugin le fait avancer) : sur place
IN_PLACE = {'e01throwf', 'd01specialsdashlw', 'd01specialsdashhi', 'd01specialsendstart', 'd03specialairlwstart', 'd03speciallwend', 'd03speciallwstart', 'a04landingfallspecial', 'e01throwhi'}
if GROUPE == 'supersonic_speciaux':
    # Super Sonic : ses courses et boules sur place (le plugin / le statut de Sonic le déplacent), saut gardé nulle part
    IN_PLACE = {'d01specialshold', 'd01specialairsendloop', 'd01specialsdashlw', 'd01specialsdashhi', 'd00specialnhit',
                'd03speciallwstart', 'd03speciallwhold', 'd03speciallwend', 'd03specialairlwstart',
                'd03specialairlwhold', 'd03specialairlwend', 'd02specialhi',
                # lancers : le plugin le déplace vraiment (avant, arrière, bas) ; montée du haut gardée dans la pose
                'e01throwf', 'e01throwb', 'e01throwlw', 'e01throwhi', 'e00catchdash'}
    KEEP_Y = {'e01throwhi', 'e01throwb'}
if GROUPE == 'chief_coups':
    # Master Chief (vidéos : « il se téléporte ») : jabs et smash avant sur place, le plugin le fait avancer de la
    # distance de REX (table AVANCE, avance_rex.py) : le mouvement de Snake ne suit pas l'avancée de ces animations
    IN_PLACE = {'c00attack11', 'c00attack12', 'c00attack13', 'c03attacks4s'}
    KEEP_Y = set()
if GROUPE == 'chief_speciaux':
    # Master Chief : spéciaux sur place (statuts du plugin ; leçon de la téléportation : rien ne doit avancer dans
    # l'animation si le statut ne suit pas son déplacement)
    IN_PLACE = set(GROUPES['chief_speciaux'])
    KEEP_Y = set()
if GROUPE == 'chief_prises':
    # sur place sauf la prise en courant (le jeu la fait glisser avec l'avancée de l'animation)
    IN_PLACE = {k for k in GROUPES['chief_prises'] if k != 'e00catchdash'}
    KEEP_Y = set()
if GROUPE == 'metal_speciaux':
    IN_PLACE = set(GROUPES['metal_speciaux'])
    KEEP_Y = set()
if GROUPE == 'chief_final':
    IN_PLACE = set(GROUPES['chief_final'])
    KEEP_Y = set()
if GROUPE == 'chief_lancers':
    # Ultimate ne déplace pas le perso avec l'animation pendant un lancer : sur place, le plugin fait le déplacement
    # (recul du lancer arrière, pas en avant du lancer haut : TransN de REX image par image)
    IN_PLACE = set(GROUPES['chief_lancers'])
    KEEP_Y = set()
if GROUPE == 'chief_neutre':
    # sur place sauf les marches / courses des jambes (comme les marches de base : le jeu règle la vitesse dessus)
    IN_PLACE = {k for k in GROUPES['chief_neutre'] if not any(w in k for w in ('slow', 'middle', 'fast', 'walk', 'dash', 'brake'))}  # (d03speciallwground : sur place)
    KEEP_Y = set()
SLICE0 = np.zeros(3)
CUR_FN = ''
for fn, src in JOBS.items():
    src = [s for s in src if s.lstrip('-').split('@')[0] in ANIMS]
    if not src:
        print('ABSENTE de REX :', fn, JOBS[fn])
        continue
    CUR_FN = fn
    frames = []
    worst = 0.0
    for name in src:
        rev = name.startswith('-')
        name = name.lstrip('-')
        cut = None
        if '@' in name:
            name, c_ = name.split('@')
            cut = tuple(int(x) for x in c_.split(':'))
        if name not in CACHE:
            d, o = ANIMS[name]
            CACHE[name] = brawl_anim.read_chr0(d, o)
        an = CACHE[name]
        if fn in HAUT_SEUL or fn in JAMBES_DE:
            an = torse_vise(an)
        if fn in JAMBES_DE:
            an = jambes(an, JAMBES_DE[fn])
        nfr = max(1, an['frames'])
        rng = list(range(nfr)) if cut is None else list(range(cut[0], min(cut[1], nfr)))
        if not frames:
            SLICE0 = brawl_anim.world_pose(BR, an, (rng[::-1] if rev else rng)[0])['TransN'][:3, 3].copy()
        for f in (rng[::-1] if rev else rng):
            L = locals_at(an, f)
            frames.append(L)
            if f % 10 == 0:
                BW = brawl_anim.world_pose(BR, an, f)
                worst = max(worst, check(L, {k: BW[k] @ KEY[k] for k in UNDER}))
    n = len(frames)
    nodes = []
    pistes = TRACKED
    if fn in HAUT_SEUL:
        pistes = [i for i in TRACKED if sous_waist(i)]
        print('   HAUT SEUL', fn, len(pistes), 'os sur', len(TRACKED), ':', sorted(UN[i] for i in pistes)[:80])
    for i in sorted(pistes, key=lambda k: UN[k]):
        vals, prev = [], None
        for k in range(n):
            m = frames[k][i]
            q, sc = quat(m)
            if prev is not None and np.dot(prev, q) < 0:
                q = -q
            prev = q
            vals.append(A.Transform([float(x) for x in sc], [float(x) for x in q], [float(x) for x in m[:3, 3]]))
        if n == 1:
            vals = vals * 2
        tr = A.TrackData('Transform')
        tr.values = vals
        nd = A.NodeData(UN[i])
        nd.tracks = [tr]
        nodes.append(nd)
    g = A.GroupData(A.GroupType.Transform)
    g.nodes = nodes
    a = A.AnimData()
    a.major_version, a.minor_version = 2, 0
    a.final_frame_index = float(max(n - 1, 1))
    a.groups = [g]
    a.save(os.path.join(DST, fn + '.nuanmb'))
    tz = [float(fr[IDX['Trans']][2, 3]) for fr in frames]
    ty = [float(fr[IDX['Trans']][1, 3]) for fr in frames]
    report[fn] = {'rex': src, 'images': n, 'trans_avance': round(tz[-1] - tz[0], 2), 'trans_monte': round(max(ty) - ty[0], 2),
                  'ecart_controle': round(worst, 4)}
    print('%-24s <- %-16s %3d images ; Trans : avance %6.2f, monte %6.2f ; ecart controle %.4f' % (
        fn, '+'.join(src), n, tz[-1] - tz[0], max(ty) - ty[0], worst))
print('ANIMATIONS CONVERTIES (exo, groupe %s) : %d' % (GROUPE, len(report)))
json.dump(report, open(os.path.join(DST, '..', 'anims_exo_%s.json' % GROUPE), 'w'), indent=1)
