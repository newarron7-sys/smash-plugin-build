//! Metal Sonic : effets (étape 1b, même procédure que Super Sonic / Super Shadow).
//! - Plus aucun effet de Sonic sur ses déplacements ni sur ses coups (demandé : nuages bleus de la course de Sonic) :
//!   seulement la poussière normale du jeu.
//! - Course (demandé, vidéo de REX) : flamme violette qui sort de son dos (réacteur), tant qu'il court.
//! - Arrière aérien « Jet Thrust » (demandé) : feu qui sort de son dos à l'image 6 de REX (effet n° 73 sur son os
//!   BlasterM, le réacteur) ; la poussée vers l'avant est dans lib.rs (POUSSEES).
//! - Aérien neutre « Body Spark » (demandé : il gardait la boule de Sonic) : étincelles électriques sur le corps, et
//!   jamais la boule de Sonic sur ses coups (corps_normal, appelé à chaque image).

use smash::app::lua_bind::*;
use smash::app::sv_animcmd::frame;
use smash::app::*;
use smash::hash40;
use smash::lib::lua_const::*;
use smash::lua2cpp::*;
use smash::phx::{Hash40, Vector3f};
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

/// violet de la flamme de course (vidéo de REX)
const VIOLET: (f32, f32, f32) = (0.75, 0.25, 1.0);

unsafe extern "C" fn vide(_agent: &mut L2CAgentBase) {}

unsafe extern "C" fn dash(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        FOOT_EFFECT(agent, Hash40::new("sys_dash_smoke"), Hash40::new("top"), -3, 0, 0, 0, 0, 0, 0.63, 0, 0, 0, 0, 0, 0, false);
    }
}

unsafe extern "C" fn course(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    for f in [2.0, 10.0] {
        frame(lua, f);
        if is_excute(agent) {
            FOOT_EFFECT(agent, Hash40::new("sys_run_smoke"), Hash40::new("top"), -2, 0, 0, 0, 0, 0, 0.8, 0, 0, 0, 0, 0, 0, false);
        }
    }
}

unsafe extern "C" fn freinage(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        FOOT_EFFECT(agent, Hash40::new("sys_turn_smoke"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.8, 0, 0, 0, 0, 0, 0, false);
    }
}

unsafe extern "C" fn saut(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        LANDING_EFFECT(agent, Hash40::new("sys_jump_smoke"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, false);
    }
}

unsafe extern "C" fn atterrissage(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        LANDING_EFFECT(agent, Hash40::new("sys_landing_smoke_s"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.9, 0, 0, 0, 0, 0, 0, false);
    }
}

/// arrière aérien : feu du réacteur dans son dos (REX : image 6), dirigé vers l'arrière
unsafe extern "C" fn e_attackairb(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 6.0);
    if is_excute(agent) {
        EFFECT_FOLLOW(agent, Hash40::new("sys_flame"), Hash40::new("top"), 0, 7.0, -3.0, 90, 0, 0, 0.8, true);
        EFFECT_FOLLOW(agent, Hash40::new("sys_rocketbelt_jet"), Hash40::new("top"), 0, 7.0, -2.5, 90, 0, 0, 0.7, true);
    }
    frame(lua, 20.0);
    if is_excute(agent) {
        EFFECT_OFF_KIND(agent, Hash40::new("sys_rocketbelt_jet"), false, true);
    }
}

/// aérien neutre : étincelles électriques sur le corps pendant le coup (REX : coups électriques)
unsafe extern "C" fn e_attackairn(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    let os = ["bust", "hip", "head", "handl", "handr", "footl", "footr"];
    let mut f = 3.0;
    let mut i = 0;
    while f <= 33.0 {
        frame(lua, f);
        if is_excute(agent) {
            EFFECT_FOLLOW(agent, Hash40::new("sys_hit_elec"), Hash40::new(os[i % os.len()]), 0, 0, 0, 0, 0, 0, 0.35, true);
            if i % 2 == 0 {
                EFFECT_FOLLOW(agent, Hash40::new("sys_damage_elec"), Hash40::new("bust"), 0, 0, 0, 0, 0, 0, 0.9, true);
            }
        }
        f += 3.0;
        i += 1;
    }
    frame(lua, 36.0);
    if is_excute(agent) {
        EFFECT_OFF_KIND(agent, Hash40::new("sys_damage_elec"), false, true);
    }
}

