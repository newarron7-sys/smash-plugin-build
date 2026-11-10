// opff import
utils::import_noreturn!(common::opff::fighter_common_opff);
use super::*;
use globals::*;

// Jump during Spin Turn
unsafe fn sonic_spindash_jump_waveland(boma: &mut BattleObjectModuleAccessor, status_kind: i32, situation_kind: i32, cat1: i32){
if status_kind == *FIGHTER_SONIC_STATUS_KIND_SPECIAL_S_TURN && boma.is_input_jump() {
    StatusModule::change_status_request_from_script(boma, *FIGHTER_SONIC_STATUS_KIND_SPIN_JUMP, true);
  }
}

// upB freefalls after one use per airtime
unsafe fn up_special_freefall(fighter: &mut L2CFighterCommon) {
    if StatusModule::is_changing(fighter.module_accessor)
    && (fighter.is_situation(*SITUATION_KIND_GROUND)
        || fighter.is_situation(*SITUATION_KIND_CLIFF)
        || fighter.is_status_one_of(&[*FIGHTER_STATUS_KIND_REBIRTH, *FIGHTER_STATUS_KIND_DEAD, *FIGHTER_STATUS_KIND_LANDING, *FIGHTER_STATUS_KIND_GIMMICK_SPRING_JUMP]))
    {
        VarModule::off_flag(fighter.battle_object, vars::sonic::instance::SPECIAL_HI_ENABLE_FREEFALL);
    }
    if fighter.is_prev_status(*FIGHTER_SONIC_STATUS_KIND_SPECIAL_HI_JUMP) {
        if StatusModule::is_changing(fighter.module_accessor) {
            VarModule::on_flag(fighter.battle_object, vars::sonic::instance::SPECIAL_HI_ENABLE_FREEFALL);
        }
    }
    if fighter.is_status(*FIGHTER_SONIC_STATUS_KIND_SPECIAL_HI_JUMP) {
        if fighter.is_situation(*SITUATION_KIND_AIR)
        && VarModule::is_flag(fighter.battle_object, vars::sonic::instance::SPECIAL_HI_ENABLE_FREEFALL) {
            if CancelModule::is_enable_cancel(fighter.module_accessor) {
                let cancel_module = *(fighter.module_accessor as *mut BattleObjectModuleAccessor as *mut u64).add(0x128 / 8) as *const u64;
                *(((cancel_module as u64) + 0x1c) as *mut bool) = false;  // CancelModule::is_enable_cancel = false
            }
            if MotionModule::is_end(fighter.module_accessor) {
                let accel_x_mul = ParamModule::get_float(fighter.battle_object, ParamType::Agent, "param_special_hi.fall_special_accel_x_mul");
                let speed_x_max_mul = ParamModule::get_float(fighter.battle_object, ParamType::Agent, "param_special_hi.fall_special_speed_x_max_mul");
                WorkModule::set_float(fighter.module_accessor, accel_x_mul, *FIGHTER_INSTANCE_WORK_ID_FLOAT_MUL_FALL_X_ACCEL);
                WorkModule::set_float(fighter.module_accessor, speed_x_max_mul, *FIGHTER_INSTANCE_WORK_ID_FLOAT_FALL_X_MAX_MUL);
                fighter.change_status_req(*FIGHTER_STATUS_KIND_FALL_SPECIAL, true);
            }
        }
    }
}

unsafe fn fastfall_specials(fighter: &mut L2CFighterCommon) {
    if !fighter.is_in_hitlag()
    && !StatusModule::is_changing(fighter.module_accessor)
    && fighter.is_status_one_of(&[
        *FIGHTER_STATUS_KIND_SPECIAL_S,
        *FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HIT,
        *FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_REBOUND,
        *FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_FAIL,
        *FIGHTER_SONIC_STATUS_KIND_SPECIAL_S_DASH,
        *FIGHTER_SONIC_STATUS_KIND_SPECIAL_HI_JUMP,
        ]) 
    && fighter.is_situation(*SITUATION_KIND_AIR) {
        fighter.sub_air_check_dive();
    }
}

pub unsafe fn moveset(fighter: &mut smash::lua2cpp::L2CFighterCommon, boma: &mut BattleObjectModuleAccessor, id: usize, cat: [i32 ; 4], status_kind: i32, situation_kind: i32, motion_kind: u64, stick_x: f32, stick_y: f32, facing: f32, frame: f32) {
    sonic_spindash_jump_waveland(boma, status_kind, situation_kind, cat[0]);
    //sonic_moveset(boma, situation_kind, status_kind, motion_kind, frame, cat[0], id);
    //sonic_lightspeed_dash(boma, status_kind, motion_kind, situation_kind, cat[0], id);
    up_special_freefall(fighter);
    fastfall_specials(fighter);
    
}

pub extern "C" fn sonic_frame_wrapper(fighter: &mut smash::lua2cpp::L2CFighterCommon) {
    unsafe {
        common::opff::fighter_common_opff(fighter);
		sonic_frame(fighter)
    }
}

pub unsafe fn sonic_frame(fighter: &mut smash::lua2cpp::L2CFighterCommon) {
    if let Some(info) = FrameInfo::update_and_get(fighter) {
        moveset(fighter, &mut *info.boma, info.id, info.cat, info.status_kind, info.situation_kind, info.motion_kind.hash, info.stick_x, info.stick_y, info.facing, info.frame);
    }
}

pub fn install(agent: &mut Agent) {
    agent.on_line(Main, sonic_frame_wrapper);
}
#![deny(deprecated)]
#![allow(unused)]
#![allow(non_snake_case)]

pub mod acmd;

pub mod opff;
pub mod status;

// articles

mod gimmickjump;

use smash::{
    lib::{
        L2CValue,
        LuaConst,
    },
    app::{
        *,
        self,
        sv_animcmd::{
            frame,
            wait
        },
        lua_bind::*
    },
    hash40,
    lib::lua_const::*,
    lua2cpp::*,
    phx::*
};
use smash_script::{
    *,
    macros::*
};
use smash_script::macros::ATTACK_ABS;
use utils::{
    *,
    util::*,
    ext::*,
    consts::*,
};
use smashline::*;
#[macro_use] extern crate smash_script;

pub fn install() {
    let agent = &mut Agent::new("sonic");
    acmd::install(agent);
    opff::install(agent);
    status::install(agent);
    agent.install();

    gimmickjump::install();
}
use super::*;

// FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HIT

// Randomized Homing Attack Poses

pub unsafe extern "C" fn special_n_hit_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    
    let ret = smashline::original_status(Main, fighter, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HIT)(fighter);
    
    let hit_pose: [&str; 8] = [
        "special_n_hit1",
        "special_n_hit2",
        "special_n_hit3",
        "special_n_hit4",
        "special_n_hit5",
        "special_n_hit6",
        "special_n_hit7",
        "special_n_hit8"
    ];

    let pose_number = VarModule::get_int(fighter.battle_object, vars::sonic::instance::SPECIAL_N_POSE);

    if pose_number != 0 {
        let pose_hash = Hash40::new(hit_pose[(pose_number - 1) as usize]);
        MotionModule::change_motion(fighter.module_accessor, pose_hash, 0.0, 1.0, false, 0.0, false, false);
    }

    if pose_number != 8 {
      VarModule::inc_int(fighter.battle_object, vars::sonic::instance::SPECIAL_N_POSE);
    } else {
      VarModule::set_int(fighter.battle_object, vars::sonic::instance::SPECIAL_N_POSE, 0);
    }
    VarModule::off_flag(fighter.battle_object, vars::sonic::instance::SPECIAL_AIR_ACTION_USED);

    ret
}

pub fn install(agent: &mut Agent) {
    agent.status(Main, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HIT, special_n_hit_main);
}
use super::*;

// FIGHTER_SONIC_STATUS_KIND_SPECIAL_LW_HOLD

unsafe extern "C" fn special_lw_hold_pre(fighter: &mut L2CFighterCommon) -> L2CValue {
    let stick_x = fighter.global_table[STICK_X].get_f32();
    let lr = PostureModule::lr(fighter.module_accessor);
    if stick_x * lr <= -0.35 {
        PostureModule::reverse_lr(fighter.module_accessor);
        PostureModule::update_rot_y_lr(fighter.module_accessor);
    }
    StatusModule::init_settings(
        fighter.module_accessor,
        SituationKind(*SITUATION_KIND_NONE),
        *FIGHTER_KINETIC_TYPE_UNIQ,
        *GROUND_CORRECT_KIND_KEEP as u32,
        GroundCliffCheckKind(*GROUND_CLIFF_CHECK_KIND_NONE),
        true,
        *FIGHTER_STATUS_WORK_KEEP_FLAG_NONE_FLAG,
        *FIGHTER_STATUS_WORK_KEEP_FLAG_NONE_INT,
        *FIGHTER_STATUS_WORK_KEEP_FLAG_NONE_FLOAT,
        0
    );
    FighterStatusModuleImpl::set_fighter_status_data(
        fighter.module_accessor,
        false,
        *FIGHTER_TREADED_KIND_NO_REAC,
        false,
        false,
        false,
        (*FIGHTER_LOG_MASK_FLAG_ATTACK_KIND_SPECIAL_LW | *FIGHTER_LOG_MASK_FLAG_ACTION_CATEGORY_ATTACK |
        *FIGHTER_LOG_MASK_FLAG_ACTION_TRIGGER_ON) as u64,
        0,
        *FIGHTER_POWER_UP_ATTACK_BIT_SPECIAL_LW as u32,
        0
    );
    0.into()
}

pub fn install(agent: &mut Agent) {
    agent.status(Pre, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_LW_HOLD, special_lw_hold_pre);
}
use super::*;

// FIGHTER_STATUS_KIND_SPECIAL_HI

pub unsafe extern "C" fn special_hi_pre(fighter: &mut L2CFighterCommon) -> L2CValue {
	StatusModule::init_settings(
        fighter.module_accessor,
        SituationKind(*SITUATION_KIND_NONE),
        *FIGHTER_KINETIC_TYPE_SONIC_SPECIAL_HI_COMP,
        *GROUND_CORRECT_KIND_NONE as u32,
        GroundCliffCheckKind(*GROUND_CLIFF_CHECK_KIND_NONE),
        true,
        *FIGHTER_STATUS_WORK_KEEP_FLAG_SONIC_SPECIAL_HI_FLAG,
        *FIGHTER_STATUS_WORK_KEEP_FLAG_SONIC_SPECIAL_HI_INT,
        *FIGHTER_STATUS_WORK_KEEP_FLAG_SONIC_SPECIAL_HI_FLOAT,
        0
    );
    FighterStatusModuleImpl::set_fighter_status_data(
        fighter.module_accessor,
        false,
        *FIGHTER_TREADED_KIND_NO_REAC,
        false,
        false,
        false,
        (
            *FIGHTER_LOG_MASK_FLAG_ACTION_CATEGORY_KEEP
        ) as u64,
        (
            *FIGHTER_STATUS_ATTR_START_TURN |
            *FIGHTER_STATUS_ATTR_DISABLE_INTERRUPT_WARP
        ) as u32,
        *FIGHTER_POWER_UP_ATTACK_BIT_SPECIAL_HI as u32,
        0
    );
    0.into()
}

pub unsafe extern "C" fn special_hi_jump_exec(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.boma();
    let min_speed_y = 1.0;
    let speed_y = KineticModule::get_sum_speed_y(fighter.module_accessor, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_ALL);

    if GroundModule::is_touch(boma, *GROUND_TOUCH_FLAG_DOWN as u32) 
    && speed_y <= min_speed_y {
        fighter.change_status_req(*FIGHTER_STATUS_KIND_LANDING_FALL_SPECIAL, false);
    }
    return 0.into();
}

unsafe extern "C" fn special_hi_jump_exit(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.boma();

    let landing_lag = WorkModule::get_param_int(fighter.module_accessor, hash40("param_special_hi"), hash40("special_hi_landing_frame")) as f32;
    WorkModule::set_float(fighter.module_accessor, landing_lag, *FIGHTER_INSTANCE_WORK_ID_FLOAT_LANDING_FRAME);

    if fighter.is_situation(*SITUATION_KIND_GROUND) {
        WorkModule::off_flag(boma, *FIGHTER_SONIC_INSTANCE_WORK_FLAG_SPECIAL_HI_FALL);
    }
    0.into()
}

pub fn install(agent: &mut Agent) {
    agent.status(Pre, *FIGHTER_STATUS_KIND_SPECIAL_HI, special_hi_pre);

    agent.status(Exec, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_HI_JUMP, special_hi_jump_exec);
    agent.status(Exit, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_HI_JUMP, special_hi_jump_exit);
}
use super::*;

// FIGHTER_SONIC_STATUS_KIND_SPECIAL_S_DASH

pub unsafe extern "C" fn special_s_dash_pre(fighter: &mut L2CFighterCommon) -> L2CValue {
    let log_mask_flags;
    let power_up_bit;
    if WorkModule::is_flag(fighter.module_accessor, *FIGHTER_SONIC_STATUS_SPECIAL_S_DASH_FLAG_SPECIAL_LW_HOLD) {
        log_mask_flags = *FIGHTER_LOG_MASK_FLAG_ATTACK_KIND_SPECIAL_LW | *FIGHTER_LOG_MASK_FLAG_ACTION_CATEGORY_ATTACK;
        power_up_bit = *FIGHTER_POWER_UP_ATTACK_BIT_SPECIAL_LW;
    }
    else {
        log_mask_flags = *FIGHTER_LOG_MASK_FLAG_ATTACK_KIND_SPECIAL_S | *FIGHTER_LOG_MASK_FLAG_ACTION_CATEGORY_ATTACK;
        power_up_bit = *FIGHTER_POWER_UP_ATTACK_BIT_SPECIAL_S;
    }
	StatusModule::init_settings(
        fighter.module_accessor,
        SituationKind(*SITUATION_KIND_NONE),
        *FIGHTER_KINETIC_TYPE_UNIQ,
        *GROUND_CORRECT_KIND_KEEP as u32,
        GroundCliffCheckKind(*GROUND_CLIFF_CHECK_KIND_NONE),
        true,
        *FIGHTER_STATUS_WORK_KEEP_FLAG_SONIC_SPECIAL_S_DASH_FLAG,
        *FIGHTER_STATUS_WORK_KEEP_FLAG_SONIC_SPECIAL_S_DASH_INT,
        *FIGHTER_STATUS_WORK_KEEP_FLAG_SONIC_SPECIAL_S_DASH_FLOAT,
        0
    );
    FighterStatusModuleImpl::set_fighter_status_data(
        fighter.module_accessor,
        false,
        *FIGHTER_TREADED_KIND_NO_REAC,
        false,
        false,
        false,
        log_mask_flags as u64,
        0,
        power_up_bit as u32,
        0
    );
    if fighter.global_table[SITUATION_KIND].get_i32() == *SITUATION_KIND_AIR {
        VarModule::on_flag(fighter.battle_object, vars::sonic::instance::SPECIAL_AIR_ACTION_USED);
    }
    0.into()
}

