//! Primid v0.30 : côté B de REX = boomerang.
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

/// chaque image (depuis la boucle de Primid) : vol du boomerang, affichage
pub unsafe fn update(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let e = e_of(boma);
    let tient = tenu(boma) && MotionModule::frame(boma) < LANCER;
    let gauche = PostureModule::lr(boma) > 0.0;
    ModelModule::set_mesh_visibility(boma, Hash40::new("pr_boomerang"), tient && gauche);
    ModelModule::set_mesh_visibility(boma, Hash40::new("pr_boomerang_r"), tient && !gauche);
    let [mut x, mut y, mut vx, mut vy, mut t, mut ph, mut a] = BM[e];
    if ph < 0.5 {
        ModelModule::set_mesh_visibility(boma, Hash40::new("pr_boom_vol"), false);
        return;
    }
    if StatusModule::status_kind(boma) == *FIGHTER_STATUS_KIND_DEAD {
        fin(fighter, e);
        ModelModule::set_mesh_visibility(boma, Hash40::new("pr_boom_vol"), false);
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
            ModelModule::set_mesh_visibility(boma, Hash40::new("pr_boom_vol"), false);
            return;
        }
    }
    // tourne sur lui-même (dans le plan de l'écran)
    a = (a + 33.0) % 360.0;
    BM[e] = [x, y, vx, vy, t, ph, a];
    ModelModule::set_joint_translate(boma, Hash40::new("hat"), &Vector3f { x, y, z: 0.0 }, true, false);
    ModelModule::set_joint_rotate(boma, Hash40::new("hat"), &Vector3f { x: a, y: 0.0, z: 0.0 },
                                  MotionNodeRotateCompose { _address: *MOTION_NODE_ROTATE_COMPOSE_AFTER as u8 },
                                  MotionNodeRotateOrder { _address: *MOTION_NODE_ROTATE_ORDER_XYZ as u8 });
    ModelModule::set_mesh_visibility(boma, Hash40::new("pr_boom_vol"), true);
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

unsafe extern "C" fn s_rien(_fighter: &mut L2CFighterCommon) -> L2CValue {
    0.into()
}

unsafe extern "C" fn rien(_agent: &mut L2CAgentBase) {}

pub fn install(a: &mut Agent) {
    a.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_S, s_main);
    a.status(Init, *FIGHTER_STATUS_KIND_SPECIAL_S, s_rien);
    a.status(Exec, *FIGHTER_STATUS_KIND_SPECIAL_S, s_rien);
    a.status(End, *FIGHTER_STATUS_KIND_SPECIAL_S, s_rien);
    // effets, sons et expressions de la cape de Mario retirés (coups : generated.rs, MAP_PLUS)
    for m in ["specials", "specialairs"] {
        for k in ["effect", "sound", "expression"] {
            a.acmd(&format!("{}_{}", k, m), rien, Priority::Low);
        }
    }
}
