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

// ------------------------------------------------------------------ side B de REX (demandé)
// Course du spin dash de Sonic avec ses animations et ses dégâts de REX (SpecialSDashHi 8 %, SpecialSDashLw 4 %,
// SpecialSSpin 4 % aux images 10 et 20). Structure des scripts de Sonic gardée (id d'attaque du statut, attaque
// prise en même temps) pour que la course réagisse comme avant aux coups donnés.
unsafe fn coup_course(agent: &mut L2CAgentBase, dmg: f32, ang: u64, kbg: i32, bkb: i32, size: f32) {
    let boma = agent.module_accessor;
    let id = *FIGHTER_SONIC_STATUS_SPECIAL_S_DASH_ATTACK_ID_DEFAULT;
    ATTACK(agent, id as u64, 0, Hash40::new("hip"), dmg, ang, kbg, 0, bkb, size, 0.0, 1.5, 0.0, None, None, None,
           1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true,
           *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
           Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_BODY);
    AttackModule::set_captured_same_time_attack(boma, id, true);
}
unsafe extern "C" fn g_specialsdashhi(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        JostleModule::set_status(agent.module_accessor, false);
        AttackModule::clear_all(agent.module_accessor);
        coup_course(agent, 8.0, 55, 72, 67, 3.1);
    }
}
unsafe extern "C" fn g_specialsdashlw(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        JostleModule::set_status(agent.module_accessor, false);
        AttackModule::clear_all(agent.module_accessor);
        coup_course(agent, 4.0, 59, 96, 64, 3.5);
    }
}
unsafe extern "C" fn g_specialsspin(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        JostleModule::set_status(agent.module_accessor, false);
    }
    for f in [10.0, 20.0] {
        frame(agent.lua_state_agent, f);
        if is_excute(agent) {
            AttackModule::clear_all(agent.module_accessor);
            ATTACK(agent, 0, 0, Hash40::new("hip"), 4.0, 65, 110, 0, 55, 2.8, 0.0, 1.5, 0.0, None, None, None, 1.0, 1.0,
                   *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true,
                   *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
                   Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_KICK,
                   *ATTACK_REGION_BODY);
        }
    }
    frame(agent.lua_state_agent, 34.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
    }
}

// ------------------------------------------------------------------ haut B : Chaos Control (demandé : celui du mod
// Shadow, qui est le même que REX) : il se concentre, puis se téléporte dans la direction du stick (invisible et
// intangible pendant le trajet, éclats bleus au départ et à l'arrivée), puis chute sans pouvoir rien faire. Pas de
// ressort de Sonic. Animations de REX (SpecialHiStart + SpecialHi bout à bout dans special_hi, vitesse x2 comme REX).
const HI_TRAJET: f32 = 31.0; // image de l'animation où commence la téléportation
const HI_FIN: f32 = 62.0;
const HI_VITESSE: f32 = 4.0; // REX : élan = stick x 4
static mut HI_PHASE: [u8; 8] = [0; 8];

unsafe fn eclat_bleu(boma: *mut BattleObjectModuleAccessor) {
    let z = smash::phx::Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    let p = smash::phx::Vector3f { x: 0.0, y: 6.0, z: 0.0 };
    let h = EffectModule::req_follow(boma, Hash40::new("sys_smash_flash"), Hash40::new("top"), &p, &z, 1.6, true, 0, 0, 0, 0, 0, false, false);
    EffectModule::set_rgb(boma, h as u32, 0.2, 0.6, 2.0);
}

unsafe extern "C" fn hi_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    HI_PHASE[e] = 0;
    if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
    } else {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_AIR_STOP);
        KineticModule::unable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    }
    sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0_f32, 0.0_f32);
    MotionModule::change_motion(boma, Hash40::new("special_hi"), 0.0, 2.0, false, 0.0, false, false);
    fighter.sub_shift_status_main(L2CValue::Ptr(hi_loop as *const () as _))
}

static mut HI_DIR: [(f32, f32); 8] = [(0.0, 1.0); 8];