pub fn install(agent: &mut Agent) {
    agent.status(Pre, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_S_DASH, special_s_dash_pre);
}
use super::*;

// FIGHTER_STATUS_KIND_SPECIAL_N

pub unsafe extern "C" fn special_n_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let mot = if fighter.global_table[SITUATION_KIND].get_i32() == *SITUATION_KIND_GROUND {
        hash40("special_n_start")
    }
    else {
        hash40("special_air_n_start")
    };
    MotionModule::change_motion(
        fighter.module_accessor,
        Hash40::new_raw(mot),
        0.0,
        1.0,
        false,
        0.0,
        false,
        false
    );
    GroundModule::correct(fighter.module_accessor, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
    fighter.set_situation(SITUATION_KIND_AIR.into());
    WorkModule::on_flag(fighter.module_accessor, *FIGHTER_INSTANCE_WORK_ID_FLAG_FORCE_LOUPE);
    WorkModule::on_flag(fighter.module_accessor, *FIGHTER_SONIC_INSTANCE_WORK_FLAG_SPECIAL_N_FALL);
    if !StopModule::is_stop(fighter.module_accessor) {
        special_n_substatus(fighter);
    }
    fighter.global_table[SUB_STATUS2].assign(&L2CValue::Ptr(special_n_substatus as *const () as _));
    VarModule::on_flag(fighter.battle_object, vars::sonic::instance::SPECIAL_AIR_ACTION_USED);

    fighter.main_shift(special_n_main_loop)
}

unsafe extern "C" fn special_n_substatus(fighter: &mut L2CFighterCommon) -> L2CValue {
    let attack_frame_coeff = WorkModule::get_param_float(fighter.module_accessor, hash40("param_special_n"), hash40("special_n_attack_frame_coeff"));
    let ratio = 1.0 / attack_frame_coeff;
    let mut advance_counter = WorkModule::get_float(fighter.module_accessor, *FIGHTER_SONIC_STATUS_SPECIAL_N_WORK_FLOAT_ADVANCE_COUNTER);
    advance_counter += ratio;
    WorkModule::set_float(fighter.module_accessor, advance_counter, *FIGHTER_SONIC_STATUS_SPECIAL_N_WORK_FLOAT_ADVANCE_COUNTER);
    let auto_attack_frame = WorkModule::get_param_int(fighter.module_accessor, hash40("param_special_n"), hash40("special_n_start_auto_attack_frame"));
    if auto_attack_frame as f32 <= advance_counter {
        fighter.global_table[SUB_STATUS2].assign(&L2CValue::I32(0));
        fighter.global_table[SUB_STATUS].assign(&L2CValue::I32(0));
    }
    0.into()
}

unsafe extern "C" fn special_n_main_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    if MotionModule::is_end(fighter.module_accessor) {
        MotionModule::change_motion(
            fighter.module_accessor,
            Hash40::new("special_n_spin"),
            0.0,
            1.0,
            false,
            0.0,
            false,
            false
        );
        return fighter.main_shift(special_n_main_loop2);
    }
    0.into()
}

unsafe extern "C" fn special_n_main_loop2(fighter: &mut L2CFighterCommon) -> L2CValue {
    let advance_counter = WorkModule::get_float(fighter.module_accessor, *FIGHTER_SONIC_STATUS_SPECIAL_N_WORK_FLOAT_ADVANCE_COUNTER);
    let enable_attack_frame = WorkModule::get_param_int(fighter.module_accessor, hash40("param_special_n"), hash40("special_n_start_enable_attack_frame"));
    let auto_attack_frame = WorkModule::get_param_int(fighter.module_accessor, hash40("param_special_n"), hash40("special_n_start_auto_attack_frame"));
    let attack_frame_diff = advance_counter - enable_attack_frame as f32;
    let mut add_power = 0.0;
    if 0.0 < attack_frame_diff {
        let auto_frame_diff = auto_attack_frame - enable_attack_frame; 
        let ratio = attack_frame_diff / auto_frame_diff as f32;
        add_power = WorkModule::get_param_float(fighter.module_accessor, hash40("param_special_n"), hash40("special_n_add_attack_power"));
        add_power *= ratio;
    }
    WorkModule::set_float(fighter.module_accessor, add_power, *FIGHTER_SONIC_INSTANCE_WORK_ID_FLOAT_SPECIAL_N_ADD_ATTACK_POWER);
    if auto_attack_frame as f32 <= advance_counter {
        fighter.change_status(FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HOMING_START.into(), false.into());
        return 0.into();
    }
    let pad_flag = fighter.global_table[PAD_FLAG].get_i32();
    if 5.0 <= advance_counter {
        if pad_flag & *FIGHTER_PAD_FLAG_ATTACK_TRIGGER != 0 {
            WorkModule::set_float(fighter.module_accessor, 0.0, *FIGHTER_SONIC_INSTANCE_WORK_ID_FLOAT_SPECIAL_N_ADD_ATTACK_POWER);
            VarModule::on_flag(fighter.battle_object, vars::sonic::status::SPECIAL_N_BLAST_ATTACK);
            VarModule::on_flag(fighter.battle_object, vars::sonic::instance::SPECIAL_AIR_ACTION_USED);
            fighter.change_status(FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HOMING_START.into(), true.into());
        }
    }
    0.into()
}

// FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HOMING_START

pub unsafe extern "C" fn special_n_homing_start_pre(fighter: &mut L2CFighterCommon) -> L2CValue {
    let blast_attack = VarModule::is_flag(fighter.battle_object, vars::sonic::status::SPECIAL_N_BLAST_ATTACK);
    let ret = smashline::original_status(Pre, fighter, *FIGHTER_STATUS_KIND_SPECIAL_N)(fighter);
    VarModule::set_flag(fighter.battle_object, vars::sonic::status::SPECIAL_N_BLAST_ATTACK, blast_attack);
    ret
}

pub unsafe extern "C" fn special_n_landing_pre(fighter: &mut L2CFighterCommon) -> L2CValue {
    if fighter.is_prev_status(*FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HIT) {
        fighter.set_status_kind_interrupt(*FIGHTER_STATUS_KIND_LANDING_LIGHT);
        return 1.into();
    }
    smashline::original_status(Pre, fighter, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_LANDING)(fighter)
}

pub fn install(agent: &mut Agent) {
    agent.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_N, special_n_main);
    
    agent.status(Pre, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HOMING_START, special_n_homing_start_pre);
    
    agent.status(Pre, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_LANDING, special_n_landing_pre);
}
use super::*;

// FIGHTER_STATUS_KIND_DASH

pub unsafe extern "C" fn dash_pre(fighter: &mut L2CFighterCommon) -> L2CValue {
	let ground_brake = WorkModule::get_param_float(fighter.module_accessor, hash40("ground_brake"), 0);
	let mut initial_speed = VarModule::get_float(fighter.battle_object, vars::common::instance::CURR_DASH_SPEED);

	if ![*FIGHTER_STATUS_KIND_DASH, *FIGHTER_STATUS_KIND_TURN_DASH].contains(&StatusModule::prev_status_kind(fighter.module_accessor, 0)) {
		// println!("not after dash/turndash");
		initial_speed = KineticModule::get_sum_speed_x(fighter.module_accessor, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_ALL) - KineticModule::get_sum_speed_x(fighter.module_accessor, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_GROUND) - KineticModule::get_sum_speed_x(fighter.module_accessor, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_EXTERN);
	}

	// println!("dash total speed: {}", initial_speed);
	
	fighter.clear_lua_stack();
	lua_args!(fighter, FIGHTER_KINETIC_ENERGY_ID_CONTROL, initial_speed);
	app::sv_kinetic_energy::set_speed(fighter.lua_state_agent);

	VarModule::set_float(fighter.battle_object, vars::common::instance::CURR_DASH_SPEED, initial_speed);
	VarModule::set_float(fighter.battle_object, vars::common::instance::MOONWALK_SPEED, 0.0);

	smashline::original_status(Pre, fighter, *FIGHTER_STATUS_KIND_DASH)(fighter)
}

pub fn install(agent: &mut Agent) {
	agent.status(Pre, *FIGHTER_STATUS_KIND_DASH, dash_pre);
}
use super::*;

// FIGHTER_STATUS_KIND_SPECIAL_S

pub unsafe extern "C" fn special_s_pre(fighter: &mut L2CFighterCommon) -> L2CValue {
	StatusModule::init_settings(
        fighter.module_accessor,
        SituationKind(*SITUATION_KIND_NONE),
        *FIGHTER_KINETIC_TYPE_UNIQ,
        *GROUND_CORRECT_KIND_KEEP as u32,
        GroundCliffCheckKind(*GROUND_CLIFF_CHECK_KIND_ON_DROP),
        true,
        *FIGHTER_STATUS_WORK_KEEP_FLAG_NONE_FLAG,
        *FIGHTER_STATUS_WORK_KEEP_FLAG_NONE_INT,
        *FIGHTER_STATUS_WORK_KEEP_FLAG_NONE_FLOAT,
        0
    );
    FighterStatusModuleImpl::set_fighter_status_data(
        fighter.module_accessor,
        false,
        *FIGHTER_TREADED_KIND_NO_REAC,
        false,
        false,
        false,
        (
            *FIGHTER_LOG_MASK_FLAG_ATTACK_KIND_SPECIAL_S |
            *FIGHTER_LOG_MASK_FLAG_ACTION_CATEGORY_ATTACK |
            *FIGHTER_LOG_MASK_FLAG_ACTION_TRIGGER_ON
        ) as u64,
        (
            *FIGHTER_STATUS_ATTR_START_TURN |
            *FIGHTER_STATUS_ATTR_DISABLE_GROUND_FRICTION
        ) as u32,
        *FIGHTER_POWER_UP_ATTACK_BIT_SPECIAL_S as u32,
        0
    );
    0.into()
}

pub unsafe extern "C" fn special_s_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    // fighter.sub_change_motion_by_situation(L2CValue::Hash40s("special_s"), L2CValue::Hash40s("special_air_s"), false.into());
    VarModule::set_int(fighter.battle_object, vars::sonic::status::SPECIAL_S_STEP, vars::sonic::SPECIAL_S_STEP_START);
    MotionModule::change_motion(
        fighter.module_accessor,
        Hash40::new("special_s_boost_start"),
        0.0,
        1.0,
        false,
        0.0,
        false,
        false
    );
    if fighter.global_table[globals::SITUATION_KIND].get_i32() != *SITUATION_KIND_GROUND {
        VarModule::on_flag(fighter.battle_object, vars::sonic::instance::SPECIAL_AIR_ACTION_USED);
        KineticModule::change_kinetic(fighter.module_accessor, *FIGHTER_KINETIC_TYPE_AIR_STOP);
        GroundModule::correct(fighter.module_accessor, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
    }
    else {
        KineticModule::change_kinetic(fighter.module_accessor, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
        GroundModule::correct(fighter.module_accessor, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND_CLIFF_STOP_ATTACK));
    }
    fighter.main_shift(special_s_main_loop)
}

