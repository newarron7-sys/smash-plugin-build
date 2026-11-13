//! B neutre de Master Chief (REX). Analyse de ses scripts (FitChief.pac : action 0x112, surcharge de l'action 0x10D,
//! sous-actions SpecialN*, sous-programmes 0x306E4 / 0x28964 / 0x2AB24) :
//! - début (SpecialN, 10 images) ; à l'image 5, B n'est plus tenu -> « Tap Pistol » : magnum, tir instantané à
//!   l'image 16 (rayon depuis ThrowN : 4 % tout près, 3 % jusqu'à 44, 1 % jusqu'à 136), 8 balles puis rechargement
//!   (SpecialNTapPistolReload, 8 balles à l'image 70) ; B appuyé entre les images 30 et 39 : nouveau tir (image 13) ;
//! - B tenu -> posture « fusil d'assaut » : jambes et haut du corps séparés (marche lente / moyenne / rapide, demi-tour,
//!   accroupi, saut) ; visée haut / bas au stick (SpecialNAimREDO, image 94 = droit devant, 3 images par image au stick
//!   à fond : de +77° à -52°) ; B tenu = tir automatique (SpecialNGunShoot, une balle toutes les 5 images, 1 %, 32 balles),
//!   chargeur vide = clic (SpecialNGunEmpty) ; A + B : recharger (AssaultReload, 32 balles à l'image 67) si le chargeur
//!   n'est pas plein, sinon A = coup de crosse (SpiecalNSmack, 6 %, image 5) ; chargeur vide en entrant : recharge ;
//!   B relâché -> fin (SpecialNEnd).
//! Sur Ultimate : la grenade de Snake a les mêmes mécaniques (début, « tenir » avec marche / saut sur les statuts
//! ItemShoot du jeu, lancer quand B est relâché). Ses statuts sont gardés, sans la grenade (scripts remplacés) ; le
//! plugin fait le reste : tirs, visée, gestes du haut du corps (mouvement partiel), magnum et fin (statut « lancer »).
//! Tirs : capsule du canon jusqu'à la portée, comme ses autres tirs instantanés (étape 1) ; le décor n'arrête pas les
//! balles (le rayon de REX s'arrête au premier obstacle).

use smash::app::lua_bind::*;
use smash::app::*;
use smash::hash40;
use smash::lib::lua_const::*;
use smash::lib::L2CValue;
use smash::lua2cpp::*;
use smash::phx::{Hash40, Vector3f};
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

use crate::speciaux::{eff_monde, finir, rien, situation, slot};

const FUSIL_MAX: i32 = 32;
const MAGNUM_MAX: i32 = 8;
const VISEE_DEPART: f32 = 94.0;
const VISEE_MAX: f32 = 151.0;
/// hauteur / avance du canon par rapport aux pieds (ThrowN de REX en posture, image 94 de la visée)
const CANON_Y: f32 = 13.6;
const CANON_Z: f32 = 4.0;
const PORTEE_FUSIL: f32 = 150.0;
/// bout du canon devant la main (vidéo v15 : fusil ~4 devant la main droite ; magnum ~2,5)
const LONG_FUSIL: f32 = 6.0;
const LONG_MAGNUM: f32 = 4.0;
/// tailles de REX × 0,82 (même échelle que ses autres coups, gen_rex.py)
const ECHELLE: f32 = 0.82;

// état du haut du corps en posture (variable LA-Basic 2 de REX)
const H_ATTENTE: u8 = 0;
const H_TIR: u8 = 1;
const H_VIDE: u8 = 2;
const H_CROSSE: u8 = 3;
const H_RECHARGE: u8 = 4;

