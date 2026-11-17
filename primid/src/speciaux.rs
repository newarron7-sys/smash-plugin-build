//! Primid v0.30 : côté B de REX = boomerang. v0.31 : neutre B = Super Scope (plus bas).
//! REX (SpecialSThrowN / SpecialSThrowAir) : il tient le boomerang (main gauche, os LHaveN), élan (vitesse x1,5
//! jusqu'à l'image 47), coup du bras à 47 (coups générés depuis REX), le boomerang part à l'image 50 (RemoveArticle :
//! l'objet lancé est créé par le module de REX). Vidéo du joueur : il part loin devant, touche, revient vers Primid.
//! Le boomerang en vol est dessiné dans le modèle de Primid (pr_boom_vol, sur l'os Hat : le chapeau de Mario, animé
//! partout, inutilisé par le maillage de Primid ; même méthode que les invocations sur Throw). Le jeu ne permet pas
//! de donner un modèle à la boule de feu de Mario (pas de model.numshb : même plantage que pumpwater).

use smash::app::lua_bind::*;
use smash::app::*;
use smash::lib::lua_const::*;
use smash::lib::L2CValue;
use smash::lua2cpp::*;
use smash::phx::{Hash40, Vector3f};
use smash_script::*;
use smashline::*;

fn e_of(boma: *mut BattleObjectModuleAccessor) -> usize {
    unsafe { (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7) }
}

/// boomerang en vol : x, y, vx, vy, temps, phase (0 aucun, 1 aller, 2 retour), angle
static mut BM: [[f32; 7]; 8] = [[0.0; 7]; 8];
/// lancé pendant ce côté B
static mut LANCE: [bool; 8] = [false; 8];

const LANCER: f32 = 50.0;
const ELAN_FIN: f32 = 47.0;
const VITESSE: f32 = 2.6;
const FREIN: f32 = 0.065;
const ID_COUP: u64 = 6;

pub unsafe fn en_vol(boma: *mut BattleObjectModuleAccessor) -> bool {
    BM[e_of(boma)][5] > 0.5
}

/// boomerang tenu (dans la main) : pendant le côté B, avant le lancer, s'il n'y en a pas déjà un en vol
pub unsafe fn tenu(boma: *mut BattleObjectModuleAccessor) -> bool {
    StatusModule::status_kind(boma) == *FIGHTER_STATUS_KIND_SPECIAL_S && !LANCE[e_of(boma)] && !en_vol(boma)
}

