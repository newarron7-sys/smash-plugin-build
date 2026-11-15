//! Metal Sonic : prise, coup pendant la prise, lancers avant / haut / bas et provocations de REX (FitMetalSonic.pac,
//! scripts relus le 15/11). Le lancer arrière reste celui de Sonic (demandé : « les throw sauf la back throw »).
//! Mêmes techniques que Super Sonic (lancers.rs) et Master Chief (prises.rs) :
//! - zones de prise de REX (060A0800 : image, taille, hauteur, distance) : prise 5, en courant 9, demi-tour 8 ;
//! - prise en courant : son animation glisse de 20,7 dans REX ; ici elle est sur place et le plugin le fait avancer
//!   (table TransN, avance_rex.py), comme la v11 de Super Sonic ;
//! - coup pendant la prise : image 2, 2 %, électrique ;
//! - lancer avant : coup de pied, lâché à 19 quand le pied le touche (4 + 3 = 7 %, 50°, croissance 60, recul 90) ;
//! - lancer haut : lâché à 23 (4 + 3 = 7 %, 90°, 70 / 99, feu) avec explosion, réacteur à 25 ;
//! - lancer bas : il saute sur l'adversaire (avance de 13 : TransN de REX, faite par le plugin), lâché à 26 quand
//!   l'adversaire touche le sol (6 %, 90°, 50 / 75) ; onde de choc + tremblement à chaque lancer ;
//! - provocations : ses animations ; scripts de Sonic remplacés ; voix de REX (sons.rs).

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

unsafe fn slot(boma: *mut BattleObjectModuleAccessor) -> usize {
    (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7)
}

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
    prise(agent, 5.0, &[(8.8, 4.5), (4.8, 3.5)], 6.0);
}
unsafe extern "C" fn g_catchdash(agent: &mut L2CAgentBase) {
    prise(agent, 9.0, &[(7.52, 3.125), (3.76, 3.125), (0.0, 3.125)], 5.64);
}
unsafe extern "C" fn g_catchturn(agent: &mut L2CAgentBase) {
    prise(agent, 8.0, &[(-13.0, 4.5), (-7.0, 3.5), (-2.0, 3.5)], 7.2);
}

/// coup pendant la prise (REX : image 2, 2 %, électrique) : coup « direct » (sinon l'adversaire se retourne : Chief v28)
unsafe extern "C" fn g_catchattack(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 2.0);
    if is_excute(agent) {
        AttackModule::set_damage_shake_scale(agent.module_accessor, 0.1);
        ATTACK(agent, 0, 0, Hash40::new("top"), 2.0, 361, 100, 30, 0, 5.0, 0.0, 6.6, 8.0, None, None, None, 1.6,
               1.0, *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true,
               *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
               Hash40::new("collision_attr_elec"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_ELEC, *ATTACK_REGION_PUNCH);
        AttackModule::set_catch_only_all(agent.module_accessor, true, false);
    }
    wait(agent.lua_state_agent, 1.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
    }
}

unsafe extern "C" fn e_catchattack(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 2.0);
    if is_excute(agent) {
        EFFECT(agent, Hash40::new("sys_hit_elec"), Hash40::new("top"), 0, 6.6, 8.0, 0, 0, 0, 0.5, 0, 0, 0, 0, 0, 0, true);
    }
}

/// lâcher : l'adversaire part avec le coup du lancer (ATTACK_ABS THROW)
unsafe fn lacher(agent: &mut L2CAgentBase) {
    let boma = agent.module_accessor;
    let target = WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_OBJECT);
    let group = WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_HIT_GROUP);
    let no = WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_HIT_NO);
    smash_script::macros::ATK_HIT_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_THROW, Hash40::new("throw"), target, group, no);
}

/// réglages du lancer (id 0) et de la prise cassée (id 1), comme les 060E1100 de REX
unsafe fn lancer_abs(agent: &mut L2CAgentBase, dmg: f32, ang: u64, kbg: i32, bkb: i32, attr: &str, c: (f32, u64, i32, i32)) {
    smash_script::macros::ATTACK_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_THROW, 0, dmg, ang, kbg, 0, bkb, 0.0, 1.0, *ATTACK_LR_CHECK_F, 0.0, true,
               Hash40::new(attr), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_NONE, *ATTACK_REGION_THROW);
    smash_script::macros::ATTACK_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_CATCH, 0, c.0, c.1, c.2, 0, c.3, 0.0, 1.0, *ATTACK_LR_CHECK_F, 0.0, true,
               Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_NONE, *ATTACK_REGION_THROW);
}