static mut FUSIL: [i32; 8] = [FUSIL_MAX; 8];
static mut MAGNUM: [i32; 8] = [MAGNUM_MAX; 8];
static mut VISEE: [f32; 8] = [VISEE_DEPART; 8];
static mut HAUT: [u8; 8] = [0; 8];
static mut HT: [i32; 8] = [0; 8];
static mut DANS: [bool; 8] = [false; 8];
static mut RAFALE: [bool; 8] = [false; 8];
static mut PART: [i32; 8] = [1; 8];
static mut EFFACER: [i32; 8] = [0; 8];
/// statut « lancer » : 0 magnum, 1 rechargement du magnum, 2 fin de la posture
static mut MODE: [u8; 8] = [0; 8];

/// axe de tangage (haut / bas) dans le repère local des os « waist » et « bust » du squelette de Snake (model.nusktb :
/// x local = le long du dos, y local = devant, z local = sur le côté) : +z penche le haut du corps en avant (canon
/// vers le bas), donc -z pour un angle « vers le haut » positif ; part de la visée donnée à chacun (REX tourne BodyN,
/// BustN et NeckN)
const AXE_TANGAGE: (f32, f32, f32) = (0.0, 0.0, -1.0);
const PART_WAIST: f32 = 0.4;
const PART_BUST: f32 = 0.6;

fn tenir(st: i32) -> bool {
    unsafe {
        [*FIGHTER_SNAKE_STATUS_KIND_SPECIAL_N_HOLD_WAIT, *FIGHTER_SNAKE_STATUS_KIND_SPECIAL_N_HOLD_WALK_F,
         *FIGHTER_SNAKE_STATUS_KIND_SPECIAL_N_HOLD_WALK_B, *FIGHTER_SNAKE_STATUS_KIND_SPECIAL_N_HOLD_WALK_BRAKE_F,
         *FIGHTER_SNAKE_STATUS_KIND_SPECIAL_N_HOLD_WALK_BRAKE_B, *FIGHTER_SNAKE_STATUS_KIND_SPECIAL_N_HOLD_DASH_F,
         *FIGHTER_SNAKE_STATUS_KIND_SPECIAL_N_HOLD_DASH_B, *FIGHTER_SNAKE_STATUS_KIND_SPECIAL_N_HOLD_JUMP_SQUAT,
         *FIGHTER_SNAKE_STATUS_KIND_SPECIAL_N_HOLD_JUMP, *FIGHTER_SNAKE_STATUS_KIND_SPECIAL_N_HOLD_JUMP_AERIAL,
         *FIGHTER_SNAKE_STATUS_KIND_SPECIAL_N_HOLD_AIR, *FIGHTER_SNAKE_STATUS_KIND_SPECIAL_N_HOLD_LANDING]
            .contains(&st)
    }
}

/// angle du tir (degrés, + = vers le haut) pour une image de la visée de REX (mesuré sur l'os ThrowN de
/// SpecialNAimREDO : image 0 = 77°, 47 = 36°, 94 = -4°, 120 = -21°, 150 = -52°)
fn angle_visee(v: f32) -> f32 {
    77.0 - 0.862 * v
}

unsafe fn tourner(boma: *mut BattleObjectModuleAccessor, os: &str, deg: f32) {
    let r = Vector3f { x: AXE_TANGAGE.0 * deg, y: AXE_TANGAGE.1 * deg, z: AXE_TANGAGE.2 * deg };
    ModelModule::set_joint_rotate(boma, Hash40::new(os), &r,
                                  MotionNodeRotateCompose { _address: *MOTION_NODE_ROTATE_COMPOSE_AFTER as u8 },
                                  MotionNodeRotateOrder { _address: *MOTION_NODE_ROTATE_ORDER_XYZ as u8 });
}

unsafe fn viser(boma: *mut BattleObjectModuleAccessor, deg: f32) {
    tourner(boma, "waist", deg * PART_WAIST);
    tourner(boma, "bust", deg * PART_BUST);
}

const HAUTS: [&str; 10] = ["special_lw_ground", "special_lw_squat_ground", "special_lw_squat_blast", "special_lw_enemy", "special_n_hold", "special_air_n_hold", "special_air_n_throw_lw", "special_air_n_throw_hi",
                           "special_n_throw_hi", "special_n_throw_lw"];

