//! Primid v0.2 : son bas B de REX (invocations de l'Émissaire subspatial).
//!
//! REX (FitPrimid.pac, action 0x115) : SummonStart (vitesse 2) -> SummonWait (B tenu, portail au-dessus de sa tête) ;
//! le stick choisit l'ennemi (au-delà de 0,75) -> SummonGo (interruptible à l'image 9) ; B lâché -> SummonEnd.
//! Recharges de REX : haut (Mite) 256 images, avant (Botron) 640 ; une seule invocation de chaque à la fois.
//! Cymul (arrière) et Boobas (bas) : modèles prêts, comportement à la version suivante.
//!
//! Les ennemis sont les modèles de REX mis sur des objets de Mario que Primid n'utilise pas :
//! - Mite -> tuyau d'entrée (dokan, statut DUMMY) : court au sol vers l'avant, fonce sur l'adversaire proche ;
//! - Botron -> J.E.T. (pump, statut WAIT) : vole au-dessus du terrain en portant son pot ;
//! - le pot de Botron -> eau du J.E.T. (pumpwater, statut REGULAR) : lâché au-dessus de l'adversaire, explose en feu.
//! Leurs statuts (Main / Exec) sont remplacés pour le costume de Primid seulement, et seulement pour les objets que le
//! bas B a créés (sinon : code d'origine, par exemple le tuyau de l'entrée en scène).

use smash::app::lua_bind::*;
use smash::app::*;
use smash::lib::lua_const::*;
use smash::lib::L2CValue;
use smash::lua2cpp::*;
use smash::phx::{Hash40, Vector2f, Vector3f};
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

use crate::PR_COSTUMES;

// ------------------------------------------------------------------ réglages (REX)
const RECH_MITE: i32 = 256;
const RECH_BOTRON: i32 = 640;
/// image de SummonGo à partir de laquelle il peut agir (REX : AllowInterrupt à 9)
const GO_LIBRE: f32 = 9.0;
/// B tenu au plus (images) avant qu'il range le portail tout seul
const ATTENTE_MAX: i32 = 180;

// Mite : animation unique (j00entry du tuyau) : course 0-24, élan 25-26, plongeon 27-28, attente 29-109
const MITE_COURSE: f32 = 25.0;
const MITE_ELAN: f32 = 25.0;
const MITE_PLONGE: f32 = 27.0;

static mut PH: [i32; 8] = [0; 8];
static mut T: [i32; 8] = [0; 8];
static mut PORTAIL: [u32; 8] = [0; 8];
static mut RECH: [[i32; 2]; 8] = [[0; 2]; 8];

/// objets créés par le bas B : 1 = à réclamer (créé à l'instant), sinon n° de l'objet
static mut MITE_ATTEND: [bool; 8] = [false; 8];
static mut MITE_ID: [u32; 8] = [0; 8];
static mut BOT_ATTEND: [bool; 8] = [false; 8];
static mut BOT_ID: [u32; 8] = [0; 8];
static mut POT_ATTEND: [bool; 8] = [false; 8];
static mut POT_ID: [u32; 8] = [0; 8];

// état des objets (par perso)
static mut M: [[f32; 6]; 8] = [[0.0; 6]; 8]; // x, y, vy, lr, phase (0 course, 1 plongeon), temps
static mut B: [[f32; 6]; 8] = [[0.0; 6]; 8]; // x, y, lr, temps, lâché (0/1), temps depuis le lâcher
static mut P: [[f32; 4]; 8] = [[0.0; 4]; 8]; // y, vy, explosé (temps), -

fn e_of(boma: *mut BattleObjectModuleAccessor) -> usize {
    unsafe { (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7) }
}

unsafe fn boma_of(id: u32) -> Option<*mut BattleObjectModuleAccessor> {
    if id == 0 || id == *BATTLE_OBJECT_ID_INVALID as u32 {
        return None;
    }
    let b = sv_battle_object::module_accessor(id);
    if b.is_null() { None } else { Some(b) }
}