unsafe fn coup(fighter: &mut L2CFighterCommon, x: f32, y: f32, dmg: f32, angle: u64, kbg: i32, bkb: i32) {
    let boma = fighter.module_accessor;
    let (px, py) = (PostureModule::pos_x(boma), PostureModule::pos_y(boma));
    let plr = PostureModule::lr(boma);
    macros::ATTACK(fighter, ID_COUP, 0, Hash40::new("top"), dmg, angle, kbg, 0, bkb, 5.5, 0.0, y - py, (x - px) * plr, None, None,
                   None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, true, true, false, false,
                   false, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
                   Hash40::new("collision_attr_cutup"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_CUTUP,
                   *ATTACK_REGION_OBJECT);
}

unsafe fn lancer(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let e = e_of(boma);
    let lr = PostureModule::lr(boma);
    let (x, y) = (PostureModule::pos_x(boma), PostureModule::pos_y(boma));
    // stick vers le haut : lancer vers le haut (REX : SpecialSThrowHi)
    let haut = ControlModule::get_stick_y(boma) > 0.5;
    BM[e] = [x + 9.0 * lr, y + 9.0, VITESSE * lr * if haut { 0.75 } else { 1.0 }, if haut { 1.4 } else { 0.0 }, 0.0, 1.0, 0.0];
    LANCE[e] = true;
}

unsafe fn fin(fighter: &mut L2CFighterCommon, e: usize) {
    let boma = fighter.module_accessor;
    AttackModule::clear(boma, ID_COUP as i32, false);
    BM[e][5] = 0.0;
    ModelModule::clear_joint_srt(boma, Hash40::new("hat"));
}

/// boomerang en vol : montre la copie k (tournée de k x 45°), cache les autres (k < 0 : tout caché)
unsafe fn boom_vol(boma: *mut BattleObjectModuleAccessor, k: i32) {
    const NOMS: [&str; 8] = ["pr_boom_vol0", "pr_boom_vol1", "pr_boom_vol2", "pr_boom_vol3", "pr_boom_vol4",
                             "pr_boom_vol5", "pr_boom_vol6", "pr_boom_vol7"];
    for (i, n) in NOMS.iter().enumerate() {
        ModelModule::set_mesh_visibility(boma, Hash40::new(n), i as i32 == k);
    }
}

/// chaque image (depuis la boucle de Primid) : vol du boomerang, affichage
pub unsafe fn update(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let e = e_of(boma);
    // v0.34 (le joueur) : au tilt bas, il frappe avec son boomerang
    let tilt_bas = MotionModule::motion_kind(boma) == smash::hash40("attack_lw3");
    let tient = (tenu(boma) && MotionModule::frame(boma) < LANCER) || (tilt_bas && !en_vol(boma));
    let gauche = PostureModule::lr(boma) > 0.0;
    ModelModule::set_mesh_visibility(boma, Hash40::new("pr_boomerang"), tient && gauche);
    ModelModule::set_mesh_visibility(boma, Hash40::new("pr_boomerang_r"), tient && !gauche);
    tirs(fighter);
    let [mut x, mut y, mut vx, mut vy, mut t, mut ph, mut a] = BM[e];
    if ph < 0.5 {
        boom_vol(boma, -1);
        return;
    }
    if StatusModule::status_kind(boma) == *FIGHTER_STATUS_KIND_DEAD {
        fin(fighter, e);
        boom_vol(boma, -1);
        return;
    }
    t += 1.0;
    let (px, py) = (PostureModule::pos_x(boma), PostureModule::pos_y(boma) + 9.0);
    if ph < 1.5 {
        // aller : il ralentit, puis revient
        x += vx;
        y += vy;
        vy *= 0.94;
        let s = vx.signum();
        vx -= FREIN * s;
        if vx * s <= 0.0 || t > 60.0 {
            ph = 2.0;
            vx = 0.0;
            vy = 0.0;
            AttackModule::clear(boma, ID_COUP as i32, false);
        }
        coup(fighter, x, y, 5.0, 45, 70, 40);
    } else {
        // retour vers Primid, de plus en plus vite
        let (dx, dy) = (px - x, py - y);
        let d = (dx * dx + dy * dy).sqrt().max(0.001);
        let v = (0.6 + (t - 20.0).max(0.0) * 0.05).min(VITESSE);
        vx = vx * 0.8 + dx / d * v * 0.4;
        vy = vy * 0.8 + dy / d * v * 0.4;
        let n = (vx * vx + vy * vy).sqrt();
        if n > v {
            vx *= v / n;
            vy *= v / n;
        }
        x += vx;
        y += vy;
        coup(fighter, x, y, 3.0, 40, 60, 30);
        if d < 7.0 || t > 240.0 {
            // rattrapé (ou perdu)
            let h = EffectModule::req(boma, Hash40::new("sys_smash_flash_s"), &Vector3f { x, y, z: 0.0 },
                                      &Vector3f { x: 0.0, y: 0.0, z: 0.0 }, 0.6, 0, 0, false, 0) as u32;
            let _ = h;
            fin(fighter, e);
            boom_vol(boma, -1);
            return;
        }
    }
    // tourne sur lui-même (dans le plan de l'écran)
    // v0.36 : rotation par copies déjà tournées (8 x 45°, une par image), plus de rotation de l'os (il basculait)
    a = (a + 1.0) % 8.0;
    BM[e] = [x, y, vx, vy, t, ph, a];
    ModelModule::set_joint_translate(boma, Hash40::new("hat"), &Vector3f { x, y, z: 0.0 }, true, false);
    boom_vol(boma, a as i32);
}

// ------------------------------------------------------------------ statut SPECIAL_S (côté B)

unsafe extern "C" fn s_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    LANCE[e_of(boma)] = false;
    let air = StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR;
    MotionModule::change_motion(boma, Hash40::new(if air { "special_air_s" } else { "special_s" }), 0.0, 1.5, false, 0.0,
                                false, false);
    if air {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_AIR_STOP);
    } else {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
    }
    fighter.sub_shift_status_main(L2CValue::Ptr(s_loop as *const () as _))
}

