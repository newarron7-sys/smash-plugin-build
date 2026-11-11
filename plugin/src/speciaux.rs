//! Super Sonic, spéciaux de REX (FitSuperSonic.pac + vidéo de REX du joueur, noms des coups affichés dedans).
//!
//! - B neutre « Homing Attack » : le homing de Sonic (boule de REX, 12 %, angle 72, croissance 120, recul 15 :
//!   SpecialNSpin de REX). À l'arrivée, REX (action 0x11C) joue AU HASARD une de ses 4 poses (SpecialNHit 1 à 4) en
//!   rebondissant ; actions permises à l'image 14. Pendant le homing, bas + B : « Bound Jump » (SpecialNBounce) : il
//!   plonge tout droit, rebondit sur le sol (élan 0,32 / 2,8 de REX) et reprend la main à l'image 13.
//! - side B « Dash Boost » : comme Super Shadow (même action 0x113 dans REX) : il fonce tant que B est tenu (Run au
//!   sol, AirBoost en l'air : coup de 4 % de REX), puis freine avec le coup de pied de fin (9 % puis 6 %).
//! - haut B « Flight » : vol libre de REX (UpBMove) : le stick le dirige (accélération 0,24 en hauteur jusqu'à 1,25,
//!   0,12 en largeur jusqu'à 1,5, freinage sans stick), pendant 45 images, puis il retombe normalement (une fois par
//!   saut). Coup léger de 1 % sur lui pendant le vol (REX).
//! - bas B « Spin Charge » : celui de Sonic (même mécanique dans REX), avec les animations de REX ; la boule qui
//!   roule fait 7 % (REX : action 0x11E).
//! Les animations de REX sont dans les fichiers de Sonic (rex/tools/retarget_exo.py, groupe supersonic_speciaux).

use smash::app::lua_bind::*;
use smash::app::sv_animcmd::frame;
use smash::app::*;
use smash::lib::lua_const::*;
use smash::lib::L2CValue;
use smash::lua2cpp::*;
use smash::phx::Hash40;
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

const OR: (f32, f32, f32) = (1.0, 0.78, 0.22);

unsafe fn slot(boma: *mut BattleObjectModuleAccessor) -> usize {
    (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7)
}

unsafe fn eclat(boma: *mut BattleObjectModuleAccessor, sc: f32) {
    let z = smash::phx::Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    let p = smash::phx::Vector3f { x: 0.0, y: 6.0, z: 0.0 };
    let h = EffectModule::req_follow(boma, Hash40::new("sys_smash_flash"), Hash40::new("top"), &p, &z, sc, true, 0, 0, 0, 0, 0, false, false);
    EffectModule::set_rgb(boma, h as u32, OR.0, OR.1, OR.2);
}

/// vitesse posée directement (énergie STOP, gravité coupée) : x dans sa direction
unsafe fn vitesse(fighter: &mut L2CFighterCommon, vx: f32, vy: f32) {
    let lr = PostureModule::lr(fighter.module_accessor);
    sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, vx * lr, vy);
}

unsafe fn en_l_air(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    StatusModule::set_situation_kind(boma, SituationKind(*SITUATION_KIND_AIR), true);
    GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
    KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_AIR_STOP);
    KineticModule::unable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
}

/// reprend la physique normale (chute) avec la vitesse verticale donnée
unsafe fn chute_normale(fighter: &mut L2CFighterCommon, vy: f32) {
    let boma = fighter.module_accessor;
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_FALL);
    sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY, vy);
}

unsafe fn finir(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND {
        fighter.change_status(FIGHTER_STATUS_KIND_LANDING.into(), false.into());
    } else {
        fighter.change_status(FIGHTER_STATUS_KIND_FALL.into(), false.into());
    }
}

// ------------------------------------------------------------------ B neutre (Homing Attack)
/// début de chaque partie dans special_n_hit : SpecialNHit, NHit2, NHit3, NHit4 (51 images chacune), SpecialNBounce
/// (50), fin
pub const SEG: [f32; 6] = [0.0, 51.0, 102.0, 153.0, 204.0, 254.0];
static mut NH_K: [usize; 8] = [0; 8];
static mut NH_T: [i32; 8] = [0; 8];
static mut NH_V: [(f32, f32); 8] = [(0.0, 0.0); 8];
static mut NH_LIBRE: [bool; 8] = [false; 8];
/// Bound Jump demandé pendant le homing (bas + B)
static mut BOUND: [bool; 8] = [false; 8];
/// 0 : plongée, 1 : rebond
static mut BOUND_PHASE: [u8; 8] = [0; 8];
static mut ALEA: u32 = 0x1234_5678;