unsafe extern "C" fn hi_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    let f = MotionModule::frame(boma);
    if HI_PHASE[e] == 0 {
        // REX : la direction est lue pendant la concentration (la dernière direction tenue compte)
        let sx = ControlModule::get_stick_x(boma);
        let sy = ControlModule::get_stick_y(boma);
        let n = (sx * sx + sy * sy).sqrt();
        if n >= 0.25 {
            HI_DIR[e] = (sx / n, sy / n);
        } else if f < 2.0 {
            HI_DIR[e] = (0.0, 1.0);
        }
        if f >= HI_TRAJET {
            HI_PHASE[e] = 1;
            let (dx, _) = HI_DIR[e];
            if dx * PostureModule::lr(boma) < -0.2 {
                PostureModule::reverse_lr(boma);
                PostureModule::update_rot_y_lr(boma);
            }
            eclat_bleu(boma);
            VisibilityModule::set_whole(boma, false);
            HitModule::set_whole(boma, HitStatus(*HIT_STATUS_XLU), 0);
            StatusModule::set_situation_kind(boma, SituationKind(*SITUATION_KIND_AIR), true);
            GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
            KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_AIR_STOP);
            KineticModule::unable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
        }
    }
    if HI_PHASE[e] == 1 {
        // trajet : déplacé image par image (REX : élan = direction x 4 à chaque image)
        let (dx, dy) = HI_DIR[e];
        KineticModule::clear_speed_all(boma);
        let d = smash::phx::Vector3f { x: dx * HI_VITESSE, y: dy * HI_VITESSE, z: 0.0 };
        PostureModule::add_pos(boma, &d);
        if f >= HI_FIN - 1.0 || MotionModule::is_end(boma) {
            // arrivée : il réapparaît et chute sans pouvoir rien faire
            HI_PHASE[e] = 2;
            VisibilityModule::set_whole(boma, true);
            HitModule::set_whole(boma, HitStatus(*HIT_STATUS_NORMAL), 0);
            eclat_bleu(boma);
            fighter.change_status(FIGHTER_STATUS_KIND_FALL_SPECIAL.into(), false.into());
        }
    }
    0.into()
}

unsafe extern "C" fn hi_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    HI_PHASE[slot(boma)] = 0;
    VisibilityModule::set_whole(boma, true);
    HitModule::set_whole(boma, HitStatus(*HIT_STATUS_NORMAL), 0);
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    0.into()
}

unsafe extern "C" fn s_specialhi(agent: &mut L2CAgentBase) {
    for f in [4.0, HI_TRAJET] {
        frame(agent.lua_state_agent, f);
        if is_excute(agent) {
            PLAY_SE(agent, Hash40::new("se_sonic_special_h01"));
        }
    }
}

// ------------------------------------------------------------------ projection vers le haut (demandé, d'après REX ThrowHi)
// L'animation de REX élève l'adversaire (os throw) dans les airs ; Super Shadow se téléporte autour de lui et lui
// envoie une sphère aux images 29, 37, 45, 53, 61, 69 (1 % chacune, feu), charge une grosse sphère à partir de
// l'image 82 et la lance à l'image 116 : explosion (7 %, électrique), puis l'adversaire est projeté (5 %, angle 90,
// croissance 35, recul de base 125 : valeurs de la projection de REX). Total 18 % comme dans REX.
const SPHERES: [f32; 6] = [29.0, 37.0, 45.0, 53.0, 61.0, 69.0];
const EXPLOSION: f32 = 116.0;

unsafe fn coup_tenu(agent: &mut L2CAgentBase, dmg: f32, ang: u64, kbg: i32, bkb: i32, size: f32, attr: &str, son: i32) {
    ATTACK(agent, 0, 0, Hash40::new("throw"), dmg, ang, kbg, 0, bkb, size, 0.0, 0.0, 0.0, None, None, None, 1.0, 1.0,
           *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true,
           *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
           Hash40::new(attr), *ATTACK_SOUND_LEVEL_M, son, *ATTACK_REGION_THROW);
    AttackModule::set_catch_only_all(agent.module_accessor, true, false);
}

unsafe extern "C" fn g_throwhi(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    if is_excute(agent) {
        smash_script::macros::ATTACK_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_THROW, 0, 5.0, 90, 35, 0, 125, 0.0, 1.0, *ATTACK_LR_CHECK_F, 0.0, true,
                   Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_NONE, *ATTACK_REGION_THROW);
        smash_script::macros::ATTACK_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_CATCH, 0, 3.0, 361, 100, 0, 60, 0.0, 1.0, *ATTACK_LR_CHECK_F, 0.0, true,
                   Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_NONE, *ATTACK_REGION_THROW);
    }
    for f in SPHERES {
        frame(lua, f);
        if is_excute(agent) {
            coup_tenu(agent, 1.0, 1, 120, 60, 4.7, "collision_attr_fire", *COLLISION_SOUND_ATTR_FIRE);
        }
        frame(lua, f + 1.0);
        if is_excute(agent) {
            AttackModule::clear_all(agent.module_accessor);
        }
    }
    frame(lua, EXPLOSION);
    if is_excute(agent) {
        coup_tenu(agent, 7.0, 90, 120, 81, 8.0, "collision_attr_elec", *COLLISION_SOUND_ATTR_ELEC);
        smash_script::macros::CHECK_FINISH_CAMERA(agent, 0, 20);
    }
    frame(lua, EXPLOSION + 1.0);
    if is_excute(agent) {
        let boma = agent.module_accessor;
        let target = WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_OBJECT);
        let group = WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_HIT_GROUP);
        let no = WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_HIT_NO);
        smash_script::macros::ATK_HIT_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_THROW, Hash40::new("throw"), target, group, no);
        AttackModule::clear_all(boma);
    }
}

