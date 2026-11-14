//! Metal Sonic, spéciaux de REX (FitMetalSonic.pac, construit sur Wolf, + vidéo du showcase REX envoyée par le joueur
//! le 14/11, noms des coups affichés dedans). Même façon de faire que Super Sonic (statuts de Sonic remplacés, sur sa
//! case seulement) ; animations de REX dans les fichiers de Sonic (retarget_exo.py, groupe metal_speciaux, sur place :
//! le déplacement est fait ici).
//!
//! - B neutre « Ring Spark Field » : appui = décharge électrique tout de suite (« no charge ») ; B tenu = il se charge
//!   (éclairs, il brille) puis lâche une décharge plus grosse (« full charge »). REX : SpecialNShot (charge),
//!   SpecialN (décharge : 14 % / 361° / élec, 10 % à la fin).
//! - side B « V.Maximum Overdrive » : départ (16 images) puis ruée vers l'avant (14) dans une aura dorée et rose,
//!   puis fin (41). REX : SpecialS 11 % / 361° (corps), SpecialSStart 15 % (tête, élec 35°). En l'air : chute
//!   spéciale à la fin.
//! - haut B « Jet Boost » (déplacement) : charge courte (10), explosion de feu au départ (REX : 9 % / 35° / feu), il
//!   file dans la direction du stick (22, 5 % puis 3 %), traînée de fumée, puis chute spéciale.
//! - bas B « Black Shield » : bouclier de cristal autour de lui ; s'il est touché pendant le bouclier, il ne prend
//!   rien et riposte (vidéo : l'adversaire prend 12 % et part) ; sinon le bouclier disparaît. En l'air, il garde son
//!   élan horizontal (« horizontal momentum »).

use smash::app::lua_bind::*;
use smash::app::sv_animcmd::frame;
use smash::app::*;
use smash::hash40;
use smash::lib::lua_const::*;
use smash::lib::L2CValue;
use smash::lua2cpp::*;
use smash::phx::{Hash40, Vector3f};
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

const ROSE: (f32, f32, f32) = (1.0, 0.35, 0.85);
const OR: (f32, f32, f32) = (1.0, 0.8, 0.25);
const CRISTAL: (f32, f32, f32) = (0.75, 0.95, 1.0);

unsafe fn slot(boma: *mut BattleObjectModuleAccessor) -> usize {
    (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7)
}

unsafe fn fx(boma: *mut BattleObjectModuleAccessor, nom: &str, os: &str, p: (f32, f32, f32), sc: f32, c: Option<(f32, f32, f32)>) -> u32 {
    let z = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    let v = Vector3f { x: p.0, y: p.1, z: p.2 };
    let h = EffectModule::req_follow(boma, Hash40::new(nom), Hash40::new(os), &v, &z, sc, true, 0, 0, 0, 0, 0, false, false) as u32;
    if let (Some(c), true) = (c, h != 0) {
        EffectModule::set_rgb(boma, h, c.0, c.1, c.2);
    }
    h
}

unsafe fn corps_normal(boma: *mut BattleObjectModuleAccessor) {
    VisibilityModule::set_whole(boma, true);
    VisibilityModule::set_int64(boma, hash40("body"), hash40("body_normal"));
}

/// vitesse posée directement (énergie STOP, gravité coupée) : x dans sa direction
unsafe fn vitesse(fighter: &mut L2CFighterCommon, vx: f32, vy: f32) {
    let lr = PostureModule::lr(fighter.module_accessor);
    sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, vx * lr, vy);
}

/// physique en l'air pilotée par le plugin (comme Super Sonic : énergie STOP libre, sans limite)
unsafe fn en_l_air(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    StatusModule::set_situation_kind(boma, SituationKind(*SITUATION_KIND_AIR), true);
    GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
    KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_AIR_STOP);
    KineticModule::unable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    sv_kinetic_energy!(reset_energy, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, *ENERGY_STOP_RESET_TYPE_FREE, 0.0, 0.0, 0.0, 0.0, 0.0);
    sv_kinetic_energy!(set_accel, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0, 0.0);
    sv_kinetic_energy!(set_stable_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0, 0.0);
    sv_kinetic_energy!(set_limit_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, -1.0, -1.0);
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_STOP);
}

