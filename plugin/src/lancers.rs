//! Lancers de Master Chief (REX : sous-actions ThrowF / ThrowB / ThrowHi / ThrowLw de FitChief.pac), sur ses animations
//! de REX (retarget_exo.py, groupe chief_lancers, sur place) :
//! - avant : coup sur l'adversaire tenu (image 22, 10 %, angle 80), lâché à 26 (2 %, angle 45) ; libre à 30 ;
//! - arrière : il le fait tourner autour de lui et le lâche derrière à 67 (5 %, angle 40 vers l'arrière), puis recule
//!   (TransN de REX : 18,8) ; il finit retourné (le jeu le retourne à la fin, comme Brawl) ; vitesse 0,8 puis 1,3 ;
//! - haut : épée à énergie, 2 coups électriques sur l'adversaire tenu (18 : 2 %, 39 : 5 %), lâché à 41 (1 %, angle 91),
//!   un pas en avant (TransN de REX : 6,6) ; libre à 58 ;
//! - bas : il le plaque au sol (18), lui tire dessus au fusil d'assaut (24 : 1 %), puis coup au sol (86 : 3 %, angle 80,
//!   touche aussi les autres) et lâché (1 %, angle 90) ; vitesse 1,4 après 28 ; libre à 110.
//! Lâchers : ATTACK_ABS THROW (n° 0 de REX) / CATCH (n° 1 de REX : l'adversaire lancé qui touche les autres).
//! Coups sur l'adversaire tenu : sur l'os « throw » (il suit ThrowN de REX), ne touchent que lui (set_catch_only_all).

use smash::app::lua_bind::*;
use smash::app::sv_animcmd::{frame, wait};
use smash::app::*;
use smash::hash40;
use smash::lib::lua_const::*;
use smash::lua2cpp::*;
use smash::phx::Hash40;
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

use crate::speciaux::slot;

const ECHELLE: f32 = 0.82;
const CYAN: (f32, f32, f32) = (0.3, 0.95, 1.0);

/// lâcher : l'adversaire part avec le coup du lancer
unsafe fn lacher(agent: &mut L2CAgentBase) {
    let boma = agent.module_accessor;
    let target = WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_OBJECT);
    let group = WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_HIT_GROUP);
    let no = WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_HIT_NO);
    smash_script::macros::ATK_HIT_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_THROW, Hash40::new("throw"), target, group, no);
}

/// coups du lancer de REX : t = n° 0 (l'adversaire lancé), c = n° 1 (dégâts, angle, croissance, fixe, base)
unsafe fn lancer_abs(agent: &mut L2CAgentBase, t: (f32, u64, i32, i32, i32), c: (f32, u64, i32, i32, i32)) {
    smash_script::macros::ATTACK_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_THROW, 0, t.0, t.1, t.2, t.3, t.4, 0.0, 1.0, *ATTACK_LR_CHECK_F, 0.0, true,
               Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_NONE, *ATTACK_REGION_THROW);
    smash_script::macros::ATTACK_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_CATCH, 0, c.0, c.1, c.2, c.3, c.4, 0.0, 1.0, *ATTACK_LR_CHECK_F, 0.0, true,
               Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_NONE, *ATTACK_REGION_THROW);
}

/// coup de REX sur l'adversaire tenu (os throw, ne touche que lui)
unsafe fn coup_tenu(agent: &mut L2CAgentBase, dmg: f32, ang: u64, kbg: i32, fkb: i32, bkb: i32, size: f32, attr: &str, son: i32) {
    ATTACK(agent, 0, 0, Hash40::new("throw"), dmg, ang, kbg, fkb, bkb, size * ECHELLE, 0.0, 0.0, 0.0, None, None, None, 1.0, 1.0,
           *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true,
           *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
           Hash40::new(attr), *ATTACK_SOUND_LEVEL_M, son, *ATTACK_REGION_THROW);
    AttackModule::set_catch_only_all(agent.module_accessor, true, false);
}

/// fin de l'arrêt du coup (FAF de REX) : il peut agir
unsafe fn libre(agent: &mut L2CAgentBase, f: f32) {
    frame(agent.lua_state_agent, f);
    if is_excute(agent) {
        CancelModule::enable_cancel(agent.module_accessor);
    }
}