/// propriétaire (Primid) d'un objet : (son module, n° de perso)
unsafe fn owner(w: *mut BattleObjectModuleAccessor) -> Option<(*mut BattleObjectModuleAccessor, usize)> {
    let o = boma_of(WorkModule::get_int(w, *WEAPON_INSTANCE_WORK_ID_INT_LINK_OWNER) as u32)?;
    Some((o, e_of(o)))
}

unsafe fn id_of(weapon: &mut L2CWeaponCommon) -> u32 {
    (*weapon.battle_object).battle_object_id
}

unsafe fn pos(b: *mut BattleObjectModuleAccessor) -> (f32, f32) {
    (PostureModule::pos_x(b), PostureModule::pos_y(b))
}

/// sol sous (x, y) à moins de `bas` : sa hauteur
unsafe fn sol(w: *mut BattleObjectModuleAccessor, x: f32, y: f32, bas: f32) -> Option<f32> {
    let de = Vector2f { x, y: y + 6.0 };
    let dir = Vector2f { x: 0.0, y: -(bas + 6.0) };
    let mut hit = Vector2f { x: 0.0, y: 0.0 };
    if GroundModule::ray_check_hit_pos(w, &de, &dir, &mut hit, true) { Some(hit.y) } else { None }
}

/// adversaire le plus proche (hors propriétaire) : (dx, dy) depuis (x, y)
unsafe fn adversaire(moi: usize, x: f32, y: f32) -> Option<(f32, f32)> {
    let mut best: Option<(f32, f32)> = None;
    for n in 0..8 {
        if n == moi {
            continue;
        }
        let id = smash::app::Fighter::get_id_from_entry_id(n as i32);
        let Some(o) = boma_of(id) else { continue };
        if smash::app::utility::get_category(&mut *o) != *BATTLE_OBJECT_CATEGORY_FIGHTER {
            continue;
        }
        let st = StatusModule::status_kind(o);
        if st == *FIGHTER_STATUS_KIND_DEAD || st == *FIGHTER_STATUS_KIND_STANDBY {
            continue;
        }
        let (ox, oy) = pos(o);
        let (dx, dy) = (ox - x, oy - y);
        if best.map_or(true, |(bx, by)| dx * dx + dy * dy < bx * bx + by * by) {
            best = Some((dx, dy));
        }
    }
    best
}

/// objet libéré de Mario (contrainte de modèle puis lien, dans cet ordre : méthode de Chief), à plat, tourné
unsafe fn liberer(w: *mut BattleObjectModuleAccessor, lr: f32) {
    if LinkModule::is_model_constraint(w) {
        LinkModule::remove_model_constraint(w, true);
    }
    LinkModule::unlink_all(w);
    PostureModule::set_rot(w, &Vector3f { x: 0.0, y: 0.0, z: 0.0 }, 0);
    PostureModule::set_lr(w, lr);
    PostureModule::update_rot_y_lr(w);
}

unsafe fn placer(w: *mut BattleObjectModuleAccessor, x: f32, y: f32) {
    PostureModule::set_pos(w, &Vector3f { x, y, z: 0.0 });
}

unsafe fn eff(w: *mut BattleObjectModuleAccessor, name: &str, x: f32, y: f32, sc: f32) {
    EffectModule::req(w, Hash40::new(name), &Vector3f { x, y, z: 0.0 }, &Vector3f { x: 0.0, y: 0.0, z: 0.0 }, sc, 0, 0, false, 0);
}

unsafe fn tuer(weapon: &mut L2CWeaponCommon) {
    notify_event_msc_cmd!(weapon, Hash40::new_raw(0x199c462b5d));
}

// ------------------------------------------------------------------ bas B de Primid (statut SPECIAL_LW de Mario)

unsafe fn motion(fighter: &mut L2CFighterCommon, sol_: &str, air: &str, rate: f32, garder: bool) {
    let boma = fighter.module_accessor;
    let m = if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND { sol_ } else { air };
    if MotionModule::motion_kind(boma) == smash::hash40(m) {
        return;
    }
    if garder {
        MotionModule::change_motion_inherit_frame(boma, Hash40::new(m), -1.0, rate, 0.0, false, false);
    } else {
        MotionModule::change_motion(boma, Hash40::new(m), 0.0, rate, false, 0.0, false, false);
    }
}