/// où le jeu met le haut du corps pendant la posture : Some(0) = mouvement principal, Some(k) = mouvement partiel k,
/// None = pas trouvé. Vidéo v13 : jambes figées en marchant ; la v13 supposait le haut du corps en mouvement partiel 1
/// sans le vérifier : si le jeu y met les jambes (h10itemlegs*), le geste du fusil les remplaçait. Cherché à chaque
/// image (le jeu le remet à chaque changement de statut)
unsafe fn ou_haut(boma: *mut BattleObjectModuleAccessor) -> Option<i32> {
    let est = |m: u64| HAUTS.iter().any(|n| hash40(n) == m);
    if est(MotionModule::motion_kind(boma)) {
        return Some(0);
    }
    (1..5).find(|&k| est(MotionModule::motion_kind_partial(boma, k)))
}

/// geste du haut du corps, là où le jeu met celui de la posture ; jamais ailleurs (les jambes restent au jeu)
unsafe fn haut(boma: *mut BattleObjectModuleAccessor, e: usize, mot: &str, reprendre: bool) {
    let h = hash40(mot);
    let p = match ou_haut(boma) {
        Some(p) => p,
        None => return,
    };
    PART[e] = p;
    if p == 0 {
        if MotionModule::motion_kind(boma) != h || reprendre {
            MotionModule::change_motion(boma, Hash40::new_raw(h), 0.0, 1.0, false, 0.0, false, false);
        }
        return;
    }
    if MotionModule::motion_kind_partial(boma, p) == h && !reprendre {
        return;
    }
    MotionModule::remove_motion_partial(boma, p, false);
    MotionModule::add_motion_partial(boma, p, Hash40::new_raw(h), 0.0, 1.0, false, false, 0.0, true, true, false);
}

unsafe fn rafale_stop(boma: *mut BattleObjectModuleAccessor, e: usize) {
    if RAFALE[e] {
        SoundModule::stop_se(boma, Hash40::new("se_snake_special_n03"), 0);
        RAFALE[e] = false;
    }
}

/// tir instantané : capsule depuis le canon dans la direction visée
unsafe fn tir(fighter: &mut L2CFighterCommon, id: u64, dmg: f32, kbg: i32, fkb: i32, bkb: i32, taille: f32,
              y0: f32, z0: f32, deg: f32, d0: f32, d1: f32, hitlag: f32) {
    let (s, c) = deg.to_radians().sin_cos();
    ATTACK(fighter, id, 0, Hash40::new("top"), dmg, 361, kbg, fkb, bkb, taille * ECHELLE,
           0.0, y0 + s * d0, z0 + c * d0, Some(0.0), Some(y0 + s * d1), Some(z0 + c * d1), hitlag, 1.0,
           *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true,
           *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
           Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_PUNCH, *ATTACK_REGION_OBJECT);
}

/// position de la main qui tient l'arme, par rapport aux pieds (y = haut, z = devant) : v16, « les effets de feu ne
/// suivent même pas le canon » (position fixe calculée) ; maintenant lue sur l'os de la main (REX : fusil sur RHaveN
/// = « haver », magnum sur LHaveN = « havel »)
unsafe fn main_arme(boma: *mut BattleObjectModuleAccessor, os: &str) -> (f32, f32) {
    let p = PostureModule::pos(boma);
    let lr = PostureModule::lr(boma);
    let mut v = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    ModelModule::joint_global_position(boma, Hash40::new(os), &mut v, true);
    if v.x == 0.0 && v.y == 0.0 {
        return (CANON_Y, CANON_Z);
    }
    (v.y - (*p).y, (v.x - (*p).x) * lr)
}

