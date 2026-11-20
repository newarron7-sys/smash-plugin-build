//! Spéciaux de Shovel Knight (REX, FitShovel.pac : actions 0x113 / 0x114 et suivantes) sur les statuts de Link enfant
//! (statuts remplacés : pas de boomerang, pas d'attaque tournoyante).
//!
//! side B (action 0x113 -> 0x117 -> 0x118 / 0x119) : SpecialS puis SpecialSCharge tant que B est tenu (compteur
//! V1100005F : 22 au départ, +1 par image), lâché -> SpecialSGo puis SpecialSLoop : élan pelle en avant, le compteur
//! descend d'une unité par image (fin de l'élan à 0 -> SpecialSEnd / SpecialSAirEnd) ; force selon le compteur
//! (>= 62 : 12 %, >= 55 : 11 %, >= 45 : 10 %, >= 30 : 9 % angle 55, sinon 9 % angle 58 ; pelle : os LHaveN de REX).
//! S'il touche -> SpecialSHit (il continue sur sa lancée, 31,6 en avant : animation de REX).
//! up B (action 0x114 -> 0x11A -> 0x11B) : SpecialHi : élan (horizontal 2,25, vertical 5 : 0E010200 de REX), coup
//! fort à l'image 7 (19 % + 12 %) ; SpecialHi2 : piqué pelle en bas (10 %), accélère vers le bas de 0,1 par image
//! jusqu'à -5, dirigeable (+-0,1 par image, jusqu'à 1,1) ; au sol SpecialHiLand (onde des deux côtés, 8 %).

use smash::app::lua_bind::*;
use smash::app::*;
use smash::hash40;
use smash::lib::lua_const::*;
use smash::lib::L2CValue;
use smash::lua2cpp::*;
use smash::phx::{Hash40, Vector3f};
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

pub(crate) unsafe fn slot(boma: *mut BattleObjectModuleAccessor) -> usize {
    (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7)
}

unsafe extern "C" fn rien_status(_fighter: &mut L2CFighterCommon) -> L2CValue {
    0.into()
}

pub(crate) unsafe extern "C" fn rien(_agent: &mut L2CAgentBase) {}

unsafe fn sol(boma: *mut BattleObjectModuleAccessor) -> bool {
    StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND
}

/// énergies remises comme le jeu les attend en sortant d'un spécial
unsafe fn energies_normales(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0, 0.0);
    KineticModule::resume_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    KineticModule::resume_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_CONTROL);
}

unsafe fn finir_sol_air(fighter: &mut L2CFighterCommon) {
    if sol(fighter.module_accessor) {
        fighter.change_status(FIGHTER_STATUS_KIND_WAIT.into(), false.into());
    } else {
        fighter.change_status(FIGHTER_STATUS_KIND_FALL.into(), false.into());
    }
}

fn table(t: &[f32], f: f32) -> f32 {
    if t.is_empty() {
        return 0.0;
    }
    let f = f.max(0.0);
    let i = f.floor() as usize;
    if i + 1 >= t.len() {
        return t[t.len() - 1];
    }
    let k = f - i as f32;
    t[i] * (1.0 - k) + t[i + 1] * k
}

/// avance de dx (vers l'avant) cette image : au sol par l'énergie STOP (s'arrête au bord comme la course), en l'air
/// en déplaçant directement (pas de gravité pendant ces mouvements, comme dans REX)
unsafe fn avancer(fighter: &mut L2CFighterCommon, dx: f32) {
    let boma = fighter.module_accessor;
    let lr = PostureModule::lr(boma);
    if sol(boma) {
        KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_STOP);
        sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, dx * lr, 0.0);
    } else {
        sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0, 0.0);
        PostureModule::add_pos(boma, &Vector3f { x: dx * lr, y: 0.0, z: 0.0 });
    }
}

/// sol / air : physique « sur place » (au sol arrêt, en l'air ni gravité ni contrôle) pour les phases où le plugin
/// fait lui-même le déplacement
unsafe fn physique_plugin(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    if sol(boma) {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
        GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND));
    } else {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_MOTION_AIR);
        GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
    }
}

/// change d'animation sol <-> air à la même image (il peut quitter le sol ou atterrir)
unsafe fn suivre_situation(fighter: &mut L2CFighterCommon, sol_mot: &str, air_mot: &str) {
    let boma = fighter.module_accessor;
    if !StatusModule::is_situation_changed(boma) {
        return;
    }
    physique_plugin(fighter);
    let m = if sol(boma) { sol_mot } else { air_mot };
    MotionModule::change_motion_inherit_frame(boma, Hash40::new(m), -1.0, 1.0, 0.0, false, false);
}

