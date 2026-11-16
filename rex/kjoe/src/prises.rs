//! Knuckle Joe : prises et lancers de REX (FitKJoe.pac, gen_rex DEBUG_CATCH / PRISE du 16/11), à la place de ceux de
//! Ness (lancers PSI). Même méthode que Metal Sonic et Master Chief (prises.rs) :
//! - zones de prise de REX : prise image 5 (z 8,8 / 3,2), en courant image 7 (z 8 / 4,5 / 1), demi-tour image 5
//!   (z -12 / -7 / -2), hauteur 5,2 ; la prise en courant avance comme dans REX (table AVANCE catch_dash, lib.rs) ;
//! - coup pendant la prise : image 3, 1 % ;
//! - lancer avant : lâché à 10, 11 %, 42°, croissance 35, recul 70 ;
//! - lancer arrière : lâché à 20, 9 %, 154°, 20 / 75 ;
//! - lancer haut : coup de pied à 6 (3 %) puis lâché à 7 (5 %) : 3 + 5 = 8 % au lâcher (pas de vrai coup juste avant
//!   le lâcher : piège de Metal Sonic), 90°, 60 / 75 ;
//! - lancer bas : lâché à 25, 9 %, 70°, 66 / 65 ;
//! - prise cassée : 3 %, 361°, 100 / 60 (060E1100 id 1 de REX).

use smash::app::lua_bind::*;
use smash::app::sv_animcmd::{frame, wait};
use smash::lib::lua_const::*;
use smash::lua2cpp::*;
use smash::phx::Hash40;
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

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
            CATCH(agent, k as i32, Hash40::new("top"), *t, 0.0, y, *z, None, None, None,
                  *FIGHTER_STATUS_KIND_CAPTURE_PULLED, *COLLISION_SITUATION_MASK_GA);
        }
    }
    wait(lua, 2.0);
    if is_excute(agent) {
        grab!(agent, *MA_MSC_CMD_GRAB_CLEAR_ALL);
        WorkModule::on_flag(boma, *FIGHTER_STATUS_CATCH_FLAG_CATCH_WAIT);
        GrabModule::set_rebound(boma, false);
    }
}

unsafe extern "C" fn g_catch(agent: &mut L2CAgentBase) {
    prise(agent, 5.0, &[(8.8, 4.4), (3.2, 3.5)], 5.2);
}
unsafe extern "C" fn g_catchdash(agent: &mut L2CAgentBase) {
    prise(agent, 7.0, &[(8.0, 4.0), (4.5, 3.5), (1.0, 3.5)], 5.2);
}
unsafe extern "C" fn g_catchturn(agent: &mut L2CAgentBase) {
    prise(agent, 5.0, &[(-12.0, 4.0), (-7.0, 3.5), (-2.0, 3.5)], 5.2);
}

/// coup pendant la prise (REX : image 3, 1 %) : coup direct sur l'adversaire tenu
unsafe extern "C" fn g_catchattack(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 3.0);
    if is_excute(agent) {
        AttackModule::set_damage_shake_scale(agent.module_accessor, 0.1);
        ATTACK(agent, 0, 0, Hash40::new("top"), 1.0, 361, 100, 30, 0, 5.0, 0.0, 9.0, 9.0, None, None, None, 1.6,
               1.0, *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true,
               *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
               Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_PUNCH, *ATTACK_REGION_PUNCH);
        AttackModule::set_catch_only_all(agent.module_accessor, true, false);
    }
    wait(agent.lua_state_agent, 1.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
    }
}
unsafe extern "C" fn e_catchattack(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 3.0);
    if is_excute(agent) {
        EFFECT(agent, Hash40::new("sys_hit_normal_s"), Hash40::new("top"), 0, 9.0, 9.0, 0, 0, 0, 0.6, 0, 0, 0, 0, 0, 0, true);
    }
}

unsafe fn lacher(agent: &mut L2CAgentBase) {
    let boma = agent.module_accessor;
    let target = WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_OBJECT);
    let group = WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_HIT_GROUP);
    let no = WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_HIT_NO);
    ATK_HIT_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_THROW, Hash40::new("throw"), target, group, no);
}

unsafe fn lancer_abs(agent: &mut L2CAgentBase, dmg: f32, ang: u64, kbg: i32, bkb: i32) {
    ATTACK_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_THROW, 0, dmg, ang, kbg, 0, bkb, 0.0, 1.0, *ATTACK_LR_CHECK_F, 0.0, true,
               Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_NONE, *ATTACK_REGION_THROW);
    ATTACK_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_CATCH, 0, 3.0, 361, 100, 0, 60, 0.0, 1.0, *ATTACK_LR_CHECK_F, 0.0, true,
               Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_NONE, *ATTACK_REGION_THROW);
}