unsafe extern "C" fn s_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let f = MotionModule::frame(boma);
    // REX : vitesse x1,5 jusqu'à l'image 47
    if f >= ELAN_FIN && MotionModule::rate(boma) > 1.01 {
        MotionModule::set_rate(boma, 1.0);
    }
    if f >= LANCER && !LANCE[e_of(boma)] && !en_vol(boma) {
        lancer(fighter);
    }
    let air = StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR;
    // atterrissage / chute de bord
    if air && MotionModule::motion_kind(boma) == smash::hash40("special_s") {
        MotionModule::change_motion_inherit_frame(boma, Hash40::new("special_air_s"), -1.0, 1.0, 0.0, false, false);
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_AIR_STOP);
    } else if !air && MotionModule::motion_kind(boma) == smash::hash40("special_air_s") {
        fighter.change_status(FIGHTER_STATUS_KIND_LANDING.into(), false.into());
        return 1.into();
    }
    if MotionModule::is_end(boma) {
        if air {
            fighter.change_status(FIGHTER_STATUS_KIND_FALL.into(), false.into());
        } else {
            fighter.change_status(FIGHTER_STATUS_KIND_WAIT.into(), false.into());
        }
        return 1.into();
    }
    // fin du geste (REX : après le lancer, interruptible)
    if f > LANCER + 14.0 {
        if air {
            if fighter.sub_air_check_fall_common().get_bool() {
                return 1.into();
            }
        } else if fighter.sub_wait_ground_check_common(false.into()).get_bool() {
            return 1.into();
        }
    }
    0.into()
}


// ------------------------------------------------------------------ neutre B : Super Scope (v0.31)
// REX : SpecialNRapidN (28 images : un tir à l'image 2, le recul de 2 à 4, rangé à partir de 14 ; un nouvel appui
// avant la fin rejoue le tir), SpecialNCharge (31, en boucle tant que B est tenu), SpecialNChargeShot (70 : tir
// chargé à l'image 2, recul vers le haut). Animations mises bout à bout dans special_n / special_air_n :
// 0-27 tir rapide, 28-58 charge, 59-128 tir chargé. Scope tenu à LHaveN (pr_scope / pr_scope_r, comme l'épée).
// Tirs : objets de REX (WpnPrimBeamS / WpnPrimBeamL) dans le modèle de Primid, sur les os libres mustacheL et Face,
// déplacés par le plugin comme le boomerang ; coup du joueur id 7 à leur place.

const N_TIR_RAPIDE: f32 = 2.0;
const N_RANGE: f32 = 14.0;
const N_FIN_RAPIDE: f32 = 27.0;
const N_CHARGE_DEB: f32 = 28.0;
const N_CHARGE_FIN: f32 = 58.0;
const N_TIR_DEB: f32 = 59.0;
const N_TIR_CHARGE: f32 = 61.0;
const N_FIN: f32 = 128.0;
const N_CHARGE_MAX: f32 = 90.0;
const ID_TIR: u64 = 7;

/// tir en vol : x, y, vx, temps, sorte (0 aucun, 1 petit, 2 chargé), charge 0..1
static mut TIR: [[f32; 6]; 8] = [[0.0; 6]; 8];
/// état du neutre B : 0 tir rapide, 1 charge, 2 tir chargé
static mut N_ETAT: [i32; 8] = [0; 8];
static mut N_CHARGE: [f32; 8] = [0.0; 8];
static mut N_TIRE: [bool; 8] = [false; 8];
/// B tenu depuis le début du coup (sinon pas de charge)
static mut N_TENU: [bool; 8] = [false; 8];
static mut N_ATTENTE: [f32; 8] = [0.0; 8];

