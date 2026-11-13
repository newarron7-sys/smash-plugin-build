//! Prises, coup pendant la prise et provocations de Master Chief (REX : sous-actions Catch, CatchDash, CatchTurn,
//! OutOfAmmoCatchAttack, AppealHi / AppealS / AppealLw de FitChief.pac).
//! - zones de prise : celles de REX (060A0800 : image, taille, position sur TopN), tailles × 0,82 comme ses coups ;
//! - coup pendant la prise : REX image 4, 3 %, angle 361, croissance 100, fixe 40, base 0 (sur la main : ThrowN) ;
//! - provocations : ses animations ; les scripts de Snake (boîte en carton) remplacés. Sons de REX (19316 / 19317) sur
//!   les étiquettes de la boîte (se_snake_cardboard_in / out).

use smash::app::lua_bind::*;
use smash::lib::lua_const::*;
use smash::lua2cpp::*;
use smash::phx::Hash40;
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

const ECHELLE: f32 = 0.82;

/// zones de prise de REX (z = devant), actives 2 images
unsafe fn prise(agent: &mut L2CAgentBase, f: f32, zs: &[(f32, f32)], y: f32) {
    let lua = agent.lua_state_agent;
    let boma = agent.module_accessor;
    frame(lua, (f - 1.0).max(1.0));
    if is_excute(agent) {
        GrabModule::set_rebound(boma, true);
    }
    frame(lua, f);
    if is_excute(agent) {
        for (k, (z, t)) in zs.iter().enumerate() {
            let sit = if k == 0 { *COLLISION_SITUATION_MASK_GA } else { *COLLISION_SITUATION_MASK_GA };
            CATCH(agent, k as i32, Hash40::new("top"), t * ECHELLE, 0.0, y, *z, None, None, None,
                  *FIGHTER_STATUS_KIND_CAPTURE_PULLED, sit);
        }
    }
    wait(lua, 2.0);
    if is_excute(agent) {
        grab!(agent, *MA_MSC_CMD_GRAB_CLEAR_ALL);
        WorkModule::on_flag(boma, *FIGHTER_STATUS_CATCH_FLAG_CATCH_WAIT);
        GrabModule::set_rebound(boma, false);
    }
}

unsafe extern "C" fn game_catch(agent: &mut L2CAgentBase) {
    prise(agent, 6.0, &[(15.31, 3.6), (9.0, 3.6)], 8.77);
}

unsafe extern "C" fn game_catchdash(agent: &mut L2CAgentBase) {
    prise(agent, 10.0, &[(11.0, 4.5), (6.5, 3.5), (2.0, 3.5)], 8.0);
}

unsafe extern "C" fn game_catchturn(agent: &mut L2CAgentBase) {
    prise(agent, 9.0, &[(-12.2, 2.93), (-7.0, 2.93), (-1.8, 2.93)], 7.09);
}

unsafe extern "C" fn sound_prise(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 5.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_common_swing_06"));
    }
}

unsafe extern "C" fn game_catchattack(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 4.0);
    if is_excute(agent) {
        AttackModule::set_damage_shake_scale(agent.module_accessor, 0.1);
        ATTACK(agent, 0, 0, Hash40::new("top"), 3.0, 361, 100, 40, 0, 4.1 * ECHELLE, 0.0, 12.0, 8.0, None, None, None, 2.1,
               1.0, *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, true, true, false, false, false,
               *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
               Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_PUNCH, *ATTACK_REGION_PUNCH);
        AttackModule::set_catch_only_all(agent.module_accessor, true, false);
    }
    wait(agent.lua_state_agent, 1.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
    }
}

unsafe extern "C" fn sound_catchattack(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 4.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_snake_catchattack"));
    }
}

unsafe extern "C" fn vide(_agent: &mut L2CAgentBase) {}

unsafe extern "C" fn expression_appeal(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        slope!(agent, *MA_MSC_CMD_SLOPE_SLOPE, *SLOPE_STATUS_LR);
        ItemModule::set_have_item_visibility(agent.module_accessor, false, 0);
    }
}

/// provocation haute (REX : son 19316 à l'image 7, 19317 à l'image 197)
unsafe extern "C" fn sound_appealhi(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 7.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_snake_cardboard_in"));
    }
    frame(agent.lua_state_agent, 197.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_snake_cardboard_out"));
    }
}

pub fn install(a: &mut Agent) {
    a.acmd("game_catch", game_catch, Priority::Low);
    a.acmd("game_catchdash", game_catchdash, Priority::Low);
    a.acmd("game_catchturn", game_catchturn, Priority::Low);
    for s in ["sound_catch", "sound_catchdash", "sound_catchturn"] {
        a.acmd(s, sound_prise, Priority::Low);
    }
    a.acmd("game_catchattack", game_catchattack, Priority::Low);
    a.acmd("sound_catchattack", sound_catchattack, Priority::Low);
    // provocations : plus de boîte en carton (créée par les scripts « expression » de Snake)
    for m in ["appealhir", "appealhil", "appealsr", "appealsl", "appeallwr", "appeallwl"] {
        let g: &'static str = Box::leak(format!("game_{}", m).into_boxed_str());
        let e: &'static str = Box::leak(format!("effect_{}", m).into_boxed_str());
        let s: &'static str = Box::leak(format!("sound_{}", m).into_boxed_str());
        let x: &'static str = Box::leak(format!("expression_{}", m).into_boxed_str());
        a.acmd(g, vide, Priority::Low);
        a.acmd(e, vide, Priority::Low);
        a.acmd(s, if m.starts_with("appealhi") { sound_appealhi } else { vide }, Priority::Low);
        a.acmd(x, expression_appeal, Priority::Low);
    }
}