#[allow(dead_code)]
/// coup sur l'adversaire tenu (os throw), quelques images avant le lâcher (piège de PROCEDURE.md : pas juste avant)
unsafe fn coup_tenu(agent: &mut L2CAgentBase, dmg: f32, size: f32, attr: &str, son: i32) {
    ATTACK(agent, 0, 0, Hash40::new("throw"), dmg, 80, 100, 30, 0, size, 0.0, 0.0, 0.0, None, None, None, 1.0, 1.0,
           *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true,
           *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
           Hash40::new(attr), *ATTACK_SOUND_LEVEL_M, son, *ATTACK_REGION_THROW);
    AttackModule::set_catch_only_all(agent.module_accessor, true, false);
}

/// lâcher + onde de choc et tremblement (demandé le 15/11 : « des ondes de choc et un tremblement à toutes ces throw »)
/// p = point de l'onde (top : y haut, z devant) ; sol = anneau au sol en plus
unsafe fn choc(agent: &mut L2CAgentBase, p: (f32, f32), sc: f32, sol: bool, quake: i32) {
    EFFECT(agent, Hash40::new("sys_crown_collision"), Hash40::new("top"), 0, p.0, p.1, 0, 0, 0, sc, 0, 0, 0, 0, 0, 0, false);
    if sol {
        LANDING_EFFECT(agent, Hash40::new("sys_crown"), Hash40::new("top"), 0, 0, p.1, 0, 0, 0, sc, 0, 0, 0, 0, 0, 0, false);
    }
    QUAKE(agent, quake);
}

// ------------------------------------------------------------------ lancer avant
// v3b (vidéo du 15/11 : « il reçoit des dégâts et il est éjecté avant que le coup de pied le touche ») : animation de REX
// relue image par image (/tmp/an/trace.py) : le pied gauche part à 17 (z 1,2) et touche à 18-19 (z 10,9 puis 12,9 ;
// l'adversaire est à z 14,3). Le coup tenu à 14 (pied encore replié) donnait les dégâts trop tôt et son gel décalait
// tout : retiré (PROCEDURE.md : pas de vrai coup juste avant le lâcher) ; ses 3 % s'ajoutent au lancer (4 + 3 = 7 %),
// lâché à 19 quand le pied est vraiment sur lui.
unsafe extern "C" fn g_throwf(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    if is_excute(agent) {
        lancer_abs(agent, 7.0, 50, 60, 90, "collision_attr_normal", (3.0, 45, 100, 40));
    }
    // REX : vitesse 1,2 jusqu'à l'image 17
    FT_MOTION_RATE(agent, 1.0 / 1.2);
    frame(lua, 17.0);
    FT_MOTION_RATE(agent, 1.0);
    frame(lua, 19.0);
    if is_excute(agent) {
        smash_script::macros::CHECK_FINISH_CAMERA(agent, 9, 6);
        lacher(agent);
    }
}
unsafe extern "C" fn e_throwf(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 19.0);
    if is_excute(agent) {
        EFFECT(agent, Hash40::new("sys_attack_impact"), Hash40::new("footl"), 0, 0, 0, 0, 0, 0, 1.4, 0, 0, 0, 0, 0, 0, true);
        choc(agent, (8.4, 13.0), 0.9, true, *CAMERA_QUAKE_KIND_M);
    }
}

// ------------------------------------------------------------------ lancer haut
// REX : il le lève au-dessus de lui (ThrowN à 26 de haut à l'image 10), il redescend sur sa tête (11 de haut à 21),
// coup de feu à 20, lâché à 23 vers le haut. v3b : même correction que le lancer avant (coup tenu retiré, 4 + 3 = 7 %)
// et explosion au lâcher (demandé, vidéo du 15/11 : « Rajoute une explosion »).
unsafe extern "C" fn g_throwhi(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    if is_excute(agent) {
        lancer_abs(agent, 7.0, 90, 70, 99, "collision_attr_fire", (3.0, 361, 100, 44));
    }
    frame(lua, 23.0);
    if is_excute(agent) {
        smash_script::macros::CHECK_FINISH_CAMERA(agent, 1, 20);
        lacher(agent);
    }
}
unsafe extern "C" fn e_throwhi(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 20.0);
    if is_excute(agent) {
        EFFECT(agent, Hash40::new("sys_flame"), Hash40::new("throw"), 0, 0, 0, 0, 0, 0, 0.7, 0, 0, 0, 0, 0, 0, true);
    }
    frame(lua, 23.0);
    if is_excute(agent) {
        // explosion sur l'adversaire (au-dessus de sa tête) au moment où il part
        EFFECT(agent, Hash40::new("sys_bomb_a"), Hash40::new("top"), 0, 12, 0, 0, 0, 0, 1.0, 0, 0, 0, 0, 0, 0, false);
        EFFECT(agent, Hash40::new("sys_bomb_b"), Hash40::new("top"), 0, 12, 0, 0, 0, 0, 0.9, 0, 0, 0, 0, 0, 0, false);
        choc(agent, (11.0, 0.0), 1.0, true, *CAMERA_QUAKE_KIND_M);
    }
    frame(lua, 25.0);
    if is_excute(agent) {
        EFFECT_FOLLOW(agent, Hash40::new("sys_flame"), Hash40::new("bust"), 0, 0, 0, 0, 0, 0, 0.8, true);
    }
}