unsafe fn au_sol_stop(boma: *mut BattleObjectModuleAccessor) {
    KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
    GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND_CLIFF_STOP));
}

unsafe fn chute_normale(fighter: &mut L2CFighterCommon, vy: f32) {
    let boma = fighter.module_accessor;
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_FALL);
    sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY, vy);
}

unsafe fn finir(fighter: &mut L2CFighterCommon, chute_speciale: bool) {
    let boma = fighter.module_accessor;
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND {
        fighter.change_status(FIGHTER_STATUS_KIND_WAIT.into(), false.into());
    } else if chute_speciale {
        fighter.change_status(FIGHTER_STATUS_KIND_FALL_SPECIAL.into(), false.into());
    } else {
        fighter.change_status(FIGHTER_STATUS_KIND_FALL.into(), false.into());
    }
}

/// coup posé par un script (os, dégâts, angle, croissance, recul, taille, hauteur, élément)
unsafe fn coup(agent: &mut L2CAgentBase, id: u64, os: &str, dmg: f32, ang: u64, kbg: i32, bkb: i32, size: f32, y: f32, z: f32, attr: &str) {
    let son = match attr {
        "collision_attr_elec" => *COLLISION_SOUND_ATTR_ELEC,
        "collision_attr_fire" => *COLLISION_SOUND_ATTR_FIRE,
        _ => *COLLISION_SOUND_ATTR_KICK,
    };
    ATTACK(agent, id, 0, Hash40::new(os), dmg, ang, kbg, 0, bkb, size, 0.0, y, z, None, None, None,
           1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true,
           *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
           Hash40::new(attr), *ATTACK_SOUND_LEVEL_L, son, *ATTACK_REGION_BODY);
}

unsafe extern "C" fn rien_acmd(_agent: &mut L2CAgentBase) {}
unsafe extern "C" fn rien(_fighter: &mut L2CFighterCommon) -> L2CValue {
    0.into()
}

// ================================================================== B neutre « Ring Spark Field »
const N_CHARGE_MAX: i32 = 60;
/// appui court : décharge tout de suite (vidéo « no charge »)
const N_APPUI: i32 = 8;
static mut N_PHASE: [u8; 8] = [0; 8];
static mut N_T: [i32; 8] = [0; 8];
/// niveau de charge de la décharge (0 à 1), lu par le script de la décharge
static mut N_NIVEAU: [f32; 8] = [0.0; 8];

unsafe extern "C" fn n_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    N_PHASE[e] = 0;
    N_T[e] = 0;
    N_NIVEAU[e] = 0.0;
    corps_normal(boma);
    if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND {
        au_sol_stop(boma);
        MotionModule::change_motion(boma, Hash40::new("special_n_start"), 0.0, 1.0, false, 0.0, false, false);
    } else {
        en_l_air(fighter);
        vitesse(fighter, 0.0, 0.0);
        MotionModule::change_motion(boma, Hash40::new("special_air_n_start"), 0.0, 1.0, false, 0.0, false, false);
    }
    fighter.sub_shift_status_main(L2CValue::Ptr(n_loop as *const () as _))
}

unsafe fn n_decharge(fighter: &mut L2CFighterCommon, niveau: f32) {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    N_PHASE[e] = 1;
    N_NIVEAU[e] = niveau;
    MotionModule::change_motion(boma, Hash40::new("special_n_spin"), 0.0, 1.0, false, 0.0, false, false);
}