unsafe extern "C" fn special_s_main_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    // Lets you grab edge once it's enabled
    if fighter.sub_transition_group_check_air_cliff().get_bool() {
        return 0.into();
    }

    if CancelModule::is_enable_cancel(fighter.module_accessor) {
        if fighter.sub_wait_ground_check_common(false.into()).get_bool()
        || fighter.sub_air_check_fall_common().get_bool() {
            return 0.into();
        }
    }

    let step = VarModule::get_int(fighter.battle_object, vars::sonic::status::SPECIAL_S_STEP);
    let situation = fighter.global_table[SITUATION_KIND].get_i32();

    if situation == *SITUATION_KIND_GROUND
    && VarModule::is_flag(fighter.battle_object, vars::sonic::status::SPECIAL_S_ENABLE_JUMP)
    && AttackModule::is_infliction_status(fighter.module_accessor, *COLLISION_KIND_MASK_HIT)
    && !AttackModule::is_infliction_status(fighter.module_accessor, *COLLISION_KIND_MASK_PARRY)
    && !fighter.global_table[IS_STOPPING].get_bool()
    && !StatusModule::is_changing(fighter.module_accessor) {
        fighter.check_jump_cancel(false, false, false);
    }
    if StatusModule::is_situation_changed(fighter.module_accessor) {
        let mut special_momentum_handle = false;
        if step == vars::sonic::SPECIAL_S_STEP_END {
            if fighter.global_table[SITUATION_KIND].get_i32() == *SITUATION_KIND_GROUND {
                WorkModule::set_float(fighter.module_accessor, 8.0, *FIGHTER_INSTANCE_WORK_ID_FLOAT_LANDING_FRAME);
                fighter.change_status(FIGHTER_STATUS_KIND_LANDING_FALL_SPECIAL.into(), false.into());
                return 1.into();
            }
            else {
                special_momentum_handle = true;
            }
        }
        else {
            if fighter.global_table[SITUATION_KIND].get_i32() != *SITUATION_KIND_GROUND {
                special_momentum_handle = true;
            }
        }
        if situation == *SITUATION_KIND_GROUND {
            KineticModule::change_kinetic(fighter.module_accessor, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
            GroundModule::correct(fighter.module_accessor, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND));
        }
        else {
            VarModule::on_flag(fighter.battle_object, vars::sonic::instance::SPECIAL_AIR_ACTION_USED);
            KineticModule::change_kinetic(fighter.module_accessor, *FIGHTER_KINETIC_TYPE_AIR_STOP);
            GroundModule::correct(fighter.module_accessor, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
        }
        let dash_brake_x = if fighter.global_table[SITUATION_KIND].get_i32() == *SITUATION_KIND_GROUND {
            ParamModule::get_float(fighter.battle_object, ParamType::Agent, "param_special_s.dash_ground_brake_x")
        } else {
            ParamModule::get_float(fighter.battle_object, ParamType::Agent, "param_special_s.dash_air_brake_x")
        };
        sv_kinetic_energy!(
            set_brake,
            fighter,
            FIGHTER_KINETIC_ENERGY_ID_STOP,
            dash_brake_x,
            0.0
        );
        if special_momentum_handle {
            sonic_special_s_ledge_cancel_helper(fighter);
            return 0.into();
        }
    }
    if VarModule::is_flag(fighter.battle_object, vars::sonic::status::SPECIAL_S_ENABLE_CONTROL) {
        sv_kinetic_energy!(
            reset_energy,
            fighter,
            FIGHTER_KINETIC_ENERGY_ID_CONTROL,
            ENERGY_CONTROLLER_RESET_TYPE_FALL_ADJUST,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0
        );
        sv_kinetic_energy!(
            set_speed,
            fighter,
            FIGHTER_KINETIC_ENERGY_ID_CONTROL,
            0.0,
            0.0
        );
        KineticModule::enable_energy(fighter.module_accessor, *FIGHTER_KINETIC_ENERGY_ID_CONTROL);
        VarModule::off_flag(fighter.battle_object, vars::sonic::status::SPECIAL_S_ENABLE_CONTROL);
    }
    if MotionModule::is_end(fighter.module_accessor) {
        if step == vars::sonic::SPECIAL_S_STEP_START {
            MotionModule::change_motion(
                fighter.module_accessor,
                Hash40::new("special_s_boost"),
                0.0,
                1.0,
                false,
                0.0,
                false,
                false
            );
            let lr = PostureModule::lr(fighter.module_accessor);
            let x_speed = ParamModule::get_float(fighter.battle_object, ParamType::Agent, "param_special_s.dash_speed_x");
            let y_speed = if fighter.global_table[SITUATION_KIND].get_i32() != *SITUATION_KIND_GROUND {
                if ControlModule::check_button_on(fighter.module_accessor, *CONTROL_PAD_BUTTON_SPECIAL) {
                    VarModule::on_flag(fighter.battle_object, vars::sonic::status::SPECIAL_S_HOP);
                    ParamModule::get_float(fighter.battle_object, ParamType::Agent, "param_special_s.dash_hop_speed_y")
                }
                else {
                    KineticModule::get_sum_speed_y(fighter.module_accessor, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN)
                }
            }
            else {
                if ControlModule::check_button_on(fighter.module_accessor, *CONTROL_PAD_BUTTON_SPECIAL) {
                    fighter.set_situation(SITUATION_KIND_AIR.into());
                    GroundModule::correct(fighter.module_accessor, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
                    sv_kinetic_energy!(
                        reset_energy,
                        fighter,
                        FIGHTER_KINETIC_ENERGY_ID_GRAVITY,
                        ENERGY_GRAVITY_RESET_TYPE_GRAVITY,
                        0.0,
                        0.0,
                        0.0,
                        0.0,
                        0.0
                    );
                    let air_speed_y_stable = WorkModule::get_param_float(fighter.module_accessor, hash40("air_speed_y_stable"), 0);
                    let dash_air_speed_y_stable_mul = ParamModule::get_float(fighter.battle_object, ParamType::Agent, "param_special_s.dash_air_speed_y_stable_mul");
                    sv_kinetic_energy!(
                        set_stable_speed,
                        fighter,
                        FIGHTER_KINETIC_ENERGY_ID_GRAVITY,
                        air_speed_y_stable * dash_air_speed_y_stable_mul
                    );
                    KineticModule::enable_energy(fighter.module_accessor, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
                    VarModule::on_flag(fighter.battle_object, vars::sonic::status::SPECIAL_S_HOP);
                    VarModule::on_flag(fighter.battle_object, vars::sonic::instance::SPECIAL_AIR_ACTION_USED);
                    ParamModule::get_float(fighter.battle_object, ParamType::Agent, "param_special_s.dash_hop_speed_y")
                }
                else {
                    GroundModule::correct(fighter.module_accessor, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND));
                    0.0
                }
            };
            sv_kinetic_energy!(
                set_limit_speed,
                fighter,
                FIGHTER_KINETIC_ENERGY_ID_STOP,
                x_speed,
                0.0
            );
            sv_kinetic_energy!(
                set_speed,
                fighter,
                FIGHTER_KINETIC_ENERGY_ID_STOP,
                x_speed * lr,
                0.0
            );
            let dash_brake_x = if fighter.global_table[SITUATION_KIND].get_i32() == *SITUATION_KIND_GROUND {
                ParamModule::get_float(fighter.battle_object, ParamType::Agent, "param_special_s.dash_ground_brake_x")
            } else {
                ParamModule::get_float(fighter.battle_object, ParamType::Agent, "param_special_s.dash_air_brake_x")
            };
            sv_kinetic_energy!(
                set_brake,
                fighter,
                FIGHTER_KINETIC_ENERGY_ID_STOP,
                dash_brake_x,
                0.0
            );
            sv_kinetic_energy!(
                set_speed,
                fighter,
                FIGHTER_KINETIC_ENERGY_ID_GRAVITY,
                y_speed
            );
            VarModule::set_int(fighter.battle_object, vars::sonic::status::SPECIAL_S_STEP, vars::sonic::SPECIAL_S_STEP_DASH);
        }
        else if step == vars::sonic::SPECIAL_S_STEP_DASH {
            special_s_transition_to_end(fighter);
        }
        else {
            if fighter.global_table[SITUATION_KIND].get_i32() != *SITUATION_KIND_GROUND {
                fighter.change_status(FIGHTER_STATUS_KIND_FALL.into(), false.into());
            }
            else {
                fighter.change_status(FIGHTER_STATUS_KIND_WAIT.into(), false.into());
            }
        }
    }
    else {
        if step == vars::sonic::SPECIAL_S_STEP_DASH {
            // Reduce speed on shield
            if AttackModule::is_infliction_status(fighter.module_accessor, *COLLISION_KIND_MASK_SHIELD | *COLLISION_KIND_MASK_PARRY)
            && !VarModule::is_flag(fighter.battle_object, vars::sonic::status::SPECIAL_S_HIT_SHIELD) {
                VarModule::on_flag(fighter.battle_object, vars::sonic::status::SPECIAL_S_HIT_SHIELD);

                let shield_hit_end_speed_x = ParamModule::get_float(fighter.battle_object, ParamType::Agent, "param_special_s.shield_hit_end_speed_x");
                let lr = PostureModule::lr(fighter.module_accessor);
                sv_kinetic_energy!(
                    set_speed,
                    fighter,
                    FIGHTER_KINETIC_ENERGY_ID_STOP,
                    shield_hit_end_speed_x * lr,
                    0.0
                );

                special_s_transition_to_end(fighter);
            }
        }
    }

    0.into()
}

unsafe fn sonic_special_s_ledge_cancel_helper(fighter: &mut L2CFighterCommon) {
    MotionModule::change_motion(fighter.module_accessor, Hash40::new("special_air_s_boost_end"), 0.0, 24.0 / 18.0, false, 0.0, false, false);
    sv_kinetic_energy!(
        reset_energy,
        fighter,
        FIGHTER_KINETIC_ENERGY_ID_GRAVITY,
        ENERGY_GRAVITY_RESET_TYPE_GRAVITY,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0
    );
    KineticModule::enable_energy(fighter.module_accessor, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    let air_speed_y_stable = WorkModule::get_param_float(fighter.module_accessor, hash40("air_speed_y_stable"), 0);
    let dash_air_speed_y_stable_mul = ParamModule::get_float(fighter.battle_object, ParamType::Agent, "param_special_s.dash_air_speed_y_stable_mul");
    sv_kinetic_energy!(
        set_stable_speed,
        fighter,
        FIGHTER_KINETIC_ENERGY_ID_GRAVITY,
        air_speed_y_stable * dash_air_speed_y_stable_mul
    );
    sv_kinetic_energy!(
        set_limit_speed,
        fighter,
        FIGHTER_KINETIC_ENERGY_ID_STOP,
        2.5,
        0.0
    );
    fighter.clear_lua_stack();
    lua_args!(fighter, FIGHTER_KINETIC_ENERGY_ID_STOP);
    let speed_x = sv_kinetic_energy::get_speed_x(fighter.lua_state_agent);
    if speed_x.abs() > 2.5 {
        sv_kinetic_energy!(
            set_speed,
            fighter,
            FIGHTER_KINETIC_ENERGY_ID_STOP,
            2.5 * speed_x.signum(),
            0.0
        );
    }
    VarModule::set_int(fighter.battle_object, vars::sonic::status::SPECIAL_S_STEP, vars::sonic::SPECIAL_S_STEP_END);
}

unsafe fn special_s_transition_to_end(fighter: &mut L2CFighterCommon) {
    let mot = if fighter.global_table[SITUATION_KIND].get_i32() == *SITUATION_KIND_GROUND {
        VarModule::off_flag(fighter.battle_object, vars::sonic::instance::SPECIAL_AIR_ACTION_USED);
        hash40("special_s_boost_end")
    }
    else {
        hash40("special_air_s_boost_end")
    };
    MotionModule::change_motion(
        fighter.module_accessor,
        Hash40::new_raw(mot),
        0.0,
        1.0,
        false,
        0.0,
        false,
        false
    );
    VarModule::set_int(fighter.battle_object, vars::sonic::status::SPECIAL_S_STEP, vars::sonic::SPECIAL_S_STEP_END);
    let dash_end_brake_x = if fighter.global_table[SITUATION_KIND].get_i32() == *SITUATION_KIND_GROUND {
        ParamModule::get_float(fighter.battle_object, ParamType::Agent, "param_special_s.dash_ground_end_brake_x")
    } else {
        ParamModule::get_float(fighter.battle_object, ParamType::Agent, "param_special_s.dash_air_end_brake_x")
    };
    if fighter.global_table[SITUATION_KIND].get_i32() != *SITUATION_KIND_GROUND {
        sv_kinetic_energy!(
            set_brake,
            fighter,
            FIGHTER_KINETIC_ENERGY_ID_STOP,
            dash_end_brake_x,
            0.0
        );
    }
}

pub fn install(agent: &mut Agent) {
    agent.status(Pre, *FIGHTER_STATUS_KIND_SPECIAL_S, special_s_pre);
    agent.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_S, special_s_main);
}
use super::*;
use globals::*;
// status script import

mod dash;
mod special_n;
mod special_s;
mod special_s_dash;
mod special_lw_hold;
mod special_hi;
mod special_n_hit;

/// Prevents side b from being used again in air
unsafe extern "C" fn should_use_special_callback(fighter: &mut L2CFighterCommon) -> L2CValue {
    if (fighter.is_situation(*SITUATION_KIND_AIR) && VarModule::is_flag(fighter.battle_object, vars::sonic::instance::SPECIAL_AIR_ACTION_USED)) || fighter.is_status(*FIGHTER_SONIC_STATUS_KIND_SPIN_JUMP)  {
        false.into()
    } else {
        true.into()
    }
}

/// Prevents side b from being used again in air
unsafe extern "C" fn should_use_special_hi_callback(fighter: &mut L2CFighterCommon) -> L2CValue {
    if fighter.is_status(*FIGHTER_SONIC_STATUS_KIND_SPIN_JUMP)  {
        false.into()
    } else {
        true.into()
    }
}

/// Re-enables the ability to use aerial specials when connecting to ground or cliff
unsafe extern "C" fn change_status_callback(fighter: &mut L2CFighterCommon) -> L2CValue {
    if fighter.is_situation(*SITUATION_KIND_GROUND) || fighter.is_situation(*SITUATION_KIND_CLIFF)
    || fighter.is_status_one_of(&[*FIGHTER_STATUS_KIND_REBIRTH, *FIGHTER_STATUS_KIND_DEAD, *FIGHTER_STATUS_KIND_LANDING, *FIGHTER_STATUS_KIND_GIMMICK_SPRING_JUMP]) {
        VarModule::off_flag(fighter.battle_object, vars::sonic::instance::SPECIAL_AIR_ACTION_USED);
    }
    true.into()
}

unsafe extern "C" fn on_start(fighter: &mut L2CFighterCommon) {
    // set the callbacks on fighter init
    fighter.global_table[globals::USE_SPECIAL_N_CALLBACK].assign(&L2CValue::Ptr(should_use_special_callback as *const () as _));
    fighter.global_table[globals::USE_SPECIAL_S_CALLBACK].assign(&L2CValue::Ptr(should_use_special_callback as *const () as _));
    fighter.global_table[globals::USE_SPECIAL_LW_CALLBACK].assign(&L2CValue::Ptr(should_use_special_callback as *const () as _));
    fighter.global_table[globals::USE_SPECIAL_HI_CALLBACK].assign(&L2CValue::Ptr(should_use_special_hi_callback as *const () as _));
    fighter.global_table[globals::STATUS_CHANGE_CALLBACK].assign(&L2CValue::Ptr(change_status_callback as *const () as _));
}

pub fn install(agent: &mut Agent) {
    agent.on_start(on_start);

    dash::install(agent);
    special_n::install(agent);
    special_s::install(agent);
    special_s_dash::install(agent);
    special_lw_hold::install(agent);
    special_hi::install(agent);
    special_n_hit::install(agent);
}
use super::*;

unsafe extern "C" fn game_fall(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 5.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("top"), 4.0, 90, 90, 0, 40, 4.5, 0.0, 5.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_SPEED, false, 0, 0.0, 0, true, false, false, false, false, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_OBJECT);
    }
}

pub fn install(agent: &mut Agent) {
    agent.acmd("game_fall", game_fall, Priority::Low);
}
use super::*;

mod acmd;

pub fn install() {
    let agent = &mut Agent::new("sonic_gimmickjump");
    acmd::install(agent);
    agent.install();
}use super::*;

unsafe extern "C" fn game_catch(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 1.0);
    FT_MOTION_RATE_RANGE(agent, 1.0, 6.0, 6.0);
    frame(lua_state, 6.0);
    FT_MOTION_RATE(agent, 1.0);
    if is_excute(agent) {
        CATCH(agent, 0, Hash40::new("top"), 4.0, 0.0, 6.0, 6.0, Some(0.0), Some(6.0), Some(7.65), *FIGHTER_STATUS_KIND_CAPTURE_PULLED, *COLLISION_SITUATION_MASK_GA);
    }
    game_CaptureCutCommon(agent);
    frame(lua_state, 8.0);
    if is_excute(agent) {
        grab!(agent, *MA_MSC_CMD_GRAB_CLEAR_ALL);
        WorkModule::on_flag(boma, *FIGHTER_STATUS_CATCH_FLAG_CATCH_WAIT);
    }
    frame(lua_state, 9.0);
    FT_MOTION_RATE(agent, 1.22);
}