// ------------------------------------------------------------------ side B : charge puis élan
const S_CHARGE_DEBUT: f32 = 16.0;   // SpecialS (16 images) puis SpecialSCharge (61)
const S_CHARGE_FIN: f32 = 77.0;
const S_BOUCLE: f32 = 5.0;          // SpecialSGo (5) puis SpecialSLoop (41)
const S_TOUCHE: f32 = 46.0;         // SpecialSHit (57)
const S_FIN: f32 = 103.0;           // SpecialSEnd (35)
const S_FIN_AIR: f32 = 138.0;       // SpecialSAirEnd (35)
const S_FIN_AIR_FIN: f32 = 173.0;
const S_COMPTEUR_DEPART: i32 = 22;
const S_COMPTEUR_MAX: i32 = 80;
/// vitesse de l'élan (le module de REX la donne ; pas lisible dans ses scripts) : ~course rapide
const S_VITESSE: f32 = 2.2;

static mut S_PHASE: [u8; 8] = [0; 8];       // 0 charge, 1 élan, 2 touché, 3 fin
static mut S_COMPTEUR: [i32; 8] = [0; 8];
static mut S_PALIER: [i32; 8] = [-1; 8];
static mut S_PREC: [f32; 8] = [0.0; 8];

unsafe extern "C" fn s_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    S_PHASE[e] = 0;
    S_COMPTEUR[e] = S_COMPTEUR_DEPART;
    S_PALIER[e] = -1;
    if sol(boma) {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
        GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND_CLIFF_STOP));
    } else {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_FALL);
        GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
    }
    let m = if sol(boma) { "special_s1" } else { "special_air_s1" };
    MotionModule::change_motion(boma, Hash40::new(m), 0.0, 1.0, false, 0.0, false, false);
    fighter.sub_shift_status_main(L2CValue::Ptr(s_loop as *const () as _))
}