unsafe extern "C" fn e_throwhi(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    // téléportations : éclat sur lui à chaque apparition (REX : images 20 puis toutes les 8)
    for f in [20.0, 28.0, 36.0, 44.0, 52.0, 60.0, 68.0, 76.0] {
        frame(lua, f);
        if is_excute(agent) {
            EFFECT_FOLLOW(agent, Hash40::new("sys_smash_flash"), Hash40::new("top"), 0, 7, 0, 0, 0, 0, 1.0, true);
            LAST_EFFECT_SET_COLOR(agent, 1.0, 0.85, 0.3);
        }
    }
    // sphères sur l'adversaire
    for f in SPHERES {
        frame(lua, f);
        if is_excute(agent) {
            EFFECT_FOLLOW(agent, Hash40::new("sys_hit_elec"), Hash40::new("throw"), 0, 0, 0, 0, 0, 0, 0.7, true);
            LAST_EFFECT_SET_COLOR(agent, 1.0, 0.8, 0.25);
            smash_script::macros::EFFECT(agent, Hash40::new("sys_bomb_a"), Hash40::new("throw"), 0, 0, 0, 0, 0, 0, 0.35, 0, 0, 0, 0, 0, 0, true);
        }
    }
    // grosse sphère chargée dans sa main
    let mut f = 82.0;
    while f < EXPLOSION {
        frame(lua, f);
        if is_excute(agent) {
            EFFECT_FOLLOW(agent, Hash40::new("sys_smash_flash"), Hash40::new("handr"), 0, 0, 0, 0, 0, 0, 1.4 + (f - 82.0) * 0.03, true);
            LAST_EFFECT_SET_COLOR(agent, 1.0, 0.8, 0.25);
        }
        f += 6.0;
    }
    // explosion
    frame(lua, EXPLOSION);
    if is_excute(agent) {
        smash_script::macros::EFFECT(agent, Hash40::new("sys_bomb_b"), Hash40::new("throw"), 0, 0, 0, 0, 0, 0, 1.3, 0, 0, 0, 0, 0, 0, true);
        smash_script::macros::EFFECT(agent, Hash40::new("sys_hit_elec"), Hash40::new("throw"), 0, 0, 0, 0, 0, 0, 1.5, 0, 0, 0, 0, 0, 0, true);
        smash_script::macros::QUAKE(agent, *CAMERA_QUAKE_KIND_L);
    }
}

unsafe extern "C" fn s_throwhi(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    for f in SPHERES {
        frame(lua, f);
        if is_excute(agent) {
            PLAY_SE(agent, Hash40::new("se_sonic_special_n02"));
        }
    }
    frame(lua, 100.0);
    if is_excute(agent) {
        PLAY_SEQUENCE(agent, Hash40::new("seq_sonic_rnd_attack"));
    }
    frame(lua, EXPLOSION);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_common_bomb_l"));
        PLAY_SE(agent, Hash40::new("se_sonic_smash_h01"));
    }
}

// ------------------------------------------------------------------ side B de REX (demandé : comme dans REX)
// REX (actions 0x115 / 0x11E / 0x121) : pas de boule ni de charge : il fonce en courant (SpecialSDashLw, vitesse 3,5
// dans sa direction, coup de 4 % sur lui), tant que B est tenu ; stick en arrière : demi-tour ; B relâché (ou
// course trop longue) : coup de pied de fin (SpecialSEnd : 9 % puis 6 %). Effet du Boost du f-smash pendant la course.
const S_VITESSE: f32 = 3.5;
const S_COURSE_MAX: i32 = 90;
static mut S_PHASE: [u8; 8] = [0; 8];
static mut S_T: [i32; 8] = [0; 8];

/// pour l'effet du Boost (effets.rs) : il est en train de foncer
pub unsafe fn side_b_course(boma: *mut BattleObjectModuleAccessor) -> bool {
    StatusModule::status_kind(boma) == *FIGHTER_STATUS_KIND_SPECIAL_S && S_PHASE[slot(boma)] == 0
}

unsafe fn s_vitesse(fighter: &mut L2CFighterCommon, v: f32) {
    let boma = fighter.module_accessor;
    let lr = PostureModule::lr(boma);
    sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, v * lr, 0.0_f32);
}

