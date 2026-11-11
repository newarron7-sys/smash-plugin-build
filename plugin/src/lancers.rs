//! Super Sonic : prise, coup pendant la prise, lancers et attaques au sol de REX (FitSuperSonic.pac, vidéo de REX du
//! joueur avec le nom des coups). Mêmes techniques que Super Shadow (rex/PROCEDURE.md) :
//! - Battering Ram (avant) : charge au sol dans un tourbillon de vent (images 0 à 54), lâché à 60 (8 %), paralysé, puis
//!   il fonce en Boost (11 %, total 20 % comme dans la vidéo) ; il avance vraiment (TransN de REX), et continue en
//!   dehors du terrain au bord comme dans REX.
//! - Interception (arrière) : il le jette derrière lui (5 %, presque sans recul), le paralyse (2 %), revient en
//!   arrière d'un bond (TransN de REX : 50 en arrière) et le frappe (14 %, angle 140).
//! - Wild Rush (haut) : il l'emmène en l'air et le frappe 4 fois (2 %) puis 7 %, le lâche (5 %, angle 76), et se
//!   retrouve là-haut (technique de Zero : placé sur l'os hip après le lâcher, puis il retombe).
//! - Point Blank Blast Off (bas) : explosion sur l'adversaire (3 %, électrique), lâché à 21 (6 %, angle 45,
//!   électrique), puis il recule d'un bond (TransN de REX : 18 en arrière).
//! - Grab Knee Strike (coup pendant la prise) : 3 %.
//! - Spinning Leg Sweep (attaques au sol) : 6 %.

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

const OR: (f32, f32, f32) = (1.0, 0.78, 0.22);
const BLANC: (f32, f32, f32) = (1.0, 1.0, 1.0);
/// vent / anneaux cyan de ses lancers dans REX
const CYAN: (f32, f32, f32) = (0.45, 1.0, 0.9);

unsafe fn slot(boma: *mut BattleObjectModuleAccessor) -> usize {
    (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7)
}

unsafe fn fx(agent: &mut L2CAgentBase, eff: &str, bone: &str, p: (f32, f32, f32), r: (f32, f32, f32), sc: f32, c: (f32, f32, f32)) {
    if is_excute(agent) {
        EFFECT_FOLLOW(agent, Hash40::new(eff), Hash40::new(bone), p.0, p.1, p.2, r.0, r.1, r.2, sc, true);
        LAST_EFFECT_SET_COLOR(agent, c.0, c.1, c.2);
    }
}
/// effet posé (il reste sur place)
unsafe fn fx_pose(agent: &mut L2CAgentBase, eff: &str, bone: &str, p: (f32, f32, f32), r: (f32, f32, f32), sc: f32, c: (f32, f32, f32)) {
    if is_excute(agent) {
        EFFECT(agent, Hash40::new(eff), Hash40::new(bone), p.0, p.1, p.2, r.0, r.1, r.2, sc, 0, 0, 0, 0, 0, 0, true);
        LAST_EFFECT_SET_COLOR(agent, c.0, c.1, c.2);
    }
}
unsafe fn off(agent: &mut L2CAgentBase, eff: &str) {
    if is_excute(agent) {
        EFFECT_OFF_KIND(agent, Hash40::new(eff), false, false);
    }
}
unsafe fn son(agent: &mut L2CAgentBase, se: &str) {
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new(se));
    }
}

/// lâcher : l'adversaire part avec le coup du lancer (ATTACK_ABS THROW)
unsafe fn lacher(agent: &mut L2CAgentBase) {
    let boma = agent.module_accessor;
    let target = WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_OBJECT);
    let group = WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_HIT_GROUP);
    let no = WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_HIT_NO);
    ATK_HIT_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_THROW, Hash40::new("throw"), target, group, no);
}