fn hasard(n: u32) -> u32 {
    unsafe {
        ALEA ^= ALEA << 13;
        ALEA ^= ALEA >> 17;
        ALEA ^= ALEA << 5;
        ALEA % n
    }
}

unsafe extern "C" fn n_hit_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    en_l_air(fighter);
    AttackModule::clear_all(boma);
    // la boule du homing laisse la place au corps (REX : changement de modèle à l'arrivée)
    VisibilityModule::set_int64(boma, smash::hash40("body"), smash::hash40("body_normal"));
    NH_T[e] = 0;
    NH_LIBRE[e] = false;
    ALEA = ALEA.wrapping_add((MotionModule::frame(boma) * 977.0) as u32).wrapping_add(e as u32 * 7919);
    if BOUND[e] {
        // Bound Jump : il plonge tout droit (pose du début de SpecialNBounce)
        NH_K[e] = 4;
        BOUND_PHASE[e] = 0;
        MotionModule::change_motion(boma, Hash40::new("special_n_hit"), SEG[4], 0.0, false, 0.0, false, false);
        vitesse(fighter, 0.0, -6.0);
        eclat(boma, 1.0);
    } else {
        // REX (action 0x11C) : une des 4 poses au hasard ; il rebondit en arrière et vers le haut
        let k = hasard(4) as usize;
        NH_K[e] = k;
        MotionModule::change_motion(boma, Hash40::new("special_n_hit"), SEG[k], 1.0, false, 0.0, false, false);
        NH_V[e] = (-0.5, 2.4);
        vitesse(fighter, NH_V[e].0, NH_V[e].1);
    }
    fighter.sub_shift_status_main(L2CValue::Ptr(n_hit_loop as *const () as _))
}

unsafe extern "C" fn n_hit_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    if StopModule::is_stop(boma) {
        return 0.into();
    }
    let k = NH_K[e];
    let f = MotionModule::frame(boma) - SEG[k];
    if k == 4 && BOUND_PHASE[e] == 0 {
        // plongée jusqu'au sol, puis rebond (REX SpecialNBounce : élan 0,32 vers l'avant, 2,8 vers le haut)
        NH_T[e] += 1;
        vitesse(fighter, 0.0, -6.0);
        let sol = StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND
            || GroundModule::is_touch(boma, *GROUND_TOUCH_FLAG_DOWN as u32);
        if sol {
            BOUND_PHASE[e] = 1;
            NH_T[e] = 0;
            en_l_air(fighter);
            MotionModule::set_rate(boma, 1.0);
            NH_V[e] = (0.32, 2.8);
            vitesse(fighter, 0.32, 2.8);
            eclat(boma, 1.2);
            let z = smash::phx::Vector3f { x: 0.0, y: 0.0, z: 0.0 };
            EffectModule::req_follow(boma, Hash40::new("sys_down_smoke"), Hash40::new("top"), &z, &z, 1.0, true, 0, 0, 0, 0, 0, false, false);
        } else if NH_T[e] > 90 {
            fighter.change_status(FIGHTER_STATUS_KIND_FALL.into(), false.into());
        }
        return 0.into();
    }
    NH_T[e] += 1;
    // actions permises : image 14 des poses, image 13 du rebond (REX 64000000)
    let libre = if k == 4 { 13 } else { 14 };
    if !NH_LIBRE[e] {
        // rebond : la gravité ralentit l'élan (pas encore de contrôle)
        NH_V[e].1 = (NH_V[e].1 - 0.11).max(-2.0);
        NH_V[e].0 *= 0.95;
        vitesse(fighter, NH_V[e].0, NH_V[e].1);
        if NH_T[e] >= libre {
            NH_LIBRE[e] = true;
            chute_normale(fighter, NH_V[e].1);
        }
    } else {
        if fighter.sub_transition_group_check_air_cliff().get_bool() {
            return 1.into();
        }
        if fighter.sub_air_check_fall_common().get_bool() {
            return 1.into();
        }
    }
    if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND && NH_T[e] > 2 {
        fighter.change_status(FIGHTER_STATUS_KIND_LANDING.into(), false.into());
        return 1.into();
    }
    if f >= SEG[k + 1] - SEG[k] - 1.0 {
        finir(fighter);
    }
    0.into()
}