// ------------------------------------------------------------------ avant
unsafe extern "C" fn game_throwf(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        lancer_abs(agent, (2.0, 45, 30, 0, 109), (1.0, 45, 9, 0, 109));
    }
    frame(agent.lua_state_agent, 21.0);
    if is_excute(agent) {
        smash_script::macros::CHECK_FINISH_CAMERA(agent, 12, 10);
    }
    frame(agent.lua_state_agent, 22.0);
    if is_excute(agent) {
        coup_tenu(agent, 10.0, 80, 100, 0, 70, 3.91, "collision_attr_normal", *COLLISION_SOUND_ATTR_PUNCH);
    }
    frame(agent.lua_state_agent, 26.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
        lacher(agent);
    }
    libre(agent, 30.0);
}

unsafe extern "C" fn sound_throwf(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 2.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_common_throw_01"));
    }
    frame(agent.lua_state_agent, 25.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_common_throw_02"));
    }
}

// ------------------------------------------------------------------ arrière
unsafe extern "C" fn game_throwb(agent: &mut L2CAgentBase) {
    // REX : vitesse 0,8 jusqu'à l'image 63, puis 1,3 (dans Ultimate : 1 / vitesse)
    FT_MOTION_RATE(agent, 1.0 / 0.8);
    if is_excute(agent) {
        // lâché derrière lui : angle 40 de REX vers l'arrière (140 vers l'avant, comme le lancer arrière de Snake)
        lancer_abs(agent, (5.0, 140, 56, 0, 80), (5.0, 135, 79, 0, 80));
    }
    frame(agent.lua_state_agent, 63.0);
    FT_MOTION_RATE(agent, 1.0 / 1.3);
    frame(agent.lua_state_agent, 66.0);
    if is_excute(agent) {
        smash_script::macros::CHECK_FINISH_CAMERA(agent, -16, 12);
    }
    frame(agent.lua_state_agent, 67.0);
    if is_excute(agent) {
        lacher(agent);
    }
}

unsafe extern "C" fn sound_throwb(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 2.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_common_throw_01"));
    }
    for f in [43.0, 59.0] {
        frame(agent.lua_state_agent, f);
        if is_excute(agent) {
            PLAY_SE(agent, Hash40::new("se_common_swing_06"));
        }
    }
    frame(agent.lua_state_agent, 66.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_common_throw_02"));
    }
}

// ------------------------------------------------------------------ haut (épée à énergie)
unsafe extern "C" fn game_throwhi(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        lancer_abs(agent, (1.0, 91, 120, 0, 85), (1.0, 45, 88, 0, 33));
    }
    frame(agent.lua_state_agent, 18.0);
    if is_excute(agent) {
        coup_tenu(agent, 2.0, 361, 100, 40, 0, 4.1, "collision_attr_elec", *COLLISION_SOUND_ATTR_ELEC);
    }
    frame(agent.lua_state_agent, 20.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
    }
    frame(agent.lua_state_agent, 39.0);
    if is_excute(agent) {
        coup_tenu(agent, 5.0, 361, 100, 40, 0, 4.9, "collision_attr_elec", *COLLISION_SOUND_ATTR_ELEC);
    }
    frame(agent.lua_state_agent, 40.0);
    if is_excute(agent) {
        smash_script::macros::CHECK_FINISH_CAMERA(agent, 4, 24);
    }
    frame(agent.lua_state_agent, 41.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
        lacher(agent);
    }
    libre(agent, 58.0);
}

/// traînée cyan de l'épée (REX : lueur de l'épée 11031400 sur RHaveN)
unsafe extern "C" fn effect_throwhi(agent: &mut L2CAgentBase) {
    for (f0, f1) in [(15.0, 19.0), (37.0, 42.0)] {
        let mut f = f0;
        while f <= f1 {
            frame(agent.lua_state_agent, f);
            if is_excute(agent) {
                EFFECT(agent, Hash40::new("sys_smash_flash_s"), Hash40::new("haver"), 0, 0, 0, 0, 0, 0, 0.6, 0, 0, 0, 0, 0, 0, true);
                LAST_EFFECT_SET_COLOR(agent, CYAN.0, CYAN.1, CYAN.2);
            }
            f += 1.0;
        }
    }
}

