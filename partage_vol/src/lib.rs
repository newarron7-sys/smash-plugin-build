#![feature(proc_macro_hygiene)]
#![allow(non_snake_case, static_mut_refs, unused_unsafe, unused_must_use)]

//! Free flight on Up Special (smashline 2) — from the Super Sonic moveset, shared as a standalone example.
//!
//! What it does:
//! - take-off: shoots straight up very fast (3.0 -> FLY_MAX_VY over TAKEOFF frames), standing pose;
//! - free flight for FLY_FRAMES frames: steered with the stick in every direction, hovers in place with no input
//!   (gravity off, the STOP energy is set every frame); holding back turns the fighter around;
//! - poses: the fighter's "wait" motion when hovering / going up / down, a "lying" motion (FLY_SIDE_MOTION) when
//!   dashing sideways, tilted along the flight slope with the "rot" joint;
//! - end: slides to a stop with dust if it touches the ground, otherwise falls normally (can still act);
//! - once per airtime (reset on ground, ledge grab, getting launched, death);
//! - effects: golden aura in the flight direction, a trail, speed lines, a flash at take-off.
//!
//! Every value below is a plain tuning constant: change them freely. Put your fighter's name in FIGHTER and the
//! costume slots in COSTUMES (leave empty for all slots). It is pure movement: no hitbox.

use smash::app::lua_bind::*;
use smash::app::*;
use smash::lib::lua_const::*;
use smash::lib::L2CValue;
use smash::lua2cpp::*;
use smash::phx::{Hash40, Vector3f};
use smash_script::*;
use smashline::*;

// ------------------------------------------------------------------ settings
const FIGHTER: &str = "sonic";
const COSTUMES: &[usize] = &[];
/// take-off length (frames), then free flight length (frames)
const TAKEOFF: i32 = 20;
const FLY_FRAMES: i32 = 150;
/// take-off starting vertical speed
const TAKEOFF_VY: f32 = 3.0;
/// free flight top speeds
const FLY_MAX_VY: f32 = 1.6;
const FLY_MAX_VX: f32 = 2.0;
/// acceleration while the stick is held, braking with no input (per frame)
const ACCEL_Y: f32 = 0.2;
const ACCEL_X: f32 = 0.1;
const BRAKE_Y: f32 = 0.12;
const BRAKE_X: f32 = 0.04;
/// motion used when flying sideways ("fall" works on every fighter; use your own if you have one) and the frame
/// range of it to loop
const FLY_SIDE_MOTION: &str = "fall";
const FLY_SIDE_LOOP: (f32, f32) = (0.0, 20.0);
/// effect colour (golden)
const TINT: (f32, f32, f32) = (1.0, 0.78, 0.22);
/// trail effect (any effect your fighter has; "" for none)
const TRAIL: &str = "";

// ------------------------------------------------------------------ state per player
static mut T: [i32; 8] = [0; 8];
static mut V: [(f32, f32); 8] = [(0.0, 0.0); 8];
static mut USED: [bool; 8] = [false; 8];
/// effect handles: aura (0), trail (1); aura direction (0 none, 1 up, 2 side, 3 down)
static mut FX: [[u32; 2]; 8] = [[0; 2]; 8];
static mut DIR: [u8; 8] = [0; 8];

unsafe fn slot(boma: *mut BattleObjectModuleAccessor) -> usize {
    (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7)
}

/// speed set directly on the STOP energy (gravity off); x is in the facing direction
unsafe fn set_speed(fighter: &mut L2CFighterCommon, vx: f32, vy: f32) {
    let lr = PostureModule::lr(fighter.module_accessor);
    sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, vx * lr, vy);
}

unsafe fn go_airborne(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    StatusModule::set_situation_kind(boma, SituationKind(*SITUATION_KIND_AIR), true);
    GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
    KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_AIR_STOP);
    KineticModule::unable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    // free STOP energy with no speed cap (otherwise its vertical speed is clamped to 0 and you can't climb)
    sv_kinetic_energy!(reset_energy, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, *ENERGY_STOP_RESET_TYPE_FREE, 0.0, 0.0, 0.0, 0.0, 0.0);
    sv_kinetic_energy!(set_accel, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0, 0.0);
    sv_kinetic_energy!(set_stable_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0, 0.0);
    sv_kinetic_energy!(set_limit_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, -1.0, -1.0);
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_STOP);
}

/// back to normal physics (falling) with the given vertical speed
unsafe fn normal_fall(fighter: &mut L2CFighterCommon, vy: f32) {
    let boma = fighter.module_accessor;
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_FALL);
    sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY, vy);
}

unsafe fn fx(boma: *mut BattleObjectModuleAccessor, eff: &str, p: (f32, f32, f32), r: (f32, f32, f32), sc: f32, tint: bool) -> u32 {
    let p = Vector3f { x: p.0, y: p.1, z: p.2 };
    let r = Vector3f { x: r.0, y: r.1, z: r.2 };
    let n = EffectModule::req_follow(boma, Hash40::new(eff), Hash40::new("top"), &p, &r, sc, false, 0, 0, 0, 0, 0, false, false) as u32;
    if n != 0 && tint {
        EffectModule::set_rgb(boma, n, TINT.0, TINT.1, TINT.2);
    }
    n
}