unsafe fn lancer_abs(agent: &mut L2CAgentBase, dmg: f32, ang: u64, kbg: i32, fkb: i32, bkb: i32, attr: &str, c: (f32, u64, i32, i32)) {
    ATTACK_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_THROW, 0, dmg, ang, kbg, fkb, bkb, 0.0, 1.0, *ATTACK_LR_CHECK_F, 0.0, true,
               Hash40::new(attr), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_NONE, *ATTACK_REGION_THROW);
    ATTACK_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_CATCH, 0, c.0, c.1, c.2, 0, c.3, 0.0, 1.0, *ATTACK_LR_CHECK_F, 0.0, true,
               Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_NONE, *ATTACK_REGION_THROW);
}

/// coup sur l'adversaire tenu (os throw)
unsafe fn coup_tenu(agent: &mut L2CAgentBase, dmg: f32, ang: u64, kbg: i32, bkb: i32, size: f32, attr: &str, son: i32) {
    ATTACK(agent, 0, 0, Hash40::new("throw"), dmg, ang, kbg, 0, bkb, size, 0.0, 0.0, 0.0, None, None, None, 1.0, 1.0,
           *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true,
           *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
           Hash40::new(attr), *ATTACK_SOUND_LEVEL_M, son, *ATTACK_REGION_THROW);
    AttackModule::set_catch_only_all(agent.module_accessor, true, false);
}

unsafe fn coup(agent: &mut L2CAgentBase, id: u64, os: &str, dmg: f32, ang: u64, kbg: i32, bkb: i32, size: f32,
               p: (f32, f32, f32), attr: &str, son: i32) {
    ATTACK(agent, id, 0, Hash40::new(os), dmg, ang, kbg, 0, bkb, size, p.0, p.1, p.2, None, None, None, 1.0, 1.0,
           *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true,
           *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
           Hash40::new(attr), *ATTACK_SOUND_LEVEL_L, son, *ATTACK_REGION_KICK);
}

// ------------------------------------------------------------------ déplacements réels pendant les lancers
// Dans Ultimate l'animation ne déplace pas le personnage pendant un lancer : TransN de REX (unités d'Ultimate), image par
// image, à partir de la première image du tableau (animations rendues sur place par retarget_exo).
const AV_F0: f32 = 56.0;
const AV_F: [f32; 28] = [1.2, 1.2, 1.2, 1.2, 1.2, 1.2, 1.2, 3.6, 6.4, 9.5, 13.0, 16.7, 20.8, 25.0, 29.5, 34.2, 39.0, 43.9,
    48.9, 54.0, 59.1, 64.2, 69.3, 74.3, 79.2, 84.1, 88.7, 93.2];
const AV_B0: f32 = 28.0;
const AV_B: [f32; 18] = [0.4, 0.3, 0.0, -0.9, -2.1, -3.0, -3.0, -5.9, -11.0, -20.0, -26.7, -32.9, -38.4, -43.1, -47.0,
    -50.0, -50.0, -50.0];
const AV_LW0: f32 = 26.0;
const AV_LW: [f32; 14] = [0.5, 0.6, 0.0, -1.3, -3.2, -5.4, -7.8, -10.4, -12.8, -15.0, -16.9, -18.2, -18.2, -18.2];

fn table(t: &[f32], f0: f32, f: f32) -> f32 {
    let n = t.len();
    let k = (f - f0).clamp(0.0, (n - 1) as f32);
    let i = (k as usize).min(n - 2);
    let a = k - i as f32;
    t[i] + (t[i + 1] - t[i]) * a
}

unsafe fn avance(fighter: &mut L2CFighterCommon, t: &[f32], f0: f32, f: f32) {
    let boma = fighter.module_accessor;
    let rate = MotionModule::rate(boma).max(0.01);
    let d = table(t, f0, f + rate) - table(t, f0, f);
    let lr = PostureModule::lr(boma);
    sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, d * lr, 0.0_f32);
}