unsafe extern "C" fn n_hit_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    BOUND[e] = false;
    BOUND_PHASE[e] = 0;
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    VisibilityModule::set_int64(boma, smash::hash40("body"), smash::hash40("body_normal"));
    0.into()
}

/// chaque image (ss_frame) : Bound Jump (bas + B pendant le homing, en l'air)
pub unsafe fn frame_n(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let st = StatusModule::status_kind(boma);
    let homing = st == *FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HOMING || st == *FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HOMING_START;
    if homing
        && StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR
        && ControlModule::get_stick_y(boma) < -0.6
        && ControlModule::check_button_trigger(boma, *CONTROL_PAD_BUTTON_SPECIAL)
    {
        BOUND[slot(boma)] = true;
        fighter.change_status(FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HIT.into(), false.into());
    }
}

/// trajet du homing : coup de REX (SpecialNSpin : 12 %, angle 72, croissance 120, recul 15)
unsafe extern "C" fn g_specialnhoming(agent: &mut L2CAgentBase) {
    let boma = agent.module_accessor;
    if is_excute(agent) {
        JostleModule::set_status(boma, false);
        MotionModule::set_rate(boma, 80.0);
        let os = if WorkModule::is_flag(boma, *FIGHTER_SONIC_STATUS_SPECIAL_N_HOMING_FLAG_IS_KIRBY) { "rot" } else { "hip" };
        ATTACK(agent, 0, 0, Hash40::new(os), 12.0, 72, 120, 0, 15, 5.0, 0.0, 1.5, 0.0, None, None, None,
               1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_SPEED, false, 0, 0.0, 0, false, false, false, false,
               true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
               Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK,
               *ATTACK_REGION_BODY);
        AttackModule::set_captured_same_time_attack(boma, 0, true);
    }
}
unsafe extern "C" fn e_nhit(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        EFFECT_FOLLOW(agent, Hash40::new("sys_attack_impact"), Hash40::new("hip"), 0, 0, 0, 0, 0, 0, 1.2, true);
        LAST_EFFECT_SET_COLOR(agent, OR.0, OR.1, OR.2);
    }
}
unsafe extern "C" fn s_nhit(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 1.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_sonic_special_n02"));
    }
}
unsafe extern "C" fn rien(_agent: &mut L2CAgentBase) {}

// ------------------------------------------------------------------ side B (Dash Boost), comme Super Shadow
const S_L_SOL: f32 = 17.0; // Run
const S_L_AIR: f32 = 31.0; // AirBoost
const S_VITESSE: f32 = 3.5;
const S_COURSE_MAX: i32 = 90;
static mut S_PHASE: [u8; 8] = [0; 8];
static mut S_T: [i32; 8] = [0; 8];
static mut S_L: [f32; 8] = [S_L_SOL; 8];

/// pour l'effet du Boost (effets.rs) : il est en train de foncer
pub unsafe fn side_b_course(boma: *mut BattleObjectModuleAccessor) -> bool {
    StatusModule::status_kind(boma) == *FIGHTER_STATUS_KIND_SPECIAL_S && S_PHASE[slot(boma)] == 0
}

unsafe fn s_vitesse(fighter: &mut L2CFighterCommon, v: f32) {
    let lr = PostureModule::lr(fighter.module_accessor);
    sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, v * lr, 0.0_f32);
}

unsafe fn coup(agent: &mut L2CAgentBase, id: u64, os: &str, dmg: f32, ang: u64, kbg: i32, bkb: i32, size: f32, y: f32) {
    ATTACK(agent, id, 0, Hash40::new(os), dmg, ang, kbg, 0, bkb, size, 0.0, y, 0.0, None, None, None,
           1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true,
           *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
           Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_BODY);
}