/// tir au bout du canon ; demandé (v13) : « rajoute les explosions comme sur les smash » : mêmes effets que canon()
/// d'effets.rs (éclair de bouche, flamme, éclat jaune), plus `cone` flammes devant pour le magnum (comme le smash avant).
/// v20 : « l'effet de feu est sur l'arme, je t'ai dit au bout du canon, fais comme pour le tilt haut, smash avant, smash
/// bas » : comme eux, effets posés sur l'os « top » (ils suivent Chief), au bout du canon : main avant + `long` dans
/// l'axe visé (REX : départ du tir à 3,8 devant LHaveN = « havel »)
unsafe fn eclair(boma: *mut BattleObjectModuleAccessor, y0: f32, z0: f32, deg: f32, long: f32, sc: f32, cone: i32) {
    let (s, c) = deg.to_radians().sin_cos();
    let y = y0 + s * long;
    let z = z0 + c * long;
    let pose = |eff: &str, y: f32, z: f32, t: f32, rgb: Option<(f32, f32, f32)>| {
        let h = EffectModule::req_follow(boma, Hash40::new(eff), Hash40::new("top"), &Vector3f { x: 0.0, y, z },
                                         &Vector3f { x: 0.0, y: 0.0, z: 0.0 }, t, false, 0, 0, 0, 0, 0, false, false) as u32;
        if let (Some((r, g, b)), true) = (rgb, h != 0) {
            EffectModule::set_rgb(boma, h, r, g, b);
        }
    };
    pose("sys_muzzleflash", y, z, 0.8 * sc, None);
    pose("sys_hit_fire", y, z, 0.35 * sc, None);
    pose("sys_smash_flash_s", y, z, 0.25 * sc, Some((1.0, 0.9, 0.55)));
    for k in 1..=cone {
        let d = 4.0 * k as f32;
        pose("sys_hit_fire", y + s * d, z + c * d, 0.4 + 0.15 * k as f32, None);
    }
}