static mut BOOST_ACTIF: [bool; 8] = [false; 8];
static mut BOOST_FRAME: [f32; 8] = [0.0; 8];
static mut BOOST_AIR: [bool; 8] = [false; 8];
static mut RECUL: [bool; 8] = [false; 8];
static mut HAUT_BASE: [(f32, f32, bool); 8] = [(0.0, 0.0, false); 8];
static mut HAUT_FAIT: [bool; 8] = [false; 8];
const HI_LACHER: f32 = 57.0;

/// lancer avant : Boost réel à partir de l'image 60 ; au bord il continue tout droit en dehors du terrain (REX)
unsafe fn frame_throw_f(fighter: &mut L2CFighterCommon, lancer: bool) {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    let st = StatusModule::status_kind(boma);
    if lancer {
        let f = MotionModule::frame(boma);
        if f < 60.0 {
            BOOST_ACTIF[e] = false;
            BOOST_AIR[e] = false;
            return;
        }
        if !BOOST_ACTIF[e] {
            BOOST_ACTIF[e] = true;
            BOOST_AIR[e] = false;
            KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
            GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND));
        }
        BOOST_FRAME[e] = f;
        avance(fighter, &AV_F, AV_F0, f);
        return;
    }
    if !BOOST_ACTIF[e] {
        return;
    }
    let air = StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR;
    let chute = st == *FIGHTER_STATUS_KIND_FALL || st == *FIGHTER_STATUS_KIND_FALL_AERIAL;
    if !(air && chute) || BOOST_FRAME[e] >= 82.0 {
        if BOOST_AIR[e] {
            KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
            KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_FALL);
            if chute {
                MotionModule::change_motion(boma, Hash40::new("fall"), 0.0, 1.0, false, 0.0, false, false);
            }
        }
        BOOST_ACTIF[e] = false;
        BOOST_AIR[e] = false;
        return;
    }
    if !BOOST_AIR[e] {
        BOOST_AIR[e] = true;
        MotionModule::change_motion(boma, Hash40::new("throw_f"), BOOST_FRAME[e], 1.6, false, 0.0, false, false);
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_AIR_STOP);
        KineticModule::unable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    }
    let f = MotionModule::frame(boma);
    BOOST_FRAME[e] = f;
    avance(fighter, &AV_F, AV_F0, f);
}

/// lancers arrière et bas : il recule vraiment (au sol, arrêt au bord)
unsafe fn frame_recul(fighter: &mut L2CFighterCommon, t: &[f32], f0: f32) {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    let f = MotionModule::frame(boma);
    if f < f0 {
        return;
    }
    if !RECUL[e] {
        RECUL[e] = true;
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
        GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND_CLIFF_STOP));
    }
    avance(fighter, t, f0, f);
}

/// lancer haut : la montée est dans la pose (hip) ; après le lâcher il est placé là-haut, vraiment en l'air, et retombe
unsafe fn frame_throw_hi(fighter: &mut L2CFighterCommon, lancer: bool) {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    if !lancer {
        HAUT_BASE[e].2 = false;
        HAUT_FAIT[e] = false;
        return;
    }
    let mut p = smash::phx::Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    ModelModule::joint_global_position(boma, Hash40::new("hip"), &mut p, true);
    let (px, py) = (PostureModule::pos_x(boma), PostureModule::pos_y(boma));
    if !HAUT_BASE[e].2 {
        if !HAUT_FAIT[e] {
            HAUT_BASE[e] = (p.x - px, p.y - py, true);
        }
        return;
    }
    if MotionModule::frame(boma) >= HI_LACHER + 2.0 {
        let dx = (p.x - px) - HAUT_BASE[e].0;
        let dy = (p.y - py) - HAUT_BASE[e].1;
        HAUT_BASE[e].2 = false;
        HAUT_FAIT[e] = true;
        if dy >= 2.0 {
            let pos = smash::phx::Vector3f { x: px + dx, y: py + dy, z: PostureModule::pos_z(boma) };
            PostureModule::set_pos(boma, &pos);
            GroundModule::set_shape_safe_pos(boma, &smash::phx::Vector2f { x: pos.x, y: pos.y });
            StatusModule::set_situation_kind(boma, SituationKind(*SITUATION_KIND_AIR), false);
            fighter.global_table[0x17].assign(&L2CValue::I32(*SITUATION_KIND_AIR));
            fighter.global_table[0x16].assign(&L2CValue::I32(*SITUATION_KIND_AIR));
            GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
            KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_FALL);
            // REX : élan de 3 vers le haut à l'image 59
            sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY, 3.0);
            fighter.change_status(FIGHTER_STATUS_KIND_FALL.into(), false.into());
        }
    }
}

