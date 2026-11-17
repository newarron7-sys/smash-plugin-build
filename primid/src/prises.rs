//! Primid v0.1 : prises de REX sur Mario : chope (sol, course, demi-tour), coup pendant la prise et les 4 lancers.
//! Même méthode que Black Shadow : zones de chope, coups et projections lus dans le PSA de REX (SUBS, générés à la
//! compilation), posés aux images de ses animations de REX (converties sur le squelette de Mario).

use smash::app::lua_bind::*;
use smash::app::sv_animcmd::{frame, wait};
use smash::app::*;
use smash::lib::lua_const::*;
use smash::lua2cpp::*;
use smash::phx::Hash40;
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

use crate::{Ev, Sub, SUBS};

fn sub(name: &str) -> Option<&'static Sub> {
    SUBS.iter().find(|s| s.name == name)
}

unsafe fn atk(agent: &mut L2CAgentBase, e: &Ev) {
    ATTACK(agent, e.id, 0, Hash40::new(e.bone), e.dmg.max(0.0), e.angle, e.kbg, e.fkb, e.bkb, e.size, e.x, e.y, e.z,
           None, None, None, e.hitlag, e.sdi, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, e.trip, 0,
           false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL,
           *COLLISION_PART_MASK_ALL, false, Hash40::new(e.attr), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_PUNCH,
           *ATTACK_REGION_PUNCH);
}

/// réglages de projection : `kind` = THROW (id 0) ou CATCH (id 1)
unsafe fn throw_abs(agent: &mut L2CAgentBase, s: &Sub, id: u64, kind: i32) {
    // dans le PSA, le dernier réglage donné l'emporte (lancer bas de Primid : 89° puis 304°, vers le bas)
    if let Some(t) = s.throws.iter().rev().find(|t| t.id == id) {
        let mut attr = t.attr;
        let mut angle = t.angle;
        if s.name == "ThrowLw" && id == 0 {
            // v0.40 (le joueur : « fais en sorte que sa down throw enterre l'adversaire ») : projection qui enterre
            // (comme le lancer bas de King K. Rool : collision_attr_bury), vers le bas
            attr = "collision_attr_bury";
            angle = 270;
        }
        smash_script::macros::ATTACK_ABS(agent, kind, 0, t.dmg, angle, t.kbg, t.fkb, t.bkb, 0.0, 1.0, *ATTACK_LR_CHECK_F, 0.0, true,
                   Hash40::new(attr), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_PUNCH, *ATTACK_REGION_NONE);
    }
}

unsafe fn throw_now(agent: &mut L2CAgentBase) {
    let boma = agent.module_accessor;
    let target = WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_OBJECT);
    let group = WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_HIT_GROUP);
    let no = WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_HIT_NO);
    ATK_HIT_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_THROW, Hash40::new("throw"), target, group, no);
}

/// coup pendant une prise : une hitbox spéciale de REX ne touche que l'adversaire tenu (posée sur l'os « throw »)
unsafe fn hold_hit(agent: &mut L2CAgentBase, e: &Ev) {
    if e.special {
        let e2 = Ev { bone: "throw", x: 0.0, y: 0.0, z: 0.0, size: e.size.max(5.0), ..*e };
        atk(agent, &e2);
        AttackModule::set_catch_only_all(agent.module_accessor, true, false);
    } else {
        atk(agent, e);
    }
}

/// rapport durée REX / durée de l'animation jouée (1 si c'est bien son animation de REX)
unsafe fn rate(agent: &mut L2CAgentBase, s: &Sub) -> f32 {
    let end = MotionModule::end_frame(agent.module_accessor);
    if s.total > 1.0 && end > 1.0 && (end - s.total).abs() > 2.0 { (s.total / end).clamp(0.25, 4.0) } else { 1.0 }
}

// ------------------------------------------------------------------ chope (sol, course, demi-tour)

unsafe fn grab_ground(agent: &mut L2CAgentBase, name: &str) {
    let lua = agent.lua_state_agent;
    let boma = agent.module_accessor;
    let Some(s) = sub(name) else { return };
    let r = rate(agent, s);
    FT_MOTION_RATE(agent, r);
    let mut fs: Vec<f32> = s.catches.iter().map(|c| c.f).collect();
    fs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    fs.dedup();
    if fs.is_empty() {
        return;
    }
    let first = fs[0];
    // fin de la chope : quelques images après la dernière zone (comme Black Shadow)
    let end = fs[fs.len() - 1] + 3.0;
    if first > 1.0 {
        frame(lua, ((first - 1.0) / r).max(1.0));
        if is_excute(agent) {
            GrabModule::set_rebound(boma, true);
        }
    }
    for f in fs {
        frame(lua, (f / r).max(1.0));
        if is_excute(agent) {
            for c in s.catches.iter().filter(|c| (c.f - f).abs() < 0.01) {
                CATCH(agent, c.id, Hash40::new("top"), c.size, c.x, c.y, c.z, None, None, None,
                      *FIGHTER_STATUS_KIND_CAPTURE_PULLED,
                      if c.ground_only { *COLLISION_SITUATION_MASK_G } else { *COLLISION_SITUATION_MASK_GA });
            }
        }
    }
    frame(lua, end / r);
    if is_excute(agent) {
        grab!(agent, *MA_MSC_CMD_GRAB_CLEAR_ALL);
        WorkModule::on_flag(boma, *FIGHTER_STATUS_CATCH_FLAG_CATCH_WAIT);
        GrabModule::set_rebound(boma, false);
    }
}
unsafe extern "C" fn game_catch(agent: &mut L2CAgentBase) { grab_ground(agent, "Catch"); }
unsafe extern "C" fn game_catchdash(agent: &mut L2CAgentBase) { grab_ground(agent, "CatchDash"); }
unsafe extern "C" fn game_catchturn(agent: &mut L2CAgentBase) { grab_ground(agent, "CatchTurn"); }