unsafe fn coup_tir(fighter: &mut L2CFighterCommon, x: f32, y: f32, dmg: f32, angle: u64, kbg: i32, bkb: i32, taille: f32) {
    let boma = fighter.module_accessor;
    let (px, py) = (PostureModule::pos_x(boma), PostureModule::pos_y(boma));
    let plr = PostureModule::lr(boma);
    macros::ATTACK(fighter, ID_TIR, 0, Hash40::new("top"), dmg, angle, kbg, 0, bkb, taille, 0.0, y - py, (x - px) * plr, None, None,
                   None, 0.6, 0.5, *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, true, true, false, false,
                   false, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
                   Hash40::new("collision_attr_elec"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_ELEC,
                   *ATTACK_REGION_ENERGY);
}

unsafe fn eclat(boma: *mut BattleObjectModuleAccessor, nom: &str, x: f32, y: f32, ech: f32, r: f32, g: f32, b: f32) {
    let h = EffectModule::req(boma, Hash40::new(nom), &Vector3f { x, y, z: 0.0 }, &Vector3f { x: 0.0, y: 0.0, z: 0.0 },
                              ech, 0, 0, false, 0) as u32;
    if h != 0 {
        EffectModule::set_rgb(boma, h, r, g, b);
    }
}

/// bouche du canon dans la pose de tir (animations de REX : 12 devant, 4,6 de haut)
unsafe fn canon(boma: *mut BattleObjectModuleAccessor) -> (f32, f32) {
    let lr = PostureModule::lr(boma);
    (PostureModule::pos_x(boma) + 12.0 * lr, PostureModule::pos_y(boma) + 4.6)
}

unsafe fn tirer(fighter: &mut L2CFighterCommon, charge: Option<f32>) {
    let boma = fighter.module_accessor;
    let e = e_of(boma);
    let lr = PostureModule::lr(boma);
    let (x, y) = canon(boma);
    AttackModule::clear(boma, ID_TIR as i32, false);
    ModelModule::clear_joint_srt(boma, Hash40::new("face"));
    ModelModule::clear_joint_srt(boma, Hash40::new("mustachel"));
    match charge {
        None => {
            TIR[e] = [x, y, 7.0 * lr, 0.0, 1.0, 0.0];
            eclat(boma, "sys_smash_flash_s", x, y, 0.5, 1.0, 0.85, 0.3);
        }
        Some(c) => {
            TIR[e] = [x, y, (3.2 + 0.8 * c) * lr, 0.0, 2.0, c];
            eclat(boma, "sys_smash_flash", x, y, 0.6 + 0.8 * c, 1.0, 0.8, 0.3);
        }
    }
}

/// chaque image : tirs en vol, scope dans la main
unsafe fn tirs(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let e = e_of(boma);
    let scope = StatusModule::status_kind(boma) == *FIGHTER_STATUS_KIND_SPECIAL_N;
    let gauche = PostureModule::lr(boma) > 0.0;
    ModelModule::set_mesh_visibility(boma, Hash40::new("pr_scope"), scope && gauche);
    ModelModule::set_mesh_visibility(boma, Hash40::new("pr_scope_r"), scope && !gauche);
    let [mut x, y, vx, mut t, sorte, c] = TIR[e];
    let mort = StatusModule::status_kind(boma) == *FIGHTER_STATUS_KIND_DEAD;
    let vie = if sorte > 1.5 { 40.0 } else { 12.0 };
    if sorte < 0.5 || mort || t >= vie {
        if sorte > 0.5 {
            AttackModule::clear(boma, ID_TIR as i32, false);
            ModelModule::clear_joint_srt(boma, Hash40::new("face"));
            ModelModule::clear_joint_srt(boma, Hash40::new("mustachel"));
            TIR[e][4] = 0.0;
        }
        ModelModule::set_mesh_visibility(boma, Hash40::new("pr_beam_s"), false);
        ModelModule::set_mesh_visibility(boma, Hash40::new("pr_beam_l"), false);
        return;
    }
    t += 1.0;
    x += vx;
    TIR[e][0] = x;
    TIR[e][3] = t;
    if sorte < 1.5 {
        // petit tir : 2 %, fait tressaillir
        coup_tir(fighter, x, y, 2.0, 361, 25, 12, 2.6);
        ModelModule::set_joint_translate(boma, Hash40::new("mustachel"), &Vector3f { x, y, z: 0.0 }, true, false);
        ModelModule::set_joint_scale(boma, Hash40::new("mustachel"), &Vector3f { x: 2.0, y: 2.0, z: 2.0 });
        ModelModule::set_mesh_visibility(boma, Hash40::new("pr_beam_s"), true);
        // v0.32 : lueur et traînée jaune-orange (tir de REX) pour mieux le voir
        let h = EffectModule::req(boma, Hash40::new("sys_attack_speedline"), &Vector3f { x: x - vx, y, z: 0.0 },
                                  &Vector3f { x: 0.0, y: 90.0 * vx.signum(), z: 0.0 }, 0.7, 0, 0, false, 0) as u32;
        if h != 0 {
            EffectModule::set_rgb(boma, h, 1.0, 0.75, 0.2);
        }
        ModelModule::set_mesh_visibility(boma, Hash40::new("pr_beam_l"), false);
    } else {
        // tir chargé : 5 à 19 %, gros et lent, grossit avec la charge (objet de REX en entier à pleine charge)
        coup_tir(fighter, x, y, 5.0 + 14.0 * c, 361, 70, (30.0 + 30.0 * c) as i32, 3.0 + 5.0 * c);
        let s = 0.5 + 0.7 * c;
        ModelModule::set_joint_translate(boma, Hash40::new("face"), &Vector3f { x, y, z: 0.0 }, true, false);
        ModelModule::set_joint_scale(boma, Hash40::new("face"), &Vector3f { x: s, y: s, z: s });
        ModelModule::set_mesh_visibility(boma, Hash40::new("pr_beam_l"), true);
        ModelModule::set_mesh_visibility(boma, Hash40::new("pr_beam_s"), false);
        {
            let h = EffectModule::req(boma, Hash40::new("sys_attack_speedline"), &Vector3f { x: x - vx * 2.0, y, z: 0.0 },
                                      &Vector3f { x: 0.0, y: 90.0 * vx.signum(), z: 0.0 }, 0.9 + 1.0 * c, 0, 0, false, 0) as u32;
            if h != 0 {
                EffectModule::set_rgb(boma, h, 1.0, 0.85, 0.3);
            }
        }
    }
}

unsafe extern "C" fn n_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = e_of(boma);
    N_ETAT[e] = 0;
    N_CHARGE[e] = 0.0;
    N_TIRE[e] = false;
    N_TENU[e] = true;
    N_ATTENTE[e] = 0.0;
    let air = StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR;
    MotionModule::change_motion(boma, Hash40::new(if air { "special_air_n" } else { "special_n" }), 0.0, 1.0, false, 0.0,
                                false, false);
    if air {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_AIR_STOP);
    } else {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
    }
    fighter.sub_shift_status_main(L2CValue::Ptr(n_loop as *const () as _))
}

