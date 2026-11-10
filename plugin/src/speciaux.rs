//! Super Shadow, spéciaux de REX.
//!
//! B neutre : le homing de Sonic, puis, s'il touche, le combo de REX (SpecialNHit2 poing, SpecialNHit3 pied,
//! SpecialNHit4 pied qui éjecte, SpecialNHit5 pose de fin) sur de nouvelles animations ajoutées à Sonic
//! (special_n_hit2..5, déclarées dans fighter/sonic/motion/body/motion_patch.yaml, lu par runtime-motion-patcher).
//! Le coup suivant part dès que le précédent a touché ; s'il rate, le combo s'arrête à la fin de l'animation.

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

const ETAPES: [&str; 4] = ["special_n_hit2", "special_n_hit3", "special_n_hit4", "special_n_hit5"];
/// image (de l'animation) à partir de laquelle on enchaîne si le coup a touché
const ENCHAINE: [f32; 3] = [9.0, 9.0, 16.0];

static mut ETAPE: [usize; 8] = [0; 8];
static mut TOUCHE: [bool; 8] = [false; 8];

unsafe fn slot(boma: *mut BattleObjectModuleAccessor) -> usize {
    (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7)
}

unsafe fn etape(boma: *mut BattleObjectModuleAccessor, e: usize, n: usize) {
    ETAPE[e] = n;
    TOUCHE[e] = false;
    MotionModule::change_motion(boma, Hash40::new(ETAPES[n]), 0.0, 1.0, false, 0.0, false, false);
}

/// le homing a touché : au lieu du rebond de Sonic, le combo de REX, en l'air sur place (ses animations le déplacent)
unsafe extern "C" fn n_hit_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    StatusModule::set_situation_kind(boma, SituationKind(*SITUATION_KIND_AIR), true);
    GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
    KineticModule::clear_speed_all(boma);
    KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_MOTION_AIR);
    etape(boma, e, 0);
    fighter.sub_shift_status_main(L2CValue::Ptr(n_hit_loop as *const () as _))
}

unsafe extern "C" fn n_hit_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    if AttackModule::is_infliction_status(boma, *COLLISION_KIND_MASK_HIT) {
        TOUCHE[e] = true;
    }
    if StopModule::is_stop(boma) {
        return 0.into();
    }
    let n = ETAPE[e];
    let f = MotionModule::frame(boma);
    if n < 3 && TOUCHE[e] && f >= ENCHAINE[n] {
        etape(boma, e, n + 1);
        return 0.into();
    }
    if MotionModule::is_end(boma) {
        if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND {
            fighter.change_status(FIGHTER_STATUS_KIND_WAIT.into(), false.into());
        } else {
            fighter.change_status(FIGHTER_STATUS_KIND_FALL.into(), false.into());
        }
    }
    0.into()
}

unsafe extern "C" fn n_hit_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    let e = slot(fighter.module_accessor);
    ETAPE[e] = 0;
    TOUCHE[e] = false;
    smashline::original_status(End, fighter, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HIT)(fighter)
}

/// homing de Sonic (même script) mais avec un petit recul fixe : l'adversaire reste près de lui pour le combo
unsafe extern "C" fn g_specialnhoming(agent: &mut L2CAgentBase) {
    let boma = agent.module_accessor;
    if is_excute(agent) {
        JostleModule::set_status(boma, false);
        MotionModule::set_rate(boma, 80.0);
        let os = if WorkModule::is_flag(boma, *FIGHTER_SONIC_STATUS_SPECIAL_N_HOMING_FLAG_IS_KIRBY) { "rot" } else { "hip" };
        ATTACK(agent, 0, 0, Hash40::new(os), 1.0, 80, 100, 30, 0, 5.0, 0.5, 1.5, 0.0, Some(0.5), Some(1.5), Some(0.0),
               1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_SPEED, false, 2, 0.0, 0, false, false, false, false,
               true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
               Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK,
               *ATTACK_REGION_BODY);
        ATK_SET_SHIELD_SETOFF_MUL(agent, 0, 0.5);
        AttackModule::set_captured_same_time_attack(boma, 0, true);
        AttackModule::set_attack_keep_rumble(boma, 0, true);
    }
}

// sons du combo (REX : coup dans le vide + cri d'attaque au hasard, image 5)
unsafe fn son(agent: &mut L2CAgentBase, se: &str) {
    frame(agent.lua_state_agent, 5.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new(se));
        PLAY_SEQUENCE(agent, Hash40::new("seq_sonic_rnd_attack"));
    }
}
unsafe extern "C" fn s_hit2(agent: &mut L2CAgentBase) { son(agent, "se_sonic_swing_ll"); }
unsafe extern "C" fn s_hit3(agent: &mut L2CAgentBase) { son(agent, "se_sonic_swing_s"); }
unsafe extern "C" fn s_hit4(agent: &mut L2CAgentBase) { son(agent, "se_sonic_swing_m"); }
unsafe extern "C" fn rien(_agent: &mut L2CAgentBase) {}

// effets du combo : éclat doré au coup
unsafe fn eclat(agent: &mut L2CAgentBase, os: &str, sc: f32) {
    frame(agent.lua_state_agent, 5.0);
    if is_excute(agent) {
        EFFECT_FOLLOW(agent, Hash40::new("sys_attack_impact"), Hash40::new(os), 0, 0, 0, 0, 0, 0, sc, true);
        LAST_EFFECT_SET_COLOR(agent, 1.0, 0.78, 0.22);
        EFFECT_FOLLOW(agent, Hash40::new("sys_attack_speedline"), Hash40::new("top"), 0, 8, 2, 0, 0, 0, 0.8, true);
        LAST_EFFECT_SET_COLOR(agent, 1.0, 0.78, 0.22);
    }
}
unsafe extern "C" fn e_hit2(agent: &mut L2CAgentBase) { eclat(agent, "handl", 1.0); }
unsafe extern "C" fn e_hit3(agent: &mut L2CAgentBase) { eclat(agent, "footr", 1.0); }
unsafe extern "C" fn e_hit4(agent: &mut L2CAgentBase) { eclat(agent, "footl", 1.4); }

pub fn install(a: &mut Agent) {
    a.acmd("game_specialnhoming", g_specialnhoming, Priority::Low);
    a.acmd("sound_specialnhit2", s_hit2, Priority::Low);
    a.acmd("sound_specialnhit3", s_hit3, Priority::Low);
    a.acmd("sound_specialnhit4", s_hit4, Priority::Low);
    a.acmd("sound_specialnhit5", rien, Priority::Low);
    a.acmd("effect_specialnhit2", e_hit2, Priority::Low);
    a.acmd("effect_specialnhit3", e_hit3, Priority::Low);
    a.acmd("effect_specialnhit4", e_hit4, Priority::Low);
    a.acmd("effect_specialnhit5", rien, Priority::Low);
    a.acmd("expression_specialnhit2", rien, Priority::Low);
    a.acmd("expression_specialnhit3", rien, Priority::Low);
    a.acmd("expression_specialnhit4", rien, Priority::Low);
    a.acmd("expression_specialnhit5", rien, Priority::Low);
    a.acmd("game_specialnhit5", rien, Priority::Low);
    a.status(Main, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HIT, n_hit_main);
    a.status(End, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HIT, n_hit_end);
}