pub fn install(a: &mut Agent) {
    a.acmd("effect_dash", dash, Priority::Low);
    a.acmd("effect_turndash", dash, Priority::Low);
    a.acmd("effect_run", course, Priority::Low);
    a.acmd("effect_runbrakel", freinage, Priority::Low);
    a.acmd("effect_runbraker", freinage, Priority::Low);
    a.acmd("effect_turnrun", freinage, Priority::Low);
    a.acmd("effect_turnrunbrake", freinage, Priority::Low);
    a.acmd("effect_jumpfront", saut, Priority::Low);
    a.acmd("effect_jumpback", saut, Priority::Low);
    a.acmd("effect_jumpfrontmini", saut, Priority::Low);
    a.acmd("effect_jumpbackmini", saut, Priority::Low);
    a.acmd("effect_jumpaerialfront", vide, Priority::Low);
    a.acmd("effect_jumpaerialback", vide, Priority::Low);
    // coups normaux : plus les effets de Sonic (traînées bleues, boule) ; ceux de REX à part
    for n in ["effect_attack11", "effect_attack12", "effect_attack13", "effect_attack100", "effect_attack100end",
              "effect_attackdash", "effect_attacks3", "effect_attacks3hi", "effect_attacks3lw", "effect_attackhi3",
              "effect_attacklw3", "effect_attacks4", "effect_attacks4hi", "effect_attacks4lw", "effect_attacks4charge",
              "effect_attackhi4", "effect_attackhi4charge", "effect_attacklw4", "effect_attacklw4charge",
              "effect_attackairf", "effect_attackairhi", "effect_attackairlw"] {
        a.acmd(n, vide, Priority::Low);
    }
    for n in ["effect_landingairn", "effect_landingairf", "effect_landingairb", "effect_landingairhi", "effect_landingairlw"] {
        a.acmd(n, atterrissage, Priority::Low);
    }
    a.acmd("effect_attackairn", e_attackairn, Priority::Low);
    a.acmd("effect_attackairb", e_attackairb, Priority::Low);
    // la boule de Sonic (expressions et sons de l'aérien neutre) : retirés
    a.acmd("expression_attackairn", vide, Priority::Low);
    a.acmd("sound_attackairn", vide, Priority::Low);
}

// ------------------------------------------------------------------ flamme violette de la course (réacteur du dos)
static mut FLAMME: [u32; 8] = [0; 8];

unsafe fn court(st: i32) -> bool {
    st == *FIGHTER_STATUS_KIND_DASH || st == *FIGHTER_STATUS_KIND_RUN || st == *FIGHTER_STATUS_KIND_TURN_RUN
        || st == *FIGHTER_STATUS_KIND_TURN_DASH
}

pub unsafe fn flamme_course(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let id = (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7);
    let h = FLAMME[id];
    if !court(StatusModule::status_kind(boma)) {
        if h != 0 && EffectModule::is_exist_effect(boma, h) {
            EffectModule::kill(boma, h, false, false);
        }
        FLAMME[id] = 0;
        return;
    }
    if h == 0 || !EffectModule::is_exist_effect(boma, h) {
        // derrière le dos, à hauteur du réacteur ; jet tourné vers l'arrière (rotations en radians)
        let pos = Vector3f { x: 0.0, y: 7.0, z: -2.5 };
        let rot = Vector3f { x: 1.5708, y: 0.0, z: 0.0 };
        let n = EffectModule::req_follow(boma, Hash40::new("sys_rocketbelt_jet"), Hash40::new("top"), &pos, &rot, 0.55,
            false, 0, 0, 0, 0, 0, false, false) as u32;
        if n != 0 {
            EffectModule::set_rgb(boma, n, VIOLET.0, VIOLET.1, VIOLET.2);
        }
        FLAMME[id] = n;
    }
}

/// jamais la boule de Sonic pendant ses coups (Metal Sonic n'en a pas dans REX)
pub unsafe fn corps_normal(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let st = StatusModule::status_kind(boma);
    let coups = [*FIGHTER_STATUS_KIND_ATTACK, *FIGHTER_STATUS_KIND_ATTACK_DASH, *FIGHTER_STATUS_KIND_ATTACK_S3,
        *FIGHTER_STATUS_KIND_ATTACK_HI3, *FIGHTER_STATUS_KIND_ATTACK_LW3, *FIGHTER_STATUS_KIND_ATTACK_S4_START,
        *FIGHTER_STATUS_KIND_ATTACK_S4_HOLD, *FIGHTER_STATUS_KIND_ATTACK_S4, *FIGHTER_STATUS_KIND_ATTACK_HI4_START,
        *FIGHTER_STATUS_KIND_ATTACK_HI4_HOLD, *FIGHTER_STATUS_KIND_ATTACK_HI4, *FIGHTER_STATUS_KIND_ATTACK_LW4_START,
        *FIGHTER_STATUS_KIND_ATTACK_LW4_HOLD, *FIGHTER_STATUS_KIND_ATTACK_LW4, *FIGHTER_STATUS_KIND_ATTACK_AIR,
        *FIGHTER_STATUS_KIND_LANDING_ATTACK_AIR];
    if coups.contains(&st) {
        VisibilityModule::set_int64(boma, hash40("body"), hash40("body_normal"));
    }
}
