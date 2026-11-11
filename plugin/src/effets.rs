//! Super Sonic : pas d'effets de Sonic sur ses déplacements ni sur ses coups (même procédure que Super Shadow) ;
//! seulement la poussière normale du jeu et son aura dorée de Super forme.

use smash::app::sv_animcmd::frame;
use smash::lua2cpp::*;
use smash::phx::Hash40;
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

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

const COUPS: [&str; 19] = ["effect_attack11", "effect_attack12", "effect_attack13", "effect_attackdash",
    "effect_attacks3", "effect_attacks3hi", "effect_attacks3lw", "effect_attackhi3", "effect_attacklw3",
    "effect_attacks4", "effect_attacks4hi", "effect_attacks4lw", "effect_attackhi4", "effect_attacklw4",
    "effect_attackairn", "effect_attackairf", "effect_attackairb", "effect_attackairhi", "effect_attackairlw"];

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
    // ses coups : les effets de Sonic (traînées bleues, boule) ne vont pas avec ses animations de REX -> vides ;
    // l'éclat doré de chaque coup est posé par run()
    for s in COUPS {
        a.acmd(s, vide, Priority::Low);
    }
}

// ------------------------------------------------------------------ aura dorée permanente (étape effets 1, demandé)
// Dans REX, son effet n° 26 (aura de Super forme) tourne en continu (attente, course, sauts…) : ici l'aura du Final
// Smash prêt du jeu, teintée or, accrochée aux hanches, plus de petites étincelles dorées sur le corps.

use smash::app::lua_bind::*;
use smash::app::*;
use smash::lib::lua_const::*;
use smash::phx::Vector3f;

const OR: (f32, f32, f32) = (1.0, 0.78, 0.22);
static mut AURA: [u32; 8] = [0; 8];
static mut T: [u32; 8] = [0; 8];

unsafe fn sans_aura(boma: *mut BattleObjectModuleAccessor) -> bool {
    let st = StatusModule::status_kind(boma);
    st == *FIGHTER_STATUS_KIND_DEAD || st == *FIGHTER_STATUS_KIND_STANDBY || st == *FIGHTER_STATUS_KIND_ENTRY
        || st == *FIGHTER_STATUS_KIND_REBIRTH
}

pub unsafe fn aura(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let id = (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7);
    let h = AURA[id];
    if sans_aura(boma) {
        if h != 0 && EffectModule::is_exist_effect(boma, h) {
            EffectModule::kill(boma, h, false, false);
        }
        AURA[id] = 0;
        return;
    }
    if h == 0 || !EffectModule::is_exist_effect(boma, h) {
        let z = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
        let n = EffectModule::req_follow(boma, Hash40::new("sys_final_aura"), Hash40::new("hip"), &z, &z, 0.9, false,
            0, 0, 0, 0, 0, false, false) as u32;
        if n != 0 {
            EffectModule::set_rgb(boma, n, OR.0, OR.1, OR.2);
        }
        AURA[id] = n;
    }
    // étincelles dorées, une toutes les 8 images, à tour de rôle sur le corps
    T[id] = T[id].wrapping_add(1);
    if T[id] % 8 == 0 {
        let os = ["head", "bust", "hip", "handl", "handr", "footl", "footr"];
        let b = os[((T[id] / 8) as usize) % os.len()];
        let z = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
        let n = EffectModule::req_follow(boma, Hash40::new("sys_smash_flash_s"), Hash40::new(b), &z, &z, 0.6, false,
            0, 0, 0, 0, 0, false, false) as u32;
        if n != 0 {
            EffectModule::set_rgb(boma, n, OR.0, OR.1, OR.2);
            EffectModule::set_alpha(boma, n, 0.7);
        }
    }
}