unsafe fn portail(fighter: &mut L2CFighterCommon, on: bool) {
    let boma = fighter.module_accessor;
    let e = e_of(boma);
    if on {
        if PORTAIL[e] == 0 || !EffectModule::is_exist_effect(boma, PORTAIL[e]) {
            // portail de Subspace au-dessus de sa tête (REX : article 0, noir et violet)
            let h = EffectModule::req_follow(boma, Hash40::new("sys_final_aura"), Hash40::new("head"),
                &Vector3f { x: 0.0, y: 9.0, z: 0.0 }, &Vector3f { x: 0.0, y: 0.0, z: 0.0 }, 0.55, true, 0, 0, 0, 0, 0, false, false) as u32;
            if h != 0 {
                EffectModule::set_rgb(boma, h, 0.35, 0.0, 0.6);
            }
            PORTAIL[e] = h;
        }
    } else if PORTAIL[e] != 0 {
        EffectModule::kill(boma, PORTAIL[e], false, true);
        PORTAIL[e] = 0;
    }
}

unsafe extern "C" fn lw_rien(_fighter: &mut L2CFighterCommon) -> L2CValue {
    0.into()
}

unsafe extern "C" fn lw_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = e_of(boma);
    PH[e] = 0;
    T[e] = 0;
    // REX : SummonStart joué à vitesse 2
    motion(fighter, "special_lw_start", "special_air_lw_start", 2.0, false);
    if StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR {
        // en l'air : il descend doucement (REX : gravité réduite pendant l'invocation)
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_AIR_STOP);
    }
    fighter.sub_shift_status_main(L2CValue::Ptr(lw_loop as *const () as _))
}

unsafe fn finir(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND {
        fighter.change_status(FIGHTER_STATUS_KIND_WAIT.into(), false.into());
    } else {
        fighter.change_status(FIGHTER_STATUS_KIND_FALL.into(), false.into());
    }
}

unsafe extern "C" fn lw_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = e_of(boma);
    T[e] += 1;
    if StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR {
        // chute lente
        let vy = KineticModule::get_sum_speed_y(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN);
        if vy < -0.8 {
            KineticModule::add_speed(boma, &Vector3f { x: 0.0, y: (-0.8 - vy).min(0.2), z: 0.0 });
        }
    }
    match PH[e] {
        0 => {
            motion(fighter, "special_lw_start", "special_air_lw_start", 2.0, true);
            portail(fighter, MotionModule::frame(boma) > 20.0);
            if MotionModule::is_end(boma) {
                PH[e] = 1;
                T[e] = 0;
                MotionModule::change_motion(boma, Hash40::new(if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND { "special_lw_hold" } else { "special_air_lw_hold" }), 0.0, 1.0, false, 0.0, false, false);
            }
        }
        1 => {
            motion(fighter, "special_lw_hold", "special_air_lw_hold", 1.0, true);
            if MotionModule::is_end(boma) {
                MotionModule::set_frame(boma, 0.0, true);
            }
            portail(fighter, true);
            let lr = PostureModule::lr(boma);
            let sx = ControlModule::get_stick_x(boma) * lr;
            let sy = ControlModule::get_stick_y(boma);
            let mut go = 0;
            if sy > 0.75 && RECH[e][0] <= 0 {
                go = 1;
            } else if sx > 0.75 && RECH[e][1] <= 0 {
                go = 2;
            }
            if go > 0 {
                if go == 1 { mite_creer(boma, e); } else { botron_creer(boma, e); }
                PH[e] = 2;
                T[e] = 0;
                MotionModule::change_motion(boma, Hash40::new(if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND { "special_lw_light" } else { "special_air_lw_light" }), 0.0, 1.0, false, 0.0, false, false);
                PLAY_SE(fighter, Hash40::new("se_common_swing_09"));
            } else if !ControlModule::check_button_on(boma, *CONTROL_PAD_BUTTON_SPECIAL) || T[e] > ATTENTE_MAX {
                PH[e] = 3;
                MotionModule::change_motion(boma, Hash40::new(if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND { "special_lw_heavy" } else { "special_air_lw_heavy" }), 0.0, 1.0, false, 0.0, false, false);
            }
        }
        2 => {
            motion(fighter, "special_lw_light", "special_air_lw_light", 1.0, true);
            portail(fighter, MotionModule::frame(boma) < 20.0);
            if MotionModule::frame(boma) >= GO_LIBRE {
                // REX : il peut agir à partir de l'image 9 (l'invocation continue seule)
                if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND {
                    if fighter.sub_wait_ground_check_common(false.into()).get_bool() {
                        return 1.into();
                    }
                } else if fighter.sub_air_check_fall_common().get_bool() {
                    return 1.into();
                }
            }
            if MotionModule::is_end(boma) {
                finir(fighter);
                return 1.into();
            }
        }
        _ => {
            motion(fighter, "special_lw_heavy", "special_air_lw_heavy", 1.0, true);
            portail(fighter, false);
            if MotionModule::is_end(boma) {
                finir(fighter);
                return 1.into();
            }
        }
    }
    0.into()
}