unsafe extern "C" fn n_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    corps_normal(boma);
    let air = StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR;
    if air {
        // il descend lentement pendant la charge et la décharge
        vitesse(fighter, 0.0, -0.4);
    }
    if N_PHASE[e] == 0 {
        N_T[e] += 1;
        let t = N_T[e];
        let tenu = ControlModule::check_button_on(boma, *CONTROL_PAD_BUTTON_SPECIAL);
        // charge : il brille et des éclairs roses courent sur lui
        if t % 6 == 0 {
            let os = ["bust", "hip", "head", "handl", "handr", "footl", "footr"];
            fx(boma, "sys_hit_elec", os[(t as usize / 6) % os.len()], (0.0, 0.0, 0.0), 0.3 + 0.3 * t as f32 / N_CHARGE_MAX as f32, Some(ROSE));
        }
        if t == N_CHARGE_MAX {
            fx(boma, "sys_smash_flash", "top", (0.0, 8.0, 0.0), 1.3, Some(ROSE));
        }
        if MotionModule::is_end(boma) {
            MotionModule::set_frame(boma, 0.0, false);
        }
        if !tenu || t >= N_CHARGE_MAX {
            let niveau = if t <= N_APPUI { 0.0 } else { (t as f32 / N_CHARGE_MAX as f32).min(1.0) };
            n_decharge(fighter, niveau);
        }
        return 0.into();
    }
    if MotionModule::is_end(boma) {
        finir(fighter, false);
        return 1.into();
    }
    if !air && StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR {
        fighter.change_status(FIGHTER_STATUS_KIND_FALL.into(), false.into());
    }
    0.into()
}

unsafe extern "C" fn n_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    N_PHASE[slot(boma)] = 0;
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    0.into()
}

/// décharge (REX SpecialN : 14 % / 361° / élec puis 10 %) : plus grosse et plus forte avec la charge
unsafe extern "C" fn g_decharge(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    let k = N_NIVEAU[slot(agent.module_accessor)];
    frame(lua, 6.0);
    if is_excute(agent) {
        coup(agent, 0, "bust", 10.0 + 8.0 * k, 361, 80, 50 + (30.0 * k) as i32, 9.0 + 6.0 * k, 0.0, 0.0, "collision_attr_elec");
    }
    frame(lua, 12.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
    }
}

unsafe extern "C" fn e_decharge(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    let boma = agent.module_accessor;
    let k = N_NIVEAU[slot(boma)];
    frame(lua, 6.0);
    if is_excute(agent) {
        fx(boma, "sys_smash_flash", "top", (0.0, 7.0, 0.0), 1.4 + 1.2 * k, Some(ROSE));
        fx(boma, "sys_thunder", "top", (0.0, 7.0, 0.0), 0.6 + 0.5 * k, Some(ROSE));
        for os in ["bust", "hip", "head", "handl", "handr", "footl", "footr"] {
            fx(boma, "sys_hit_elec", os, (0.0, 0.0, 0.0), 0.5 + 0.4 * k, Some(ROSE));
        }
    }
    frame(lua, 9.0);
    if is_excute(agent) {
        fx(boma, "sys_hit_elec", "top", (0.0, 7.0, 0.0), 1.0 + 0.8 * k, Some(ROSE));
    }
}

unsafe extern "C" fn s_decharge(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 6.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_common_electric_hit_l"));
    }
}

// ================================================================== side B « V.Maximum Overdrive »
/// parties de special_s_hold / special_air_s_end_loop : départ 0-16, ruée 16-30, fin 30-71
const S_RUEE: f32 = 16.0;
const S_FIN: f32 = 30.0;
const S_VITESSE: f32 = 3.6;
static mut S_AIR: [bool; 8] = [false; 8];