unsafe extern "C" fn game_catchdash(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 10.0);
    FT_MOTION_RATE(agent, 1.0);
    if is_excute(agent) {
        CATCH(agent, 0, Hash40::new("top"), 4.0, 0.0, 6.0, 6.0, Some(0.0), Some(6.0), Some(9.6), *FIGHTER_STATUS_KIND_CAPTURE_PULLED, *COLLISION_SITUATION_MASK_GA);
    }
    game_CaptureCutCommon(agent);
    frame(lua_state, 12.0);
    FT_MOTION_RATE(agent, 1.28);
    if is_excute(agent) {
        grab!(agent, *MA_MSC_CMD_GRAB_CLEAR_ALL);
        WorkModule::on_flag(boma, *FIGHTER_STATUS_CATCH_FLAG_CATCH_WAIT);
    }
}

unsafe extern "C" fn game_catchturn(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 1.0);
    FT_MOTION_RATE_RANGE(agent, 1.0, 10.0, 10.0);
    frame(lua_state, 10.0);
    if is_excute(agent) {
        CATCH(agent, 0, Hash40::new("top"), 4.0, 0.0, 6.0, -4.0, Some(0.0), Some(6.0), Some(-13.3), *FIGHTER_STATUS_KIND_CAPTURE_PULLED, *COLLISION_SITUATION_MASK_GA);
    }
    game_CaptureCutCommon(agent);
    frame(lua_state, 12.0);
    if is_excute(agent) {
        grab!(agent, *MA_MSC_CMD_GRAB_CLEAR_ALL);
        WorkModule::on_flag(boma, *FIGHTER_STATUS_CATCH_FLAG_CATCH_WAIT);
    }
    frame(lua_state, 13.0);
    FT_MOTION_RATE(agent, 1.125);
}

unsafe extern "C" fn game_throwb(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    if is_excute(agent) {
        ATTACK_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_THROW, 0, 9.0, 135, 60, 0, 70, 0.0, 1.0, *ATTACK_LR_CHECK_F, 0.0, true, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_NONE, *ATTACK_REGION_THROW);
        ATTACK_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_CATCH, 0, 3.0, 361, 100, 0, 60, 0.0, 1.0, *ATTACK_LR_CHECK_F, 0.0, true, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_NONE, *ATTACK_REGION_THROW);
    }
    frame(lua_state, 42.0);
    if is_excute(agent) {
        CHECK_FINISH_CAMERA(agent, -9, 8);
    }
    frame(lua_state, 43.0);
    if is_excute(agent) {
        let target = WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_OBJECT);
        let target_group = WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_HIT_GROUP);
        let target_no = WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_HIT_NO);
        ATK_HIT_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_THROW, Hash40::new("throw"), target, target_group, target_no);
    }
    frame(lua_state, 57.0);
    FT_MOTION_RATE_RANGE(agent, 57.0, 67.0, 14.0);
    frame(lua_state, 67.0);
    FT_MOTION_RATE(agent, 1.0);
}

unsafe extern "C" fn game_throwhi(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    if is_excute(agent) {
        ATTACK_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_THROW, 0, 5.0, 76, 60, 0, 95, 0.0, 1.0, *ATTACK_LR_CHECK_F, 0.0, true, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_NONE, *ATTACK_REGION_THROW);
        ATTACK_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_CATCH, 0, 3.0, 361, 100, 0, 60, 0.0, 1.0, *ATTACK_LR_CHECK_F, 0.0, true, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_NONE, *ATTACK_REGION_THROW);
    }
    frame(lua_state, 22.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("top"), 1.0, 80, 180, 0, 60, 5.0, 0.0, 11.0, 3.0, None, None, None, 1.25, 1.0, *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_cutup"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_CUTUP, *ATTACK_REGION_HEAD);
        AttackModule::set_catch_only_all(boma, true, false);
        CHECK_FINISH_CAMERA(agent, 5, 4);
        //FighterCutInManager::set_throw_finish_zoom_rate(boma, 1.2);
    }
    frame(lua_state, 23.0);
    if is_excute(agent) {
        ATK_HIT_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_THROW, Hash40::new("throw"), WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_OBJECT), WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_HIT_GROUP), WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_HIT_NO));
        AttackModule::clear_all(boma);
    }
}

unsafe extern "C" fn game_throwlw(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    if is_excute(agent) {
        ATTACK_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_THROW, 0, 5.0, 20, 35, 0, 75, 0.0, 1.0, *ATTACK_LR_CHECK_F, 0.5, true, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_NONE, *ATTACK_REGION_THROW);
        ATTACK_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_CATCH, 0, 3.0, 361, 100, 0, 60, 0.0, 1.0, *ATTACK_LR_CHECK_F, 0.0, true, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_NONE, *ATTACK_REGION_THROW);
    }
    frame(lua_state, 18.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("top"), 1.0, 45, 100, 0, 0, 5.0, 0.0, 3.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 8, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_rush"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_BODY);
        AttackModule::set_catch_only_all(boma, true, false);
    }
    frame(lua_state, 41.0);
    if is_excute(agent) {
        CHECK_FINISH_CAMERA(agent, 2, 0);
        //FighterCutInManager::set_throw_finish_zoom_rate(boma, 1.2);
        //FighterCutInManager::set_throw_finish_offset(boma, 0, -5, 0);
    }
    frame(lua_state, 42.0);
    if is_excute(agent) {
        ATK_HIT_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_THROW, Hash40::new("throw"), WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_OBJECT), WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_HIT_GROUP), WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_HIT_NO));
        AttackModule::clear_all(boma);
    }
    frame(lua_state, 43.0);{
        MotionModule::set_rate(boma, 2.0);
    }
    frame(lua_state, 70.0);{
        MotionModule::set_rate(boma, 1.0)
    }
}

pub fn install(agent: &mut Agent) {
    agent.acmd("game_catch", game_catch, Priority::Low);
    agent.acmd("game_catchdash", game_catchdash, Priority::Low);
    agent.acmd("game_catchturn", game_catchturn, Priority::Low);

    agent.acmd("game_throwb", game_throwb, Priority::Low);

    agent.acmd("game_throwhi", game_throwhi, Priority::Low);

    agent.acmd("game_throwlw", game_throwlw, Priority::Low);
}
use super::*;

unsafe extern "C" fn game_specialnhomingstart(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    if !VarModule::is_flag(agent.battle_object, vars::sonic::status::SPECIAL_N_BLAST_ATTACK) {
        if is_excute(agent) {
            SEARCH(agent, 0, 0, Hash40::new("top"), 40.0, 0.0, 10.0, 10.0, None, None, None, *COLLISION_KIND_MASK_HIT, *HIT_STATUS_MASK_NORMAL, 1, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_FIEB, *COLLISION_PART_MASK_BODY_HEAD, false);
        }
    }
}

unsafe extern "C" fn game_specialnhoming(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    let mut bone_hash = match WorkModule::is_flag(boma, *FIGHTER_SONIC_STATUS_SPECIAL_N_HOMING_FLAG_IS_KIRBY){
        true => Hash40::new("rot"),
        false => Hash40::new("hip")
    };
    if is_excute(agent) {
        JostleModule::set_status(boma, false);
        FT_MOTION_RATE(agent, 0.013);
        ATTACK(agent, 0, 0, bone_hash, 8.0, 80, 90, 0, 50, 5.0, 0.5, 1.5, 0.0, Some(0.5), Some(1.5), Some(0.0), 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_SPEED, false, 2, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_PUNCH, *ATTACK_REGION_BODY);
        ATK_SET_SHIELD_SETOFF_MUL(agent, 0, 0.5);
        AttackModule::set_captured_same_time_attack(boma, 0, true);
        AttackModule::set_attack_keep_rumble(boma, 0, true);
    }
}

unsafe extern "C" fn sound_specialnhoming(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    let rng = app::sv_math::rand(smash::hash40("fighter"), 2);
    let mut sound = "";
    if rng == 0 { sound = "vc_sonic_004"; }
    else { sound = "vc_sonic_attack02"; }
    if is_excute(agent) {
        STOP_SE(agent, Hash40::new("se_sonic_rounddash"));
        PLAY_SE(agent, Hash40::new(sound));
        PLAY_SE(agent, Hash40::new("se_sonic_special_n01"));
    }
}

unsafe extern "C" fn game_specialnhit(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    if is_excute(agent) {
        FT_MOTION_RATE(agent, 0.3);
    }
    frame(lua_state, 3.0);
    if is_excute(agent) {
         KineticUtility::reset_enable_energy(
        *FIGHTER_KINETIC_ENERGY_ID_CONTROL,
        boma,
        *ENERGY_CONTROLLER_RESET_TYPE_FALL_ADJUST,
        &Vector2f{x: 0.0, y: 0.0},
        &Vector3f{x: 0.0, y: 0.0, z: 0.0}
         );    
        let temp = Vector3f { x: 0.3, y: 1.0, z: 0.0 };
		KineticModule::add_speed(boma, &temp);
        
    }
    frame(lua_state, 10.0);
    if is_excute(agent) {
        FT_MOTION_RATE(agent, 1.0);
    }
}

unsafe extern "C" fn game_specialsbooststart(agent: &mut L2CAgentBase) {
    // FT_MOTION_RATE(agent, 0.75);
}

unsafe extern "C" fn sound_specialsbooststart(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    frame(lua_state, 6.0);
    if is_excute(agent) {
        PLAY_SEQUENCE(agent, Hash40::new("seq_sonic_rnd_attack"));
    }
}

unsafe extern "C" fn expression_specialsbooststart(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 5.0);
    if is_excute(agent) {
        ControlModule::set_rumble(boma, Hash40::new("rbkind_dash"), 0, true, *BATTLE_OBJECT_ID_INVALID as u32);
    }
}

unsafe extern "C" fn game_specialsboost(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    FT_MOTION_RATE(agent, 1.0 / 3.0);
    frame(lua_state, 3.0);
    FT_MOTION_RATE(agent, 1.0);
    if is_excute(agent) {
        notify_event_msc_cmd!(agent, Hash40::new_raw(0x2127e37c07), *GROUND_CLIFF_CHECK_KIND_NONE);
        ATTACK(agent, 0, 0, Hash40::new("top"), 4.0, 361, 100, 0, 45, 4.0, 0.0, 5.5, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_BODY);
    }
    frame(lua_state, 11.0);
    if is_excute(agent) {
         ATTACK(agent, 0, 0, Hash40::new("top"), 1.0, 25, 120, 0, 50, 3.0, 0.0, 4.0, -0.5, Some(0.0), Some(9.0), Some(-0.5), 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_BODY);
         VarModule::on_flag(agent.battle_object, vars::sonic::status::SPECIAL_S_ENABLE_JUMP);
        }
    frame(lua_state, 14.0);
    FT_MOTION_RATE(agent, 0.25);
}

unsafe extern "C" fn effect_specialsboost(agent: &mut L2CAgentBase) {
    let boma = agent.boma();
    if is_excute(agent) {
        let eff = if VarModule::is_flag(agent.battle_object, vars::sonic::status::SPECIAL_S_HOP) {
            Hash40::new("sonic_spintrace_max")
        }
        else {
            Hash40::new("sonic_spintrace")
        };
        EFFECT_FOLLOW_NO_STOP(agent, eff, Hash40::new("top"), 0, 6, 0, 0, 0, 0, 1.25, true);
        EffectModule::enable_sync_init_pos_last(boma);
    }
}

unsafe extern "C" fn sound_specialsboost(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_sonic_special_s01"));
    }
}

unsafe extern "C" fn expression_specialsboost(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    if is_excute(agent) {
        RUMBLE_HIT(agent, Hash40::new("rbkind_attackm"), 0);
    }
    frame(lua_state, 7.0);
    if is_excute(agent) {
        RUMBLE_HIT(agent, Hash40::new("rbkind_attacks"), 0);
    }
}

unsafe extern "C" fn game_specialsboostend(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    FT_MOTION_RATE(agent, 0.5);
    if is_excute(agent) {
        AttackModule::clear_all(boma);
    }
    frame(lua_state, 8.0);
    FT_MOTION_RATE(agent, 2.0);
    frame(lua_state, 20.0);
    FT_MOTION_RATE(agent, 0.75);
}

unsafe extern "C" fn effect_specialsboostend(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    if is_excute(agent) {
        let eff = if VarModule::is_flag(agent.battle_object, vars::sonic::status::SPECIAL_S_HOP) {
            Hash40::new("sonic_spintrace_max")
        }
        else {
            Hash40::new("sonic_spintrace")
        };
        EFFECT_OFF_KIND(agent, eff, false, false);
    }
    for _ in 0..4 {
        if is_excute(agent) {
            LANDING_EFFECT(agent, Hash40::new("sys_sliding_smoke"), Hash40::new("top"), 4, 0, 0, 0, 0, 0, 0.6, 0, 0, 3, 0, 0, 0, false);
        }
        wait(lua_state, 4.0);
    }
}

unsafe extern "C" fn sound_specialsboostend(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_sonic_dash_stop"));
    }
}

unsafe extern "C" fn expression_specialsboostend(agent: &mut L2CAgentBase) {
    let boma = agent.boma();
    if is_excute(agent) {
        ControlModule::set_rumble(boma, Hash40::new("rbkind_dash"), 0, false, *BATTLE_OBJECT_ID_INVALID as u32);
    }
}

unsafe extern "C" fn game_specialairsboostend(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    FT_MOTION_RATE(agent, 0.8);
    if is_excute(agent) {
        AttackModule::clear_all(boma);
    }
    frame(lua_state, 8.0);
    FT_MOTION_RATE(agent, 1.3);
    if is_excute(agent) {
        notify_event_msc_cmd!(agent, Hash40::new_raw(0x2127e37c07), *GROUND_CLIFF_CHECK_KIND_ON_DROP);
    }
    frame(lua_state, 24.0);
    FT_MOTION_RATE(agent, 0.5);
    if is_excute(agent) {
        VarModule::on_flag(agent.battle_object, vars::sonic::status::SPECIAL_S_ENABLE_CONTROL);
    }
}

unsafe extern "C" fn effect_specialairsboostend(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        let eff = if VarModule::is_flag(agent.battle_object, vars::sonic::status::SPECIAL_S_HOP) {
            Hash40::new("sonic_spintrace_max")
        }
        else {
            Hash40::new("sonic_spintrace")
        };
        EFFECT_OFF_KIND(agent, eff, false, false);
    }
}