/// chaque image (ss_frame)
pub unsafe fn frame_throw(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    let lancer = StatusModule::status_kind(boma) == *FIGHTER_STATUS_KIND_THROW;
    let mk = MotionModule::motion_kind(boma);
    frame_throw_f(fighter, lancer && mk == smash::hash40("throw_f"));
    frame_throw_hi(fighter, lancer && mk == smash::hash40("throw_hi"));
    if lancer && mk == smash::hash40("throw_b") {
        frame_recul(fighter, &AV_B, AV_B0);
    } else if lancer && mk == smash::hash40("throw_lw") {
        frame_recul(fighter, &AV_LW, AV_LW0);
    } else {
        RECUL[e] = false;
    }
}

// ------------------------------------------------------------------ Battering Ram (avant)
unsafe extern "C" fn g_throwf(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    if is_excute(agent) {
        // lâché juste devant lui, sans être éjecté (puis paralysé et percuté par le Boost, comme Super Shadow)
        lancer_abs(agent, 8.0, 361, 0, 0, 10, "collision_attr_normal", (3.0, 361, 100, 60));
    }
    // REX : vitesse 1,15 pendant la charge (6 x 9 images), puis 1,6
    FT_MOTION_RATE(agent, 1.0 / 1.15);
    frame(lua, 54.0);
    FT_MOTION_RATE(agent, 1.0 / 1.6);
    frame(lua, 60.0);
    if is_excute(agent) {
        lacher(agent);
    }
    frame(lua, 61.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
        coup(agent, 0, "top", 1.0, 270, 0, 30, 7.5, (0.0, 9.0, 14.0), "collision_attr_paralyze", *COLLISION_SOUND_ATTR_ELEC);
    }
    frame(lua, 63.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
        // la charge du Boost (REX : 12 %, angle 361, 144 / 50)
        coup(agent, 0, "hip", 11.0, 361, 144, 50, 7.5, (0.0, 0.0, 0.0), "collision_attr_normal", *COLLISION_SOUND_ATTR_KICK);
        CHECK_FINISH_CAMERA(agent, 12, 6);
    }
    frame(lua, 74.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
    }
}
unsafe extern "C" fn e_throwf(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    // vidéo REX : tourbillon de vent cyan au sol autour de lui pendant la charge
    for k in 0..6 {
        frame(lua, 1.0 + 9.0 * k as f32);
        fx(agent, "sys_whirlwind_l", "top", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 1.1, CYAN);
        fx_pose(agent, "sys_spin_wind", "top", (0.0, 1.0, 0.0), (90.0, 0.0, 0.0), 1.4, CYAN);
    }
    frame(lua, 54.0);
    off(agent, "sys_whirlwind_l");
    // lâcher : éclat blanc et doré
    frame(lua, 60.0);
    fx_pose(agent, "sys_smash_flash", "throw", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 1.8, BLANC);
    fx_pose(agent, "sys_hit_elec", "throw", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 1.0, OR);
    // Boost (boule dorée et traînée blanche)
    fx(agent, "sys_score_aura", "top", (0.0, 6.0, 4.0), (-90.0, 0.0, 0.0), 1.4, OR);
    fx(agent, "sonic_spintrace", "top", (0.0, 7.0, 0.0), (0.0, 0.0, 0.0), 0.9, OR);
    for f in [60.0, 64.0, 68.0, 72.0, 76.0] {
        frame(lua, f);
        fx(agent, "sys_attack_speedline", "top", (0.0, 8.0, -4.0), (0.0, 0.0, 0.0), 1.3, BLANC);
        if is_excute(agent) {
            FOOT_EFFECT(agent, Hash40::new("sys_dash_smoke"), Hash40::new("top"), -4, 0, 0, 0, 0, 0, 0.9, 0, 0, 0, 0, 0, 0, false);
        }
    }
    frame(lua, 82.0);
    off(agent, "sys_score_aura");
    off(agent, "sonic_spintrace");
}
unsafe extern "C" fn s_throwf(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 1.0);
    son(agent, "se_sonic_special_s01");
    frame(lua, 60.0);
    if is_excute(agent) {
        PLAY_SEQUENCE(agent, Hash40::new("seq_sonic_rnd_attack"));
    }
    son(agent, "se_sonic_smash_s01");
    son(agent, "se_common_throw_02");
}