unsafe extern "C" fn s_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    corps_normal(boma);
    let sol = StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND;
    S_AIR[e] = !sol;
    if sol {
        au_sol_stop(boma);
    } else {
        en_l_air(fighter);
        vitesse(fighter, 0.0, 0.0);
    }
    let mot = if sol { "special_s_hold" } else { "special_air_s_end_loop" };
    MotionModule::change_motion(boma, Hash40::new(mot), 0.0, 1.0, false, 0.0, false, false);
    fighter.sub_shift_status_main(L2CValue::Ptr(s_loop as *const () as _))
}

unsafe extern "C" fn s_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    corps_normal(boma);
    let f = MotionModule::frame(boma);
    let v = if f < S_RUEE { 0.2 } else if f < S_FIN { S_VITESSE } else { (S_VITESSE * (1.0 - (f - S_FIN) / 14.0)).max(0.0) };
    let sol = StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND;
    if sol {
        let lr = PostureModule::lr(boma);
        sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, v * lr, 0.0_f32);
    } else {
        vitesse(fighter, v, if f < S_FIN { 0.0 } else { -0.6 });
    }
    if MotionModule::is_end(boma) || f >= MotionModule::end_frame(boma) - 1.0 {
        finir(fighter, S_AIR[slot(boma)] && !sol);
        return 1.into();
    }
    0.into()
}

unsafe extern "C" fn s_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    KineticModule::enable_energy(fighter.module_accessor, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    0.into()
}

/// ruée : coup de tête au départ de la ruée (15 %, élec 35°), puis le corps (11 % / 361°) jusqu'à la fin
unsafe extern "C" fn g_ruee(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, S_RUEE);
    if is_excute(agent) {
        JostleModule::set_status(agent.module_accessor, false);
        coup(agent, 0, "head", 15.0, 35, 75, 55, 5.0, 0.0, 0.0, "collision_attr_elec");
        coup(agent, 1, "waist", 11.0, 361, 70, 60, 6.0, 0.0, 0.0, "collision_attr_normal");
    }
    frame(lua, S_RUEE + 4.0);
    if is_excute(agent) {
        AttackModule::clear(agent.module_accessor, 0, false);
    }
    frame(lua, S_FIN);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
        JostleModule::set_status(agent.module_accessor, true);
    }
}

/// vidéo : anneau doré et rose au départ, traînée dorée pendant la ruée
unsafe extern "C" fn e_ruee(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    let boma = agent.module_accessor;
    frame(lua, 2.0);
    if is_excute(agent) {
        fx(boma, "sys_smash_flash", "top", (0.0, 7.0, 0.0), 1.2, Some(OR));
    }
    frame(lua, S_RUEE - 2.0);
    if is_excute(agent) {
        fx(boma, "sys_smash_flash_s", "top", (0.0, 7.0, 0.0), 1.6, Some(ROSE));
    }
    let mut f = S_RUEE;
    while f < S_FIN {
        frame(lua, f);
        if is_excute(agent) {
            EFFECT_FOLLOW(agent, Hash40::new("sys_attack_speedline"), Hash40::new("top"), 0, 7.0, -3.0, 0, 0, 0, 1.2, true);
            LAST_EFFECT_SET_COLOR(agent, OR.0, OR.1, OR.2);
            fx(boma, "sys_hit_elec", "bust", (0.0, 0.0, 0.0), 0.3, Some(ROSE));
        }
        f += 3.0;
    }
}

unsafe extern "C" fn s_ruee(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, S_RUEE);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_sonic_dash_start"));
    }
}

// ================================================================== haut B « Jet Boost »
/// special_hi : charge 0-10, envol 10-32, chute 32-52
const HI_ENVOL: i32 = 10;
const HI_FIN: i32 = 32;
const HI_VITESSE: f32 = 3.2;
static mut HI_T: [i32; 8] = [0; 8];
static mut HI_V: [(f32, f32); 8] = [(0.0, 0.0); 8];
static mut HI_FAIT: [bool; 8] = [false; 8];