unsafe extern "C" fn lw_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    portail(fighter, false);
    0.into()
}

/// chaque image (depuis lib.rs) : recharges
pub unsafe fn update(fighter: &mut L2CFighterCommon) {
    let e = e_of(fighter.module_accessor);
    for k in 0..2 {
        if RECH[e][k] > 0 {
            RECH[e][k] -= 1;
        }
    }
}

// ------------------------------------------------------------------ Mite (tuyau d'entrée de Mario)

unsafe fn mite_creer(boma: *mut BattleObjectModuleAccessor, e: usize) {
    if ArticleModule::is_exist(boma, *FIGHTER_MARIO_GENERATE_ARTICLE_DOKAN) {
        ArticleModule::remove_exist(boma, *FIGHTER_MARIO_GENERATE_ARTICLE_DOKAN, ArticleOperationTarget(*ARTICLE_OPE_TARGET_ALL));
    }
    let lr = PostureModule::lr(boma);
    let (x, y) = pos(boma);
    M[e] = [x + 10.0 * lr, y + 6.0, 0.0, lr, 0.0, 0.0];
    MITE_ATTEND[e] = true;
    MITE_ID[e] = 0;
    ArticleModule::generate_article(boma, *FIGHTER_MARIO_GENERATE_ARTICLE_DOKAN, false, -1);
    if let Some(w) = article_boma(boma, *FIGHTER_MARIO_GENERATE_ARTICLE_DOKAN) {
        if MITE_ATTEND[e] {
            // le statut n'a pas encore réclamé l'objet : il passe au statut DUMMY (remplacé ci-dessous)
            StatusModule::change_status_force(w, *WEAPON_MARIO_DOKAN_STATUS_KIND_DUMMY, false);
        }
    }
    RECH[e][0] = RECH_MITE;
    eff(boma, "sys_flash", M[e][0], M[e][1] + 4.0, 0.8);
}

unsafe fn article_boma(boma: *mut BattleObjectModuleAccessor, kind: i32) -> Option<*mut BattleObjectModuleAccessor> {
    if !ArticleModule::is_exist(boma, kind) {
        return None;
    }
    let a = ArticleModule::get_article(boma, kind);
    if a.is_null() {
        return None;
    }
    boma_of(smash::app::lua_bind::Article::get_battle_object_id(a) as u32)
}

/// ce tuyau est-il le Mite du bas B ? (réclamé à sa première image)
unsafe fn est_mite(w: *mut BattleObjectModuleAccessor, id: u32) -> Option<usize> {
    let (_, e) = owner(w)?;
    if MITE_ATTEND[e] {
        MITE_ATTEND[e] = false;
        MITE_ID[e] = id;
    }
    if MITE_ID[e] == id { Some(e) } else { None }
}

