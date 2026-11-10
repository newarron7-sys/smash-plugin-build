//! Super Shadow, spéciaux de REX.
//!
//! B neutre « Chaos Attack », d'après ses scripts de REX (FitSuperShadow.pac, actions 0x117/0x118/0x11C) :
//! - le homing de Sonic sert de trajet ; pendant le trajet il est invisible (REX : il se téléporte), un éclat au départ
//!   et à l'arrivée ;
//! - à l'arrivée (SpecialNHit) : un coup qui PARALYSE (1 %, angle 270, recul 80, taille 10, élément paralysie) ;
//! - ensuite c'est le joueur qui enchaîne : un appui sur A ou B pendant la fenêtre de REX lance le coup suivant
//!   (SpecialNHit : images 6 à 33 -> poing SpecialNHit2 ; poing / pied : images 9 à 24 -> coup suivant) ;
//!   sans appui, à l'image 25 du poing ou du pied, pose de fin (SpecialNHit5), et à l'image 45 de SpecialNHit, fin ;
//! - le dernier pied (SpecialNHit4) éjecte et le fait monter (élan 2,8), la pose de fin le fait sauter (élan 3).
//! Les 5 animations de REX sont mises bout à bout dans special_n_hit de Sonic (de nouvelles animations restaient en
//! T-pose) ; hitbox, dégâts et vitesses : ceux de REX (generated.rs, assemblés dans retouches.rs).

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

/// début de chaque partie dans l'animation special_n_hit : SpecialNHit (88 images), SpecialNHit2 (26),
/// SpecialNHit3 (26), SpecialNHit4 (30), SpecialNHit5 (44), et la fin
pub const SEG: [f32; 6] = [0.0, 88.0, 114.0, 140.0, 170.0, 214.0];

static mut ETAPE: [usize; 8] = [0; 8];
static mut APPUI: [bool; 8] = [false; 8];
static mut ELAN: [bool; 8] = [false; 8];
static mut CACHE: [bool; 8] = [false; 8];

unsafe fn slot(boma: *mut BattleObjectModuleAccessor) -> usize {
    (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7)
}

/// partie du combo en cours (les sons / éclats des parties sautées ne sont pas joués)
unsafe fn en_cours(agent: &mut L2CAgentBase, k: usize) -> bool {
    ETAPE[slot(agent.module_accessor)] == k
}

unsafe fn eclat(boma: *mut BattleObjectModuleAccessor) {
    let z = smash::phx::Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    let p = smash::phx::Vector3f { x: 0.0, y: 6.0, z: 0.0 };
    let h = EffectModule::req_follow(boma, Hash40::new("sys_smash_flash"), Hash40::new("top"), &p, &z, 1.4, true, 0, 0, 0, 0, 0, false, false);
    EffectModule::set_rgb(boma, h as u32, 1.0, 0.85, 0.3);
}

/// chaque image (ss_frame) : invisible pendant le trajet du homing (téléportation de REX), visible sinon
pub unsafe fn frame_b(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    let trajet = StatusModule::status_kind(boma) == *FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HOMING;
    if trajet && !CACHE[e] {
        eclat(boma);
        VisibilityModule::set_whole(boma, false);
        CACHE[e] = true;
    } else if !trajet && CACHE[e] {
        VisibilityModule::set_whole(boma, true);
        eclat(boma);
        CACHE[e] = false;
    }
}

unsafe fn etape(boma: *mut BattleObjectModuleAccessor, e: usize, n: usize) {
    ETAPE[e] = n;
    APPUI[e] = false;
    ELAN[e] = false;
    // REX : chaque coup repart de zéro (06040000) -> l'adversaire peut être retouché ; le script de coups suit le saut
    // d'image (sinon il restait en retard et les coups ne sortaient pas : 0 dégât)
    AttackModule::clear_all(boma);
    MotionModule::set_frame_sync_anim_cmd(boma, SEG[n], true, true, false);
}

/// arrivée : le coup qui paralyse, puis le combo au rythme des appuis du joueur
unsafe extern "C" fn n_hit_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    StatusModule::set_situation_kind(boma, SituationKind(*SITUATION_KIND_AIR), true);
    GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
    KineticModule::clear_speed_all(boma);
    KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_MOTION_AIR);
    // l'adversaire vient d'être touché par le trajet du homing : sans ça, le coup qui paralyse ne pouvait plus le toucher
    AttackModule::clear_all(boma);
    ETAPE[e] = 0;
    APPUI[e] = false;
    ELAN[e] = false;
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

/// élan vers le haut (REX : 0E010200 0 2,8 sur le dernier pied, 0 3 sur la pose de fin)
unsafe fn elan(fighter: &mut L2CFighterCommon, vy: f32) {
    let boma = fighter.module_accessor;
    KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_FALL);
    sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY, vy);
}