unsafe extern "C" fn hi_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    // ressort de Sonic (article GIMMICKJUMP) : jamais créé, retiré s'il en reste un (Super Sonic v12b)
    ArticleModule::remove_exist(boma, *FIGHTER_SONIC_GENERATE_ARTICLE_GIMMICKJUMP, ArticleOperationTarget(*ARTICLE_OPE_TARGET_ALL));
    HI_FAIT[e] = true;
    HI_T[e] = 0;
    HI_V[e] = (0.0, 0.0);
    corps_normal(boma);
    en_l_air(fighter);
    vitesse(fighter, 0.0, 0.0);
    MotionModule::change_motion(boma, Hash40::new("special_hi"), 0.0, 1.0, false, 0.0, false, false);
    fighter.sub_shift_status_main(L2CValue::Ptr(hi_loop as *const () as _))
}

unsafe extern "C" fn hi_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    if StopModule::is_stop(boma) {
        return 0.into();
    }
    corps_normal(boma);
    HI_T[e] += 1;
    let t = HI_T[e];
    if t < HI_ENVOL {
        vitesse(fighter, 0.0, 0.1);
        return 0.into();
    }
    if t == HI_ENVOL {
        // direction du stick (vers le haut sans stick), jamais vers le bas
        let lr = PostureModule::lr(boma);
        let mut sx = ControlModule::get_stick_x(boma) * lr;
        let mut sy = ControlModule::get_stick_y(boma);
        if sx < -0.3 {
            PostureModule::reverse_lr(boma);
            PostureModule::update_rot_y_lr(boma);
            sx = -sx;
        }
        if sx.abs() < 0.2 && sy.abs() < 0.2 {
            sx = 0.0;
            sy = 1.0;
        }
        sy = sy.max(-0.2);
        let n = (sx * sx + sy * sy).sqrt().max(0.01);
        HI_V[e] = (HI_VITESSE * sx / n, HI_VITESSE * sy / n);
        // explosion de feu au départ (vidéo), là où il était
        fx(boma, "sys_bomb_a", "top", (0.0, 2.0, 0.0), 0.8, None);
    }
    let (vx, vy) = HI_V[e];
    vitesse(fighter, vx, vy);
    if t % 3 == 0 {
        fx(boma, "sys_atk_smoke", "top", (0.0, 6.0, -2.0), 0.9, None);
    }
    if fighter.sub_transition_group_check_air_cliff().get_bool() {
        return 1.into();
    }
    if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND && t > HI_ENVOL + 4 {
        fighter.change_status(FIGHTER_STATUS_KIND_LANDING_FALL_SPECIAL.into(), false.into());
        return 1.into();
    }
    if t >= HI_FIN {
        chute_normale(fighter, vy * 0.4);
        fighter.change_status(FIGHTER_STATUS_KIND_FALL_SPECIAL.into(), false.into());
        return 1.into();
    }
    0.into()
}

unsafe extern "C" fn hi_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    HI_T[slot(boma)] = 0;
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    0.into()
}

/// une fois par saut (comme le Flight de Super Sonic) : bloqué en l'air tant qu'il n'a pas touché le sol / le bord
pub unsafe fn frame_hi(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let st = StatusModule::status_kind(boma);
    let e = slot(boma);
    if HI_FAIT[e] && StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR && st != *FIGHTER_STATUS_KIND_SPECIAL_HI {
        WorkModule::unable_transition_term(boma, *FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_SPECIAL_HI);
    }
    if StatusModule::situation_kind(boma) != *SITUATION_KIND_AIR || st == *FIGHTER_STATUS_KIND_CLIFF_CATCH
        || st == *FIGHTER_STATUS_KIND_DAMAGE_FLY || st == *FIGHTER_STATUS_KIND_DEAD
    {
        HI_FAIT[e] = false;
    }
}