unsafe extern "C" fn game_specialhi(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    if is_excute(agent) {
        GroundModule::select_cliff_hangdata(boma, 2);
        ArticleModule::shoot_exist(boma, *FIGHTER_SONIC_GENERATE_ARTICLE_GIMMICKJUMP, app::ArticleOperationTarget(*ARTICLE_OPE_TARGET_ALL), false);
    }
    frame(lua_state, 8.0);
    if is_excute(agent) {
        notify_event_msc_cmd!(agent, Hash40::new_raw(0x2127e37c07), *GROUND_CLIFF_CHECK_KIND_ON_DROP_BOTH_SIDES);
    }   
}

unsafe extern "C" fn game_speciallwhold(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    for i in 1..50 {
        wait(lua_state, 6.0);
        if is_excute(agent) {
            ATTACK(agent, 0, 0, Hash40::new("rot"), 1.0, 90, 0, 0, 58, 5.0, 0.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_BODY);
        }
        wait(lua_state, 3.0);
        if is_excute(agent) {
            AttackModule::clear_all(boma);
        }
    }
}

unsafe extern "C" fn game_speciallwstart(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 5.0);
    if is_excute(agent) {
        StatusModule::change_status_request_from_script(boma, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_LW_HOLD, false);
    }
}

unsafe extern "C" fn game_specialairlwstart(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 5.0);
    if is_excute(agent) {
        StatusModule::change_status_request_from_script(boma, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_LW_HOLD, false);
    }
}

unsafe extern "C" fn game_speciallwdash(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    if is_excute(agent) {
        JostleModule::set_status(boma, false);
        AttackModule::clear_all(boma);
        ATTACK(agent, 0, 0, Hash40::new("hip"), 5.0, 60, 57, 0, 97, 3.5, 0.0, 1.5, 0.0, None, None, None, 1.0, 0.5, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_BODY);
        ATK_SET_SHIELD_SETOFF_MUL(agent,0, 0.5);
        AttackModule::set_captured_same_time_attack(boma, *FIGHTER_SONIC_STATUS_SPECIAL_S_DASH_ATTACK_ID_DEFAULT, true);
        AttackModule::set_attack_keep_rumble(boma, 0, true);
    }
}

unsafe extern "C" fn game_specialairlwdash(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    if is_excute(agent) {
        JostleModule::set_status(boma, false);
        UNABLE_AREA(agent, *FIGHTER_AREA_KIND_TREAD_JUMP_CHECK);
    }
    frame(lua_state, 3.0);
    if is_excute(agent) {
        AttackModule::set_captured_same_time_attack(boma, 0, true);
    }
    frame(lua_state, 10.0);
    if is_excute(agent) {
        WorkModule::on_flag(boma, *FIGHTER_SONIC_STATUS_SPIN_JUMP_WORK_ID_FLAG_ENABLE_JUMP_AERIAL);
        ENABLE_AREA(agent, *FIGHTER_AREA_KIND_TREAD_JUMP_CHECK);
    }
    frame(lua_state, 12.0);
    if is_excute(agent) {
    ATTACK(agent, 0, 0, Hash40::new("hip"), 6.0, 60, 60, 0, 80, 4.0, 0.0, 1.5, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_BODY);
    }
    frame(lua_state, 34.0);
    if is_excute(agent) {
        AttackModule::clear_all(boma);
    }
    frame(lua_state, 40.0);
    if is_excute(agent) {
        notify_event_msc_cmd!(agent, Hash40::new_raw(0x2127e37c07), *GROUND_CLIFF_CHECK_KIND_ON_DROP);
    }
}

unsafe extern "C" fn stub(agent: &mut L2CAgentBase) {} 

pub fn install(agent: &mut Agent) {
    agent.acmd("game_specialnhomingstart", game_specialnhomingstart, Priority::Low);
    agent.acmd("game_specialnhoming", game_specialnhoming, Priority::Low);
    agent.acmd("sound_specialnhoming", sound_specialnhoming, Priority::Low);
    agent.acmd("game_specialnhit", game_specialnhit, Priority::Low);
    agent.acmd("effect_specialnhit", acmd_stub, Priority::Low);

    agent.acmd("game_specialsbooststart", game_specialsbooststart, Priority::Low);
    agent.acmd("sound_specialsbooststart", sound_specialsbooststart, Priority::Low);
    agent.acmd("expression_specialsbooststart", expression_specialsbooststart, Priority::Low);
    agent.acmd("game_specialsboost", game_specialsboost, Priority::Low);
    agent.acmd("effect_specialsboost", effect_specialsboost, Priority::Low);
    agent.acmd("sound_specialsboost", sound_specialsboost, Priority::Low);
    agent.acmd("expression_specialsboost", expression_specialsboost, Priority::Low);
    agent.acmd("game_specialsboostend", game_specialsboostend, Priority::Low);
    agent.acmd("effect_specialsboostend", effect_specialsboostend, Priority::Low);
    agent.acmd("sound_specialsboostend", sound_specialsboostend, Priority::Low);
    agent.acmd("expression_specialsboostend", expression_specialsboostend, Priority::Low);
    agent.acmd("game_specialairsboostend", game_specialairsboostend, Priority::Low);
    agent.acmd("effect_specialairsboostend", effect_specialairsboostend, Priority::Low);

    agent.acmd("game_specialhi", game_specialhi, Priority::Low);

    agent.acmd("game_speciallwhold", game_speciallwhold, Priority::Low);
    agent.acmd("game_specialairlwhold", game_speciallwhold, Priority::Low);
    agent.acmd("game_speciallwholdchargehi", game_speciallwhold, Priority::Low);
    agent.acmd("game_speciallwholdchargelw", game_speciallwhold, Priority::Low);
    agent.acmd("game_speciallwholdchargemiddle", game_speciallwhold, Priority::Low);
    agent.acmd("game_speciallwstart", game_speciallwstart, Priority::Low);
    agent.acmd("game_specialairlwstart", game_specialairlwstart, Priority::Low);
    agent.game_acmd(0x1b07509826, game_speciallwdash, Priority::Low);
    agent.game_acmd(0x195dc47911, game_specialairlwdash, Priority::Low);
}
use super::*;

unsafe extern "C" fn sound_damagefly(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 2.0);
    if is_excute(agent) {
        if VarModule::is_flag(agent.battle_object, vars::common::instance::IS_KILLING_BLOW) {
            PLAY_SE(agent, Hash40::new("vc_sonic_damagefly02"));
        } else {
            let damage_speed_x = agent.get_speed_x(*FIGHTER_KINETIC_ENERGY_ID_DAMAGE);
            let damage_speed_y = agent.get_speed_y(*FIGHTER_KINETIC_ENERGY_ID_DAMAGE);

            let speed_vector = sv_math::vec2_length(damage_speed_x, damage_speed_y);

            let play_vc = if speed_vector < 3.8 {
                app::sv_math::rand(hash40("fighter"), 3)
            } else {
                0
            };
            if play_vc == 0 {
                PLAY_FLY_VOICE(agent, Hash40::new("seq_sonic_rnd_futtobi01"), Hash40::new("seq_sonic_rnd_futtobi02"));
            }
        }
    }
}

unsafe extern "C" fn sound_damageflyroll(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 2.0);
    if is_excute(agent) {
        if VarModule::is_flag(agent.battle_object, vars::common::instance::IS_KILLING_BLOW) {
            PLAY_SE(agent, Hash40::new("vc_sonic_damagefly02"));
        } else {
            PLAY_FLY_VOICE(agent, Hash40::new("seq_sonic_rnd_futtobi01"), Hash40::new("seq_sonic_rnd_futtobi02"));
        }
    }
}

unsafe extern "C" fn sound_dash(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 4.0);
    if is_excute(agent) {
        let dash_sfx_handle = SoundModule::play_se(boma, Hash40::new("se_sonic_dash_start"), true, false, false, false, app::enSEType(0));
        SoundModule::set_se_vol(boma, dash_sfx_handle as i32, 0.5, 0);
    }
    wait(lua_state, 9.0);
    if is_excute(agent) {
        PLAY_STEP(agent, Hash40::new("se_sonic_step_right_m"));
    }
    wait(lua_state, 5.0);
    if is_excute(agent) {
        PLAY_STEP(agent, Hash40::new("se_sonic_step_left_m"));
    }
}

unsafe extern "C" fn game_turndash(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 3.0);
    if is_excute(agent) {
        WorkModule::on_flag(boma, *FIGHTER_STATUS_DASH_FLAG_TURN_DASH);
    }
    frame(lua_state, 11.0);
    if is_excute(agent) {
        WorkModule::enable_transition_term(boma, *FIGHTER_STATUS_TRANSITION_TERM_ID_DASH_TO_RUN);
    }
    frame(lua_state, 16.0);
    if is_excute(agent) {
        WorkModule::enable_transition_term(boma, *FIGHTER_SONIC_STATUS_DASH_FLAG_BRAKE_INCREASE);
    }
}

unsafe extern "C" fn game_escapeair(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();

    frame(lua_state, 29.0);
    if is_excute(agent) {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_FALL);
    }

}

unsafe extern "C" fn game_escapeairslide(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();

    frame(lua_state, 29.0);
    if is_excute(agent) {
        WorkModule::on_flag(boma, *FIGHTER_STATUS_ESCAPE_AIR_FLAG_SLIDE_ENABLE_CONTROL);
    }

}

unsafe extern "C" fn game_jumpaerialfront(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();

    if is_excute(agent) {
        GroundModule::select_cliff_hangdata(boma, 1);
    }
}

unsafe extern "C" fn game_jumpaerialback(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();

    if is_excute(agent) {
        GroundModule::select_cliff_hangdata(boma, 1);
    }
}

pub fn install(agent: &mut Agent) {
    agent.acmd("game_cliffescape", acmd_stub, Priority::Low);

    agent.acmd("sound_damageflyhi", sound_damagefly, Priority::Low);
    agent.acmd("sound_damageflylw", sound_damagefly, Priority::Low);
    agent.acmd("sound_damageflyn", sound_damagefly, Priority::Low);
    agent.acmd("sound_damageflytop", sound_damagefly, Priority::Low);
    agent.acmd("sound_damageflyroll", sound_damageflyroll, Priority::Low);

    agent.acmd("sound_dash", sound_dash, Priority::Low);
    agent.acmd("game_turndash", game_turndash, Priority::Low);

    agent.acmd("game_escapeair", game_escapeair, Priority::Low);
    agent.acmd("game_escapeairslide", game_escapeairslide, Priority::Low);

    agent.acmd("game_jumpaerialfront", game_jumpaerialfront, Priority::Low);
    agent.acmd("game_jumpaerialback", game_jumpaerialback, Priority::Low);
}
use super::*;

unsafe extern "C" fn game_attacks4(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 13.0);
    if is_excute(agent) {
        WorkModule::on_flag(boma, *FIGHTER_STATUS_ATTACK_FLAG_START_SMASH_HOLD);
    }
    frame(lua_state, 18.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("handr"), 16.0, 361, 98, 0, 32, 4.5, 4.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 10, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_PUNCH, *ATTACK_REGION_PUNCH);
        ATTACK(agent, 1, 0, Hash40::new("armr"), 16.0, 361, 98, 0, 32, 3.5, 1.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 10, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_PUNCH, *ATTACK_REGION_PUNCH);
        ATTACK(agent, 2, 0, Hash40::new("shoulderr"), 16.0, 361, 98, 0, 32, 2.5, -1.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 10, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_PUNCH, *ATTACK_REGION_PUNCH);
    }
    wait(lua_state, 3.0);
    if is_excute(agent) {
        AttackModule::clear_all(boma);
    }
}

unsafe extern "C" fn game_attacks4hi(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 13.0);
    if is_excute(agent) {
        WorkModule::on_flag(boma, *FIGHTER_STATUS_ATTACK_FLAG_START_SMASH_HOLD);
    }
    frame(lua_state, 18.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("handr"), 16.0, 361, 98, 0, 32, 4.5, 4.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 10, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_PUNCH, *ATTACK_REGION_PUNCH);
        ATTACK(agent, 1, 0, Hash40::new("armr"), 16.0, 361, 98, 0, 32, 3.5, 1.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 10, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_PUNCH, *ATTACK_REGION_PUNCH);
        ATTACK(agent, 2, 0, Hash40::new("shoulderr"), 16.0, 361, 98, 0, 32, 2.5, -1.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 10, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_PUNCH, *ATTACK_REGION_PUNCH);
        AttackModule::set_attack_height_all(boma, app::AttackHeight(*ATTACK_HEIGHT_HIGH), false);
    }
    wait(lua_state, 3.0);
    if is_excute(agent) {
        AttackModule::clear_all(boma);
    }
}

unsafe extern "C" fn game_attacks4lw(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 13.0);
    if is_excute(agent) {
        WorkModule::on_flag(boma, *FIGHTER_STATUS_ATTACK_FLAG_START_SMASH_HOLD);
    }
    frame(lua_state, 18.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("handr"), 16.0, 361, 98, 0, 32, 4.5, 4.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 10, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_PUNCH, *ATTACK_REGION_PUNCH);
        ATTACK(agent, 1, 0, Hash40::new("armr"), 16.0, 361, 98, 0, 32, 3.5, 1.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 10, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_PUNCH, *ATTACK_REGION_PUNCH);
        ATTACK(agent, 2, 0, Hash40::new("shoulderr"), 16.0, 361, 98, 0, 32, 2.5, -1.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 10, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_PUNCH, *ATTACK_REGION_PUNCH);
        AttackModule::set_attack_height_all(boma, app::AttackHeight(*ATTACK_HEIGHT_LOW), false);
    }
    wait(lua_state, 3.0);
    if is_excute(agent) {
        AttackModule::clear_all(boma);
    }
}

unsafe extern "C" fn game_attackhi4(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 7.0);
    if is_excute(agent) {
        WorkModule::on_flag(boma, *FIGHTER_STATUS_ATTACK_FLAG_START_SMASH_HOLD);
    }
    frame(lua_state, 9.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("kneer"), 16.0, 80, 100, 0, 27, 4.5, 0.0, 0.3, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 10, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 1, 0, Hash40::new("footr"), 16.0, 80, 100, 0, 27, 5.25, 3.0, 0.0, -2.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 10, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
    }
    wait(lua_state, 4.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("kneer"), 12.0, 361, 100, 0, 15, 4.5, 0.0, 0.3, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 10, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 1, 0, Hash40::new("footr"), 12.0, 361, 100, 0, 15, 5.0, 2.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 10, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
    }
    wait(lua_state, 4.0);
    if is_excute(agent) {
        AttackModule::clear_all(boma);
        FT_MOTION_RATE(agent, 30.0/(42.0 - 17.0));
    }
}