unsafe extern "C" fn n_hit_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    // REX : un appui sur A ou B est retenu pendant toute la partie (bit 0x1200007A)
    if ControlModule::check_button_trigger(boma, *CONTROL_PAD_BUTTON_ATTACK)
        || ControlModule::check_button_trigger(boma, *CONTROL_PAD_BUTTON_SPECIAL)
    {
        APPUI[e] = true;
    }
    if StopModule::is_stop(boma) {
        return 0.into();
    }
    let n = ETAPE[e];
    let f = MotionModule::frame(boma) - SEG[n];
    match n {
        // arrivée (coup qui paralyse) : appui entre les images 6 et 33 -> poing ; sinon fin à l'image 45
        0 => {
            if APPUI[e] && (6.0..=33.0).contains(&f) {
                etape(boma, e, 1);
            } else if f >= 45.0 {
                fin(fighter);
            }
        }
        // poing, 1er pied : appui entre les images 9 et 24 -> coup suivant ; sinon pose de fin à l'image 25
        1 | 2 => {
            if APPUI[e] && (9.0..25.0).contains(&f) {
                etape(boma, e, n + 1);
            } else if f >= 25.0 {
                etape(boma, e, 4);
            }
        }
        // dernier pied : il monte à l'image 5, fin avec l'animation
        3 => {
            if f >= 5.0 && !ELAN[e] {
                ELAN[e] = true;
                elan(fighter, 2.8);
            }
            if f >= SEG[4] - SEG[3] - 1.0 {
                fin(fighter);
            }
        }
        // pose de fin : petit saut, puis il reprend la main (REX : actions permises à l'image 5)
        _ => {
            if !ELAN[e] {
                ELAN[e] = true;
                elan(fighter, 3.0);
            }
            if f >= 20.0 || MotionModule::is_end(boma) {
                fin(fighter);
            }
        }
    }
    0.into()
}

unsafe extern "C" fn n_hit_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    let e = slot(fighter.module_accessor);
    ETAPE[e] = 0;
    APPUI[e] = false;
    ELAN[e] = false;
    smashline::original_status(End, fighter, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HIT)(fighter)
}

/// trajet du homing : demandé, c'est lui qui PARALYSE (comme dans REX, l'adversaire est figé à l'arrivée) :
/// valeurs du coup d'arrivée de REX (1 %, angle 270, recul 80 fixe, élément paralysie), taille du homing de Sonic
unsafe extern "C" fn g_specialnhoming(agent: &mut L2CAgentBase) {
    let boma = agent.module_accessor;
    if is_excute(agent) {
        JostleModule::set_status(boma, false);
        MotionModule::set_rate(boma, 80.0);
        let os = if WorkModule::is_flag(boma, *FIGHTER_SONIC_STATUS_SPECIAL_N_HOMING_FLAG_IS_KIRBY) { "rot" } else { "hip" };
        ATTACK(agent, 0, 0, Hash40::new(os), 1.0, 270, 0, 0, 80, 5.0, 0.5, 1.5, 0.0, Some(0.5), Some(1.5), Some(0.0),
               1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_SPEED, false, 0, 0.0, 0, false, false, false, false,
               true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
               Hash40::new("collision_attr_paralyze"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_ELEC,
               *ATTACK_REGION_BODY);
        AttackModule::set_captured_same_time_attack(boma, 0, true);
    }
}

// combo : hitbox de REX ; sons et éclats dorés à chaque coup (image 5 de chaque partie)
unsafe extern "C" fn g_combo(agent: &mut L2CAgentBase) { crate::run(agent, "game_specialnhit"); }
unsafe extern "C" fn s_combo(agent: &mut L2CAgentBase) {
    for (k, se) in [(0usize, "se_sonic_special_n02"), (1, "se_sonic_swing_ll"), (2, "se_sonic_swing_s"), (3, "se_sonic_swing_m")] {
        frame(agent.lua_state_agent, SEG[k] + if k == 0 { 1.0 } else { 5.0 });
        if is_excute(agent) && en_cours(agent, k) {
            PLAY_SE(agent, Hash40::new(se));
            if k > 0 {
                PLAY_SEQUENCE(agent, Hash40::new("seq_sonic_rnd_attack"));
            }
        }
    }
}
unsafe extern "C" fn e_combo(agent: &mut L2CAgentBase) {
    // paralysie à l'arrivée : éclat électrique doré
    frame(agent.lua_state_agent, 1.0);
    if is_excute(agent) {
        EFFECT_FOLLOW(agent, Hash40::new("sys_hit_elec"), Hash40::new("hip"), 0, 0, 0, 0, 0, 0, 1.2, true);
        LAST_EFFECT_SET_COLOR(agent, 1.0, 0.85, 0.3);
    }
    for (k, os, sc) in [(1usize, "handl", 1.0), (2, "footr", 1.0), (3, "footl", 1.4)] {
        frame(agent.lua_state_agent, SEG[k] + 5.0);
        if is_excute(agent) && en_cours(agent, k) {
            EFFECT_FOLLOW(agent, Hash40::new("sys_attack_impact"), Hash40::new(os), 0, 0, 0, 0, 0, 0, sc, true);
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