/// envol : explosion (REX : 9 % / 35° / feu), puis le corps (5 % / 64° / feu, puis 3 % / 70°)
unsafe extern "C" fn g_envol(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, HI_ENVOL as f32);
    if is_excute(agent) {
        coup(agent, 0, "top", 9.0, 35, 80, 60, 9.0, 3.0, 0.0, "collision_attr_fire");
    }
    frame(lua, HI_ENVOL as f32 + 2.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
        coup(agent, 0, "hip", 5.0, 64, 70, 40, 5.0, 0.0, 0.0, "collision_attr_fire");
    }
    frame(lua, HI_ENVOL as f32 + 8.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
        coup(agent, 0, "hip", 3.0, 70, 70, 30, 4.5, 0.0, 0.0, "collision_attr_normal");
    }
    frame(lua, HI_FIN as f32);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
    }
}

unsafe extern "C" fn e_envol(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, HI_ENVOL as f32);
    if is_excute(agent) {
        EFFECT(agent, Hash40::new("sys_bomb_b"), Hash40::new("top"), 0, 1.0, 0, 0, 0, 0, 0.7, 0, 0, 0, 0, 0, 0, true);
    }
}

unsafe extern "C" fn s_envol(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, HI_ENVOL as f32);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_common_bomb_m"));
    }
}

// ================================================================== bas B « Black Shield »
/// bouclier actif de l'image 4 à 34 du statut ; riposte si touché pendant ce temps
const LW_DEBUT: i32 = 4;
const LW_DUREE: i32 = 34;
static mut LW_T: [i32; 8] = [0; 8];
/// 0 : bouclier, 1 : riposte, 2 : fin sans riposte
static mut LW_PHASE: [u8; 8] = [0; 8];
static mut LW_VX: [f32; 8] = [0.0; 8];
static mut LW_FX: [u32; 8] = [0; 8];
static mut LW_RIPOSTE: [bool; 8] = [false; 8];

unsafe fn lw_bouclier(boma: *mut BattleObjectModuleAccessor, on: bool) {
    if on {
        DamageModule::set_no_reaction_mode_status(boma, DamageNoReactionMode { _address: *DAMAGE_NO_REACTION_MODE_ALWAYS as u8 }, -1.0, -1.0, -1);
        DamageModule::set_damage_mul(boma, 0.0);
    } else {
        DamageModule::reset_no_reaction_mode_status(boma);
        DamageModule::set_damage_mul(boma, 1.0);
    }
}

unsafe fn lw_fx_off(boma: *mut BattleObjectModuleAccessor, e: usize) {
    if LW_FX[e] != 0 && EffectModule::is_exist_effect(boma, LW_FX[e]) {
        EffectModule::kill(boma, LW_FX[e], false, true);
    }
    LW_FX[e] = 0;
}

unsafe extern "C" fn lw_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    LW_T[e] = 0;
    LW_PHASE[e] = 0;
    LW_RIPOSTE[e] = false;
    LW_FX[e] = 0;
    corps_normal(boma);
    let lr = PostureModule::lr(boma);
    // vidéo : il garde son élan horizontal (« horizontal momentum during dashing »)
    LW_VX[e] = KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN) * lr;
    if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND {
        au_sol_stop(boma);
        MotionModule::change_motion(boma, Hash40::new("special_lw_start"), 0.0, 1.0, false, 0.0, false, false);
    } else {
        en_l_air(fighter);
        MotionModule::change_motion(boma, Hash40::new("special_air_lw_start"), 0.0, 1.0, false, 0.0, false, false);
    }
    fighter.sub_shift_status_main(L2CValue::Ptr(lw_loop as *const () as _))
}