// ------------------------------------------------------------------ coup pendant la prise (pummel)

unsafe extern "C" fn game_catchattack(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    let boma = agent.module_accessor;
    let Some(s) = sub("CatchAttack") else { return };
    let r = rate(agent, s);
    FT_MOTION_RATE(agent, r);
    let mut fs: Vec<f32> = s.evs.iter().map(|e| e.f).collect();
    fs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    fs.dedup();
    for f in fs {
        frame(lua, (f / r).max(1.0));
        if !is_excute(agent) {
            continue;
        }
        for e in s.evs.iter().filter(|e| (e.f - f).abs() < 0.01) {
            if e.clear {
                AttackModule::clear_all(boma);
            } else {
                // le coup pendant la prise ne vise que l'adversaire tenu, comme dans Ultimate
                hold_hit(agent, &Ev { special: true, ..*e });
            }
        }
    }
    wait(lua, 1.0);
    if is_excute(agent) {
        AttackModule::clear_all(boma);
    }
}

// ------------------------------------------------------------------ lancers

unsafe fn throw_move(agent: &mut L2CAgentBase, name: &str) {
    let lua = agent.lua_state_agent;
    let boma = agent.module_accessor;
    let Some(s) = sub(name) else { return };
    let r = rate(agent, s);
    FT_MOTION_RATE(agent, r);
    let rel = if s.release > 0.0 { s.release } else { 1.0 };
    if is_excute(agent) {
        throw_abs(agent, s, 0, *FIGHTER_ATTACK_ABSOLUTE_KIND_THROW);
        throw_abs(agent, s, 1, *FIGHTER_ATTACK_ABSOLUTE_KIND_CATCH);
    }
    // coups avant le lâcher
    let mut fs: Vec<f32> = s.evs.iter().map(|e| e.f).filter(|f| *f < rel - 0.01).collect();
    fs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    fs.dedup();
    for f in fs {
        frame(lua, (f / r).max(1.0));
        if !is_excute(agent) {
            continue;
        }
        for e in s.evs.iter().filter(|e| (e.f - f).abs() < 0.01) {
            if e.clear {
                AttackModule::clear_all(boma);
            } else {
                hold_hit(agent, e);
            }
        }
    }
    // le lâcher
    frame(lua, (rel / r).max(1.0));
    if is_excute(agent) {
        throw_now(agent);
    }
    // après le lâcher : coups et effacements
    let mut fs: Vec<f32> = s.evs.iter().map(|e| e.f).filter(|f| *f >= rel - 0.01).collect();
    fs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    fs.dedup();
    let mut cleared = false;
    for f in fs {
        if f > rel {
            frame(lua, f / r);
        }
        if !is_excute(agent) {
            continue;
        }
        for e in s.evs.iter().filter(|e| (e.f - f).abs() < 0.01) {
            if e.clear {
                AttackModule::clear_all(boma);
                cleared = true;
            } else {
                atk(agent, e);
            }
        }
    }
    if !cleared {
        wait(lua, 1.0);
        if is_excute(agent) {
            AttackModule::clear_all(boma);
        }
    }
}
unsafe extern "C" fn game_throwf(agent: &mut L2CAgentBase) { throw_move(agent, "ThrowF"); }
unsafe extern "C" fn game_throwb(agent: &mut L2CAgentBase) { throw_move(agent, "ThrowB"); }
unsafe extern "C" fn game_throwhi(agent: &mut L2CAgentBase) { throw_move(agent, "ThrowHi"); }
unsafe extern "C" fn game_throwlw(agent: &mut L2CAgentBase) { throw_move(agent, "ThrowLw"); }

pub unsafe fn update(_fighter: &mut L2CFighterCommon) {}

/// effets de Mario (placés pour ses animations à lui) : retirés sur celles de Primid
unsafe extern "C" fn rien(_agent: &mut L2CAgentBase) {}

pub fn install(a: &mut Agent) {
    if SUBS.is_empty() {
        return;
    }
    let list: [(&str, unsafe extern "C" fn(&mut L2CAgentBase), &str); 8] = [
        ("game_catch", game_catch, "Catch"),
        ("game_catchdash", game_catchdash, "CatchDash"),
        ("game_catchturn", game_catchturn, "CatchTurn"),
        ("game_catchattack", game_catchattack, "CatchAttack"),
        ("game_throwf", game_throwf, "ThrowF"),
        ("game_throwb", game_throwb, "ThrowB"),
        ("game_throwhi", game_throwhi, "ThrowHi"),
        ("game_throwlw", game_throwlw, "ThrowLw"),
    ];
    for (script, f, name) in list {
        if sub(name).is_some() {
            a.acmd(script, f, Priority::Low);
        }
    }
    for s in ["effect_catchattack", "effect_throwf", "effect_throwb", "effect_throwhi", "effect_throwlw"] {
        a.acmd(s, rien, Priority::Low);
    }
}