unsafe extern "C" fn effect_attackhi4(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 1.0);
    if is_excute(agent) {
        EFFECT(agent, Hash40::new("sys_smash_flash"), Hash40::new("top"), 0, 17, 4, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, true);
    }
    frame(lua_state, 8.0);
    if is_excute(agent) {
        LANDING_EFFECT(agent, Hash40::new("sys_h_smoke_a"), Hash40::new("top"), 2, 0, 0, 0, 0, 0, 0.9, 0, 0, 0, 0, 0, 0, false);
    }
    frame(lua_state, 9.0);
    if is_excute(agent) {
        EFFECT_FOLLOW(agent, Hash40::new("sys_attack_arc_d"), Hash40::new("top"), 0, 13, 0, 0, 50, 90, 1.2, true);
        LAST_EFFECT_SET_COLOR(agent, 3.0, 0.15, 0.15);
        LAST_EFFECT_SET_RATE(agent, 1.3);
    }
    frame(lua_state, 27.0);
    if is_excute(agent) {
        FOOT_EFFECT(agent, Hash40::new("sys_landing_smoke_s"), Hash40::new("top"), -1.5, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, false);
    }
}

unsafe extern "C" fn sound_attackhi4(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    if is_excute(agent) {
        STOP_SE(agent, Hash40::new_raw(0x13589f8893));
    }
    frame(lua_state, 9.0);
    if is_excute(agent) {
        STOP_SE(agent, Hash40::new("se_common_smash_start"));
        PLAY_SE(agent, Hash40::new("se_sonic_attackair_f03"));
    }
    wait(lua_state, 4.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("vc_sonic_attack06"));
        PLAY_STATUS(agent, Hash40::new("se_sonic_smash_l01"));
        //PLAY_SE(agent, Hash40::new("se_sonic_smash_h01"));
    }
    wait(lua_state, 27.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_sonic_landing02"));
    }
}

unsafe extern "C" fn expression_attackhi4(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    if is_excute(agent) {
        slope!(agent, MA_MSC_CMD_SLOPE_SLOPE, SLOPE_STATUS_LR);
    }
    frame(lua_state, 7.0); // make this the frame you can charge the attack
    sv_animcmd::execute(lua_state , 7.0); // make this the frame you can charge the attack
    if is_excute(agent) {
        slope!(agent, MA_MSC_CMD_SLOPE_SLOPE, SLOPE_STATUS_LR);
    }
    frame(lua_state, 8.0);
    if is_excute(agent) {
        ControlModule::set_rumble(
            boma,
            Hash40::new("rbkind_nohitm"),
            0,
            false,
            *BATTLE_OBJECT_ID_INVALID as u32
        );
    }
    frame(lua_state, 9.0);
    if is_excute(agent) {
        RUMBLE_HIT(agent, Hash40::new("rbkind_attackl"), 0);
    }
    frame(lua_state, 13.0);
    if is_excute(agent) {
        RUMBLE_HIT(agent, Hash40::new("rbkind_attackm"), 0);
    }
}

unsafe extern "C" fn game_attacklw4(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    if is_excute(agent) {
        FT_MOTION_RATE(agent, 4.0/10.0);
    }
    frame(lua_state, 4.0);
    if is_excute(agent) {
        WorkModule::on_flag(boma, *FIGHTER_STATUS_ATTACK_FLAG_START_SMASH_HOLD);
    }
    frame(lua_state, 10.0);
    if is_excute(agent) {
        FT_MOTION_RATE(agent, 16.0/(32.0 - 10.0));
        ATTACK(agent, 0, 0, Hash40::new("hip"), 15.0, 361, 90, 0, 35, 4.0, 0.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, true, 10, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 1, 0, Hash40::new("kneer"), 15.0, 361, 90, 0, 35, 5.0, 0.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, true, 10, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 2, 0, Hash40::new("footr"), 15.0, 361, 90, 0, 35, 5.0, 0.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, true, 10, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
    }
    frame(lua_state, 15.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("hip"), 12.0, 361, 94, 0, 20, 4.0, 0.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, true, 10, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 1, 0, Hash40::new("kneer"), 12.0, 361, 94, 0, 20, 5.0, 0.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, true, 10, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 2, 0, Hash40::new("footr"), 12.0, 361, 94, 0, 20, 5.0, 0.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, true, 10, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
    }
    frame(lua_state, 32.0);
    if is_excute(agent) {
        AttackModule::clear_all(boma);
        FT_MOTION_RATE(agent, 28.0/(54.0 - 32.0));
    }
    frame(lua_state, 54.0);
    if is_excute(agent) {
        FT_MOTION_RATE(agent, 1.0);
    }
}

unsafe extern "C" fn effect_attacklw4(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 1.0);
    if is_excute(agent) {
        EFFECT(agent, Hash40::new("sys_smash_flash"), Hash40::new("top"), 0, 15, 4, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, true);
    }
    frame(lua_state, 8.0);
    if is_excute(agent) {
        LANDING_EFFECT_FLIP(agent, Hash40::new("sys_whirlwind_r"), Hash40::new("sys_whirlwind_l"), Hash40::new("top"), 0, 0, 1, 0, 0, 0, 1.2, 0, 0, 0, 0, 0, 0, false, *EF_FLIP_NONE);
    }
    frame(lua_state, 10.0);
    if is_excute(agent) {
        EFFECT_FOLLOW_FLIP(agent, Hash40::new("sys_attack_arc"), Hash40::new("sys_attack_arc"), Hash40::new("top"), 0, 5, -4, -180, 150, 7, 1.4, true, *EF_FLIP_YZ);
    }
    frame(lua_state, 18.0);
    if is_excute(agent) {
        EFFECT_FOLLOW_FLIP(agent, Hash40::new("sys_attack_arc"), Hash40::new("sys_attack_arc"), Hash40::new("top"), 0, 5, 2.5, -180, 20, 5, 1.4, true, *EF_FLIP_YZ);
    }
    frame(lua_state, 27.0);
    if is_excute(agent) {
        EFFECT_FOLLOW_FLIP(agent, Hash40::new("sys_attack_arc"), Hash40::new("sys_attack_arc"), Hash40::new("top"), 0, 5, -5.5, -180, 150, 7, 1.4, true, *EF_FLIP_YZ);
    }
}

unsafe extern "C" fn expression_attacklw4(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    if is_excute(agent) {
        slope!(agent, *MA_MSC_CMD_SLOPE_SLOPE, *SLOPE_STATUS_L);
    }
    frame(lua_state, 6.0);
    app::sv_animcmd::execute(lua_state, 6.0);
    if WorkModule::is_flag(boma, *FIGHTER_STATUS_ATTACK_FLAG_SMASH_SMASH_HOLD_TO_ATTACK) {
        if is_excute(agent) {
            slope!(agent, *MA_MSC_CMD_SLOPE_SLOPE, *SLOPE_STATUS_L);
        }
    }
    frame(lua_state, 8.0);
    if is_excute(agent) {
        slope!(agent, *MA_MSC_CMD_SLOPE_SLOPE_INTP, *SLOPE_STATUS_TOP, 4);
        ControlModule::set_rumble(boma, Hash40::new("rbkind_nohitl"), 0, false, *BATTLE_OBJECT_ID_INVALID as u32);
    }
    frame(lua_state, 10.0);
    if is_excute(agent) {
        RUMBLE_HIT(agent, Hash40::new("rbkind_attackl"), 0);
    }
    frame(lua_state, 40.0);
    if is_excute(agent) {
        slope!(agent, *MA_MSC_CMD_SLOPE_SLOPE_INTP, *SLOPE_STATUS_NONE, 8);
    }
    frame(lua_state, 52.0);
    if is_excute(agent) {
        slope!(agent, *MA_MSC_CMD_SLOPE_SLOPE_INTP, *SLOPE_STATUS_LR, 4);
    }
}

pub fn install(agent: &mut Agent) {
    agent.acmd("game_attacks4", game_attacks4, Priority::Low);
    agent.acmd("game_attacks4hi", game_attacks4hi, Priority::Low);
    agent.acmd("game_attacks4lw", game_attacks4lw, Priority::Low);

    agent.acmd("game_attackhi4", game_attackhi4, Priority::Low);
    agent.acmd("effect_attackhi4", effect_attackhi4, Priority::Low);
    agent.acmd("sound_attackhi4", sound_attackhi4, Priority::Low);
    agent.acmd("expression_attackhi4", expression_attackhi4, Priority::Low);

    agent.acmd("game_attacklw4", game_attacklw4, Priority::Low);
    agent.acmd("effect_attacklw4", effect_attacklw4, Priority::Low);
    agent.acmd("expression_attacklw4", expression_attacklw4, Priority::Low);
}use super::*;

unsafe extern "C" fn game_attackairn(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 1.0);
    FT_MOTION_RATE(agent, 0.5);
    if is_excute(agent) {
        WorkModule::on_flag(boma, *FIGHTER_STATUS_ATTACK_AIR_FLAG_ENABLE_LANDING);
    }
    frame(lua_state, 4.0);
    FT_MOTION_RATE(agent, 1.0);
    frame(lua_state, 6.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("top"), 14.0, 361, 105, 0, 15, 4.25, 0.0, 10.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_cutup"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_CUTUP, *ATTACK_REGION_BODY);
        ATTACK(agent, 1, 0, Hash40::new("top"), 11.0, 361, 105, 0, 15, 6.0, 0.0, 10.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_cutup"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_CUTUP, *ATTACK_REGION_BODY);
    }
    frame(lua_state, 10.0);
    if is_excute(agent) {
        AttackModule::clear(boma,1,false);
        ATTACK(agent, 0, 0, Hash40::new("top"), 8.0, 361, 95, 0, 25, 4.0, 0.0, 10.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_cutup"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_CUTUP, *ATTACK_REGION_BODY);
    }
    frame(lua_state, 20.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("top"), 5.0, 361, 100, 0, 25, 3.5, 0.0, 10.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_cutup"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_CUTUP, *ATTACK_REGION_BODY);
    }
    frame(lua_state, 24.0);
    if is_excute(agent) {
        AttackModule::clear_all(boma);
    }
    frame(lua_state, 25.0);
    FT_MOTION_RATE(agent, 0.33);
    frame(lua_state, 45.0);
    FT_MOTION_RATE(agent, 1.0);
    frame(lua_state, 48.0);
    if is_excute(agent) {
        WorkModule::off_flag(boma, *FIGHTER_STATUS_ATTACK_AIR_FLAG_ENABLE_LANDING);
    }
}

unsafe extern "C" fn game_attackairf(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    FT_MOTION_RATE(agent, 5.0/6.0);
    frame(lua_state, 4.8);  // effectively f4
    if is_excute(agent) {
        WorkModule::on_flag(boma, *FIGHTER_STATUS_ATTACK_AIR_FLAG_ENABLE_LANDING);
    }
    frame(lua_state, 8.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("kneer"), 9.0, 55, 100, 0, 30, 3.0, -2.0, 0.3, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 1, 0, Hash40::new("footr"), 11.0, 361, 90, 0, 45, 5.0, 1.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
    }
    frame(lua_state, 9.6);
    FT_MOTION_RATE(agent, 1.0);
    frame(lua_state, 10.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("kneer"),  9.0,  55, 100, 0, 30, 3.0, -2.0, 0.3, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 1, 0, Hash40::new("footr"), 11.0, 270,  82, 0,  0, 5.0,  1.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
    }
    frame(lua_state, 14.0);
    if is_excute(agent) {
        AttackModule::clear_all(boma);
    }
    frame(lua_state, 33.0);
    if is_excute(agent) {
        WorkModule::off_flag(boma, *FIGHTER_STATUS_ATTACK_AIR_FLAG_ENABLE_LANDING);
    }
    frame(lua_state, 34.0);
    FT_MOTION_RATE(agent, 0.5);
    frame(lua_state, 40.0);
    FT_MOTION_RATE(agent, 1.0);
}

unsafe extern "C" fn effect_attackairf(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 9.0);
    if is_excute(agent) {
        EFFECT_FOLLOW(agent, Hash40::new("sys_attack_arc_d"), Hash40::new("top"), 0, 9.5, 1.0, 180, -150 , 90, 0.92, true);
        LAST_EFFECT_SET_COLOR(agent, 3.0, 0.15, 0.15);
        LAST_EFFECT_SET_RATE(agent, 2.0);
    }
    frame(lua_state, 12.0);
    if is_excute(agent) {
        EFFECT_FOLLOW_ALPHA(agent, Hash40::new("sys_attack_impact"), Hash40::new("footr"), 0, 0, 0, 0, 0, 0, 1.3, true, 0.9);
        LAST_EFFECT_SET_RATE(agent, 2);
    }
}

unsafe extern "C" fn sound_attackairf(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 9.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_sonic_swing_l"));
    }
    frame(lua_state, 13.0);
    if is_excute(agent) {
        PLAY_SEQUENCE(agent, Hash40::new("seq_sonic_rnd_attack"));
        PLAY_SE(agent, Hash40::new("se_sonic_attackair_f03"));
    }
}

unsafe extern "C" fn expression_attackairf(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 7.0);
    if is_excute(agent) {
        ControlModule::set_rumble(boma, Hash40::new("rbkind_nohitm"), 0, false, *BATTLE_OBJECT_ID_INVALID as u32);
    }
    frame(lua_state, 8.0);
    if is_excute(agent) {
        RUMBLE_HIT(agent, Hash40::new("rbkind_attackm"), 0);
    }
}

unsafe extern "C" fn game_attackairb(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 1.0);
    FT_MOTION_RATE(agent, 0.7);
    frame(lua_state, 5.0);
    FT_MOTION_RATE(agent, 0.75);
    if is_excute(agent) {
        WorkModule::on_flag(boma, *FIGHTER_STATUS_ATTACK_AIR_FLAG_ENABLE_LANDING);
    }
    frame(lua_state, 15.0);
    FT_MOTION_RATE(agent, 1.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("hip"), 11.0, 361, 94, 0, 30, 4.0, 0.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 1, 0, Hash40::new("kneer"), 14.0, 361, 94, 0, 30, 5.0, 0.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 2, 0, Hash40::new("kneer"), 14.0, 361, 94, 0, 30, 5.5, 4.5, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
    }
    frame(lua_state, 17.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("hip"), 7.0, 361, 80, 0, 40, 3.8, 0.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 1, 0, Hash40::new("kneer"), 10.0, 361, 80, 0, 40, 4.8, 0.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 2, 0, Hash40::new("kneer"), 10.0, 361, 80, 0, 40, 5.3, 4.5, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
    }
    frame(lua_state, 22.0);
    if is_excute(agent) {
        AttackModule::clear_all(boma);
    }
    frame(lua_state, 35.0);
    if is_excute(agent) {
        WorkModule::off_flag(boma, *FIGHTER_STATUS_ATTACK_AIR_FLAG_ENABLE_LANDING);
    }
}