unsafe extern "C" fn lw_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    corps_normal(boma);
    LW_T[e] += 1;
    let t = LW_T[e];
    let sol = StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND;
    LW_VX[e] *= 0.92;
    let lr = PostureModule::lr(boma);
    if sol {
        sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, LW_VX[e] * lr, 0.0_f32);
    } else {
        vitesse(fighter, LW_VX[e], -0.35);
    }
    match LW_PHASE[e] {
        0 => {
            if t == LW_DEBUT {
                lw_bouclier(boma, true);
                // cristal autour de lui (vidéo : losange transparent) + éclat en croix à l'apparition
                LW_FX[e] = fx(boma, "sys_shield", "top", (0.0, 7.0, 0.0), 1.15, Some(CRISTAL));
                fx(boma, "sys_smash_flash_s", "top", (0.0, 7.0, 0.0), 1.5, Some(CRISTAL));
            }
            if t > LW_DEBUT && LW_FX[e] != 0 && !EffectModule::is_exist_effect(boma, LW_FX[e]) {
                LW_FX[e] = fx(boma, "sys_shield", "top", (0.0, 7.0, 0.0), 1.15, Some(CRISTAL));
            }
            let mot_hold = if sol { "special_lw_hold" } else { "special_air_lw_hold" };
            if MotionModule::is_end(boma) {
                MotionModule::change_motion(boma, Hash40::new(mot_hold), 0.0, 1.0, false, 0.0, false, false);
            }
            // touché pendant le bouclier : il ne prend rien et riposte
            if t > LW_DEBUT && StopModule::is_damage(boma) {
                LW_PHASE[e] = 1;
                LW_RIPOSTE[e] = true;
                lw_fx_off(boma, e);
                fx(boma, "sys_hit_elec", "top", (0.0, 7.0, 0.0), 1.2, Some(CRISTAL));
                let mot = if sol { "special_lw_end" } else { "special_air_lw_end" };
                MotionModule::change_motion(boma, Hash40::new(mot), 0.0, 1.0, false, 0.0, false, false);
                return 0.into();
            }
            if t >= LW_DUREE {
                LW_PHASE[e] = 2;
                lw_bouclier(boma, false);
                lw_fx_off(boma, e);
                fx(boma, "sys_smash_flash_s", "top", (0.0, 7.0, 0.0), 0.9, Some(CRISTAL));
                let mot = if sol { "special_lw_end" } else { "special_air_lw_end" };
                MotionModule::change_motion(boma, Hash40::new(mot), 0.0, 1.0, false, 0.0, false, false);
            }
        }
        _ => {
            if LW_PHASE[e] == 1 && MotionModule::frame(boma) > 6.0 {
                lw_bouclier(boma, false);
            }
            if MotionModule::is_end(boma) {
                finir(fighter, false);
                return 1.into();
            }
        }
    }
    0.into()
}

unsafe extern "C" fn lw_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    lw_bouclier(boma, false);
    lw_fx_off(boma, e);
    LW_PHASE[e] = 0;
    LW_RIPOSTE[e] = false;
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    0.into()
}

/// riposte (REX SpecialLwEnd : 7 % élec puis 4 % feu ; vidéo : 12 % au total) : seulement s'il a été touché
unsafe extern "C" fn g_riposte(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 1.0);
    if is_excute(agent) && LW_RIPOSTE[slot(agent.module_accessor)] {
        coup(agent, 0, "top", 12.0, 45, 85, 60, 13.0, 7.0, 0.0, "collision_attr_elec");
    }
    frame(lua, 5.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
    }
}

unsafe extern "C" fn e_riposte(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 1.0);
    let boma = agent.module_accessor;
    if is_excute(agent) && LW_RIPOSTE[slot(boma)] {
        fx(boma, "sys_smash_flash", "top", (0.0, 7.0, 0.0), 1.4, Some(CRISTAL));
        fx(boma, "sys_thunder", "top", (0.0, 7.0, 0.0), 0.7, Some(CRISTAL));
    }
}

unsafe extern "C" fn s_riposte(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 1.0);
    if is_excute(agent) && LW_RIPOSTE[slot(agent.module_accessor)] {
        PLAY_SE(agent, Hash40::new("se_common_electric_hit_l"));
    }
}

unsafe extern "C" fn s_bouclier(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, LW_DEBUT as f32);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_common_guardon"));
    }
}