// ------------------------------------------------------------------ lancer bas
// v3b (vidéo du 15/11 : « il l'a même pas vraiment balancé au sol qu'il part déjà, et l'onde de choc est même pas au
// sol ») : REX relu image par image : il saute avec l'adversaire au-dessus de lui (ThrowN à 22 de haut à l'image 22,
// le sommet), le jette, et l'adversaire touche le sol à 26 (ThrowN y 1,3, à 11 devant lui) ; lui atterrit à 25-26.
// L'onde était à 22 sur l'os throw (= au sommet, en l'air) et le lâcher à 25 (encore à 10 de haut) : lâcher à 26,
// onde de choc au sol, à l'endroit où l'adversaire s'écrase.
unsafe extern "C" fn g_throwlw(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    if is_excute(agent) {
        lancer_abs(agent, 6.0, 90, 50, 75, "collision_attr_normal", (3.0, 45, 100, 40));
    }
    frame(lua, 26.0);
    if is_excute(agent) {
        smash_script::macros::CHECK_FINISH_CAMERA(agent, 2, 0);
        lacher(agent);
    }
    // REX : vitesse 1,35 après le lâcher
    FT_MOTION_RATE(agent, 1.0 / 1.35);
}
unsafe extern "C" fn e_throwlw(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 26.0);
    if is_excute(agent) {
        LANDING_EFFECT(agent, Hash40::new("sys_down_smoke"), Hash40::new("top"), 0, 0, 11, 0, 0, 0, 1.0, 0, 0, 0, 0, 0, 0, false);
        EFFECT(agent, Hash40::new("sys_attack_impact"), Hash40::new("top"), 0, 1, 11, 0, 0, 0, 1.3, 0, 0, 0, 0, 0, 0, false);
        choc(agent, (1.0, 11.0), 1.1, true, *CAMERA_QUAKE_KIND_L);
    }
}

// ------------------------------------------------------------------ déplacements réels (animations sur place)
// TransN de REX, image par image depuis l'image 0 (rex/tools/avance_rex.py catch_dash=CatchDash throw_lw=ThrowLw)
const AV_CD: [f32; 40] = [0.000, 0.925, 2.438, 3.595, 4.705, 5.767, 6.794, 7.785, 8.730, 9.626, 10.483, 11.297, 12.067,
    12.797, 13.483, 14.134, 14.746, 15.319, 15.861, 16.368, 16.838, 17.272, 17.679, 18.061, 18.411, 18.726, 19.011,
    19.274, 19.516, 19.731, 19.917, 20.079, 20.223, 20.347, 20.449, 20.531, 20.595, 20.645, 20.677, 20.688];
const AV_LW: [f32; 51] = [0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.487, 1.041, 1.778, 2.622, 3.494,
    4.313, 5.000, 5.519, 5.921, 6.258, 6.580, 6.938, 7.384, 8.000, 8.653, 9.579, 11.351, 13.000, 13.300, 13.206, 13.248,
    13.300, 13.291, 13.265, 13.224, 13.166, 13.091, 13.000, 12.872, 12.710, 12.542, 12.400, 12.286, 12.182, 12.086,
    11.997, 11.912, 11.831, 11.751, 11.670, 11.587, 11.500];
static mut CD_ON: [bool; 8] = [false; 8];
static mut LW_ON: [bool; 8] = [false; 8];

fn table(t: &[f32], f: f32) -> f32 {
    let n = t.len();
    let k = f.clamp(0.0, (n - 1) as f32);
    let i = (k as usize).min(n - 2);
    let a = k - i as f32;
    t[i] + (t[i + 1] - t[i]) * a
}