// ------------------------------------------------------------------ Interception (arrière)
unsafe extern "C" fn g_throwb(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    if is_excute(agent) {
        // REX : 5 %, angle 90, reculs de 4 : il retombe derrière lui
        lancer_abs(agent, 5.0, 90, 4, 0, 4, "collision_attr_normal", (3.0, 361, 4, 60));
    }
    frame(lua, 21.0);
    if is_excute(agent) {
        lacher(agent);
    }
    frame(lua, 22.0);
    if is_excute(agent) {
        // REX : paralysie derrière lui (2 %, angle 270, recul 36, à 20 derrière)
        coup(agent, 0, "top", 2.0, 270, 0, 36, 5.0, (0.0, 8.0, -20.0), "collision_attr_paralyze", *COLLISION_SOUND_ATTR_ELEC);
    }
    frame(lua, 26.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
    }
    FT_MOTION_RATE(agent, 1.0 / 1.5);
    frame(lua, 37.0);
    FT_MOTION_RATE(agent, 1.0);
    if is_excute(agent) {
        // le coup (14 %, angle 140, 112 / 42) : il l'envoie derrière lui
        coup(agent, 0, "top", 14.0, 140, 112, 42, 8.0, (0.0, 8.0, 0.0), "collision_attr_normal", *COLLISION_SOUND_ATTR_KICK);
        CHECK_FINISH_CAMERA(agent, -10, 8);
    }
    frame(lua, 45.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
    }
}
unsafe extern "C" fn e_throwb(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 14.0);
    fx_pose(agent, "sys_smash_flash", "hip", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 1.2, BLANC);
    frame(lua, 21.0);
    fx_pose(agent, "sys_hit_elec", "throw", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 0.9, OR);
    // REX (images 32 à 45) : il revient en boule dorée, anneau de vent cyan, traînée
    frame(lua, 31.0);
    fx(agent, "sys_score_aura", "hip", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 1.0, OR);
    fx(agent, "sys_spin_wind", "hip", (0.0, 0.0, 0.0), (0.0, 90.0, 0.0), 1.2, CYAN);
    fx(agent, "sys_attack_speedline", "top", (0.0, 8.0, 6.0), (0.0, 180.0, 0.0), 1.2, BLANC);
    frame(lua, 37.0);
    fx_pose(agent, "sys_attack_impact", "top", (0.0, 8.0, -4.0), (0.0, 0.0, 0.0), 1.6, BLANC);
    fx_pose(agent, "sys_smash_flash", "top", (0.0, 8.0, -4.0), (0.0, 0.0, 0.0), 1.3, CYAN);
    frame(lua, 45.0);
    off(agent, "sys_score_aura");
    off(agent, "sys_spin_wind");
}
unsafe extern "C" fn s_throwb(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 14.0);
    son(agent, "se_common_throw_01");
    frame(lua, 32.0);
    son(agent, "se_sonic_special_s01");
    frame(lua, 37.0);
    if is_excute(agent) {
        PLAY_SEQUENCE(agent, Hash40::new("seq_sonic_rnd_attack"));
    }
    son(agent, "se_common_kick_hit_l");
}