unsafe extern "C" fn sound_throwhi(agent: &mut L2CAgentBase) {
    // sons de REX : 19313 (n° 77) à 9, 19238 (n° 2 : épée) à 15 et 38
    frame(agent.lua_state_agent, 9.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_snake_attackair_f01"));
    }
    for f in [15.0, 38.0] {
        frame(agent.lua_state_agent, f);
        if is_excute(agent) {
            PLAY_SE(agent, Hash40::new("se_snake_attackair_n02"));
        }
    }
    frame(agent.lua_state_agent, 41.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_common_throw_02"));
    }
}

// ------------------------------------------------------------------ bas (fusil d'assaut)
unsafe extern "C" fn game_throwlw(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        lancer_abs(agent, (1.0, 90, 57, 0, 85), (3.0, 45, 100, 0, 40));
    }
    frame(agent.lua_state_agent, 24.0);
    if is_excute(agent) {
        coup_tenu(agent, 1.0, 361, 100, 40, 0, 4.1, "collision_attr_normal", *COLLISION_SOUND_ATTR_PUNCH);
    }
    frame(agent.lua_state_agent, 27.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
    }
    frame(agent.lua_state_agent, 28.0);
    FT_MOTION_RATE(agent, 1.0 / 1.4);
    frame(agent.lua_state_agent, 85.0);
    if is_excute(agent) {
        smash_script::macros::CHECK_FINISH_CAMERA(agent, 0, 3);
    }
    frame(agent.lua_state_agent, 86.0);
    FT_MOTION_RATE(agent, 1.0 / 1.34783);
    if is_excute(agent) {
        // coup au sol de REX (pas spécial : touche aussi les autres autour de lui)
        ATTACK(agent, 1, 0, Hash40::new("top"), 3.0, 80, 61, 30, 5, 4.1 * ECHELLE, 0.0, 0.0, 0.0, None, None, None, 1.88, 1.0,
               *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true,
               *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
               Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KICK);
        lacher(agent);
    }
    wait(agent.lua_state_agent, 2.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
    }
    libre(agent, 110.0);
}

unsafe extern "C" fn effect_throwlw(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 18.0);
    if is_excute(agent) {
        LANDING_EFFECT(agent, Hash40::new("sys_down_smoke"), Hash40::new("top"), 0, 0, 1, 0, 0, 0, 0.9, 0, 0, 0, 0, 0, 0, false);
    }
    // tir au fusil d'assaut : bout du canon de REX (effet 0x1FB0007 à 10 devant la main : (0 ; 4,1 ; 1,8) sur top à
    // l'image 24), éclair jaune sur Chief (REX : 21050600 jaune, images 24 à 27)
    frame(agent.lua_state_agent, 24.0);
    if is_excute(agent) {
        EFFECT(agent, Hash40::new("sys_muzzleflash"), Hash40::new("top"), 0, 4.1, 1.8, 0, 0, 0, 0.6, 0, 0, 0, 0, 0, 0, true);
        EFFECT(agent, Hash40::new("sys_hit_fire"), Hash40::new("top"), 0, 4.1, 1.8, 0, 0, 0, 0.3, 0, 0, 0, 0, 0, 0, true);
        smash_script::macros::FLASH(agent, 1, 1, 0, 0.33);
    }
    frame(agent.lua_state_agent, 27.0);
    if is_excute(agent) {
        smash_script::macros::COL_NORMAL(agent);
    }
    frame(agent.lua_state_agent, 86.0);
    if is_excute(agent) {
        LANDING_EFFECT(agent, Hash40::new("sys_down_smoke"), Hash40::new("top"), 0, 0, 1, 0, 0, 0, 1.0, 0, 0, 0, 0, 0, 0, false);
    }
}