unsafe extern "C" fn dokan_main(weapon: &mut L2CWeaponCommon) -> L2CValue {
    let w = weapon.module_accessor;
    let Some(e) = est_mite(w, id_of(weapon)) else {
        return smashline::original_status(Main, weapon, *WEAPON_MARIO_DOKAN_STATUS_KIND_DUMMY)(weapon);
    };
    liberer(w, M[e][3]);
    placer(w, M[e][0], M[e][1]);
    MotionModule::change_motion(w, Hash40::new("entry"), 0.0, 0.0, false, 0.0, false, false);
    weapon.fastshift(L2CValue::Ptr(mite_loop as *const () as _))
}

unsafe extern "C" fn dokan_exec(weapon: &mut L2CWeaponCommon) -> L2CValue {
    let w = weapon.module_accessor;
    if est_mite(w, id_of(weapon)).is_none() {
        return smashline::original_status(Exec, weapon, *WEAPON_MARIO_DOKAN_STATUS_KIND_DUMMY)(weapon);
    }
    0.into()
}

unsafe fn mite_coup(weapon: &mut L2CWeaponCommon, dmg: f32, taille: f32) {
    ATTACK(weapon, 0, 0, Hash40::new("top"), dmg, 50, 70, 0, 45, taille, 0.0, 3.0, 0.0, None, None, None, 0.8, 1.0,
           *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, true, true, false, false, false,
           *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
           Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_OBJECT);
}

unsafe extern "C" fn mite_loop(weapon: &mut L2CWeaponCommon) -> L2CValue {
    let w = weapon.module_accessor;
    let Some((_, e)) = owner(w) else {
        tuer(weapon);
        return 0.into();
    };
    if MITE_ID[e] != id_of(weapon) {
        tuer(weapon);
        return 0.into();
    }
    let [mut x, mut y, mut vy, lr, phase, mut t] = M[e];
    t += 1.0;
    // touché (adversaire ou bouclier) ou trop vieux : il disparaît dans un nuage
    if t > 2.0 && (AttackModule::is_infliction(w, *COLLISION_KIND_MASK_HIT) || AttackModule::is_infliction(w, *COLLISION_KIND_MASK_SHIELD))
        || t > 300.0
    {
        eff(w, "sys_erace_smoke", x, y + 3.0, 0.8);
        MITE_ID[e] = 0;
        tuer(weapon);
        return 0.into();
    }
    let mut ph = phase;
    let f;
    if ph < 0.5 {
        // course (REX EnmMiteRun) vers l'avant, collé au sol
        x += 1.4 * lr;
        match sol(w, x, y, 14.0) {
            Some(g) => { y = g; vy = 0.0; }
            None => { vy = (vy - 0.15).max(-3.0); y += vy; }
        }
        f = t % MITE_COURSE;
        mite_coup(weapon, 4.0, 3.5);
        // adversaire tout près devant : il fonce (JumpTackle)
        if let Some((dx, dy)) = adversaire(e, x, y) {
            if dx * lr > 0.0 && dx * lr < 18.0 && dy.abs() < 14.0 && vy == 0.0 {
                ph = 1.0;
                vy = 1.9;
            }
        }
    } else {
        // plongeon : élan puis vol tendu, il retombe au sol et reprend sa course
        x += 1.9 * lr;
        vy = (vy - 0.14).max(-3.0);
        y += vy;
        f = if vy > 0.6 { MITE_ELAN } else { MITE_PLONGE };
        mite_coup(weapon, 6.0, 4.5);
        if vy < 0.0 {
            if let Some(g) = sol(w, x, y, 2.0) {
                y = g;
                vy = 0.0;
                ph = 0.0;
            }
        }
    }
    M[e] = [x, y, vy, lr, ph, t];
    placer(w, x, y);
    MotionModule::set_frame(w, f, true);
    // tombé du terrain
    if y < -300.0 {
        MITE_ID[e] = 0;
        tuer(weapon);
    }
    0.into()
}

// ------------------------------------------------------------------ Botron (J.E.T. de Mario) et son pot (eau du J.E.T.)