// ------------------------------------------------------------------ Wild Rush (haut)
const HI_COUPS: [f32; 4] = [25.0, 32.0, 39.0, 46.0];
unsafe fn camera_haut(agent: &mut L2CAgentBase, y: i32) {
    if is_excute(agent) {
        camera!(agent, *MA_MSC_CMD_CAMERA_CAM_OFFSET, 0, y);
    }
}
unsafe extern "C" fn g_throwhi(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    if is_excute(agent) {
        // REX : 5 %, angle 76, croissance 60, recul 95
        lancer_abs(agent, 5.0, 76, 60, 0, 95, "collision_attr_normal", (3.0, 76, 100, 60));
    }
    FT_MOTION_RATE(agent, 1.0 / 2.0);
    // REX : l'adversaire monte à 47 de haut, lui autour de lui jusqu'à 59 : la caméra suit
    for (f, y) in [(8.0, 20), (16.0, 35), (22.0, 45)] {
        frame(lua, f);
        camera_haut(agent, y);
    }
    for f in HI_COUPS {
        frame(lua, f);
        if is_excute(agent) {
            coup_tenu(agent, 2.0, 361, 120, 60, 4.7, "collision_attr_normal", *COLLISION_SOUND_ATTR_KICK);
        }
        frame(lua, f + 1.0);
        if is_excute(agent) {
            AttackModule::clear_all(agent.module_accessor);
        }
    }
    frame(lua, 52.0);
    FT_MOTION_RATE(agent, 1.0);
    frame(lua, 53.0);
    if is_excute(agent) {
        coup_tenu(agent, 7.0, 90, 120, 81, 8.0, "collision_attr_normal", *COLLISION_SOUND_ATTR_KICK);
    }
    frame(lua, 54.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
    }
    camera_haut(agent, 60);
    frame(lua, HI_LACHER);
    if is_excute(agent) {
        CHECK_FINISH_CAMERA(agent, 0, 20);
        lacher(agent);
    }
    // il est placé là-haut à l'image 59 (frame_throw) : la caméra le suit alors normalement
    frame(lua, 59.0);
    camera_haut(agent, 0);
}
unsafe extern "C" fn e_throwhi(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    // départ : coup de pied qui l'envoie en l'air (éclat blanc)
    frame(lua, 6.0);
    fx_pose(agent, "sys_smash_flash", "throw", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 1.3, BLANC);
    // vidéo REX : à chaque coup, éclat blanc, anneau de vent et arc cyan autour de l'adversaire
    for f in HI_COUPS {
        frame(lua, f);
        fx_pose(agent, "sys_attack_impact", "throw", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 1.2, BLANC);
        fx_pose(agent, "sys_spin_wind", "throw", (0.0, 0.0, 0.0), (0.0, f * 7.0, 90.0), 1.0, CYAN);
        fx(agent, "sys_attack_arc", "hip", (0.0, 0.0, 0.0), (0.0, 0.0, f * 13.0), 1.1, CYAN);
    }
    frame(lua, 53.0);
    fx_pose(agent, "sys_smash_flash", "throw", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 2.2, BLANC);
    fx_pose(agent, "sys_attack_impact", "throw", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 1.8, CYAN);
    // il monte en colonne de lumière (REX : TransN monte à 72)
    frame(lua, 54.0);
    fx(agent, "sys_attack_speedline", "top", (0.0, 4.0, 0.0), (-90.0, 0.0, 0.0), 1.6, CYAN);
    frame(lua, 60.0);
    off(agent, "sys_attack_speedline");
}
unsafe extern "C" fn s_throwhi(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 6.0);
    son(agent, "se_common_throw_03");
    for f in HI_COUPS {
        frame(lua, f);
        son(agent, "se_common_kick_hit_m");
    }
    frame(lua, 53.0);
    if is_excute(agent) {
        PLAY_SEQUENCE(agent, Hash40::new("seq_sonic_rnd_attack"));
    }
    son(agent, "se_common_kick_hit_l");
}