/// coup de l'élan selon le compteur (REX : paliers 62 / 55 / 45 / 30), posé seulement quand le palier change
unsafe fn s_coup(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    let c = S_COMPTEUR[e];
    let (palier, dmg, angle, bkb) = if c >= 62 {
        (4, 12.0, 361u64, 64)
    } else if c >= 55 {
        (3, 11.0, 361, 51)
    } else if c >= 45 {
        (2, 10.0, 361, 40)
    } else if c >= 30 {
        (1, 9.0, 55, 32)
    } else {
        (0, 9.0, 58, 32)
    };
    if palier == S_PALIER[e] {
        return;
    }
    S_PALIER[e] = palier;
    ATTACK(fighter, 0, 0, Hash40::new("havel"), dmg, angle, 102, 0, bkb, 2.7, 8.982, -0.572, -0.021, None, None, None,
           0.6, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true,
           *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
           Hash40::new("collision_attr_cutup"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_CUTUP, *ATTACK_REGION_SWORD);
    // corps de la pelle (taille 2,7 dans REX : petite ; le corps de Shovel Knight touche aussi, comme une charge)
    ATTACK(fighter, 1, 0, Hash40::new("top"), dmg, angle, 102, 0, bkb, 4.5, 0.0, 6.0, 5.0, None, None, None,
           0.6, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true,
           *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
           Hash40::new("collision_attr_cutup"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_CUTUP, *ATTACK_REGION_SWORD);
}

unsafe extern "C" fn s_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    let f = MotionModule::frame(boma);
    match S_PHASE[e] {
        0 => {
            // début + charge : tenir B charge (REX : +1 par image), l'animation de charge boucle
            if StatusModule::is_situation_changed(boma) {
                if sol(boma) {
                    KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
                    GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND_CLIFF_STOP));
                } else {
                    KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_FALL);
                    GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
                }
                let m = if sol(boma) { "special_s1" } else { "special_air_s1" };
                MotionModule::change_motion_inherit_frame(boma, Hash40::new(m), -1.0, 1.0, 0.0, false, false);
            }
            let tenu = ControlModule::check_button_on(boma, *CONTROL_PAD_BUTTON_SPECIAL);
            helice(fighter, f);
            if f >= S_CHARGE_DEBUT {
                if tenu {
                    if S_COMPTEUR[e] < S_COMPTEUR_MAX {
                        S_COMPTEUR[e] += 1;
                        if S_COMPTEUR[e] == 62 {
                            // pleine charge : éclat sur la pelle
                            EFFECT_FOLLOW(fighter, Hash40::new("sys_smash_flash"), Hash40::new("havel"), 6, 0, 0, 0, 0, 0, 1.0, true);
                        }
                    }
                    if f >= S_CHARGE_FIN - 1.0 {
                        MotionModule::set_frame(boma, S_CHARGE_DEBUT, true);
                    }
                } else {
                    // lâché : élan
                    S_PHASE[e] = 1;
                    physique_plugin(fighter);
                    let m = if sol(boma) { "special_s2" } else { "special_air_s2" };
                    MotionModule::change_motion(boma, Hash40::new(m), 0.0, 1.0, false, 0.0, false, false);
                    if sol(boma) {
                        FOOT_EFFECT(fighter, Hash40::new("sys_dash_smoke"), Hash40::new("top"), -4, 0, 0, 0, 0, 0, 1.0, 0, 0, 0, 0, 0, 0, false);
                    }
                    AFTER_IMAGE4_ON_arg29(fighter, Hash40::new("tex_younglink_sword1"), Hash40::new("tex_younglink_sword2"), 4,
                                          Hash40::new("sword"), 1.0, 0, 0, Hash40::new("sword"), 12.5, 0, 0, true,
                                          Hash40::new("null"), Hash40::new("sword"), 0, 0, 0, 0, 0, 0, 1, 0,
                                          *EFFECT_AXIS_X, 0, *TRAIL_BLEND_ALPHA, 101, *TRAIL_CULL_NONE, 1.5, 0.2);
                }
            }
        }
        1 => {
            suivre_situation(fighter, "special_s2", "special_air_s2");
            helice(fighter, f);
            if f >= S_BOUCLE {
                avancer(fighter, S_VITESSE);
                s_coup(fighter);
                S_COMPTEUR[e] -= 1;
                if (f as i32) % 8 == 0 {
                    EFFECT_FOLLOW(fighter, Hash40::new("sys_attack_speedline"), Hash40::new("top"), 0, 7, -3, 0, 0, 0, 0.9, true);
                    if sol(boma) {
                        FOOT_EFFECT(fighter, Hash40::new("sys_dash_smoke"), Hash40::new("top"), -3, 0, 0, 0, 0, 0, 0.8, 0, 0, 0, 0, 0, 0, false);
                    }
                }
            } else {
                avancer(fighter, 0.0);
            }
            if AttackModule::is_infliction_status(boma, *COLLISION_KIND_MASK_HIT) {
                // touché : il continue sur sa lancée (SpecialSHit)
                S_PHASE[e] = 2;
                S_PREC[e] = 0.0;
                AttackModule::clear_all(boma);
                AFTER_IMAGE_OFF(fighter, 3);
                MotionModule::set_frame(boma, S_TOUCHE, true);
            } else if S_COMPTEUR[e] < 0 {
                S_PHASE[e] = 3;
                AttackModule::clear_all(boma);
                AFTER_IMAGE_OFF(fighter, 3);
                if sol(boma) {
                    MotionModule::set_frame(boma, S_FIN, true);
                    // fin au sol : dernier coup de pelle (4 %, REX : images 0-3)
                    ATTACK(fighter, 0, 0, Hash40::new("havel"), 4.0, 75, 102, 0, 32, 2.7, 8.982, -0.572, -0.021, None, None, None,
                           0.6, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true,
                           *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
                           Hash40::new("collision_attr_cutup"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_CUTUP, *ATTACK_REGION_SWORD);
                } else {
                    MotionModule::set_frame(boma, S_FIN_AIR, true);
                }
            } else if f >= S_TOUCHE - 1.0 {
                MotionModule::set_frame(boma, S_BOUCLE, true);
            }
        }
        2 => {
            suivre_situation(fighter, "special_s2", "special_air_s2");
            if let Some((_, tz, _)) = crate::AVANCE_AIR.iter().find(|(n, _, _)| *n == "sk_s_hit") {
                let g = (f - S_TOUCHE).max(0.0);
                let d = table(tz, g) - table(tz, S_PREC[e]);
                S_PREC[e] = g;
                avancer(fighter, d);
            }
            if f >= S_FIN - 1.0 {
                finir_sol_air(fighter);
            }
        }
        _ => {
            suivre_situation(fighter, "special_s2", "special_air_s2");
            avancer(fighter, 0.0);
            if f >= S_FIN + 3.0 && f < S_FIN_AIR {
                AttackModule::clear_all(boma);
            }
            let fin = if f >= S_FIN_AIR { S_FIN_AIR_FIN } else { S_FIN_AIR };
            if f >= fin - 1.0 {
                finir_sol_air(fighter);
            }
        }
    }
    0.into()
}