unsafe fn fx_off(boma: *mut BattleObjectModuleAccessor, e: usize, k: usize) {
    let h = FX[e][k];
    if h != 0 && EffectModule::is_exist_effect(boma, h) {
        EffectModule::kill(boma, h, false, false);
    }
    FX[e][k] = 0;
}

/// every frame: aura pointing where it flies, trail, speed lines, a small ring now and then
unsafe fn flight_effects(boma: *mut BattleObjectModuleAccessor, e: usize, vx: f32, vy: f32) {
    let t = T[e];
    let dir: u8 = if vx.abs() < 0.5 && vy.abs() < 0.5 {
        0
    } else if vx.abs() >= vy.abs() {
        2
    } else if vy > 0.0 {
        1
    } else {
        3
    };
    if dir != DIR[e] || (dir != 0 && (FX[e][0] == 0 || !EffectModule::is_exist_effect(boma, FX[e][0]))) {
        fx_off(boma, e, 0);
        DIR[e] = dir;
        FX[e][0] = match dir {
            // up / down: a cup around the feet; sideways: laid down in front of him
            1 | 3 => fx(boma, "sys_score_aura", (0.0, -1.0, 0.0), (0.0, 0.0, 0.0), 1.1, true),
            2 => fx(boma, "sys_score_aura", (0.0, 6.0, 9.0), (-90.0, 0.0, 0.0), 1.0, true),
            _ => 0,
        };
    }
    if !TRAIL.is_empty() && (FX[e][1] == 0 || !EffectModule::is_exist_effect(boma, FX[e][1])) {
        let n = fx(boma, TRAIL, (0.0, 7.0, 0.0), (0.0, 0.0, 0.0), 0.8, false);
        if n != 0 {
            EffectModule::set_alpha(boma, n, 0.7);
        }
        FX[e][1] = n;
    }
    let step = if t <= TAKEOFF { 3 } else { 6 };
    if dir != 0 && t % step == 1 {
        match dir {
            1 => { fx(boma, "sys_attack_speedline", (0.0, 2.0, 0.0), (-90.0, 0.0, 0.0), 1.3, true); }
            3 => { fx(boma, "sys_attack_speedline", (0.0, 14.0, 0.0), (90.0, 0.0, 0.0), 1.2, true); }
            _ => { fx(boma, "sys_attack_speedline", (0.0, 8.0, -4.0), (0.0, 0.0, 0.0), 1.2, true); }
        }
    }
    if t > TAKEOFF && t % 40 == 0 {
        fx(boma, "sys_smash_flash_s", (0.0, 7.0, 0.0), (0.0, 0.0, 0.0), 1.4, false);
    }
}

unsafe extern "C" fn hi_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    let grounded = StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND;
    USED[e] = true;
    T[e] = 0;
    DIR[e] = 0;
    FX[e] = [0; 2];
    V[e] = (0.0, TAKEOFF_VY);
    go_airborne(fighter);
    set_speed(fighter, V[e].0, V[e].1);
    let flash = fx(boma, "sys_smash_flash", (0.0, 6.0, 0.0), (0.0, 0.0, 0.0), 1.6, true);
    let _ = flash;
    if grounded {
        fx(boma, "sys_landing_smoke", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 1.2, false);
    }
    MotionModule::change_motion(boma, Hash40::new("wait"), 0.0, 1.0, false, 0.0, false, false);
    fighter.sub_shift_status_main(L2CValue::Ptr(hi_loop as *const () as _))
}