unsafe extern "C" fn effect_attackairb(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();

    frame(lua_state, 13.0);
    if is_excute(agent) {
        EFFECT_FOLLOW_FLIP(agent, Hash40::new("sys_attack_arc"), Hash40::new("sys_attack_arc"), Hash40::new("top"), 0, 6.0, -4.0, 165, 65, 150, 1.1, true, *EF_FLIP_YZ);
	    LAST_EFFECT_SET_RATE(agent, 1.75);
    }
    frame(lua_state, 23.0);
    if is_excute(agent) {
        EFFECT_OFF_KIND(agent, Hash40::new("sys_attack_arc"), true, true);
    }
}

unsafe extern "C" fn game_attackairhi(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    FT_MOTION_RATE(agent, 5.0/(4.0-0.0));
    frame(lua_state, 3.0);
    if is_excute(agent) {
        WorkModule::on_flag(boma, *FIGHTER_STATUS_ATTACK_AIR_FLAG_ENABLE_LANDING);
    }
    frame(lua_state, 4.0);
    FT_MOTION_RATE(agent, 1.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("top"), 4.0, 110, 100, 45, 0, 3.5, 0.0, 7.0, 1.5, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 1, 0, Hash40::new("top"), 4.0, 110, 100, 45, 0, 3.5, 0.0, 7.0, -1.5, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 2, 0, Hash40::new("top"), 4.0, 123, 100, 60, 0, 4.0, 0.0, 8.0, 7.5, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 3, 0, Hash40::new("top"), 4.0, 123, 100, 60, 0, 4.0, 0.0, 8.0, -7.5, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
    }
    frame(lua_state, 7.0);
    FT_MOTION_RATE(agent, 0.333);
    if is_excute(agent) {
        AttackModule::clear_all(boma);
    }
    frame(lua_state, 13.0);
    FT_MOTION_RATE(agent, 1.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("top"), 10.0, 90, 85, 0, 52, 5.0, 0.0, 19.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 1, 0, Hash40::new("top"), 10.0, 90, 85, 0, 52, 4.5, 0.0, 15.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 2, 0, Hash40::new("top"), 10.0, 90, 85, 0, 52, 4.0, 0.0, 11.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
    }
    frame(lua_state, 18.0);
    if is_excute(agent) {
        AttackModule::clear_all(boma);
    }
    frame(lua_state, 36.0);
    if is_excute(agent) {
        WorkModule::off_flag(boma, *FIGHTER_STATUS_ATTACK_AIR_FLAG_ENABLE_LANDING);
    }
}

unsafe extern "C" fn effect_attackairhi(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 4.0);
    if is_excute(agent) {
        EFFECT_FOLLOW(agent, Hash40::new("sys_attack_speedline"), Hash40::new("top"), 0, 7.5, 2.5, -10, 0, 0, 0.5, true);
        LAST_EFFECT_SET_COLOR(agent, 3.0, 0.7, 0.2);
        LAST_EFFECT_SET_COLOR(agent, 3.0, 0.15, 0.15);
        EFFECT_FOLLOW(agent, Hash40::new("sys_attack_speedline"), Hash40::new("top"), -1, 7.5, -2.5, 190, 0, 0, 0.5, true);
        LAST_EFFECT_SET_COLOR(agent, 3.0, 0.7, 0.2);
        LAST_EFFECT_SET_COLOR(agent, 3.0, 0.15, 0.15);
    }
    frame(lua_state, 12.0);
    if is_excute(agent) {
        EFFECT_FOLLOW_NO_STOP_FLIP(agent, Hash40::new("sonic_atk_hi"), Hash40::new("sonic_atk_hi"), Hash40::new("top"), 0.85, 11.5, 0, 59, 74, 154, 1.1, true, *EF_FLIP_YZ);
        LAST_EFFECT_SET_COLOR(agent, 3.0, 0.7, 0.2);
        LAST_EFFECT_SET_COLOR(agent, 3.0, 0.15, 0.15);
        EFFECT_FOLLOW_NO_STOP_FLIP(agent, Hash40::new("sonic_atk_hi"), Hash40::new("sonic_atk_hi"), Hash40::new("top"), -0.95, 11.5, 0, 245, -77, 22, 1.1, true, *EF_FLIP_YZ);
        LAST_EFFECT_SET_COLOR(agent, 3.0, 0.7, 0.2);
        LAST_EFFECT_SET_COLOR(agent, 3.0, 0.15, 0.15);
    }
    frame(lua_state, 14.0);
    if is_excute(agent) {
        EFFECT(agent, Hash40::new("sys_attack_impact"), Hash40::new("top"), 0.0, 19.5, 0.0, 0, 0, 0, 1.55, 0, 0, 0, 0, 0, 361, true);
    }
}

unsafe extern "C" fn expression_attackairhi(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 4.0);
    if is_excute(agent) {
        RUMBLE_HIT(agent, Hash40::new("rbkind_attacks"), 0);
    }
    frame(lua_state, 10.0);
    if is_excute(agent) {
        ControlModule::set_rumble(boma, Hash40::new("rbkind_nohitm"), 0, false, *BATTLE_OBJECT_ID_INVALID as u32);
    }
    frame(lua_state, 13.0);
    if is_excute(agent) {
        RUMBLE_HIT(agent, Hash40::new("rbkind_attackm"), 0);
    }
}

unsafe extern "C" fn game_attackairlw(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    FT_MOTION_RATE(agent, 10.0/(16.0-0.0));
    frame(lua_state, 6.0);
    if is_excute(agent) {
        WorkModule::on_flag(boma, *FIGHTER_STATUS_ATTACK_AIR_FLAG_ENABLE_LANDING);
    }
    frame(lua_state, 16.0);
    FT_MOTION_RATE(agent, 1.0);
    if is_excute(agent) {
        WorkModule::on_flag(boma, *FIGHTER_INSTANCE_WORK_ID_FLAG_NO_SPEED_OPERATION_CHK);
        SET_SPEED_EX(agent, 1.4, -4, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN);
        WorkModule::off_flag(boma, *FIGHTER_INSTANCE_WORK_ID_FLAG_NO_SPEED_OPERATION_CHK);
        WorkModule::on_flag(boma, *FIGHTER_STATUS_WORK_ID_FLAG_RESERVE_GRAVITY_STABLE_UNABLE);
    }
    frame(lua_state, 16.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("top"), 10.0, 287, 92, 0, 10, 5.0, -4.0, 5.0, 1.8, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_A, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 1, 0, Hash40::new("top"), 10.0, 37, 88, 0, 45, 4.1, 0.0, -0.3, 3.1, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_G, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
    }
    frame(lua_state, 19.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("top"), 9.0, 60, 66, 0, 30, 5.0, -4.0, 5.0, 1.8, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_A, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 1, 0, Hash40::new("top"), 9.0, 361, 88, 0, 45, 4.1, 0.0, 1.3, 3.1, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_G, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
    }
    frame(lua_state, 22.0);
    if is_excute(agent) {
        AttackModule::clear(boma, 1, false);
        ATTACK(agent, 0, 0, Hash40::new("top"), 6.0, 361, 75, 0, 45, 4.3, -4.0, 6.0, 1.8, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
    }
    frame(lua_state, 33.0);
    FT_MOTION_RATE(agent, 14.0/(49.0-33.0));
    if is_excute(agent) {
        AttackModule::clear_all(boma);
    }
    frame(lua_state, 40.0);
    if is_excute(agent) {
        WorkModule::off_flag(boma, *FIGHTER_STATUS_ATTACK_AIR_FLAG_ENABLE_LANDING);
    }
    frame(lua_state, 61.0);
    if is_excute(agent) {
        WorkModule::off_flag(boma, *FIGHTER_STATUS_WORK_ID_FLAG_RESERVE_GRAVITY_STABLE_UNABLE);
        KineticModule::resume_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_CONTROL);
    }
}

pub fn install(agent: &mut Agent) {
    agent.acmd("game_attackairn", game_attackairn, Priority::Low);

    agent.acmd("game_attackairf", game_attackairf, Priority::Low);
    agent.acmd("effect_attackairf", effect_attackairf, Priority::Low);
    agent.acmd("sound_attackairf", sound_attackairf, Priority::Low);
    agent.acmd("expression_attackairf", expression_attackairf, Priority::Low);

    agent.acmd("game_attackairb", game_attackairb, Priority::Low);
    agent.acmd("effect_attackairb", effect_attackairb, Priority::Low);

    agent.acmd("game_attackairhi", game_attackairhi, Priority::Low);
    agent.acmd("effect_attackairhi", effect_attackairhi, Priority::Low);
    agent.acmd("expression_attackairhi", expression_attackairhi, Priority::Low);

    agent.acmd("game_attackairlw", game_attackairlw, Priority::Low);
}
use super::*;

mod ground;
mod tilts;
mod smashes;
mod aerials;
mod specials;
mod throws;
mod other;

pub fn install(agent: &mut Agent) {
    ground::install(agent);
    tilts::install(agent);
    smashes::install(agent);
    aerials::install(agent);
    specials::install(agent);
    throws::install(agent);
    other::install(agent);
}use super::*;

unsafe extern "C" fn game_attacks3(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    if is_excute(agent) {
        JostleModule::set_status(boma, false);
    }
    frame(lua_state, 7.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("kneel"), 11.0, 361, 90, 0, 20, 4.5, 6.0, 0.0, -1.6, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 1, 0, Hash40::new("kneel"), 9.0, 361, 90, 0, 20, 3.5, 0.0, 0.0, -1.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 2, 0, Hash40::new("legl"), 9.0, 361, 90, 0, 20, 3.5, -1.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
    }
    frame(lua_state, 12.0);
    if is_excute(agent) {
        AttackModule::clear_all(boma);
    }
    frame(lua_state, 26.0);
    if is_excute(agent) {
        JostleModule::set_status(boma, true);
    }
}

unsafe extern "C" fn effect_attacks3(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 4.0);
    if is_excute(agent) {
        LANDING_EFFECT(agent, Hash40::new("sys_atk_smoke"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.5, 0, 0, 0, 0, 0, 0, false);
    }
    frame(lua_state, 7.0);
    if is_excute(agent) {
        EFFECT_FOLLOW_FLIP(agent, Hash40::new("sys_attack_speedline"), Hash40::new("sys_attack_speedline"), Hash40::new("top"), 0, 1, 1, -22, 0, 0, 1.1, true, *EF_FLIP_YZ);
        LAST_EFFECT_SET_COLOR(agent, 3, 0.7, 0.2);
    }
    frame(lua_state, 8.0);
    if is_excute(agent) {
        EFFECT(agent, Hash40::new("sys_attack_impact"), Hash40::new("top"), 0, 9, 19.5, 0, 0, 0, 1.1, 0, 0, 0, 0, 0, 360, true);
    }
}

unsafe extern "C" fn game_attacks3hi(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    if is_excute(agent) {
        JostleModule::set_status(boma, false);
    }
    frame(lua_state, 7.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("kneel"), 11.0, 361, 90, 0, 20, 4.5, 6.0, 0.0, -1.6, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 1, 0, Hash40::new("kneel"), 9.0, 361, 90, 0, 20, 3.5, 0.0, 0.0, -1.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 2, 0, Hash40::new("legl"), 9.0, 361, 90, 0, 20, 3.5, -1.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
    }
    frame(lua_state, 12.0);
    if is_excute(agent) {
        AttackModule::clear_all(boma);
    }
    frame(lua_state, 26.0);
    if is_excute(agent) {
        JostleModule::set_status(boma, true);
    }
}

unsafe extern "C" fn effect_attacks3hi(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 4.0);
    if is_excute(agent) {
        LANDING_EFFECT(agent, Hash40::new("sys_atk_smoke"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.4, 0, 0, 0, 0, 0, 0, false);
    }
    frame(lua_state, 7.0);
    if is_excute(agent) {
        EFFECT_FOLLOW_FLIP(agent, Hash40::new("sys_attack_speedline"), Hash40::new("sys_attack_speedline"), Hash40::new("top"), 0, 0, 1, -42, 0, 0, 1.1, true, *EF_FLIP_YZ);
        LAST_EFFECT_SET_COLOR(agent, 3, 0.7, 0.2);
    }
    frame(lua_state, 8.0);
    if is_excute(agent) {
        EFFECT(agent, Hash40::new("sys_attack_impact"), Hash40::new("top"), 0, 13.2, 16.2, 0, 0, 0, 1.1, 0, 0, 0, 0, 0, 360, true);
    }
}

unsafe extern "C" fn game_attacks3lw(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    if is_excute(agent) {
        JostleModule::set_status(boma, false);
    }
   frame(lua_state, 7.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("kneel"), 11.0, 361, 90, 0, 20, 4.5, 6.0, 0.0, -1.6, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 1, 0, Hash40::new("kneel"), 9.0, 361, 90, 0, 20, 3.5, 0.0, 0.0, -1.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 2, 0, Hash40::new("legl"), 9.0, 361, 90, 0, 20, 3.5, -1.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        AttackModule::set_attack_height_all(boma, AttackHeight(*ATTACK_HEIGHT_LOW), false);
    }
    frame(lua_state, 12.0);
    if is_excute(agent) {
        AttackModule::clear_all(boma);
    }
    frame(lua_state, 26.0);
    if is_excute(agent) {
        JostleModule::set_status(boma, true);
    }
}

unsafe extern "C" fn effect_attacks3lw(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 4.0);
    if is_excute(agent) {
        LANDING_EFFECT(agent, Hash40::new("sys_atk_smoke"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.4, 0, 0, 0, 0, 0, 0, false);
    }
    frame(lua_state, 7.0);
    if is_excute(agent) {
        EFFECT_FOLLOW_FLIP(agent, Hash40::new("sys_attack_speedline"), Hash40::new("sys_attack_speedline"), Hash40::new("top"), 0, 3.5, 0, -5, 0, 0, 1.1, true, *EF_FLIP_YZ);
        LAST_EFFECT_SET_COLOR(agent, 3, 0.7, 0.2);
    }
    frame(lua_state, 8.0);
    if is_excute(agent) {
        EFFECT(agent, Hash40::new("sys_attack_impact"), Hash40::new("top"), 0, 6, 19.5, 0, 0, 0, 1.1, 0, 0, 0, 0, 0, 360, true);
    }
}