// ------------------------------------------------------------------ Point Blank Blast Off (bas)
unsafe extern "C" fn g_throwlw(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    if is_excute(agent) {
        // REX : 6 %, angle 45, croissance 65, recul 65, électrique
        lancer_abs(agent, 6.0, 45, 65, 0, 65, "collision_attr_elec", (3.0, 361, 100, 60));
    }
    frame(lua, 14.0);
    if is_excute(agent) {
        // explosion sur l'adversaire (REX : 3 %, électrique)
        coup_tenu(agent, 3.0, 80, 100, 30, 9.0, "collision_attr_elec", *COLLISION_SOUND_ATTR_ELEC);
    }
    frame(lua, 15.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
    }
    frame(lua, 21.0);
    if is_excute(agent) {
        CHECK_FINISH_CAMERA(agent, 8, 4);
        lacher(agent);
    }
}
unsafe extern "C" fn e_throwlw(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    // REX : la charge sur l'adversaire (image 7), puis l'explosion blanche et dorée (image 14)
    frame(lua, 7.0);
    fx(agent, "sys_smash_flash", "throw", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 0.9, OR);
    frame(lua, 14.0);
    fx_pose(agent, "sys_bomb_b", "throw", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 1.3, BLANC);
    fx_pose(agent, "sys_smash_flash", "throw", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 2.6, BLANC);
    fx_pose(agent, "sys_hit_elec", "throw", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 1.6, OR);
    if is_excute(agent) {
        QUAKE(agent, *CAMERA_QUAKE_KIND_M);
    }
    // il recule d'un bond : petit éclat
    frame(lua, 28.0);
    fx(agent, "sys_smash_flash", "hip", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 1.0, BLANC);
    if is_excute(agent) {
        FOOT_EFFECT(agent, Hash40::new("sys_turn_smoke"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.9, 0, 0, 0, 0, 0, 0, false);
    }
}
unsafe extern "C" fn s_throwlw(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 7.0);
    son(agent, "se_common_elec_s_damage");
    frame(lua, 14.0);
    if is_excute(agent) {
        PLAY_SEQUENCE(agent, Hash40::new("seq_sonic_rnd_attack"));
    }
    son(agent, "se_common_bomb_m");
}

// ------------------------------------------------------------------ Grab Knee Strike, prises
unsafe extern "C" fn g_catchattack(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    FT_MOTION_RATE(agent, 1.0 / 1.5);
    frame(lua, 3.0);
    if is_excute(agent) {
        ATTACK(agent, 0, 0, Hash40::new("throw"), 3.0, 361, 100, 30, 0, 5.0, 0.0, 0.0, 0.0, None, None, None, 2.1, 1.0,
               *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true,
               *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
               Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_KNEE);
        AttackModule::set_catch_only_all(agent.module_accessor, true, false);
    }
    frame(lua, 4.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
    }
}
unsafe extern "C" fn e_catchattack(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 3.0);
    // vidéo REX : éclat blanc sur le genou, anneau de lumière autour de lui
    fx_pose(agent, "sys_attack_impact", "throw", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 1.0, BLANC);
    fx(agent, "sys_smash_flash", "hip", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 1.0, BLANC);
    fx(agent, "sys_smash_flash_s", "kneer", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 0.8, OR);
}
unsafe extern "C" fn e_catch(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 4.0);
    fx(agent, "sys_smash_flash", "hip", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 0.9, BLANC);
}
/// prise en courant : traînée de lumière blanche derrière lui (vidéo REX)
unsafe extern "C" fn e_catchdash(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 1.0);
    fx(agent, "sys_attack_speedline", "top", (0.0, 8.0, -4.0), (0.0, 0.0, 0.0), 1.2, BLANC);
    if is_excute(agent) {
        FOOT_EFFECT(agent, Hash40::new("sys_dash_smoke"), Hash40::new("top"), -4, 0, 0, 0, 0, 0, 0.8, 0, 0, 0, 0, 0, 0, false);
    }
    frame(lua, 6.0);
    fx(agent, "sys_smash_flash", "hip", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 0.9, BLANC);
}