/// chaque image (ligne principale) : munitions, posture
pub unsafe fn chaque_image(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    let st = StatusModule::status_kind(boma);
    if st == *FIGHTER_STATUS_KIND_ENTRY || st == *FIGHTER_STATUS_KIND_REBIRTH {
        FUSIL[e] = FUSIL_MAX;
        MAGNUM[e] = MAGNUM_MAX;
    }
    if EFFACER[e] > 0 {
        EFFACER[e] -= 1;
        if EFFACER[e] == 0 {
            AttackModule::clear_all(boma);
        }
    }
    let dans = tenir(st);
    if !dans {
        if DANS[e] {
            DANS[e] = false;
            rafale_stop(boma, e);
            viser(boma, 0.0);
        }
        return;
    }
    if !DANS[e] {
        DANS[e] = true;
        VISEE[e] = VISEE_DEPART;
        HAUT[e] = if FUSIL[e] <= 0 { H_RECHARGE } else { H_ATTENTE };
        HT[e] = 0;
        PART[e] = -1;
    }
    let air = StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR;
    // visée : stick haut = image de la visée qui baisse = canon qui monte (REX : -3 × stick par image)
    let sy = ControlModule::get_stick_y(boma);
    VISEE[e] = (VISEE[e] - 3.0 * sy).clamp(0.0, VISEE_MAX);
    let deg = angle_visee(VISEE[e]);
    viser(boma, deg - angle_visee(VISEE_DEPART));

    // v18 / v19 (vidéo) : immobile en posture, jambes droites et serrées : à l'arrêt le jeu ne met pas de jambes sous
    // le geste du haut du corps (en marchant, celles de la marche) ; « special_lw_ground » = le tir avec les jambes de
    // la posture de REX (SpecialNLegWait)
    let debout = st == *FIGHTER_SNAKE_STATUS_KIND_SPECIAL_N_HOLD_WAIT;
    let a = ControlModule::check_button_trigger(boma, *CONTROL_PAD_BUTTON_ATTACK);
    let libre = HAUT[e] == H_ATTENTE || HAUT[e] == H_TIR || HAUT[e] == H_VIDE;
    if libre && a {
        rafale_stop(boma, e);
        HAUT[e] = if FUSIL[e] < FUSIL_MAX { H_RECHARGE } else { H_CROSSE };
        HT[e] = 0;
    } else if libre && FUSIL[e] > 0 {
        if HAUT[e] != H_TIR {
            HAUT[e] = H_TIR;
            HT[e] = 0;
        }
    } else if libre && HAUT[e] != H_VIDE {
        rafale_stop(boma, e);
        HAUT[e] = H_VIDE;
        HT[e] = 0;
    }
    let t = HT[e];
    match HAUT[e] {
        H_TIR => {
            // immobile : la version avec les jambes ; en marchant : celle du jeu (haut du corps seul)
            haut(boma, e, if debout { "special_lw_ground" } else if air { "special_air_n_hold" } else { "special_n_hold" }, false);
            if t % 5 == 0 {
                // une balle (REX : sous-programme 0x28964 ; 1 %, croissance 25, base 8, taille 1,85)
                FUSIL[e] -= 1;
                AttackModule::clear_all(boma);
                let (my, mz) = main_arme(boma, "havel");
                tir(fighter, 0, 1.0, 25, 0, 8, 1.85, my, mz, deg, 2.0, PORTEE_FUSIL, 0.5);
                EFFACER[e] = 3;
                eclair(boma, my, mz, deg, LONG_FUSIL, 0.55, 0);
                // geste de tir : c'est l'animation « tenir » du jeu elle-même (SpecialNGunShoot en boucle) : v13 / v14,
                // jambes figées en marchant quand le plugin changeait le haut du corps à chaque balle
                if debout {
                    haut(boma, e, "special_lw_ground", true);
                }
                if !RAFALE[e] {
                    SoundModule::play_se(boma, Hash40::new("se_snake_special_n03"), true, false, false, false, enSEType(0));
                    RAFALE[e] = true;
                }
            }
        }
        H_VIDE => {
            if t == 0 {
                if debout {
                    haut(boma, e, "special_lw_enemy", true);
                }
                PLAY_SE(fighter, Hash40::new("se_snake_special_n05"));
            }
        }
        H_CROSSE => {
            if t == 0 {
                PLAY_SE(fighter, Hash40::new("se_snake_special_n06"));
                haut(boma, e, if debout { "special_lw_squat_blast" } else { "special_n_throw_hi" }, true);
            }
            if t == 5 {
                // REX : 2 coups sur BustN (6 %) : le 1er pour les persos au sol, le 2e (vers le bas) en l'air
                AttackModule::clear_all(boma);
                for (id, ang, kbg, bkb, sit) in [(1u64, 361u64, 80, 70, *COLLISION_SITUATION_MASK_G),
                                                 (2u64, 270u64, 10, 64, *COLLISION_SITUATION_MASK_A)] {
                    ATTACK(fighter, id, 0, Hash40::new("top"), 6.0, ang, kbg, 0, bkb, 7.2 * ECHELLE, 0.0, 15.0, 7.5,
                           None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0,
                           false, false, false, false, true, sit, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL,
                           false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_PUNCH,
                           *ATTACK_REGION_OBJECT);
                }
                EFFACER[e] = 2;
            }
            if t >= 55 {
                HAUT[e] = H_ATTENTE;
                HT[e] = -1;
            }
        }
        H_RECHARGE => {
            if t == 0 {
                PLAY_SE(fighter, Hash40::new("se_snake_special_n04"));
                haut(boma, e, if debout { "special_lw_squat_ground" } else { "special_n_throw_lw" }, true);
            }
            if t == 67 {
                FUSIL[e] = FUSIL_MAX;
            }
            if t >= 85 {
                HAUT[e] = H_ATTENTE;
                HT[e] = -1;
            }
        }
        _ => {
            haut(boma, e, if debout { "special_lw_ground" } else if air { "special_air_n_hold" } else { "special_n_hold" }, false);
        }
    }
    HT[e] += 1;
}

// ------------------------------------------------------------------ statut « lancer » : magnum ou fin de la posture

unsafe fn mots(e: usize) -> (&'static str, &'static str) {
    match MODE[e] {
        0 => ("special_n_throw_m", "special_air_n_throw_m"),
        1 => ("special_lw_squat_start", "special_air_lw_wall"),
        _ => ("special_n_close", "special_air_lw_blast"),
    }
}