unsafe extern "C" fn s_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    AttackModule::clear_all(fighter.module_accessor);
    AFTER_IMAGE_OFF(fighter, 3);
    energies_normales(fighter);
    0.into()
}

/// hélice de la Propeller Dagger (REX : effet 0x24, anneaux de plus en plus petits le long de la dague, toutes les 3
/// images) : la dague est le long de X de l'os havel (dague_shovel.py : C . Y de LHaveN = X de HaveL)
unsafe fn helice(fighter: &mut L2CFighterCommon, f: f32) {
    if (f as i32) % 3 == 0 {
        for (x, t) in [(4.0f32, 0.4f32), (6.0f32, 0.25f32), (8.0f32, 0.2f32)] {
            EFFECT_FOLLOW(fighter, Hash40::new("sys_spin_wind"), Hash40::new("havel"), x, 0, 0, 0, 0, 90, t, true);
            LAST_EFFECT_SET_RATE(fighter, 2.0);
        }
    }
}

// ------------------------------------------------------------------ up B : élan puis piqué
// v10 (vidéo du joueur : « va beaucoup trop loin », il garde son élan) : trajectoire de REX refaite.
// REX (SpecialHi, 12 images) : vitesse 0,68 jusqu'à l'image 7, coups 7-9, à l'image 10 élan (+2,25 avant, +5 haut) ;
// fin de l'animation -> SpecialHi2 (piqué) : -0,1 par image tant que la chute est plus lente que 5 (main), et -0,1 de
// plus par image dès l'image 25 (GFX) ; dirigeable : +-0,1 jusqu'à 1,1. Gravité de Shovel Knight dans REX : 0,125
// (vitesse de chute max 2,3) ; vitesse en l'air max 0,95, freinage 0,016 (attributs de REX).
const H_LANCE: f32 = 10.0;
const H_VX: f32 = 2.25;
const H_VY: f32 = 5.0;
const H_G: f32 = 0.125;
const H_CHUTE_MAX: f32 = 2.3;
const H_VX_MAX: f32 = 0.95;
const H_FREIN_X: f32 = 0.016;
const H_ACC_PIQUE: f32 = 0.1;
const H_VY_MIN: f32 = -5.0;
const H_VX_STICK: f32 = 1.1;
static mut H_PHASE: [u8; 8] = [0; 8];       // 0 départ (avant l'élan), 1 élan, 2 piqué, 3 au sol
static mut H_V: [(f32, f32); 8] = [(0.0, 0.0); 8];
static mut H_T: [i32; 8] = [0; 8];

unsafe extern "C" fn hi_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    H_PHASE[e] = 0;
    H_T[e] = 0;
    let m = if sol(boma) { "special_hi_start" } else { "special_air_hi" };
    if sol(boma) {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
        GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND_CLIFF_STOP));
    } else {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_FALL);
        GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
        KineticModule::suspend_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_CONTROL);
        KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_STOP);
    }
    // l'élan d'avant (saut, course) ne compte pas
    KineticModule::clear_speed_all(boma);
    H_V[e] = (0.0, 0.0);
    MotionModule::change_motion(boma, Hash40::new(m), 0.0, 1.0, false, 0.0, false, false);
    fighter.sub_shift_status_main(L2CValue::Ptr(hi_loop as *const () as _))
}