unsafe extern "C" fn s_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    S_PHASE[e] = 0;
    S_T[e] = 0;
    if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
        GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND_CLIFF_STOP));
    } else {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_AIR_STOP);
        KineticModule::unable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    }
    MotionModule::change_motion(boma, Hash40::new("special_s_dash_lw"), 0.0, 1.0, false, 0.0, false, false);
    s_vitesse(fighter, S_VITESSE);
    fighter.sub_shift_status_main(L2CValue::Ptr(s_loop as *const () as _))
}

unsafe extern "C" fn s_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    if S_PHASE[e] == 0 {
        S_T[e] += 1;
        // demi-tour : stick franchement en arrière
        if ControlModule::get_stick_x(boma) * PostureModule::lr(boma) < -0.6 {
            PostureModule::reverse_lr(boma);
            PostureModule::update_rot_y_lr(boma);
        }
        s_vitesse(fighter, S_VITESSE);
        let tenu = ControlModule::check_button_on(boma, *CONTROL_PAD_BUTTON_SPECIAL);
        if (!tenu && S_T[e] > 8) || S_T[e] >= S_COURSE_MAX {
            // coup de pied de fin, il glisse en ralentissant
            S_PHASE[e] = 1;
            MotionModule::change_motion(boma, Hash40::new("special_s_end_start"), 0.0, 1.0, false, 0.0, false, false);
        }
        return 0.into();
    }
    let f = MotionModule::frame(boma);
    s_vitesse(fighter, (S_VITESSE * (1.0 - f / 20.0)).max(0.0));
    if MotionModule::is_end(boma) {
        KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
        if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND {
            fighter.change_status(FIGHTER_STATUS_KIND_WAIT.into(), false.into());
        } else {
            fighter.change_status(FIGHTER_STATUS_KIND_FALL_SPECIAL.into(), false.into());
        }
    }
    0.into()
}

unsafe extern "C" fn s_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    S_PHASE[slot(boma)] = 0;
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    0.into()
}

/// coup de pied de fin : REX SpecialSEnd (9 % à l'image 5, 6 % à l'image 10, fin à 25)
unsafe extern "C" fn g_specialsendstart(agent: &mut L2CAgentBase) { crate::run(agent, "game_specialsendstart"); }

/// projection vers le haut : l'animation de REX le fait monter très haut (99 unités) avec l'adversaire. Le statut de
/// projection reste au sol : on le passe en l'air pour que la montée de l'animation s'applique, puis il retombe.
static mut THROW_AIR: [bool; 8] = [false; 8];
pub unsafe fn frame_throw(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    let lancer = StatusModule::status_kind(boma) == *FIGHTER_STATUS_KIND_THROW
        && MotionModule::motion_kind(boma) == smash::hash40("throw_hi");
    if !lancer {
        THROW_AIR[e] = false;
        return;
    }
    let f = MotionModule::frame(boma);
    if !THROW_AIR[e] && f >= 1.0 {
        THROW_AIR[e] = true;
        StatusModule::set_situation_kind(boma, SituationKind(*SITUATION_KIND_AIR), true);
        GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_MOTION_AIR);
    }
    if THROW_AIR[e] && f >= MotionModule::end_frame(boma) - 2.0 {
        THROW_AIR[e] = false;
        fighter.change_status(FIGHTER_STATUS_KIND_FALL.into(), false.into());
    }
}

pub fn install(a: &mut Agent) {
    a.acmd("game_specialnhoming", g_specialnhoming, Priority::Low);
    a.acmd("game_specialnhit", g_combo, Priority::Low);
    a.acmd("sound_specialnhit", s_combo, Priority::Low);
    a.acmd("effect_specialnhit", e_combo, Priority::Low);
    a.acmd("expression_specialnhit", rien, Priority::Low);
    a.acmd("game_specialsdashhi", g_specialsdashhi, Priority::Low);
    a.acmd("game_specialsdashlw", g_specialsdashlw, Priority::Low);
    a.acmd("game_specialsspin", g_specialsspin, Priority::Low);
    a.acmd("game_specialsendstart", g_specialsendstart, Priority::Low);
    a.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_S, s_main);
    a.status(End, *FIGHTER_STATUS_KIND_SPECIAL_S, s_end);
    a.acmd("game_throwhi", g_throwhi, Priority::Low);
    a.acmd("effect_throwhi", e_throwhi, Priority::Low);
    a.acmd("sound_throwhi", s_throwhi, Priority::Low);
    a.acmd("game_specialhi", rien, Priority::Low);
    a.acmd("effect_specialhi", rien, Priority::Low);
    a.acmd("sound_specialhi", s_specialhi, Priority::Low);
    a.acmd("expression_specialhi", rien, Priority::Low);
    a.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_HI, hi_main);
    a.status(End, *FIGHTER_STATUS_KIND_SPECIAL_HI, hi_end);
    a.status(Main, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HIT, n_hit_main);
    a.status(End, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HIT, n_hit_end);
}