unsafe extern "C" fn game_attackhi3(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 8.0);
    if is_excute(agent) {
        // ground-only
        ATTACK(agent, 0, 0, Hash40::new("legr"), 2.0, 80, 100, 35, 0, 3.5, 0.0, 0.0, 0.0, None, None, None, 1.0, 0.5, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_G, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 1, 0, Hash40::new("kneer"), 2.0, 80, 100, 35, 0, 4.0, 2.0, 0.0, 0.0, None, None, None, 1.0, 0.5, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_G, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 2, 0, Hash40::new("kneer"), 2.0, 361, 100, 45, 0, 4.5, 7.0, 0.0, -1.5, None, None, None, 1.0, 0.5, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_G, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        // air-only
        ATTACK(agent, 3, 0, Hash40::new("legr"), 2.0, 366, 100, 35, 0, 3.5, 0.0, 0.0, 0.0, None, None, None, 1.0, 0.5, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_A, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 4, 0, Hash40::new("kneer"), 2.0, 366, 100, 35, 0, 4.0, 2.0, 0.0, 0.0, None, None, None, 1.0, 0.5, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_A, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 5, 0, Hash40::new("kneer"), 2.0, 366, 100, 45, 0, 4.5, 7.0, 0.0, -1.5, None, None, None, 1.0, 0.5, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_A, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
    }
    frame(lua_state, 13.0);
    if is_excute(agent) {
        AttackModule::clear_all(boma);
    }
    frame(lua_state, 13.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("legl"), 6.0, 75, 120, 0, 40, 3.5, 0.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 1, 0, Hash40::new("kneel"), 6.0, 75, 120, 0, 40, 4.0, 2.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 2, 0, Hash40::new("kneel"), 6.0, 75, 120, 0, 40, 4.5, 7.0, 0.0, 1.5, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
    }
    frame(lua_state, 22.0);
    if is_excute(agent) {
        AttackModule::clear_all(boma);
    }
}

unsafe extern "C" fn effect_attackhi3(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 6.0);
    if is_excute(agent) {
        LANDING_EFFECT(agent, Hash40::new("sys_h_smoke_a"), Hash40::new("top"), 2, 0, 0, 0, 0, 0, 0.7, 0, 0, 0, 0, 0, 0, false);
    }
    frame(lua_state, 8.0);
    if is_excute(agent) {
        EFFECT_FOLLOW_FLIP(agent, Hash40::new("sys_attack_arc_b"), Hash40::new("sys_attack_arc_b"), Hash40::new("top"), 2, 8, 4, 180, 325, 90, 1.4, true, *EF_FLIP_YZ);
        LAST_EFFECT_SET_RATE(agent, 0.55);
    }
    frame(lua_state, 14.0);
    if is_excute(agent) {
        EFFECT_FOLLOW_FLIP(agent, Hash40::new("sys_attack_arc"), Hash40::new("sys_attack_arc"), Hash40::new("top"), 2, 8.5, -0.2, 180, 260, 90, 1.15, true, *EF_FLIP_YZ);
        LAST_EFFECT_SET_RATE(agent, 0.7);
    }
    frame(lua_state, 16.0);
    if is_excute(agent) {
        EFFECT_OFF_KIND(agent, Hash40::new("sys_attack_arc_b"), false, true);
    }
    frame(lua_state, 24.0);
    if is_excute(agent) {
        EFFECT_OFF_KIND(agent, Hash40::new("sys_attack_arc"), false, true);
    }
    frame(lua_state, 27.0);
    if is_excute(agent) {
        FOOT_EFFECT(agent, Hash40::new("sys_landing_smoke_s"), Hash40::new("top"), -1.5, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, false);
    }
}

unsafe extern "C" fn game_attacklw3(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 3.0);
    if is_excute(agent) {
        JostleModule::set_status(boma, false);
    }
    frame(lua_state, 6.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("legr"), 6.0, 100, 100, 0, 50, 3.0, 0.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 1, 0, Hash40::new("kneer"), 6.0, 100, 100, 0, 50, 3.5, 0.5, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 2, 0, Hash40::new("kneer"), 6.0, 106, 106, 0, 50, 4.0, 6.5, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        AttackModule::set_attack_height_all(boma, app::AttackHeight(*ATTACK_HEIGHT_LOW), false);
    }
    frame(lua_state, 9.0);
    if is_excute(agent) {
        AttackModule::clear_all(boma);
        JostleModule::set_status(boma, true);
    }
}

unsafe extern "C" fn effect_attacklw3(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 5.0);
    if is_excute(agent) {
        EFFECT_FOLLOW_FLIP(agent, Hash40::new("sys_attack_arc_d"), Hash40::new("sys_attack_arc_d"), Hash40::new("top"), 5, 2, 2, 1, 5, 5, 0.925, true, *EF_FLIP_YZ);
        LAST_EFFECT_SET_RATE(agent, 1.5);
    }
    frame(lua_state, 6.0);
    if is_excute(agent) {
        FOOT_EFFECT(agent, Hash40::new("sys_run_smoke"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, false);
    }
}

pub fn install(agent: &mut Agent) {
    agent.acmd("game_attacks3", game_attacks3, Priority::Low);
    agent.acmd("effect_attacks3", effect_attacks3, Priority::Low);
    agent.acmd("game_attacks3hi", game_attacks3hi, Priority::Low);
    agent.acmd("effect_attacks3hi", effect_attacks3hi, Priority::Low);
    agent.acmd("game_attacks3lw", game_attacks3lw, Priority::Low);
    agent.acmd("effect_attacks3lw", effect_attacks3lw, Priority::Low);

    agent.acmd("game_attackhi3", game_attackhi3, Priority::Low);
    agent.acmd("effect_attackhi3", effect_attackhi3, Priority::Low);

    agent.acmd("game_attacklw3", game_attacklw3, Priority::Low);
    agent.acmd("effect_attacklw3", effect_attacklw3, Priority::Low);
}
use super::*;

unsafe extern "C" fn game_attack11(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 2.0);
    if is_excute(agent) {
        FighterAreaModuleImpl::enable_fix_jostle_area(boma, 3.0, 3.0);
    }
    frame(lua_state, 3.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("top"), 3.0, 85, 100, 20, 0, 2.0, 0.0, 7.0, 6.7, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA_d, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_PUNCH, *ATTACK_REGION_PUNCH);
        ATTACK(agent, 1, 0, Hash40::new("top"), 3.0, 83, 100, 20, 0, 2.5, 0.0, 7.0, 9.2, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA_d, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_PUNCH, *ATTACK_REGION_PUNCH);
        ATTACK(agent, 2, 0, Hash40::new("top"), 3.0, 83, 100, 20, 0, 3.0, 0.0, 7.0, 12.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA_d, *COLLISION_CATEGORY_MASK_FIGHTER, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_PUNCH, *ATTACK_REGION_PUNCH);
        ATTACK(agent, 3, 0, Hash40::new("top"), 3.0, 361, 30, 0, 20, 2.0, 0.0, 3.3, 8.0, Some(0.0), Some(3.3), Some(11.0), 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_G, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_PUNCH, *ATTACK_REGION_PUNCH);
    }
    frame(lua_state, 5.0);
    if is_excute(agent) {
        AttackModule::clear_all(boma);
    }
    frame(lua_state, 8.0);
    if is_excute(agent) {
        WorkModule::on_flag(boma,  *FIGHTER_STATUS_ATTACK_FLAG_ENABLE_COMBO);
    }
}

unsafe extern "C" fn game_attack12(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    FT_MOTION_RATE(agent, 4.0/6.0);
    frame(lua_state, 6.0);
    FT_MOTION_RATE(agent, 1.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("legr"), 3.0, 85, 25, 0, 20, 3.3, 4.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_PUNCH);
        ATTACK(agent, 1, 0, Hash40::new("kneer"), 3.0, 85, 25, 0, 20, 3.3, 0.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_PUNCH);
        ATTACK(agent, 2, 0, Hash40::new("kneer"), 3.0, 85, 25, 0, 20, 4.3, 4.0, 0.0, 1.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_PUNCH);
    }
    frame(lua_state, 9.0);
    if is_excute(agent) {
        AttackModule::clear_all(boma);
    }
    frame(lua_state, 10.0);
    if is_excute(agent) {
        WorkModule::on_flag(boma, *FIGHTER_STATUS_ATTACK_FLAG_ENABLE_COMBO);
    }
}

unsafe extern "C" fn effect_attack12(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 5.0);
    if is_excute(agent) {
        FOOT_EFFECT(agent, Hash40::new("null"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.8, 0, 0, 0, 0, 0, 0, false);
        EFFECT_FOLLOW(agent, Hash40::new("sys_attack_arc_b"), Hash40::new("footl"), -6, -5, 0, 90, 180, 60, 0.7, true);
    }
    frame(lua_state, 6.0);
    if is_excute(agent) {
        EFFECT(agent, Hash40::new("sys_attack_impact"), Hash40::new("top"), 13, 7.5, 0, 0, 0, 0, 0.9, 0, 0, 0, 0, 0, 360, false);
        LAST_EFFECT_SET_ALPHA(agent, 0.7);
    }
}

unsafe extern "C" fn game_attack13(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 5.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("hip"), 4.0, 361, 110, 0, 50, 3.0, 0.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 1, 0, Hash40::new("kneel"), 4.0, 361, 110, 0, 50, 3.5, 0.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 2, 0, Hash40::new("kneel"), 4.0, 361, 110, 0, 50, 5.0, 5.0, -1.5, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
    }
    frame(lua_state, 8.0);
    if is_excute(agent) {
        AttackModule::clear_all(boma);
    }
}

unsafe extern "C" fn effect_attack13(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 2.0);
    if is_excute(agent) {
        FOOT_EFFECT(agent, Hash40::new("sys_run_smoke"), Hash40::new("top"), -3, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, false);
    }
    frame(lua_state, 5.0);
    if is_excute(agent) {
        EFFECT_FOLLOW(agent, Hash40::new("sys_attack_arc_b"), Hash40::new("footr"), -7, 1.5, 0, 90, 180, -40, 0.8, true);
        EFFECT(agent, Hash40::new("sys_attack_impact"), Hash40::new("top"), 13, 8.5, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 360, false);
        LAST_EFFECT_SET_ALPHA(agent, 0.7);
    }
}

unsafe extern "C" fn game_attackdash(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    sv_kinetic_energy!(set_speed_mul, agent, FIGHTER_KINETIC_ENERGY_ID_MOTION, 0.8);
    frame(lua_state, 1.0);
    FT_MOTION_RATE(agent, 8.0/11.0);
    frame(lua_state, 11.0);
    FT_MOTION_RATE_RANGE(agent, 11.0, 21.0, 15.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("top"), 3.0, 361, 100, 30, 0, 4.5, 0.0, 4.5, 3.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
    }
    frame(lua_state, 14.0);
    if is_excute(agent) {
        AttackModule::clear_all(boma);
    }
    frame(lua_state, 16.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("kneel"), 6.0, 55, 130, 0, 40, 4.0, 3.5, 0.0, 2.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        ATTACK(agent, 1, 0, Hash40::new("top"), 6.0, 55, 130, 0, 40, 3.5, 0.0, 3.5, -3.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
    }
    frame(lua_state, 18.0);
    if is_excute(agent) {
        AttackModule::clear(boma, 1, false);
    }
    frame(lua_state, 20.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("kneel"), 4.0, 48, 130, 0, 40, 3.6, 3.5, 0.0, 2.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
    }
    frame(lua_state, 21.0);
    FT_MOTION_RATE_RANGE(agent, 21.0, 50.0, 38.0);
    frame(lua_state, 25.0);
    if is_excute(agent) {
        AttackModule::clear_all(boma);
    }
    frame(lua_state, 26.0);
    FT_MOTION_RATE(agent, 1.5);
    frame(lua_state, 30.0);
    FT_MOTION_RATE(agent, 1.0);
}

unsafe extern "C" fn effect_attackdash(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 2.0);
    if is_excute(agent) {
        LANDING_EFFECT(agent, Hash40::new("sys_dash_smoke"), Hash40::new("top"), -10, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, false);
        EFFECT_FOLLOW_FLIP(agent, Hash40::new("sys_attack_speedline"), Hash40::new("sys_attack_speedline"), Hash40::new("top"), 0, 11, -5, 0, 180, 0, 1.0, true, *EF_FLIP_YZ);
    }
    frame(lua_state, 11.0);
    if is_excute(agent) {
        LANDING_EFFECT(agent, Hash40::new("sys_down_smoke"), Hash40::new("top"), 4, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, false);
        FOOT_EFFECT(agent, Hash40::new("sys_turn_smoke"), Hash40::new("top"), 3, 0, 10, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, true);
        LAST_EFFECT_SET_RATE(agent, 1.3);
    }
    frame(lua_state, 16.5);
    if is_excute(agent) {
        EFFECT_FOLLOW(agent, Hash40::new("sys_attack_line"), Hash40::new("top"), 0, 0, -10, -15, 0, 0, 1.3, true);
    }
    frame(lua_state, 20.0);
    if is_excute(agent) {
        EFFECT_FOLLOW(agent, Hash40::new("sys_turn_smoke"), Hash40::new("top"), -1, 0, 10, 0, 0, 0, 1.4, true);
    }
}

unsafe extern "C" fn sound_attackdash(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.boma();
    frame(lua_state, 4.0);
    if is_excute(agent) {
        PLAY_SEQUENCE(agent, Hash40::new("seq_sonic_rnd_attack"));
    }
    frame(lua_state, 5.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_sonic_swing_ll"));
    }
    frame(lua_state, 16.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_sonic_swing_l"));
    }
}

pub fn install(agent: &mut Agent) {
    agent.acmd("game_attack11", game_attack11, Priority::Low);
    agent.acmd("game_attack12", game_attack12, Priority::Low);
    agent.acmd("effect_attack12", effect_attack12, Priority::Low);
    agent.acmd("game_attack13", game_attack13, Priority::Low);
    agent.acmd("effect_attack13", effect_attack13, Priority::Low);

    agent.acmd("game_attackdash", game_attackdash, Priority::Low);
    agent.acmd("effect_attackdash", effect_attackdash, Priority::Low);
    agent.acmd("sound_attackdash", sound_attackdash, Priority::Low);
}