unsafe extern "C" fn sound_throwlw(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 17.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_common_throw_01"));
        smash_script::macros::PLAY_DOWN_SE(agent, Hash40::new("se_common_down_soil_s"));
    }
    // rafale de REX (19328 en boucle, images 24 à 28 : son du fusil d'assaut du B neutre)
    frame(agent.lua_state_agent, 24.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_snake_special_n03"));
    }
    frame(agent.lua_state_agent, 28.0);
    if is_excute(agent) {
        smash_script::macros::STOP_SE(agent, Hash40::new("se_snake_special_n03"));
    }
    frame(agent.lua_state_agent, 85.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_common_throw_02"));
    }
}

unsafe extern "C" fn vide(_agent: &mut L2CAgentBase) {}

/// expression : inclinaison au sol + vibrations aux images de REX (07070200)
unsafe fn vibre(agent: &mut L2CAgentBase, fs: &[f32]) {
    if is_excute(agent) {
        slope!(agent, *MA_MSC_CMD_SLOPE_SLOPE, *SLOPE_STATUS_LR);
    }
    for f in fs {
        frame(agent.lua_state_agent, *f);
        if is_excute(agent) {
            ControlModule::set_rumble(agent.module_accessor, Hash40::new("rbkind_attackm"), 0, false, 0x50000000);
        }
    }
}
unsafe extern "C" fn expression_throwf(agent: &mut L2CAgentBase) { vibre(agent, &[22.0]); }
unsafe extern "C" fn expression_throwb(agent: &mut L2CAgentBase) { vibre(agent, &[34.0, 52.0]); }
unsafe extern "C" fn expression_throwhi(agent: &mut L2CAgentBase) { vibre(agent, &[38.0]); }
unsafe extern "C" fn expression_throwlw(agent: &mut L2CAgentBase) { vibre(agent, &[70.0, 78.0]); }

// ------------------------------------------------------------------ déplacements (animations sur place)
// TransN de REX (axe avant, cumulé), image par image depuis l'image 0 (rex/tools/avance_rex.py)
const AV_B: [f32; 132] = [
    0.000, -0.099, -0.215, -0.338, -0.457, -0.563, -0.645, -0.692, -0.695, -0.684, -0.663, -0.581, -0.388,
    -0.032, 0.691, 1.747, 2.828, 3.628, 3.840, 3.040, 1.600, 0.406, -0.001, -0.863, -1.628, -2.452, -3.290,
    -4.046, -4.604, -4.000, -3.154, -2.188, -1.273, -0.581, -0.219, -0.077, 0.000, 0.087, 0.110, 0.000, -0.440,
    -1.013, -1.192, -0.703, 0.181, 1.159, 1.930, 2.506, 3.017, 3.341, 3.356, 2.895, 2.069, 1.167, 0.482, 0.208,
    0.000, -0.369, -0.775, -1.211, -1.670, -2.147, -2.633, -3.123, -3.609, -4.086, -4.569, -4.931, -5.279,
    -5.612, -5.930, -6.231, -6.516, -6.785, -7.035, -7.267, -7.481, -7.675, -7.849, -8.003, -8.136, -8.223,
    -8.253, -8.240, -8.199, -8.147, -8.098, -8.067, -7.998, -7.926, -8.002, -8.259, -8.612, -9.035, -9.507,
    -10.003, -10.499, -10.973, -11.401, -11.795, -12.182, -12.559, -12.926, -13.281, -13.621, -13.933, -14.214,
    -14.446, -14.644, -14.876, -15.209, -15.744, -16.384, -16.900, -17.185, -17.346, -17.473, -17.601, -17.706,
    -17.792, -17.863, -17.924, -17.978, -18.029, -18.082, -18.140, -18.209, -18.291, -18.392, -18.515, -18.664,
    -18.843
];
const AV_HI: [f32; 170] = [
    0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.000,
    0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.000,
    0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 3.900, 5.600, 6.314, 6.685, 6.803, 6.755, 6.629,
    6.515, 6.500, 6.552, 6.583, 6.595, 6.594, 6.583, 6.564, 6.543, 6.523, 6.507, 6.500, 6.498, 6.496, 6.494,
    6.491, 6.489, 6.486, 6.484, 6.481, 6.478, 6.476, 6.473, 6.470, 6.467, 6.463, 6.460, 6.457, 6.454, 6.450,
    6.447, 6.443, 6.439, 6.436, 6.432, 6.428, 6.424, 6.420, 6.416, 6.412, 6.408, 6.404, 6.400, 6.396, 6.391,
    6.387, 6.382, 6.378, 6.373, 6.369, 6.364, 6.360, 6.355, 6.350, 6.345, 6.341, 6.336, 6.331, 6.326, 6.321,
    6.316, 6.311, 6.306, 6.301, 6.296, 6.291, 6.286, 6.281, 6.276, 6.271, 6.265, 6.260, 6.255, 6.250, 6.245,
    6.239, 6.234, 6.229, 6.224, 6.218, 6.213, 6.208, 6.203, 6.197, 6.192, 6.187, 6.182, 6.176, 6.171, 6.166,
    6.161, 6.156, 6.150, 6.145, 6.140, 6.135, 6.130, 6.125, 6.120, 6.115, 6.110, 6.105, 6.100, 6.095, 6.090,
    6.085, 6.080, 6.075, 6.071, 6.066, 6.061, 6.056, 6.052, 6.047, 6.043, 6.038, 6.034, 6.029, 6.025, 6.021,
    6.016, 6.012, 6.008, 6.004, 6.000
];
/// lancer arrière : il est retourné à partir de cette image (REX : il finit de dos, le jeu le retourne à la fin)
const B_DOS: f32 = 64.0;
static mut BOUGE: [bool; 8] = [false; 8];
static mut DOS: [bool; 8] = [false; 8];