// ------------------------------------------------------------------ Spinning Leg Sweep (attaques au sol)
/// REX DownAttackU : 6 %, angle 361, croissance 50, recul 80, jambe gauche, images 18 et 21
unsafe extern "C" fn g_downattacku(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    for f in [18.0, 21.0] {
        frame(lua, f);
        if is_excute(agent) {
            coup(agent, 0, "legl", 6.0, 361, 50, 80, 3.3, (0.0, 0.0, 0.0), "collision_attr_normal", *COLLISION_SOUND_ATTR_KICK);
            coup(agent, 1, "kneel", 6.0, 361, 50, 80, 3.3, (0.0, 0.0, 0.0), "collision_attr_normal", *COLLISION_SOUND_ATTR_KICK);
            coup(agent, 2, "kneel", 6.0, 361, 50, 80, 4.1, (5.6, 0.0, 0.0), "collision_attr_normal", *COLLISION_SOUND_ATTR_KICK);
        }
        frame(lua, f + 2.0);
        if is_excute(agent) {
            AttackModule::clear_all(agent.module_accessor);
        }
    }
}
/// REX DownAttackD : 6 %, images 17 à 25
unsafe extern "C" fn g_downattackd(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 17.0);
    if is_excute(agent) {
        coup(agent, 0, "footl", 6.0, 361, 50, 80, 4.1, (0.0, 0.0, 0.0), "collision_attr_normal", *COLLISION_SOUND_ATTR_KICK);
        coup(agent, 1, "footr", 6.0, 361, 50, 80, 4.1, (0.0, 0.0, 0.0), "collision_attr_normal", *COLLISION_SOUND_ATTR_KICK);
    }
    frame(lua, 25.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
    }
}
unsafe extern "C" fn e_downattack(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 16.0);
    fx(agent, "sys_attack_arc_d", "top", (0.0, 3.0, 0.0), (0.0, 0.0, 0.0), 1.2, OR);
    if is_excute(agent) {
        LANDING_EFFECT(agent, Hash40::new("sys_atk_smoke"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.8, 0, 0, 0, 0, 0, 0, false);
    }
}

pub fn install(a: &mut Agent) {
    a.acmd("game_throwf", g_throwf, Priority::Low);
    a.acmd("effect_throwf", e_throwf, Priority::Low);
    a.acmd("sound_throwf", s_throwf, Priority::Low);
    a.acmd("game_throwb", g_throwb, Priority::Low);
    a.acmd("effect_throwb", e_throwb, Priority::Low);
    a.acmd("sound_throwb", s_throwb, Priority::Low);
    a.acmd("game_throwhi", g_throwhi, Priority::Low);
    a.acmd("effect_throwhi", e_throwhi, Priority::Low);
    a.acmd("sound_throwhi", s_throwhi, Priority::Low);
    a.acmd("game_throwlw", g_throwlw, Priority::Low);
    a.acmd("effect_throwlw", e_throwlw, Priority::Low);
    a.acmd("sound_throwlw", s_throwlw, Priority::Low);
    a.acmd("game_catchattack", g_catchattack, Priority::Low);
    a.acmd("effect_catchattack", e_catchattack, Priority::Low);
    a.acmd("effect_catch", e_catch, Priority::Low);
    a.acmd("effect_catchturn", e_catch, Priority::Low);
    a.acmd("effect_catchdash", e_catchdash, Priority::Low);
    a.acmd("game_downattacku", g_downattacku, Priority::Low);
    a.acmd("game_downattackd", g_downattackd, Priority::Low);
    a.acmd("effect_downattacku", e_downattack, Priority::Low);
    a.acmd("effect_downattackd", e_downattack, Priority::Low);
}