unsafe extern "C" fn s_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    S_PHASE[e] = 0;
    S_T[e] = 0;
    VisibilityModule::set_whole(boma, true);
    VisibilityModule::set_int64(boma, smash::hash40("body"), smash::hash40("body_normal"));
    let sol = StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND;
    if sol {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
        GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND_CLIFF_STOP));
    } else {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_AIR_STOP);
        KineticModule::unable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    }
    // REX (Dash Boost) : au sol sa course (Run), en l'air son vol plané (AirBoost), suivis du freinage, rangés dans des
    // fichiers que le side B de Sonic n'utilise plus (special_s_dash_* servent à la boule du bas B)
    let mot = if sol { "special_s_hold" } else { "special_air_s_end_loop" };
    S_L[e] = if sol { S_L_SOL } else { S_L_AIR };
    MotionModule::change_motion(boma, Hash40::new(mot), 0.0, 1.0, false, 0.0, false, false);
    s_vitesse(fighter, S_VITESSE);
    fighter.sub_shift_status_main(L2CValue::Ptr(s_loop as *const () as _))
}

unsafe extern "C" fn s_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    if S_PHASE[e] == 0 {
        S_T[e] += 1;
        if ControlModule::get_stick_x(boma) * PostureModule::lr(boma) < -0.6 {
            PostureModule::reverse_lr(boma);
            PostureModule::update_rot_y_lr(boma);
        }
        s_vitesse(fighter, S_VITESSE);
        if MotionModule::frame(boma) >= S_L[e] - 1.0 {
            MotionModule::set_frame(boma, 0.0, false);
        }
        let tenu = ControlModule::check_button_on(boma, *CONTROL_PAD_BUTTON_SPECIAL);
        if (!tenu && S_T[e] > 15) || S_T[e] >= S_COURSE_MAX {
            S_PHASE[e] = 1;
            MotionModule::set_frame_sync_anim_cmd(boma, S_L[e], true, true, false);
        }
        return 0.into();
    }
    let fm = MotionModule::frame(boma);
    let f = (fm - S_L[e]).max(0.0);
    s_vitesse(fighter, (S_VITESSE * (1.0 - f / 20.0)).max(0.0));
    let fin = fm >= MotionModule::end_frame(boma) - 1.5 || fm < S_L[e] - 0.5;
    if fin {
        KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
        if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND {
            fighter.change_status(FIGHTER_STATUS_KIND_WAIT.into(), false.into());
        } else {
            fighter.change_status(FIGHTER_STATUS_KIND_FALL_SPECIAL.into(), false.into());
        }
    }
    0.into()
}

unsafe extern "C" fn s_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    S_PHASE[slot(boma)] = 0;
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    0.into()
}

/// course : coup de REX (AirBoost : 4 %, angle 361, recul fixe 46), puis coup de pied de fin (SpecialSEnd : 9 % puis 6 %)
unsafe fn g_course(agent: &mut L2CAgentBase, l: f32) {
    let lua = agent.lua_state_agent;
    if is_excute(agent) {
        JostleModule::set_status(agent.module_accessor, false);
        AttackModule::clear_all(agent.module_accessor);
        coup(agent, 0, "hip", 4.0, 361, 0, 46, 5.0, 1.5);
    }
    for (k, d, a, kbg, bkb) in [(2.0, 9.0, 40u64, 85, 65), (7.0, 6.0, 55u64, 100, 24)] {
        frame(lua, l + k);
        if is_excute(agent) {
            AttackModule::clear_all(agent.module_accessor);
            for (id, os, sz, x, y) in [(0u64, "legr", 2.5, 0.0, 0.0), (1u64, "kneer", 2.9, 0.0, 0.0), (2u64, "kneer", 3.5, 4.0, -2.0)] {
                ATTACK(agent, id, 0, Hash40::new(os), d, a, kbg, 0, bkb, sz, x, y, 0.0, None, None, None, 1.0, 1.0,
                       *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true,
                       *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
                       Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK,
                       *ATTACK_REGION_KICK);
            }
        }
    }
    frame(lua, l + 12.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
    }
}
unsafe extern "C" fn g_boost_sol(agent: &mut L2CAgentBase) { g_course(agent, S_L_SOL); }
unsafe extern "C" fn g_boost_air(agent: &mut L2CAgentBase) { g_course(agent, S_L_AIR); }
unsafe fn e_freinage(agent: &mut L2CAgentBase, l: f32) {
    frame(agent.lua_state_agent, l);
    if is_excute(agent) {
        EFFECT(agent, Hash40::new("sys_smash_flash"), Hash40::new("top"), 0, 9, 2, 0, 0, 0, 0.8, 0, 0, 0, 0, 0, 0, true);
        LAST_EFFECT_SET_COLOR(agent, OR.0, OR.1, OR.2);
        FOOT_EFFECT(agent, Hash40::new("sys_turn_smoke"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 1.0, 0, 0, 0, 0, 0, 0, false);
    }
}
unsafe extern "C" fn e_boost_sol(agent: &mut L2CAgentBase) { e_freinage(agent, S_L_SOL); }
unsafe extern "C" fn e_boost_air(agent: &mut L2CAgentBase) { e_freinage(agent, S_L_AIR); }
unsafe extern "C" fn s_boost(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_sonic_special_s01"));
    }
}