pub fn install(a: &mut Agent) {
    // B neutre
    a.status(Init, *FIGHTER_STATUS_KIND_SPECIAL_N, rien);
    a.status(Exec, *FIGHTER_STATUS_KIND_SPECIAL_N, rien);
    a.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_N, n_main);
    a.status(End, *FIGHTER_STATUS_KIND_SPECIAL_N, n_end);
    for n in ["game_specialnstart", "game_specialairnstart", "effect_specialnstart", "effect_specialairnstart",
              "sound_specialnstart", "sound_specialairnstart", "expression_specialnstart", "expression_specialairnstart",
              "expression_specialnspin"] {
        a.acmd(n, rien_acmd, Priority::Low);
    }
    a.acmd("game_specialnspin", g_decharge, Priority::Low);
    a.acmd("effect_specialnspin", e_decharge, Priority::Low);
    a.acmd("sound_specialnspin", s_decharge, Priority::Low);
    // side B
    a.status(Init, *FIGHTER_STATUS_KIND_SPECIAL_S, rien);
    a.status(Exec, *FIGHTER_STATUS_KIND_SPECIAL_S, rien);
    a.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_S, s_main);
    a.status(End, *FIGHTER_STATUS_KIND_SPECIAL_S, s_end);
    for (g, ef, so, ex) in [("game_specialshold", "effect_specialshold", "sound_specialshold", "expression_specialshold"),
                            ("game_specialairsendloop", "effect_specialairsendloop", "sound_specialairsendloop", "expression_specialairsendloop")] {
        a.acmd(g, g_ruee, Priority::Low);
        a.acmd(ef, e_ruee, Priority::Low);
        a.acmd(so, s_ruee, Priority::Low);
        a.acmd(ex, rien_acmd, Priority::Low);
    }
    // haut B
    a.status(Init, *FIGHTER_STATUS_KIND_SPECIAL_HI, rien);
    a.status(Exec, *FIGHTER_STATUS_KIND_SPECIAL_HI, rien);
    a.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_HI, hi_main);
    a.status(End, *FIGHTER_STATUS_KIND_SPECIAL_HI, hi_end);
    a.acmd("game_specialhi", g_envol, Priority::Low);
    a.acmd("effect_specialhi", e_envol, Priority::Low);
    a.acmd("sound_specialhi", s_envol, Priority::Low);
    a.acmd("expression_specialhi", rien_acmd, Priority::Low);
    // bas B
    a.status(Init, *FIGHTER_STATUS_KIND_SPECIAL_LW, rien);
    a.status(Exec, *FIGHTER_STATUS_KIND_SPECIAL_LW, rien);
    a.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_LW, lw_main);
    a.status(End, *FIGHTER_STATUS_KIND_SPECIAL_LW, lw_end);
    for n in ["game_speciallwstart", "game_specialairlwstart", "game_speciallwhold", "game_specialairlwhold",
              "effect_speciallwstart", "effect_specialairlwstart", "effect_speciallwhold", "effect_specialairlwhold",
              "sound_speciallwhold", "sound_specialairlwhold",
              "expression_speciallwstart", "expression_specialairlwstart", "expression_speciallwhold",
              "expression_specialairlwhold", "expression_speciallwend", "expression_specialairlwend"] {
        a.acmd(n, rien_acmd, Priority::Low);
    }
    a.acmd("sound_speciallwstart", s_bouclier, Priority::Low);
    a.acmd("sound_specialairlwstart", s_bouclier, Priority::Low);
    for (g, ef, so) in [("game_speciallwend", "effect_speciallwend", "sound_speciallwend"),
                        ("game_specialairlwend", "effect_specialairlwend", "sound_specialairlwend")] {
        a.acmd(g, g_riposte, Priority::Low);
        a.acmd(ef, e_riposte, Priority::Low);
        a.acmd(so, s_riposte, Priority::Low);
    }
}