unsafe fn botron_creer(boma: *mut BattleObjectModuleAccessor, e: usize) {
    if ArticleModule::is_exist(boma, *FIGHTER_MARIO_GENERATE_ARTICLE_PUMP) {
        ArticleModule::remove_exist(boma, *FIGHTER_MARIO_GENERATE_ARTICLE_PUMP, ArticleOperationTarget(*ARTICLE_OPE_TARGET_ALL));
    }
    let lr = PostureModule::lr(boma);
    let (x, y) = pos(boma);
    B[e] = [x + 6.0 * lr, y + 24.0, lr, 0.0, 0.0, 0.0];
    P[e] = [y + 24.0, 0.0, 0.0, 0.0];
    BOT_ATTEND[e] = true;
    BOT_ID[e] = 0;
    ArticleModule::generate_article(boma, *FIGHTER_MARIO_GENERATE_ARTICLE_PUMP, false, -1);
    if let Some(w) = article_boma(boma, *FIGHTER_MARIO_GENERATE_ARTICLE_PUMP) {
        if BOT_ATTEND[e] {
            StatusModule::change_status_force(w, *WEAPON_MARIO_PUMP_STATUS_KIND_WAIT, false);
        }
    }
    RECH[e][1] = RECH_BOTRON;
    eff(boma, "sys_flash", B[e][0], B[e][1], 0.8);
}

unsafe fn est_botron(w: *mut BattleObjectModuleAccessor, id: u32) -> Option<usize> {
    let (_, e) = owner(w)?;
    if BOT_ATTEND[e] {
        BOT_ATTEND[e] = false;
        BOT_ID[e] = id;
    }
    if BOT_ID[e] == id { Some(e) } else { None }
}

unsafe extern "C" fn pump_main(weapon: &mut L2CWeaponCommon) -> L2CValue {
    let w = weapon.module_accessor;
    let Some(e) = est_botron(w, id_of(weapon)) else {
        return smashline::original_status(Main, weapon, *WEAPON_MARIO_PUMP_STATUS_KIND_WAIT)(weapon);
    };
    liberer(w, B[e][2]);
    placer(w, B[e][0], B[e][1]);
    MotionModule::change_motion(w, Hash40::new("start"), 0.0, 1.0, false, 0.0, false, false);
    EffectModule::kill_all(w, 0, true, false);
    // son pot (eau du J.E.T.) : créé par le J.E.T. lui-même
    POT_ATTEND[e] = true;
    POT_ID[e] = 0;
    ArticleModule::generate_article(w, *WEAPON_MARIO_PUMP_GENERATE_ARTICLE_WATER, false, -1);
    if let Some(p) = article_boma(w, *WEAPON_MARIO_PUMP_GENERATE_ARTICLE_WATER) {
        if POT_ATTEND[e] {
            StatusModule::change_status_force(p, *WEAPON_MARIO_PUMP_WATER_STATUS_KIND_REGULAR, false);
        }
    }
    weapon.fastshift(L2CValue::Ptr(botron_loop as *const () as _))
}

unsafe extern "C" fn pump_exec(weapon: &mut L2CWeaponCommon) -> L2CValue {
    let w = weapon.module_accessor;
    if est_botron(w, id_of(weapon)).is_none() {
        return smashline::original_status(Exec, weapon, *WEAPON_MARIO_PUMP_STATUS_KIND_WAIT)(weapon);
    }
    0.into()
}