unsafe extern "C" fn lancer_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    // relâché pendant le début = B tapé (magnum) ; relâché en posture = fin
    MODE[e] = if StatusModule::prev_status_kind(boma, 0) == *FIGHTER_STATUS_KIND_SPECIAL_N { 0 } else { 2 };
    // geste du fusil laissé en mouvement partiel : retiré (jamais un partiel que le jeu n'a pas mis pour le haut du corps)
    if PART[e] > 0 && HAUTS.iter().any(|n| hash40(n) == MotionModule::motion_kind_partial(boma, PART[e])) {
        MotionModule::remove_motion_partial(boma, PART[e], false);
    }
    let (s, a) = mots(e);
    situation(fighter, s, a, true);
    fighter.sub_shift_status_main(L2CValue::Ptr(lancer_boucle as *const () as _))
}

unsafe extern "C" fn lancer_boucle(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    let (s, a) = mots(e);
    situation(fighter, s, a, false);
    let f = MotionModule::frame(boma);
    let fp = MotionModule::prev_frame(boma);
    let passe = |k: f32| fp < k && f >= k;
    match MODE[e] {
        0 => {
            if passe(7.0) {
                PLAY_SE(fighter, Hash40::new("se_snake_special_n01"));
            }
            if passe(16.0) {
                // REX : rayon depuis ThrowN (à l'image 16 : 15,2 de haut, 11,7 devant, droit devant)
                MAGNUM[e] -= 1;
                PLAY_SE(fighter, Hash40::new("se_snake_special_n02"));
                AttackModule::clear_all(boma);
                let (my, mz) = main_arme(boma, "havel");
                tir(fighter, 0, 4.0, 76, 20, 75, 5.4, my, mz, 0.0, 0.0, 4.0, 1.0);
                tir(fighter, 1, 3.0, 60, 20, 56, 2.5, my, mz, 0.0, 0.0, 44.0, 1.0);
                tir(fighter, 2, 1.0, 0, 0, 7, 1.0, my, mz, 0.0, 0.0, 136.0, 1.0);
                EFFACER[e] = 3;
                eclair(boma, my, mz, 0.0, LONG_MAGNUM, 1.6, 2);
            }
            if f >= 30.0 && f < 40.0 && MAGNUM[e] > 0
                && ControlModule::check_button_trigger(boma, *CONTROL_PAD_BUTTON_SPECIAL) {
                MotionModule::set_frame(boma, 13.0, true);
            }
            if MotionModule::is_end(boma) && MAGNUM[e] <= 0 {
                MODE[e] = 1;
                let (s, a) = mots(e);
                situation(fighter, s, a, true);
                return 0.into();
            }
        }
        1 => {
            if passe(70.0) {
                MAGNUM[e] = MAGNUM_MAX;
            }
        }
        _ => {}
    }
    finir(fighter);
    0.into()
}

pub fn install(a: &mut Agent) {
    // scripts de la grenade de Snake (création de la grenade et de la goupille, effets, sons) et des mouvements de
    // son B bas réutilisés (création de la C4) : vides, le plugin fait tout
    for m in ["specialnstart", "specialairnstart", "specialnhold", "specialairnhold", "specialnthrowm",
              "specialnthrowhi", "specialnthrowlw", "specialairnthrowm", "specialairnthrowhi", "specialairnthrowlw",
              "specialnthrowfail", "specialairnthrowfail", "specialnclose", "speciallwsquatstart", "speciallwground", "speciallwsquatground", "speciallwsquatblast", "speciallwenemy",
              "specialairlwwall", "specialairlwblast"] {
        for k in ["game_", "effect_", "sound_", "expression_"] {
            let n: &'static str = Box::leak(format!("{}{}", k, m).into_boxed_str());
            a.acmd(n, rien, Priority::Low);
        }
    }
    a.status(Main, *FIGHTER_SNAKE_STATUS_KIND_SPECIAL_N_THROW, lancer_main);
}