// ------------------------------------------------------------------ bas B (Spin Charge) : boule de REX qui roule
/// REX (action 0x11E) : 7 %, angle 55, croissance 46, recul 65 ; structure du script de Sonic gardée (id du statut)
unsafe extern "C" fn g_roule(agent: &mut L2CAgentBase) {
    let boma = agent.module_accessor;
    if is_excute(agent) {
        JostleModule::set_status(boma, false);
        AttackModule::clear_all(boma);
        let id = *FIGHTER_SONIC_STATUS_SPECIAL_S_DASH_ATTACK_ID_DEFAULT;
        coup(agent, id as u64, "hip", 7.0, 55, 46, 65, 3.5, 1.5);
        AttackModule::set_captured_same_time_attack(boma, id, true);
        AttackModule::set_attack_keep_rumble(boma, 0, true);
    }
}

// ------------------------------------------------------------------ haut B (Flight)
const VOL_DUREE: i32 = 45;
static mut VOL_T: [i32; 8] = [0; 8];
static mut VOL_V: [(f32, f32); 8] = [(0.0, 0.0); 8];
/// déjà volé depuis le dernier passage au sol
static mut VOL_FAIT: [bool; 8] = [false; 8];

unsafe extern "C" fn hi_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    if VOL_FAIT[e] && StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR {
        // une fois par saut : rien ne se passe
        fighter.change_status(FIGHTER_STATUS_KIND_FALL.into(), false.into());
        return 0.into();
    }
    VOL_FAIT[e] = true;
    VOL_T[e] = 0;
    // REX : il part avec un petit élan vers le haut
    VOL_V[e] = (0.0, 0.6);
    en_l_air(fighter);
    vitesse(fighter, VOL_V[e].0, VOL_V[e].1);
    eclat(boma, 1.3);
    MotionModule::change_motion(boma, Hash40::new("special_hi"), 0.0, 1.0, false, 0.0, false, false);
    fighter.sub_shift_status_main(L2CValue::Ptr(hi_loop as *const () as _))
}

unsafe extern "C" fn hi_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    if StopModule::is_stop(boma) {
        return 0.into();
    }
    VOL_T[e] += 1;
    let lr = PostureModule::lr(boma);
    let sx = ControlModule::get_stick_x(boma) * lr;
    let sy = ControlModule::get_stick_y(boma);
    let (mut vx, mut vy) = VOL_V[e];
    // REX (UpBMove) : accélération selon le stick, freinage sans stick
    if sy > 0.2 {
        if vy < 1.25 { vy += 0.24; }
    } else if sy < -0.2 {
        if vy > -1.25 { vy -= 0.24; }
    } else if vy > 0.0 {
        vy = (vy - 0.12).max(0.0);
    } else {
        vy = (vy + 0.12).min(0.0);
    }
    if sx > 0.2 {
        if vx < 1.5 { vx += 0.12; }
    } else if sx < -0.2 {
        if vx > -1.5 { vx -= 0.12; }
    } else if vx > 0.0 {
        vx = (vx - 0.04).max(0.0);
    } else {
        vx = (vx + 0.04).min(0.0);
    }
    VOL_V[e] = (vx, vy);
    vitesse(fighter, vx, vy);
    if MotionModule::frame(boma) >= MotionModule::end_frame(boma) - 1.0 {
        MotionModule::set_frame(boma, 0.0, false);
    }
    if fighter.sub_transition_group_check_air_cliff().get_bool() {
        return 1.into();
    }
    if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND && VOL_T[e] > 4 {
        fighter.change_status(FIGHTER_STATUS_KIND_LANDING.into(), false.into());
        return 1.into();
    }
    if VOL_T[e] >= VOL_DUREE {
        // fin du vol : il retombe normalement (vidéo REX)
        chute_normale(fighter, vy);
        fighter.change_status(FIGHTER_STATUS_KIND_FALL.into(), false.into());
    }
    0.into()
}