fn table(t: &[f32], f: f32) -> f32 {
    let n = t.len();
    let k = f.clamp(0.0, (n - 1) as f32);
    let i = (k as usize).min(n - 2);
    t[i] + (t[i + 1] - t[i]) * (k - i as f32)
}

/// chaque image : déplacement réel des lancers arrière / haut, demi-tour à la fin du lancer arrière
pub unsafe fn chaque_image(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    if StatusModule::status_kind(boma) != *FIGHTER_STATUS_KIND_THROW {
        if DOS[e] {
            DOS[e] = false;
            PostureModule::reverse_lr(boma);
            PostureModule::update_rot_y_lr(boma);
        }
        BOUGE[e] = false;
        return;
    }
    let m = MotionModule::motion_kind(boma);
    let f = MotionModule::frame(boma);
    let t: &[f32] = if m == hash40("throw_b") {
        if f >= B_DOS {
            DOS[e] = true;
        }
        &AV_B
    } else if m == hash40("throw_hi") {
        &AV_HI
    } else {
        return;
    };
    if !BOUGE[e] {
        BOUGE[e] = true;
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
        GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND_CLIFF_STOP));
    }
    let rate = MotionModule::rate(boma).max(0.01);
    let d = table(t, f + rate) - table(t, f);
    let lr = PostureModule::lr(boma);
    sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, d * lr, 0.0_f32);
}

pub fn install(a: &mut Agent) {
    a.acmd("game_throwf", game_throwf, Priority::Low);
    a.acmd("game_throwb", game_throwb, Priority::Low);
    a.acmd("game_throwhi", game_throwhi, Priority::Low);
    a.acmd("game_throwlw", game_throwlw, Priority::Low);
    a.acmd("effect_throwf", vide, Priority::Low);
    a.acmd("effect_throwb", vide, Priority::Low);
    a.acmd("effect_throwhi", effect_throwhi, Priority::Low);
    a.acmd("effect_throwlw", effect_throwlw, Priority::Low);
    a.acmd("sound_throwf", sound_throwf, Priority::Low);
    a.acmd("sound_throwb", sound_throwb, Priority::Low);
    a.acmd("sound_throwhi", sound_throwhi, Priority::Low);
    a.acmd("sound_throwlw", sound_throwlw, Priority::Low);
    a.acmd("expression_throwf", expression_throwf, Priority::Low);
    a.acmd("expression_throwb", expression_throwb, Priority::Low);
    a.acmd("expression_throwhi", expression_throwhi, Priority::Low);
    a.acmd("expression_throwlw", expression_throwlw, Priority::Low);
}