unsafe extern "C" fn botron_loop(weapon: &mut L2CWeaponCommon) -> L2CValue {
    let w = weapon.module_accessor;
    let Some((_, e)) = owner(w) else {
        tuer(weapon);
        return 0.into();
    };
    if BOT_ID[e] != id_of(weapon) {
        tuer(weapon);
        return 0.into();
    }
    let [mut x, mut y, lr, mut t, mut lache, mut tl] = B[e];
    t += 1.0;
    let mk = MotionModule::motion_kind(w);
    if lache < 0.5 {
        // vol (REX : StartFloat puis FloatFly) vers l'avant, à hauteur constante au-dessus du sol
        x += 1.1 * lr;
        if let Some(g) = sol(w, x, y, 80.0) {
            let cible = g + 32.0;
            y += (cible - y).clamp(-0.6, 0.6);
        }
        if MotionModule::is_end(w) || (mk != smash::hash40("start") && mk != smash::hash40("wait")) {
            MotionModule::change_motion(w, Hash40::new("wait"), 0.0, 1.0, false, 0.0, false, false);
        }
        let au_dessus = adversaire(e, x, y).map_or(false, |(dx, dy)| dx.abs() < 7.0 && dy < 0.0 && dy > -70.0);
        if au_dessus || t > 150.0 {
            // il lâche son pot (REX : ShootStart + ShootFire)
            lache = 1.0;
            tl = 0.0;
            MotionModule::change_motion(w, Hash40::new("light"), 0.0, 1.0, false, 0.0, false, false);
        }
    } else {
        // après le lâcher : il remonte et s'en va (REX : ShootEnd)
        tl += 1.0;
        if mk == smash::hash40("light") && MotionModule::is_end(w) {
            MotionModule::change_motion(w, Hash40::new("end"), 0.0, 1.0, false, 0.0, false, false);
        }
        if tl > 30.0 {
            y += 1.2;
            x += 0.6 * lr;
        }
        if tl > 90.0 {
            eff(w, "sys_erace_smoke", x, y, 1.0);
            BOT_ID[e] = 0;
            B[e] = [x, y, lr, t, lache, tl];
            tuer(weapon);
            return 0.into();
        }
    }
    B[e] = [x, y, lr, t, lache, tl];
    placer(w, x, y);
    0.into()
}

unsafe fn est_pot(w: *mut BattleObjectModuleAccessor, id: u32) -> Option<usize> {
    // l'eau appartient au J.E.T. : son propriétaire final est Primid
    let mut o = boma_of(WorkModule::get_int(w, *WEAPON_INSTANCE_WORK_ID_INT_LINK_OWNER) as u32)?;
    if smash::app::utility::get_category(&mut *o) != *BATTLE_OBJECT_CATEGORY_FIGHTER {
        o = boma_of(WorkModule::get_int(o, *WEAPON_INSTANCE_WORK_ID_INT_LINK_OWNER) as u32)?;
    }
    let e = e_of(o);
    if POT_ATTEND[e] {
        POT_ATTEND[e] = false;
        POT_ID[e] = id;
    }
    if POT_ID[e] == id { Some(e) } else { None }
}

unsafe extern "C" fn water_main(weapon: &mut L2CWeaponCommon) -> L2CValue {
    let w = weapon.module_accessor;
    let Some(e) = est_pot(w, id_of(weapon)) else {
        return smashline::original_status(Main, weapon, *WEAPON_MARIO_PUMP_WATER_STATUS_KIND_REGULAR)(weapon);
    };
    liberer(w, B[e][2]);
    placer(w, B[e][0], B[e][1]);
    KineticModule::clear_speed_all(w);
    EffectModule::kill_all(w, 0, true, false);
    weapon.fastshift(L2CValue::Ptr(pot_loop as *const () as _))
}

unsafe extern "C" fn water_exec(weapon: &mut L2CWeaponCommon) -> L2CValue {
    let w = weapon.module_accessor;
    if est_pot(w, id_of(weapon)).is_none() {
        return smashline::original_status(Exec, weapon, *WEAPON_MARIO_PUMP_WATER_STATUS_KIND_REGULAR)(weapon);
    }
    0.into()
}