unsafe fn n_aller(boma: *mut BattleObjectModuleAccessor, f: f32) {
    let air = StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR;
    MotionModule::change_motion(boma, Hash40::new(if air { "special_air_n" } else { "special_n" }), f, 1.0, false, 0.0,
                                false, false);
}

unsafe extern "C" fn n_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = e_of(boma);
    let f = MotionModule::frame(boma);
    let air = StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR;
    let b_tenu = ControlModule::check_button_on(boma, *CONTROL_PAD_BUTTON_SPECIAL);
    if !b_tenu {
        N_TENU[e] = false;
    }
    // atterrissage / chute de bord : même image, autre animation
    if air && MotionModule::motion_kind(boma) == smash::hash40("special_n") {
        MotionModule::change_motion_inherit_frame(boma, Hash40::new("special_air_n"), -1.0, 1.0, 0.0, false, false);
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_AIR_STOP);
    } else if !air && MotionModule::motion_kind(boma) == smash::hash40("special_air_n") {
        if N_ETAT[e] == 1 {
            MotionModule::change_motion_inherit_frame(boma, Hash40::new("special_n"), -1.0, 1.0, 0.0, false, false);
            KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
        } else {
            fighter.change_status(FIGHTER_STATUS_KIND_LANDING.into(), false.into());
            return 1.into();
        }
    }
    let fin = |fighter: &mut L2CFighterCommon| {
        if air {
            fighter.change_status(FIGHTER_STATUS_KIND_FALL.into(), false.into());
        } else {
            fighter.change_status(FIGHTER_STATUS_KIND_WAIT.into(), false.into());
        }
    };
    match N_ETAT[e] {
        0 => {
            // v0.32 (le joueur : la charge touchait avant de lâcher) : B encore tenu à l'image 2 = pas de tir, il
            // attend (pose de visée) ; relâché avant 12 images = petit tir ; tenu 12 images = charge
            if f >= N_TIR_RAPIDE && !N_TIRE[e] {
                if N_TENU[e] {
                    N_ATTENTE[e] += 1.0;
                    MotionModule::set_rate(boma, 0.0);
                    if N_ATTENTE[e] >= 12.0 {
                        MotionModule::set_rate(boma, 1.0);
                        N_ETAT[e] = 1;
                        n_aller(boma, N_CHARGE_DEB);
                        return 0.into();
                    }
                } else {
                    MotionModule::set_rate(boma, 1.0);
                    N_TIRE[e] = true;
                    tirer(fighter, None);
                }
            }
            if f >= 8.0 && ControlModule::check_button_trigger(boma, *CONTROL_PAD_BUTTON_SPECIAL) {
                N_TIRE[e] = false;
                N_TENU[e] = true;
                N_ATTENTE[e] = 0.0;
                n_aller(boma, 0.0);
                return 0.into();
            }
            if f >= N_FIN_RAPIDE {
                fin(fighter);
                return 1.into();
            }
            if f > N_RANGE + 2.0 {
                if air {
                    if fighter.sub_air_check_fall_common().get_bool() {
                        return 1.into();
                    }
                } else if fighter.sub_wait_ground_check_common(false.into()).get_bool() {
                    return 1.into();
                }
            }
        }
        1 => {
            N_CHARGE[e] = (N_CHARGE[e] + 1.0).min(N_CHARGE_MAX);
            let c = N_CHARGE[e] / N_CHARGE_MAX;
            if (N_CHARGE[e] as i32) % 6 == 1 {
                let (x, y) = canon(boma);
                eclat(boma, "sys_smash_flash_s", x, y, 0.25 + 0.5 * c, 1.0, 0.8, 0.3);
            }
            if f >= N_CHARGE_FIN {
                n_aller(boma, N_CHARGE_DEB);
            }
            if !b_tenu {
                N_ETAT[e] = 2;
                N_TIRE[e] = false;
                n_aller(boma, N_TIR_DEB);
            } else if ControlModule::check_button_on(boma, *CONTROL_PAD_BUTTON_GUARD) && !air {
                // bouclier : annule la charge
                fighter.change_status(FIGHTER_STATUS_KIND_GUARD_ON.into(), true.into());
                return 1.into();
            }
        }
        _ => {
            if f >= N_TIR_CHARGE && !N_TIRE[e] {
                N_TIRE[e] = true;
                let c = N_CHARGE[e] / N_CHARGE_MAX;
                tirer(fighter, Some(c));
            }
            if f >= N_FIN || MotionModule::is_end(boma) {
                fin(fighter);
                return 1.into();
            }
            if f > N_TIR_CHARGE + 40.0 {
                if air {
                    if fighter.sub_air_check_fall_common().get_bool() {
                        return 1.into();
                    }
                } else if fighter.sub_wait_ground_check_common(false.into()).get_bool() {
                    return 1.into();
                }
            }
        }
    }
    0.into()
}

