//! Super Sonic, spéciaux de REX (FitSuperSonic.pac + vidéo de REX du joueur, noms des coups affichés dedans).
//!
//! - B neutre « Homing Attack » : le homing de Sonic (boule de REX, 12 %, angle 72, croissance 120, recul 15 :
//!   SpecialNSpin de REX). À l'arrivée, REX (action 0x11C) joue AU HASARD une de ses 4 poses (SpecialNHit 1 à 4) en
//!   rebondissant ; actions permises à l'image 14. Pendant le homing, bas + B : « Bound Jump » (SpecialNBounce) : il
//!   plonge tout droit, rebondit sur le sol (élan 0,32 / 2,8 de REX) et reprend la main à l'image 13.
//! - side B « Dash Boost » : comme Super Shadow (même action 0x113 dans REX) : il fonce tant que B est tenu (Run au
//!   sol, AirBoost en l'air : coup de 4 % de REX), puis freine avec le coup de pied de fin (9 % puis 6 %).
//! - haut B « Flight » (v12, d'après la vidéo de REX) : décollage vertical très rapide, puis vol libre de 2,5 s dirigé
//!   au stick (accélérations de REX), sur place sans stick ; il se pose en glissant ou retombe normalement à la fin
//!   (une fois par saut). Déplacement pur : pas de coup. Effets : le Boost doré du smash avant.
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
    // vidéo : la vitesse verticale de l'énergie STOP était bridée à 0 (il ne montait / descendait pas, restait en
    // l'air) : énergie libre, sans limite (même réglage que le haut B de Palutena dans HewDraw-Remix)
    sv_kinetic_energy!(reset_energy, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, *ENERGY_STOP_RESET_TYPE_FREE, 0.0, 0.0, 0.0, 0.0, 0.0);
    sv_kinetic_energy!(set_accel, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0, 0.0);
    sv_kinetic_energy!(set_stable_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0, 0.0);
    sv_kinetic_energy!(set_limit_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, -1.0, -1.0);
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_STOP);
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
        // demandé (vidéo) : il retombe pendant sa pose (avant : il restait en l'air jusqu'à la fin de l'animation) :
        // petit rebond puis la vraie gravité ; au sol, la pose s'arrête (atterrissage)
        chute_normale(fighter, 2.0);
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
        // vidéo REX : colonne de lumière blanche derrière lui pendant la plongée
        let mut p = smash::phx::Vector3f { x: 0.0, y: 0.0, z: 0.0 };
        ModelModule::joint_global_position(boma, Hash40::new("hip"), &mut p, true);
        let r = smash::phx::Vector3f { x: 0.0, y: 0.0, z: 0.0 };
        let h = EffectModule::req(boma, Hash40::new("sys_smash_flash_s"), &p, &r, 1.3, 0, 0, false, 0) as u32;
        if h != 0 {
            EffectModule::set_rgb(boma, h, 1.0, 1.0, 1.0);
        }
        let sol = StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND
            || GroundModule::is_touch(boma, *GROUND_TOUCH_FLAG_DOWN as u32);
        if sol {
            BOUND_PHASE[e] = 1;
            NH_T[e] = 0;
            StatusModule::set_situation_kind(boma, SituationKind(*SITUATION_KIND_AIR), true);
            GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
            MotionModule::set_rate(boma, 1.0);
            // rebond (REX : 2,8 vers le haut), puis la vraie gravité
            chute_normale(fighter, 2.8);
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
        if NH_T[e] >= libre {
            NH_LIBRE[e] = true;
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
    // demandé : le B neutre utilisable plusieurs fois de suite en l'air (Sonic : une seule fois, marqueur
    // SPECIAL_N_FALL posé par le homing) : marqueur retiré dès qu'il n'est plus dans son B neutre
    let b_neutre = st == *FIGHTER_STATUS_KIND_SPECIAL_N
        || st == *FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HOMING_START
        || st == *FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HOMING
        || st == *FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_REBOUND
        || st == *FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_FAIL;
    if !b_neutre {
        WorkModule::off_flag(boma, *FIGHTER_SONIC_INSTANCE_WORK_FLAG_SPECIAL_N_FALL);
    }
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
        // vidéo REX : éclat blanc en étoile à l'impact
        EFFECT(agent, Hash40::new("sys_attack_impact"), Hash40::new("hip"), 0, 0, 0, 0, 0, 0, 1.6, 0, 0, 0, 0, 0, 0, true);
        EFFECT(agent, Hash40::new("sys_smash_flash"), Hash40::new("hip"), 0, 0, 0, 0, 0, 0, 1.4, 0, 0, 0, 0, 0, 0, true);
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
        PLAY_SE(agent, Hash40::new("se_sonic_dash_start"));
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
// v12 (demandé, d'après la vidéo de REX du joueur, analysée image par image) :
// - décollage : il part d'un coup tout droit vers le haut, très vite (20 images : 3,0 par image puis ralentit),
//   debout, sans accroupi ;
// - vol libre : 150 images (2,5 s), dirigé au stick dans toutes les directions, sur place sans stick (pas de gravité) ;
//   accélérations de REX (UpBMove : 0,24 en hauteur, 0,12 en largeur), vitesses maximales un peu plus hautes ;
// - poses : debout (Fall de REX) quand il monte, descend ou plane ; allongé (UpBMove) quand il file sur le côté,
//   penché vers le bas quand il redescend en diagonale ;
// - fin : il se pose en glissant (poussière) s'il touche le sol, sinon il retombe normalement au bout du vol ;
// - c'est un déplacement (« MOVEMENT OPTION » dans la vidéo) : plus de coup de 1 % ;
// - effets : le « Boost doré » de son smash avant / side B (sys_score_aura dorée tournée dans la direction du vol,
//   traînée sonic_spintrace, traits de vitesse dorés), éclat doré au départ, petit anneau blanc de temps en temps.
const VOL_DECOL: i32 = 20;
const VOL_DUREE: i32 = VOL_DECOL + 150;
/// vitesses maximales du vol libre (REX : 1,25 / 1,5 ; vidéo : un peu plus vif)
const VOL_VY: f32 = 1.6;
const VOL_VX: f32 = 2.0;
/// special_hi : Fall de REX (debout, 21 images) puis UpBMove (couché, 17 images)
const VOL_SEG: [f32; 2] = [21.0, 38.0];
static mut VOL_T: [i32; 8] = [0; 8];
static mut VOL_V: [(f32, f32); 8] = [(0.0, 0.0); 8];
/// déjà volé depuis le dernier passage au sol
static mut VOL_FAIT: [bool; 8] = [false; 8];
/// effets du vol : aura (0), traînée (1) ; direction de l'aura (0 aucune, 1 haut, 2 côté, 3 bas)
static mut VOL_FX: [[u32; 2]; 8] = [[0; 2]; 8];
static mut VOL_DIR: [u8; 8] = [0; 8];

unsafe fn vol_fx_off(boma: *mut BattleObjectModuleAccessor, e: usize, k: usize) {
    let h = VOL_FX[e][k];
    if h != 0 && EffectModule::is_exist_effect(boma, h) {
        EffectModule::kill(boma, h, false, false);
    }
    VOL_FX[e][k] = 0;
}

unsafe fn vol_fx(boma: *mut BattleObjectModuleAccessor, eff: &str, p: (f32, f32, f32), r: (f32, f32, f32), sc: f32, or: bool) -> u32 {
    let p = smash::phx::Vector3f { x: p.0, y: p.1, z: p.2 };
    let r = smash::phx::Vector3f { x: r.0, y: r.1, z: r.2 };
    let n = EffectModule::req_follow(boma, Hash40::new(eff), Hash40::new("top"), &p, &r, sc, false, 0, 0, 0, 0, 0, false, false) as u32;
    if n != 0 && or {
        EffectModule::set_rgb(boma, n, OR.0, OR.1, OR.2);
    }
    n
}

/// effets du vol, à chaque image : aura tournée dans la direction, traînée, traits de vitesse, anneau blanc
unsafe fn vol_effets(boma: *mut BattleObjectModuleAccessor, e: usize, vx: f32, vy: f32) {
    let t = VOL_T[e];
    // direction dominante (rien quand il plane presque sur place)
    let dir: u8 = if vx.abs() < 0.5 && vy.abs() < 0.5 {
        0
    } else if vx.abs() >= vy.abs() {
        2
    } else if vy > 0.0 {
        1
    } else {
        3
    };
    if dir != VOL_DIR[e] || (dir != 0 && (VOL_FX[e][0] == 0 || !EffectModule::is_exist_effect(boma, VOL_FX[e][0]))) {
        vol_fx_off(boma, e, 0);
        VOL_DIR[e] = dir;
        // sys_score_aura : debout telle quelle (vers le haut), couchée devant lui à -90 (comme le smash avant)
        VOL_FX[e][0] = match dir {
            1 => vol_fx(boma, "sys_score_aura", (0.0, 9.0, 0.0), (0.0, 0.0, 0.0), 1.1, true),
            2 => vol_fx(boma, "sys_score_aura", (0.0, 6.0, 9.0), (-90.0, 0.0, 0.0), 1.0, true),
            3 => vol_fx(boma, "sys_score_aura", (0.0, 4.0, 0.0), (180.0, 0.0, 0.0), 1.0, true),
            _ => 0,
        };
    }
    // la traînée jaune du smash avant, pendant tout le vol
    if VOL_FX[e][1] == 0 || !EffectModule::is_exist_effect(boma, VOL_FX[e][1]) {
        let n = vol_fx(boma, "sonic_spintrace", (0.0, 7.0, 0.0), (0.0, 0.0, 0.0), 0.8, false);
        if n != 0 {
            EffectModule::set_alpha(boma, n, 0.7);
        }
        VOL_FX[e][1] = n;
    }
    // traits de vitesse dorés, dans le sens du mouvement (plus serrés pendant le décollage)
    let pas = if t <= VOL_DECOL { 3 } else { 6 };
    if dir != 0 && t % pas == 1 {
        match dir {
            1 => { vol_fx(boma, "sys_attack_speedline", (0.0, 2.0, 0.0), (-90.0, 0.0, 0.0), 1.3, true); }
            3 => { vol_fx(boma, "sys_attack_speedline", (0.0, 14.0, 0.0), (90.0, 0.0, 0.0), 1.2, true); }
            _ => { vol_fx(boma, "sys_attack_speedline", (0.0, 8.0, -4.0), (0.0, 0.0, 0.0), 1.2, true); }
        }
    }
    // vidéo : un petit anneau blanc autour de lui de temps en temps
    if t > VOL_DECOL && t % 40 == 0 {
        vol_fx(boma, "sys_smash_flash_s", (0.0, 7.0, 0.0), (0.0, 0.0, 0.0), 1.4, false);
    }
}

unsafe extern "C" fn hi_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    // v12b (crash au 2e haut B) : le ressort de Sonic (article GIMMICKJUMP) n'est jamais créé ; s'il en reste un, il
    // est retiré avant le vol
    ArticleModule::remove_exist(boma, *FIGHTER_SONIC_GENERATE_ARTICLE_GIMMICKJUMP, ArticleOperationTarget(*ARTICLE_OPE_TARGET_ALL));
    let sol = StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND;
    VOL_FAIT[e] = true;
    VOL_T[e] = 0;
    VOL_DIR[e] = 0;
    VOL_FX[e] = [0; 2];
    // vidéo : il partait en boule (statut de Sonic) : corps normal
    VisibilityModule::set_whole(boma, true);
    VisibilityModule::set_int64(boma, smash::hash40("body"), smash::hash40("body_normal"));
    // vidéo REX : il décolle d'un coup, tout droit, très vite
    VOL_V[e] = (0.0, 3.0);
    en_l_air(fighter);
    vitesse(fighter, VOL_V[e].0, VOL_V[e].1);
    eclat(boma, 1.6);
    if sol {
        vol_fx(boma, "sys_landing_smoke", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 1.2, false);
    }
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
    let t = VOL_T[e];
    let lr = PostureModule::lr(boma);
    let sx = ControlModule::get_stick_x(boma) * lr;
    let sy = ControlModule::get_stick_y(boma);
    let (mut vx, mut vy) = VOL_V[e];
    if t <= VOL_DECOL {
        // décollage : la montée ralentit de 3,0 à la vitesse du vol ; le stick ne fait que le déporter un peu
        vy = 3.0 - (3.0 - VOL_VY) * (t as f32 / VOL_DECOL as f32);
    } else if sy > 0.2 {
        // REX (UpBMove) : accélération selon le stick, freinage sans stick
        if vy < VOL_VY { vy += 0.24; }
    } else if sy < -0.2 {
        if vy > -VOL_VY { vy -= 0.24; }
    } else if vy > 0.0 {
        vy = (vy - 0.12).max(0.0);
    } else {
        vy = (vy + 0.12).min(0.0);
    }
    if sx > 0.2 {
        if vx < VOL_VX { vx += 0.12; }
    } else if sx < -0.2 {
        if vx > -VOL_VX { vx -= 0.12; }
    } else if vx > 0.0 {
        vx = (vx - 0.04).max(0.0);
    } else {
        vx = (vx + 0.04).min(0.0);
    }
    VOL_V[e] = (vx, vy);
    vitesse(fighter, vx, vy);
    // poses (vidéo REX) : debout quand il monte, descend ou plane ; allongé quand il file sur le côté, et penché vers
    // le bas quand il redescend en diagonale (l'os « rot » est incliné selon la pente du vol)
    let fm = MotionModule::frame(boma);
    let couche = t > VOL_DECOL && vx.abs() > 0.9 && sx.abs() > 0.3;
    let (a, b) = if couche { (VOL_SEG[0], VOL_SEG[1]) } else { (0.0, VOL_SEG[0]) };
    if fm < a || fm >= b - 1.0 {
        MotionModule::set_frame(boma, a, false);
    }
    if couche {
        let pente = (-vy).atan2(vx.abs()).to_degrees().clamp(-35.0, 50.0);
        let r = smash::phx::Vector3f { x: pente, y: 0.0, z: 0.0 };
        ModelModule::set_joint_rotate(boma, Hash40::new("rot"), &r,
            MotionNodeRotateCompose { _address: *MOTION_NODE_ROTATE_COMPOSE_AFTER as u8 },
            MotionNodeRotateOrder { _address: *MOTION_NODE_ROTATE_ORDER_XYZ as u8 });
    }
    vol_effets(boma, e, vx, vy);
    if fighter.sub_transition_group_check_air_cliff().get_bool() {
        return 1.into();
    }
    if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND && t > 4 {
        // vidéo : il se pose en glissant, avec de la poussière
        vol_fx(boma, "sys_sliding_smoke", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 1.0, false);
        vol_fx(boma, "sys_landing_smoke", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 1.0, false);
        fighter.change_status(FIGHTER_STATUS_KIND_LANDING.into(), false.into());
        return 1.into();
    }
    if t >= VOL_DUREE {
        // fin du vol : il retombe normalement (il peut encore agir)
        chute_normale(fighter, vy);
        fighter.change_status(FIGHTER_STATUS_KIND_FALL.into(), false.into());
    }
    0.into()
}

unsafe extern "C" fn hi_rien(_fighter: &mut L2CFighterCommon) -> L2CValue {
    0.into()
}

unsafe extern "C" fn hi_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    VOL_T[e] = 0;
    VOL_DIR[e] = 0;
    vol_fx_off(boma, e, 0);
    vol_fx_off(boma, e, 1);
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    0.into()
}