unsafe extern "C" fn hi_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    VOL_T[slot(boma)] = 0;
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    0.into()
}

/// chaque image : le vol redevient possible au sol, au bord ou après un coup reçu
pub unsafe fn frame_hi(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let st = StatusModule::status_kind(boma);
    if StatusModule::situation_kind(boma) != *SITUATION_KIND_AIR
        || st == *FIGHTER_STATUS_KIND_CLIFF_CATCH
        || st == *FIGHTER_STATUS_KIND_DAMAGE_FLY
        || st == *FIGHTER_STATUS_KIND_DEAD
    {
        VOL_FAIT[slot(boma)] = false;
    }
}

/// vol : coup léger de REX sur lui (1 %, recul 30), traînée dorée à chaque image, éclat au départ
unsafe extern "C" fn g_vol(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
        coup(agent, 0, "hip", 1.0, 361, 100, 30, 6.0, 0.0);
    }
}
unsafe extern "C" fn e_vol(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    let mut f = 0.0;
    while f < 17.0 {
        frame(lua, f);
        if is_excute(agent) {
            EFFECT(agent, Hash40::new("sys_smash_flash_s"), Hash40::new("hip"), 0, 0, 0, 0, 0, 0, 0.9, 0, 0, 0, 0, 0, 0, true);
            LAST_EFFECT_SET_COLOR(agent, OR.0, OR.1, OR.2);
        }
        f += 1.0;
    }
}
unsafe extern "C" fn s_vol(agent: &mut L2CAgentBase) {
    if is_excute(agent) && VOL_T[slot(agent.module_accessor)] < 3 {
        PLAY_SE(agent, Hash40::new("se_sonic_special_h01"));
    }
}

pub fn install(a: &mut Agent) {
    // B neutre
    a.acmd("game_specialnhoming", g_specialnhoming, Priority::Low);
    a.acmd("game_specialnhit", rien, Priority::Low);
    a.acmd("effect_specialnhit", e_nhit, Priority::Low);
    a.acmd("sound_specialnhit", s_nhit, Priority::Low);
    a.acmd("expression_specialnhit", rien, Priority::Low);
    a.status(Main, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HIT, n_hit_main);
    a.status(End, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HIT, n_hit_end);
    // side B
    a.acmd("game_specialshold", g_boost_sol, Priority::Low);
    a.acmd("game_specialairsendloop", g_boost_air, Priority::Low);
    a.acmd("effect_specialshold", e_boost_sol, Priority::Low);
    a.acmd("effect_specialairsendloop", e_boost_air, Priority::Low);
    a.acmd("sound_specialshold", s_boost, Priority::Low);
    a.acmd("sound_specialairsendloop", s_boost, Priority::Low);
    a.acmd("expression_specialshold", rien, Priority::Low);
    a.acmd("expression_specialairsendloop", rien, Priority::Low);
    a.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_S, s_main);
    a.status(End, *FIGHTER_STATUS_KIND_SPECIAL_S, s_end);
    // bas B : la boule qui roule
    a.acmd("game_specialsdashlw", g_roule, Priority::Low);
    a.acmd("game_specialsdashhi", g_roule, Priority::Low);
    // haut B
    a.acmd("game_specialhi", g_vol, Priority::Low);
    a.acmd("effect_specialhi", e_vol, Priority::Low);
    a.acmd("sound_specialhi", s_vol, Priority::Low);
    a.acmd("expression_specialhi", rien, Priority::Low);
    a.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_HI, hi_main);
    a.status(End, *FIGHTER_STATUS_KIND_SPECIAL_HI, hi_end);
}