unsafe extern "C" fn s_rien(_fighter: &mut L2CFighterCommon) -> L2CValue {
    0.into()
}

unsafe extern "C" fn rien(_agent: &mut L2CAgentBase) {}

pub fn install(a: &mut Agent) {
    a.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_S, s_main);
    a.status(Init, *FIGHTER_STATUS_KIND_SPECIAL_S, s_rien);
    a.status(Exec, *FIGHTER_STATUS_KIND_SPECIAL_S, s_rien);
    a.status(End, *FIGHTER_STATUS_KIND_SPECIAL_S, s_rien);
    a.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_N, n_main);
    a.status(Init, *FIGHTER_STATUS_KIND_SPECIAL_N, s_rien);
    a.status(Exec, *FIGHTER_STATUS_KIND_SPECIAL_N, s_rien);
    a.status(End, *FIGHTER_STATUS_KIND_SPECIAL_N, s_rien);
    // neutre B : la boule de feu de Mario (créée par son script game) et ses effets retirés
    for m in ["specialn", "specialairn"] {
        for k in ["game", "effect", "sound", "expression"] {
            a.acmd(&format!("{}_{}", k, m), rien, Priority::Low);
        }
    }
    // effets, sons et expressions de la cape de Mario retirés (coups : generated.rs, MAP_PLUS)
    for m in ["specials", "specialairs"] {
        for k in ["effect", "sound", "expression"] {
            a.acmd(&format!("{}_{}", k, m), rien, Priority::Low);
        }
    }
}