/// chaque image : le vol redevient possible au sol, au bord ou après un coup reçu
pub unsafe fn frame_hi(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let st = StatusModule::status_kind(boma);
    // v12b : une fois par saut : le haut B est bloqué avant d'entrer dans le statut (avant : le statut démarrait puis
    // repartait aussitôt en chute, ce qui faisait planter le jeu au 2e haut B)
    if VOL_FAIT[slot(boma)] && StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR && st != *FIGHTER_STATUS_KIND_SPECIAL_HI {
        WorkModule::unable_transition_term(boma, *FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_SPECIAL_HI);
    }
    if StatusModule::situation_kind(boma) != *SITUATION_KIND_AIR
        || st == *FIGHTER_STATUS_KIND_CLIFF_CATCH
        || st == *FIGHTER_STATUS_KIND_DAMAGE_FLY
        || st == *FIGHTER_STATUS_KIND_DEAD
    {
        VOL_FAIT[slot(boma)] = false;
    }
}

/// vol : plus de coup (déplacement pur dans la vidéo de REX) ; les effets sont posés par le statut
unsafe extern "C" fn g_vol(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
    }
}
unsafe extern "C" fn e_vol(_agent: &mut L2CAgentBase) {}
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
    // v12b : statuts de Sonic qui posent le ressort et le gèrent : remplacés par rien (le vol est tout dans hi_main)
    a.status(Init, *FIGHTER_STATUS_KIND_SPECIAL_HI, hi_rien);
    a.status(Exec, *FIGHTER_STATUS_KIND_SPECIAL_HI, hi_rien);
    a.status(End, *FIGHTER_STATUS_KIND_SPECIAL_HI, hi_end);
}