unsafe fn pot_coup(weapon: &mut L2CWeaponCommon, dmg: f32, taille: f32, kbg: i32, bkb: i32) {
    ATTACK(weapon, 0, 0, Hash40::new("top"), dmg, 70, kbg, 0, bkb, taille, 0.0, 0.0, 0.0, None, None, None, 1.0, 1.0,
           *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, true, true, false, false, false,
           *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
           Hash40::new("collision_attr_fire"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_BOMB, *ATTACK_REGION_BOMB);
}

unsafe extern "C" fn pot_loop(weapon: &mut L2CWeaponCommon) -> L2CValue {
    let w = weapon.module_accessor;
    let Some(e) = est_pot(w, id_of(weapon)) else {
        tuer(weapon);
        return 0.into();
    };
    let [bx, by, _lr, _t, lache, _tl] = B[e];
    let [mut y, mut vy, mut boum, _] = P[e];
    if boum > 0.0 {
        // explosion de feu (REX : le pot explose en flammes), quelques images de coup puis fin
        boum += 1.0;
        if boum > 6.0 {
            POT_ID[e] = 0;
            tuer(weapon);
            return 0.into();
        }
        P[e] = [y, vy, boum, 0.0];
        return 0.into();
    }
    if lache < 0.5 {
        // porté par Botron (même origine que lui : le pot est dessiné sous son corps)
        y = by;
        vy = 0.0;
        placer(w, bx, y);
        if BOT_ID[e] == 0 {
            POT_ID[e] = 0;
            tuer(weapon);
            return 0.into();
        }
    } else {
        // lâché : il tombe et frappe en tombant
        vy = (vy - 0.22).max(-4.5);
        y += vy;
        placer(w, PostureModule::pos_x(w), y);
        pot_coup(weapon, 6.0, 5.0, 60, 40);
        let touche = AttackModule::is_infliction(w, *COLLISION_KIND_MASK_HIT) || AttackModule::is_infliction(w, *COLLISION_KIND_MASK_SHIELD);
        let au_sol = sol(w, PostureModule::pos_x(w), y, (-vy).max(1.0)).is_some();
        if touche || au_sol || y < by - 150.0 {
            let x = PostureModule::pos_x(w);
            AttackModule::clear_all(w);
            pot_coup(weapon, 13.0, 13.0, 80, 55);
            eff(w, "sys_bomb_a", x, y, 1.2);
            eff(w, "sys_fireflower_shot", x, y + 2.0, 1.5);
            PLAY_SE(weapon, Hash40::new("se_common_bomb_l"));
            // le pot disparaît dans l'explosion
            ModelModule::set_visibility(w, false);
            boum = 1.0;
        }
    }
    P[e] = [y, vy, boum, 0.0];
    0.into()
}

unsafe extern "C" fn rien(_agent: &mut L2CAgentBase) {}

pub fn install(a: &mut Agent) {
    a.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_LW, lw_main);
    a.status(Init, *FIGHTER_STATUS_KIND_SPECIAL_LW, lw_rien);
    a.status(Exec, *FIGHTER_STATUS_KIND_SPECIAL_LW, lw_rien);
    a.status(End, *FIGHTER_STATUS_KIND_SPECIAL_LW, lw_end);
    // scripts du J.E.T. de Mario sur ses animations : retirés (effets d'eau, son, voix)
    for m in ["speciallwstart", "specialairlwstart", "speciallwhold", "specialairlwhold", "speciallwlight",
              "specialairlwlight", "speciallwheavy", "specialairlwheavy"] {
        for k in ["game", "effect", "sound", "expression"] {
            a.acmd(&format!("{}_{}", k, m), rien, Priority::Low);
        }
    }

    let mut d = Agent::new("mario_dokan");
    d.set_costume(PR_COSTUMES.to_vec());
    d.status(Main, *WEAPON_MARIO_DOKAN_STATUS_KIND_DUMMY, dokan_main);
    d.status(Exec, *WEAPON_MARIO_DOKAN_STATUS_KIND_DUMMY, dokan_exec);
    d.install();

    let mut p = Agent::new("mario_pump");
    p.set_costume(PR_COSTUMES.to_vec());
    p.status(Main, *WEAPON_MARIO_PUMP_STATUS_KIND_WAIT, pump_main);
    p.status(Exec, *WEAPON_MARIO_PUMP_STATUS_KIND_WAIT, pump_exec);
    for m in ["hold", "light", "heavy", "start", "end", "wait"] {
        for k in ["game", "effect", "sound"] {
            p.acmd(&format!("{}_{}", k, m), rien, Priority::Low);
        }
    }
    p.install();

    let mut wa = Agent::new("mario_pumpwater");
    wa.set_costume(PR_COSTUMES.to_vec());
    wa.status(Main, *WEAPON_MARIO_PUMP_WATER_STATUS_KIND_REGULAR, water_main);
    wa.status(Exec, *WEAPON_MARIO_PUMP_WATER_STATUS_KIND_REGULAR, water_exec);
    for k in ["game", "effect", "sound"] {
        wa.acmd(&format!("{}_regular", k), rien, Priority::Low);
    }
    wa.install();
}