/// lancer : réglages au début, lâcher à l'image de REX
unsafe fn lancer(agent: &mut L2CAgentBase, f: f32, dmg: f32, ang: u64, kbg: i32, bkb: i32, cam: (i32, i32)) {
    if is_excute(agent) {
        lancer_abs(agent, dmg, ang, kbg, bkb);
    }
    frame(agent.lua_state_agent, f);
    if is_excute(agent) {
        CHECK_FINISH_CAMERA(agent, cam.0, cam.1);
        lacher(agent);
    }
}

unsafe extern "C" fn g_throwf(agent: &mut L2CAgentBase) {
    lancer(agent, 10.0, 11.0, 42, 35, 70, (9, 6));
}
unsafe extern "C" fn g_throwb(agent: &mut L2CAgentBase) {
    lancer(agent, 20.0, 9.0, 154, 20, 75, (-9, 6));
}
unsafe extern "C" fn g_throwhi(agent: &mut L2CAgentBase) {
    lancer(agent, 7.0, 8.0, 90, 60, 75, (1, 20));
}
unsafe extern "C" fn g_throwlw(agent: &mut L2CAgentBase) {
    lancer(agent, 25.0, 9.0, 70, 66, 65, (2, 0));
}

/// impact au lâcher (sur l'adversaire, os throw), poussière au sol pour le lancer bas
unsafe fn impact(agent: &mut L2CAgentBase, f: f32, sc: f32) {
    frame(agent.lua_state_agent, f);
    if is_excute(agent) {
        EFFECT(agent, Hash40::new("sys_attack_impact"), Hash40::new("throw"), 0, 0, 0, 0, 0, 0, sc, 0, 0, 0, 0, 0, 0, true);
        EFFECT(agent, Hash40::new("sys_hit_normal"), Hash40::new("throw"), 0, 0, 0, 0, 0, 0, 0.8, 0, 0, 0, 0, 0, 0, true);
    }
}
unsafe extern "C" fn e_throwf(agent: &mut L2CAgentBase) {
    impact(agent, 10.0, 1.2);
}
unsafe extern "C" fn e_throwb(agent: &mut L2CAgentBase) {
    impact(agent, 20.0, 1.1);
}
unsafe extern "C" fn e_throwhi(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 6.0);
    if is_excute(agent) {
        EFFECT(agent, Hash40::new("sys_attack_arc_d"), Hash40::new("top"), 0, 10, 2, 0, 0, 0, 0.9, 0, 0, 0, 0, 0, 0, true);
    }
    impact(agent, 7.0, 1.1);
}
unsafe extern "C" fn e_throwlw(agent: &mut L2CAgentBase) {
    impact(agent, 25.0, 1.2);
    if is_excute(agent) {
        LANDING_EFFECT(agent, Hash40::new("sys_down_smoke"), Hash40::new("top"), 0, 0, 8, 0, 0, 0, 0.9, 0, 0, 0, 0, 0, 0, false);
    }
}

unsafe extern "C" fn s_swing(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 4.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_common_punch_kick_swing_m"));
    }
}

unsafe extern "C" fn vide(_agent: &mut L2CAgentBase) {}

pub fn install(a: &mut Agent) {
    a.acmd("game_catch", g_catch, Priority::Low);
    a.acmd("game_catchdash", g_catchdash, Priority::Low);
    a.acmd("game_catchturn", g_catchturn, Priority::Low);
    a.acmd("game_catchattack", g_catchattack, Priority::Low);
    a.acmd("effect_catchattack", e_catchattack, Priority::Low);
    a.acmd("game_throwf", g_throwf, Priority::Low);
    a.acmd("game_throwb", g_throwb, Priority::Low);
    a.acmd("game_throwhi", g_throwhi, Priority::Low);
    a.acmd("game_throwlw", g_throwlw, Priority::Low);
    a.acmd("effect_throwf", e_throwf, Priority::Low);
    a.acmd("effect_throwb", e_throwb, Priority::Low);
    a.acmd("effect_throwhi", e_throwhi, Priority::Low);
    a.acmd("effect_throwlw", e_throwlw, Priority::Low);
    // sons PSI de Ness remplacés par un simple bruit de coup
    for m in ["throwf", "throwb", "throwhi", "throwlw"] {
        a.acmd(Box::leak(format!("sound_{}", m).into_boxed_str()), s_swing, Priority::Low);
    }
    // effets de Ness à la prise : rien
    for n in ["effect_catch", "effect_catchdash", "effect_catchturn"] {
        a.acmd(n, vide, Priority::Low);
    }
}