unsafe extern "C" fn hi_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    if StopModule::is_stop(boma) {
        return 0.into();
    }
    T[e] += 1;
    let t = T[e];
    let lr = PostureModule::lr(boma);
    let mut sx = ControlModule::get_stick_x(boma) * lr;
    let sy = ControlModule::get_stick_y(boma);
    let (mut vx, mut vy) = V[e];
    // holding back: turn around (the aura follows)
    if t > TAKEOFF && sx < -0.3 {
        PostureModule::reverse_lr(boma);
        PostureModule::update_rot_y_lr(boma);
        sx = -sx;
        vx = -vx;
        DIR[e] = 0;
        fx_off(boma, e, 0);
    }
    if t <= TAKEOFF {
        // take-off: climbs from TAKEOFF_VY down to the flight speed
        vy = TAKEOFF_VY - (TAKEOFF_VY - FLY_MAX_VY) * (t as f32 / TAKEOFF as f32);
    } else if sy > 0.2 {
        if vy < FLY_MAX_VY { vy += ACCEL_Y; }
    } else if sy < -0.2 {
        if vy > -FLY_MAX_VY { vy -= ACCEL_Y; }
    } else if vy > 0.0 {
        vy = (vy - BRAKE_Y).max(0.0);
    } else {
        vy = (vy + BRAKE_Y).min(0.0);
    }
    if sx > 0.2 {
        if vx < FLY_MAX_VX { vx += ACCEL_X; }
    } else if sx < -0.2 {
        if vx > -FLY_MAX_VX { vx -= ACCEL_X; }
    } else if vx > 0.0 {
        vx = (vx - BRAKE_X).max(0.0);
    } else {
        vx = (vx + BRAKE_X).min(0.0);
    }
    V[e] = (vx, vy);
    set_speed(fighter, vx, vy);
    // poses: wait when hovering / going up / down; lying when dashing sideways, tilted along the slope
    let lying = t > TAKEOFF && vx.abs() > 0.9 && sx.abs() > 0.3;
    if lying {
        let fm = MotionModule::frame(boma);
        if MotionModule::motion_kind(boma) != smash::hash40(FLY_SIDE_MOTION) || fm < FLY_SIDE_LOOP.0 || fm >= FLY_SIDE_LOOP.1 - 1.0 {
            MotionModule::change_motion(boma, Hash40::new(FLY_SIDE_MOTION), FLY_SIDE_LOOP.0, 1.0, false, 0.0, false, false);
        }
        let slope = (-vy).atan2(vx.abs()).to_degrees().clamp(-35.0, 50.0);
        let r = Vector3f { x: slope, y: 0.0, z: 0.0 };
        ModelModule::set_joint_rotate(boma, Hash40::new("rot"), &r,
            MotionNodeRotateCompose { _address: *MOTION_NODE_ROTATE_COMPOSE_AFTER as u8 },
            MotionNodeRotateOrder { _address: *MOTION_NODE_ROTATE_ORDER_XYZ as u8 });
    } else if MotionModule::motion_kind(boma) != smash::hash40("wait") || MotionModule::is_end(boma) {
        MotionModule::change_motion(boma, Hash40::new("wait"), 0.0, 1.0, false, 0.0, false, false);
    }
    flight_effects(boma, e, vx, vy);
    if fighter.sub_transition_group_check_air_cliff().get_bool() {
        return 1.into();
    }
    if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND && t > 4 {
        // lands sliding, with dust
        fx(boma, "sys_sliding_smoke", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 1.0, false);
        fx(boma, "sys_landing_smoke", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 1.0, false);
        fighter.change_status(FIGHTER_STATUS_KIND_LANDING.into(), false.into());
        return 1.into();
    }
    if t >= TAKEOFF + FLY_FRAMES {
        // end of the flight: normal fall, can still act
        normal_fall(fighter, vy);
        fighter.change_status(FIGHTER_STATUS_KIND_FALL.into(), false.into());
    }
    0.into()
}

/// the vanilla init / exec of the fighter's Up Special are replaced by nothing (everything is in hi_main)
unsafe extern "C" fn hi_nothing(_fighter: &mut L2CFighterCommon) -> L2CValue {
    0.into()
}

unsafe extern "C" fn hi_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    T[e] = 0;
    DIR[e] = 0;
    fx_off(boma, e, 0);
    fx_off(boma, e, 1);
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    0.into()
}

/// every frame: once per airtime. The Up Special is blocked BEFORE entering the status (starting the status and
/// leaving it right away crashed the game on the 2nd use)
unsafe extern "C" fn each_frame(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let st = StatusModule::status_kind(boma);
    let e = slot(boma);
    if USED[e] && StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR && st != *FIGHTER_STATUS_KIND_SPECIAL_HI {
        WorkModule::unable_transition_term(boma, *FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_SPECIAL_HI);
    }
    if StatusModule::situation_kind(boma) != *SITUATION_KIND_AIR
        || st == *FIGHTER_STATUS_KIND_CLIFF_CATCH
        || st == *FIGHTER_STATUS_KIND_DAMAGE_FLY
        || st == *FIGHTER_STATUS_KIND_DEAD
    {
        USED[e] = false;
    }
}

/// no hitbox, no vanilla effects / sounds during the flight (effects are spawned by the status)
unsafe extern "C" fn acmd_nothing(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
    }
}

pub fn install() {
    let mut a = Agent::new(FIGHTER);
    if !COSTUMES.is_empty() {
        a.set_costume(COSTUMES.to_vec());
    }
    for s in ["game_specialhi", "effect_specialhi", "expression_specialhi", "game_specialairhi",
              "effect_specialairhi", "expression_specialairhi"] {
        a.acmd(s, acmd_nothing, Priority::Low);
    }
    a.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_HI, hi_main);
    a.status(Init, *FIGHTER_STATUS_KIND_SPECIAL_HI, hi_nothing);
    a.status(Exec, *FIGHTER_STATUS_KIND_SPECIAL_HI, hi_nothing);
    a.status(End, *FIGHTER_STATUS_KIND_SPECIAL_HI, hi_end);
    a.on_line(Main, each_frame);
    a.install();
}

#[skyline::main(name = "flight_example")]
pub fn main() {
    install();
}