unsafe extern "C" fn hi_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    let lr = PostureModule::lr(boma);
    let f = MotionModule::frame(boma);
    if H_PHASE[e] == 0 && f >= H_LANCE {
        // élan de REX
        H_PHASE[e] = 1;
        if sol(boma) {
            StatusModule::set_situation_kind(boma, SituationKind(*SITUATION_KIND_AIR), true);
            GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
            KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_FALL);
            KineticModule::suspend_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_CONTROL);
            KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_STOP);
        }
        H_V[e] = (H_VX.min(H_VX_MAX), H_VY);
        EFFECT_FOLLOW(fighter, Hash40::new("sys_attack_speedline"), Hash40::new("top"), 0, 4, 0, -90, 0, 0, 1.0, true);
        FOOT_EFFECT(fighter, Hash40::new("sys_v_smoke_a"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.8, 0, 0, 0, 0, 0, 0, false);
    }
    if H_PHASE[e] == 1 || H_PHASE[e] == 2 {
        let (mut vx, mut vy) = H_V[e];
        if vy > -H_CHUTE_MAX {
            vy = (vy - H_G).max(-H_CHUTE_MAX);
        }
        let sx = ControlModule::get_stick_x(boma) * lr;
        if H_PHASE[e] == 2 {
            H_T[e] += 1;
            if vy > H_VY_MIN {
                vy -= H_ACC_PIQUE;
            }
            if H_T[e] >= 25 {
                vy = (vy - H_ACC_PIQUE).max(-8.0);
            }
            if sx > 0.3 && vx < H_VX_STICK {
                vx += 0.1;
            } else if sx < -0.3 && vx > -H_VX_STICK {
                vx -= 0.1;
            }
        }
        if !(H_PHASE[e] == 2 && sx.abs() > 0.3) {
            // freinage en l'air (REX)
            vx = if vx > 0.0 { (vx - H_FREIN_X).max(0.0) } else { (vx + H_FREIN_X).min(0.0) };
        }
        H_V[e] = (vx, vy);
        sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY, vy);
        sv_kinetic_energy!(set_accel, fighter, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY, 0.0);
        sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, vx * lr, 0.0);
    } else if H_PHASE[e] == 0 && !sol(boma) {
        // avant l'élan, en l'air : il reste sur place
        sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY, 0.0);
        sv_kinetic_energy!(set_accel, fighter, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY, 0.0);
        sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0, 0.0);
    }
    match H_PHASE[e] {
        0 | 1 => {
            if MotionModule::is_end(boma) {
                H_PHASE[e] = 2;
                H_T[e] = 0;
                MotionModule::change_motion(boma, Hash40::new("special_hi"), 0.0, 1.0, false, 0.0, false, false);
            }
        }
        2 => {
            if sol(boma) {
                // atterrissage : onde des deux côtés (SpecialHiLand), il s'arrête net
                H_PHASE[e] = 3;
                energies_normales(fighter);
                KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
                GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND_CLIFF_STOP));
                KineticModule::clear_speed_all(boma);
                sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0, 0.0);
                AttackModule::clear_all(boma);
                MotionModule::change_motion(boma, Hash40::new("special_hi_hold"), 0.0, 1.0, false, 0.0, false, false);
                LANDING_EFFECT(fighter, Hash40::new("sys_down_smoke"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 1.1, 0, 0, 0, 0, 0, 0, false);
                LANDING_EFFECT(fighter, Hash40::new("sys_crown"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.7, 0, 0, 0, 0, 0, 0, false);
                QUAKE(fighter, *CAMERA_QUAKE_KIND_S);
            } else if MotionModule::is_end(boma) {
                MotionModule::change_motion(boma, Hash40::new("special_hi"), 0.0, 1.0, false, 0.0, false, false);
            }
        }
        _ => {
            sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0, 0.0);
            if !sol(boma) {
                fighter.change_status(FIGHTER_STATUS_KIND_FALL.into(), false.into());
            } else if MotionModule::is_end(boma) {
                fighter.change_status(FIGHTER_STATUS_KIND_WAIT.into(), false.into());
            }
        }
    }
    0.into()
}

unsafe extern "C" fn hi_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    AttackModule::clear_all(fighter.module_accessor);
    // gravité du jeu remise (coupée pendant l'élan : trajectoire de REX)
    let g = WorkModule::get_param_float(fighter.module_accessor, hash40("air_accel_y"), 0);
    if g > 0.0 {
        sv_kinetic_energy!(set_accel, fighter, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY, -g);
    }
    energies_normales(fighter);
    0.into()
}

pub fn install(a: &mut Agent) {
    for st in [*FIGHTER_STATUS_KIND_SPECIAL_S, *FIGHTER_STATUS_KIND_SPECIAL_HI] {
        a.status(Init, st, rien_status);
        a.status(Exec, st, rien_status);
    }
    a.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_S, s_main);
    a.status(End, *FIGHTER_STATUS_KIND_SPECIAL_S, s_end);
    a.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_HI, hi_main);
    a.status(End, *FIGHTER_STATUS_KIND_SPECIAL_HI, hi_end);
    // scripts de Link enfant de ces animations (boomerang, effets / sons d'épée) : vides ; side B piloté ici
    for s in ["game_specials1", "game_specialairs1", "game_specials2", "game_specialairs2",
              "effect_specials1", "effect_specialairs1", "effect_specials2", "effect_specialairs2",
              "sound_specials1", "sound_specialairs1", "sound_specials2", "sound_specialairs2",
              "expression_specials1", "expression_specialairs1", "expression_specials2", "expression_specialairs2"] {
        a.acmd(s, rien, Priority::Low);
    }
}