unsafe fn avance(fighter: &mut L2CFighterCommon, t: &[f32], f: f32) {
    let boma = fighter.module_accessor;
    let rate = MotionModule::rate(boma).max(0.01);
    let d = table(t, f + rate) - table(t, f);
    let lr = PostureModule::lr(boma);
    sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, d * lr, 0.0_f32);
}

/// chaque image (ss_frame)
pub unsafe fn chaque_image(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    let st = StatusModule::status_kind(boma);
    let mk = MotionModule::motion_kind(boma);
    let f = MotionModule::frame(boma);
    let sol = StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND;
    // prise en courant : il glisse vraiment, comme dans REX (arrêt au bord)
    if st == *FIGHTER_STATUS_KIND_CATCH_DASH && mk == hash40("catch_dash") && sol {
        if !CD_ON[e] {
            CD_ON[e] = true;
            KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_STOP);
        }
        avance(fighter, &AV_CD, f);
    } else if CD_ON[e] {
        CD_ON[e] = false;
        sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0_f32, 0.0_f32);
    }
    // lancer bas : il saute sur l'adversaire (avance de REX)
    if st == *FIGHTER_STATUS_KIND_THROW && mk == hash40("throw_lw") && sol {
        if !LW_ON[e] {
            LW_ON[e] = true;
            KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
            GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND_CLIFF_STOP));
        }
        avance(fighter, &AV_LW, f);
    } else if LW_ON[e] {
        LW_ON[e] = false;
        sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0_f32, 0.0_f32);
    }
    // provocations : retour à l'attente à la fin de son animation de REX (animations de Sonic parfois en boucle)
    if st == *FIGHTER_STATUS_KIND_APPEAL {
        if f + MotionModule::rate(boma) >= MotionModule::end_frame(boma) - 1.0 {
            StatusModule::change_status_request_from_script(boma, *FIGHTER_STATUS_KIND_WAIT, false);
        }
    }
}

unsafe extern "C" fn vide(_agent: &mut L2CAgentBase) {}

unsafe extern "C" fn x_appeal(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        slope!(agent, *MA_MSC_CMD_SLOPE_SLOPE, *SLOPE_STATUS_LR);
    }
}

/// provocation côté (REX : vitesse 1,2 à partir de l'image 20)
unsafe extern "C" fn g_appeals(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 20.0);
    FT_MOTION_RATE(agent, 1.0 / 1.2);
}

/// provocation bas (REX : éclat à l'image 48)
unsafe extern "C" fn e_appeallw(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 48.0);
    if is_excute(agent) {
        EFFECT_FOLLOW(agent, Hash40::new("sys_smash_flash_s"), Hash40::new("head"), 0.5, 3.4, 1.7, 0, 0, 0, 0.5, true);
    }
}

pub fn install(a: &mut Agent) {
    a.acmd("game_catch", g_catch, Priority::Low);
    a.acmd("game_catchdash", g_catchdash, Priority::Low);
    a.acmd("game_catchturn", g_catchturn, Priority::Low);
    for n in ["effect_catch", "effect_catchdash", "effect_catchturn", "expression_catchattack"] {
        a.acmd(n, vide, Priority::Low);
    }
    a.acmd("game_catchattack", g_catchattack, Priority::Low);
    a.acmd("effect_catchattack", e_catchattack, Priority::Low);
    a.acmd("game_throwf", g_throwf, Priority::Low);
    a.acmd("effect_throwf", e_throwf, Priority::Low);
    a.acmd("game_throwhi", g_throwhi, Priority::Low);
    a.acmd("effect_throwhi", e_throwhi, Priority::Low);
    a.acmd("game_throwlw", g_throwlw, Priority::Low);
    a.acmd("effect_throwlw", e_throwlw, Priority::Low);
    // provocations : scripts de Sonic remplacés
    for m in ["appealhir", "appealhil", "appealsr", "appealsl", "appeallwr", "appeallwl"] {
        let g: &'static str = Box::leak(format!("game_{}", m).into_boxed_str());
        let e: &'static str = Box::leak(format!("effect_{}", m).into_boxed_str());
        let x: &'static str = Box::leak(format!("expression_{}", m).into_boxed_str());
        let gf: unsafe extern "C" fn(&mut L2CAgentBase) = if m.starts_with("appeals") { g_appeals } else { vide };
        let ef: unsafe extern "C" fn(&mut L2CAgentBase) = if m.starts_with("appeallw") { e_appeallw } else { vide };
        a.acmd(g, gf, Priority::Low);
        a.acmd(e, ef, Priority::Low);
        a.acmd(x, x_appeal, Priority::Low);
    }
}
