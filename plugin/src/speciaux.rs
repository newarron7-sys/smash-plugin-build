//! Super Shadow, spéciaux de REX.
//!
//! B neutre : le homing de Sonic, puis, s'il touche, le combo de REX (SpecialNHit2 poing, SpecialNHit3 pied,
//! SpecialNHit4 pied qui éjecte, SpecialNHit5 pose de fin), mis bout à bout dans l'animation special_n_hit de Sonic.
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

/// début de chaque partie du combo dans l'animation special_n_hit (REX : SpecialNHit2 26 images, SpecialNHit3 26,
/// SpecialNHit4 30, SpecialNHit5 44), et sa fin
pub const SEG: [f32; 5] = [0.0, 26.0, 52.0, 82.0, 126.0];
/// images d'animation après le coup avant de passer à la partie suivante (poing, pied, dernier pied -> pose)
const ENCHAINE: [f32; 3] = [2.0, 2.0, 8.0];

static mut ETAPE: [usize; 8] = [0; 8];
static mut TOUCHE: [bool; 8] = [false; 8];
static mut F_TOUCHE: [f32; 8] = [0.0; 8];

unsafe fn slot(boma: *mut BattleObjectModuleAccessor) -> usize {
    (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7)
}

/// le homing a touché : au lieu du rebond de Sonic, le combo de REX en l'air sur place. Ses 4 animations sont mises
/// bout à bout dans special_n_hit (une animation que le jeu charge déjà : de nouvelles animations restaient en T-pose)
unsafe extern "C" fn n_hit_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    StatusModule::set_situation_kind(boma, SituationKind(*SITUATION_KIND_AIR), true);
    GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
    KineticModule::clear_speed_all(boma);
    KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_MOTION_AIR);
    ETAPE[e] = 0;
    TOUCHE[e] = false;
    MotionModule::change_motion(boma, Hash40::new("special_n_hit"), 0.0, 1.0, false, 0.0, false, false);
    fighter.sub_shift_status_main(L2CValue::Ptr(n_hit_loop as *const () as _))
}

unsafe fn fin(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND {
        fighter.change_status(FIGHTER_STATUS_KIND_WAIT.into(), false.into());
    } else {
        fighter.change_status(FIGHTER_STATUS_KIND_FALL.into(), false.into());
    }
}

unsafe extern "C" fn n_hit_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    if AttackModule::is_infliction(boma, *COLLISION_KIND_MASK_HIT) && !TOUCHE[e] {
        TOUCHE[e] = true;
        F_TOUCHE[e] = MotionModule::frame(boma);
    }
    if StopModule::is_stop(boma) {
        return 0.into();
    }
    let n = ETAPE[e];
    let f = MotionModule::frame(boma);
    if n < 3 {
        if TOUCHE[e] && f >= F_TOUCHE[e] + ENCHAINE[n] {
            // touché : partie suivante tout de suite
            ETAPE[e] = n + 1;
            TOUCHE[e] = false;
            MotionModule::set_frame(boma, SEG[n + 1], true);
        } else if f >= SEG[n + 1] - 1.0 && !TOUCHE[e] {
            // raté : le combo s'arrête
            fin(fighter);
        }
        return 0.into();
    }
    if MotionModule::is_end(boma) || f >= SEG[4] - 1.0 {
        fin(fighter);
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
        ATTACK(agent, 0, 0, Hash40::new(os), 1.0, 366, 100, 40, 0, 5.0, 0.5, 1.5, 0.0, Some(0.5), Some(1.5), Some(0.0),
               1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_SPEED, false, 2, 0.0, 0, false, false, false, false,
               true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
               Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK,
               *ATTACK_REGION_BODY);
        ATK_SET_SHIELD_SETOFF_MUL(agent, 0, 0.5);
        AttackModule::set_captured_same_time_attack(boma, 0, true);
        AttackModule::set_attack_keep_rumble(boma, 0, true);
    }
}

// sons et effets du combo : à chaque coup (image 5 de chaque partie)
unsafe extern "C" fn g_combo(agent: &mut L2CAgentBase) { crate::run(agent, "game_specialnhit"); }
unsafe extern "C" fn s_combo(agent: &mut L2CAgentBase) {
    for (k, se) in ["se_sonic_swing_ll", "se_sonic_swing_s", "se_sonic_swing_m"].iter().enumerate() {
        frame(agent.lua_state_agent, SEG[k] + 3.0);
        if is_excute(agent) {
            PLAY_SE(agent, Hash40::new(se));
            PLAY_SEQUENCE(agent, Hash40::new("seq_sonic_rnd_attack"));
        }
    }
}
unsafe extern "C" fn e_combo(agent: &mut L2CAgentBase) {
    for (k, (os, sc)) in [("handl", 1.0), ("footr", 1.0), ("footl", 1.4)].iter().enumerate() {
        frame(agent.lua_state_agent, SEG[k] + 3.0);
        if is_excute(agent) {
            EFFECT_FOLLOW(agent, Hash40::new("sys_attack_impact"), Hash40::new(os), 0, 0, 0, 0, 0, 0, *sc, true);
            LAST_EFFECT_SET_COLOR(agent, 1.0, 0.78, 0.22);
            EFFECT_FOLLOW(agent, Hash40::new("sys_attack_speedline"), Hash40::new("top"), 0, 8, 2, 0, 0, 0, 0.8, true);
            LAST_EFFECT_SET_COLOR(agent, 1.0, 0.78, 0.22);
        }
    }
}
unsafe extern "C" fn rien(_agent: &mut L2CAgentBase) {}

pub fn install(a: &mut Agent) {
    a.acmd("game_specialnhoming", g_specialnhoming, Priority::Low);
    a.acmd("game_specialnhit", g_combo, Priority::Low);
    a.acmd("sound_specialnhit", s_combo, Priority::Low);
    a.acmd("effect_specialnhit", e_combo, Priority::Low);
    a.acmd("expression_specialnhit", rien, Priority::Low);
    a.status(Main, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HIT, n_hit_main);
    a.status(End, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HIT, n_hit_end);
}
