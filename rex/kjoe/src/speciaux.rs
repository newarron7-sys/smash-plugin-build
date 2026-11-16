//! Knuckle Joe : spéciaux de REX (FitKJoe.pac, décompilés le 16/11), statuts de Ness remplacés (méthode de Metal
//! Sonic / Master Chief : Init / Exec / Main / End). Animations de REX sur place (groupe kjoe_speciaux) : le plugin le
//! déplace image par image avec le trajet de REX (table TRAJET de trajet_rex.py), à l'échelle du modèle.
//! - B neutre (SpecialNStart / NHold / NFire) : charge tant que B est tenu (niveaux à 15, 30, 60, 90 et 180 images,
//!   variable 0x1D de REX), puis tir d'une boule d'énergie (projectile : le PK Feu de Ness redessiné, projectile.rs),
//!   plus forte avec la charge ;
//! - côté B (SpecialAirSStart puis SpecialS) : il fonce devant lui (66 dans REX) : coup de 12 % au départ de la ruée,
//!   puis 15 % (x 1,25) de l'image 11 à 21 ;
//! - haut B « Rising Break » (SpecialHi) : uppercut qui monte (30 de haut, 12,7 devant) : 16 % (80°) à l'image 8,
//!   12 / 11 % (60°) de 11 à 31, puis chute spéciale ;
//! - bas B (SpecialLwStart / SpecialLw / SpecialLwEnd / SpecialLwHit) : au sol il saute (26 de haut, coup de 6 %),
//!   en l'air il s'arrête un instant ; puis il plonge en coup de pied (vitesse de REX 0,5 avant / -5 bas) : 10 % vers
//!   le bas (270°) les 5 premières images puis 10 % (88°) ; atterrissage SpecialLwEnd ; s'il touche, il rebondit en
//!   arrière (SpecialLwHit : -7 / +8).

use smash::app::lua_bind::*;
use smash::app::sv_animcmd::frame;
use smash::app::*;
use smash::hash40;
use smash::lib::lua_const::*;
use smash::lib::L2CValue;
use smash::lua2cpp::*;
use smash::phx::Hash40;
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

use crate::ECHELLE;

const BLEU: (f32, f32, f32) = (0.45, 0.7, 1.0);
const BLANC: (f32, f32, f32) = (1.0, 1.0, 1.0);

unsafe fn slot(boma: *mut BattleObjectModuleAccessor) -> usize {
    (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7)
}

fn trajet(nom: &str) -> &'static [(f32, f32)] {
    crate::TRAJET.iter().find(|(n, _)| *n == nom).map(|(_, t)| *t).unwrap_or(&[])
}

fn point(t: &[(f32, f32)], f: f32) -> (f32, f32) {
    if t.is_empty() {
        return (0.0, 0.0);
    }
    let f = f.max(0.0);
    let i = f.floor() as usize;
    if i + 1 >= t.len() {
        return t[t.len() - 1];
    }
    let k = f - i as f32;
    (t[i].0 * (1.0 - k) + t[i + 1].0 * k, t[i].1 * (1.0 - k) + t[i + 1].1 * k)
}

/// vitesse posée (énergie STOP) : x dans sa direction
unsafe fn vitesse(fighter: &mut L2CFighterCommon, vx: f32, vy: f32) {
    let lr = PostureModule::lr(fighter.module_accessor);
    sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, vx * lr, vy);
}

/// déplacement de l'image f0 à f1 du trajet de REX (à l'échelle du modèle)
unsafe fn suivre(fighter: &mut L2CFighterCommon, t: &[(f32, f32)], f0: f32, f1: f32) {
    let a = point(t, f0);
    let b = point(t, f1);
    vitesse(fighter, (b.0 - a.0) * ECHELLE, (b.1 - a.1) * ECHELLE);
}

/// physique en l'air pilotée par le plugin (énergie STOP libre, sans limite)
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

unsafe fn au_sol(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    StatusModule::set_situation_kind(boma, SituationKind(*SITUATION_KIND_GROUND), true);
    KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
    GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND_CLIFF_STOP));
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_STOP);
}

unsafe fn sol(boma: *mut BattleObjectModuleAccessor) -> bool {
    StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND
}

unsafe fn finir(fighter: &mut L2CFighterCommon, chute_speciale: bool) {
    let boma = fighter.module_accessor;
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    if sol(boma) {
        fighter.change_status(FIGHTER_STATUS_KIND_WAIT.into(), false.into());
    } else if chute_speciale {
        fighter.change_status(FIGHTER_STATUS_KIND_FALL_SPECIAL.into(), false.into());
    } else {
        fighter.change_status(FIGHTER_STATUS_KIND_FALL.into(), false.into());
    }
}

/// fin de l'animation (certaines animations de Ness sont en boucle dans son motion_list : is_end n'arrive jamais)
unsafe fn fini(boma: *mut BattleObjectModuleAccessor) -> bool {
    MotionModule::frame(boma) >= MotionModule::end_frame(boma) - 1.0
}

unsafe fn mouvement(boma: *mut BattleObjectModuleAccessor, nom: &str, rate: f32) {
    MotionModule::change_motion(boma, Hash40::new(nom), 0.0, rate, false, 0.0, false, false);
}

/// coup (os, dégâts, angle, croissance, recul, taille, hauteur, avant) ; tailles et positions à l'échelle du modèle
#[allow(clippy::too_many_arguments)]
unsafe fn coup(agent: &mut L2CAgentBase, id: u64, os: &str, dmg: f32, ang: u64, kbg: i32, bkb: i32, size: f32, y: f32, z: f32, son: i32) {
    ATTACK(agent, id, 0, Hash40::new(os), dmg, ang, kbg, 0, bkb, size * ECHELLE, 0.0, y * ECHELLE, z * ECHELLE, None, None, None,
           1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true,
           *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
           Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, son, *ATTACK_REGION_PUNCH);
}

unsafe fn fx(agent: &mut L2CAgentBase, nom: &str, os: &str, p: (f32, f32, f32), sc: f32, c: (f32, f32, f32)) {
    if is_excute(agent) {
        EFFECT_FOLLOW(agent, Hash40::new(nom), Hash40::new(os), p.0, p.1, p.2, 0, 0, 0, sc, true);
        LAST_EFFECT_SET_COLOR(agent, c.0, c.1, c.2);
    }
}

unsafe extern "C" fn rien_acmd(_agent: &mut L2CAgentBase) {}
unsafe extern "C" fn rien(_fighter: &mut L2CFighterCommon) -> L2CValue {
    0.into()
}

// ================================================================== B neutre (charge puis boule d'énergie)
static mut N_PHASE: [u8; 8] = [0; 8];
static mut N_T: [i32; 8] = [0; 8];
static mut N_AIR: [bool; 8] = [false; 8];
/// niveau de charge (0 à 5) du dernier tir : lu par le projectile (projectile.rs)
pub static mut N_NIVEAU: i32 = 0;
/// le projectile lancé vient du B neutre (sinon : smash avant, 9 %)
pub static mut N_TIR: bool = false;
const N_PALIERS: [i32; 5] = [15, 30, 60, 90, 180];
const N_LACHER: f32 = 7.0;

unsafe extern "C" fn n_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    N_PHASE[e] = 0;
    N_T[e] = 0;
    N_AIR[e] = !sol(boma);
    if N_AIR[e] {
        en_l_air(fighter);
        vitesse(fighter, 0.0, 0.0);
        mouvement(boma, "special_air_n_start", 0.8);
    } else {
        au_sol(fighter);
        vitesse(fighter, 0.0, 0.0);
        mouvement(boma, "special_n_start", 0.8);
    }
    fighter.sub_shift_status_main(L2CValue::Ptr(n_loop as *const () as _))
}

unsafe extern "C" fn n_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    if N_AIR[e] {
        // REX : il reste presque sur place en l'air pendant la charge
        vitesse(fighter, 0.0, if N_PHASE[e] == 2 { -0.3 } else { -0.15 });
        if sol(boma) && N_PHASE[e] == 2 && MotionModule::frame(boma) > N_LACHER + 2.0 {
            fighter.change_status(FIGHTER_STATUS_KIND_LANDING.into(), false.into());
            return 1.into();
        }
    }
    match N_PHASE[e] {
        0 => {
            if fini(boma) {
                N_PHASE[e] = 1;
                mouvement(boma, if N_AIR[e] { "special_air_n_hold" } else { "special_n_hold" }, 1.0);
            }
        }
        1 => {
            N_T[e] += 1;
            let t = N_T[e];
            let niveau = N_PALIERS.iter().filter(|p| t >= **p).count() as i32;
            if N_PALIERS.contains(&t) {
                // un éclat à chaque palier de charge
                let mut p = smash::phx::Vector3f { x: 0.0, y: 0.0, z: 0.0 };
                ModelModule::joint_global_position(boma, Hash40::new("throw"), &mut p, true);
                let h = EffectModule::req(boma, Hash40::new("sys_smash_flash_s"), &p, &smash::phx::Vector3f { x: 0.0, y: 0.0, z: 0.0 }, 0.8 + 0.2 * niveau as f32, 0, 0, false, 0) as u32;
                if h != 0 {
                    EffectModule::set_rgb(boma, h, BLEU.0, BLEU.1, BLEU.2);
                }
            }
            if t % 4 == 0 {
                let p = smash::phx::Vector3f { x: 0.0, y: 0.0, z: 0.0 };
                let h = EffectModule::req_follow(boma, Hash40::new("sys_hit_elec"), Hash40::new("throw"), &p, &p, 0.25 + 0.08 * niveau as f32, true, 0, 0, 0, 0, 0, false, false) as u32;
                if h != 0 {
                    EffectModule::set_rgb(boma, h, BLEU.0, BLEU.1, BLEU.2);
                }
            }
            // vidéo du showcase : pendant la charge, le corps clignote en blanc-lilas et une boule bleue brille au poing
            if t % 6 == 0 {
                FLASH(fighter, 0.9, 0.85, 1.0, 0.55 + 0.05 * niveau as f32);
            } else if t % 6 == 3 {
                FLASH(fighter, 0.9, 0.85, 1.0, 0.25);
            }
            if t % 5 == 1 {
                let p = smash::phx::Vector3f { x: 0.0, y: 0.0, z: 0.0 };
                let h = EffectModule::req_follow(boma, Hash40::new("sys_smash_flash_s"), Hash40::new("handr"), &p, &p, 0.7 + 0.1 * niveau as f32, true, 0, 0, 0, 0, 0, false, false) as u32;
                if h != 0 {
                    EffectModule::set_rgb(boma, h, BLEU.0, BLEU.1, BLEU.2);
                }
            }
            let tenu = ControlModule::check_button_on(boma, *CONTROL_PAD_BUTTON_SPECIAL);
            if (!tenu && t > 3) || t >= 180 {
                COL_NORMAL(fighter);
                N_PHASE[e] = 2;
                N_NIVEAU = niveau;
                mouvement(boma, if N_AIR[e] { "special_n_air_fire" } else { "special_n_fire" }, 1.0);
                if MotionModule::motion_kind(boma) != hash40(if N_AIR[e] { "special_n_air_fire" } else { "special_n_fire" }) {
                    mouvement(boma, if N_AIR[e] { "special_air_n_end" } else { "special_n_end" }, 1.0);
                }
            }
        }
        _ => {
            if fini(boma) {
                finir(fighter, false);
                return 1.into();
            }
        }
    }
    0.into()
}

unsafe extern "C" fn n_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    COL_NORMAL(fighter);
    KineticModule::enable_energy(fighter.module_accessor, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    0.into()
}

/// tir (REX SpecialNFire : article à l'image 7)
unsafe extern "C" fn g_n_tir(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, N_LACHER);
    if is_excute(agent) {
        N_TIR = true;
        ArticleModule::generate_article(agent.module_accessor, *FIGHTER_NESS_GENERATE_ARTICLE_PK_FIRE, false, -1);
    }
}
unsafe extern "C" fn e_n_tir(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, N_LACHER);
    fx(agent, "sys_smash_flash", "handr", (0.0, 0.0, 0.0), 1.0 + 0.15 * N_NIVEAU as f32, BLEU);
    // traits blancs derrière lui au coup de poing (showcase)
    fx(agent, "sys_attack_speedline", "top", (0.0, 5.0, -3.0), 1.2, BLANC);
}
unsafe extern "C" fn s_n_tir(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, N_LACHER);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_common_punch_kick_swing_l"));
    }
}

// ================================================================== côté B (ruée)
static mut S_PREC: [f32; 8] = [0.0; 8];

unsafe extern "C" fn s_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    S_PREC[slot(boma)] = 0.0;
    if sol(boma) {
        au_sol(fighter);
        mouvement(boma, "special_s", 1.2);
    } else {
        en_l_air(fighter);
        mouvement(boma, "special_air_s", 1.2);
    }
    vitesse(fighter, 0.0, 0.0);
    fighter.sub_shift_status_main(L2CValue::Ptr(s_loop as *const () as _))
}

unsafe extern "C" fn s_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    let f = MotionModule::frame(boma);
    suivre(fighter, trajet("s"), S_PREC[e], f);
    S_PREC[e] = f;
    if fini(boma) {
        finir(fighter, false);
        return 1.into();
    }
    0.into()
}

unsafe extern "C" fn s_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    KineticModule::enable_energy(fighter.module_accessor, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    0.into()
}

/// SpecialAirSStart (15 images, vitesse 1,2) puis SpecialS : 12 % à l'image 0 de la ruée, 15 % de 11 à 21
unsafe extern "C" fn g_s(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 15.0);
    if is_excute(agent) {
        coup(agent, 0, "handr", 12.0, 40, 50, 80, 3.9, 0.0, 0.0, *COLLISION_SOUND_ATTR_PUNCH);
        coup(agent, 1, "top", 12.0, 40, 50, 80, 3.5, 6.0, 5.0, *COLLISION_SOUND_ATTR_PUNCH);
    }
    frame(lua, 26.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
        coup(agent, 0, "top", 15.0, 40, 50, 70, 2.3, 6.0, 6.0, *COLLISION_SOUND_ATTR_PUNCH);
        coup(agent, 1, "handr", 15.0, 40, 50, 70, 3.0, 0.0, 0.0, *COLLISION_SOUND_ATTR_PUNCH);
    }
    frame(lua, 36.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
    }
}
unsafe extern "C" fn e_s(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 1.0);
    fx(agent, "sys_smash_flash_s", "handr", (0.0, 0.0, 0.0), 1.0, BLANC);
    frame(lua, 15.0);
    if is_excute(agent) {
        FOOT_EFFECT(agent, Hash40::new("sys_dash_smoke"), Hash40::new("top"), -4, 0, 0, 0, 0, 0, 1.0, 0, 0, 0, 0, 0, 0, false);
    }
    fx(agent, "sys_attack_speedline", "top", (0.0, 6.0, -4.0), 1.1, BLANC);
    let mut f = 15.0;
    while f <= 36.0 {
        frame(lua, f);
        if is_excute(agent) {
            EFFECT(agent, Hash40::new("sys_smash_flash_s"), Hash40::new("handr"), 0, 0, 0, 0, 0, 0, 0.6, 0, 0, 0, 0, 0, 0, true);
            LAST_EFFECT_SET_COLOR(agent, BLEU.0, BLEU.1, BLEU.2);
        }
        f += 2.0;
    }
}
unsafe extern "C" fn s_s(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 15.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_common_punch_kick_swing_l"));
    }
}

// ================================================================== haut B « Rising Break »
static mut HI_PREC: [f32; 8] = [0.0; 8];

unsafe extern "C" fn hi_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    HI_PREC[slot(boma)] = 0.0;
    let air = !sol(boma);
    en_l_air(fighter);
    vitesse(fighter, 0.0, 0.0);
    mouvement(boma, if air { "special_air_hi_start" } else { "special_hi_start" }, 1.0);
    fighter.sub_shift_status_main(L2CValue::Ptr(hi_loop as *const () as _))
}

unsafe extern "C" fn hi_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    let f = MotionModule::frame(boma);
    suivre(fighter, trajet("hi"), HI_PREC[e], f);
    HI_PREC[e] = f;
    if fini(boma) {
        finir(fighter, true);
        return 1.into();
    }
    if f > 40.0 && sol(boma) {
        fighter.change_status(FIGHTER_STATUS_KIND_LANDING_FALL_SPECIAL.into(), false.into());
        return 1.into();
    }
    0.into()
}

unsafe extern "C" fn hi_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    COL_NORMAL(fighter);
    KineticModule::enable_energy(fighter.module_accessor, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    0.into()
}

/// REX SpecialHi : 16 % (80°, 95 / 40) à l'image 8, 12 / 11 % (60°, 82 / 50) de 11 à 31
unsafe extern "C" fn g_hi(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 8.0);
    if is_excute(agent) {
        coup(agent, 0, "handr", 16.0, 80, 95, 40, 5.0, 0.0, 0.0, *COLLISION_SOUND_ATTR_PUNCH);
        coup(agent, 1, "top", 16.0, 80, 95, 40, 5.0, 8.0, 3.0, *COLLISION_SOUND_ATTR_PUNCH);
    }
    frame(lua, 11.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
        coup(agent, 0, "handr", 12.0, 60, 82, 50, 5.0, 0.0, 0.0, *COLLISION_SOUND_ATTR_PUNCH);
        coup(agent, 1, "top", 11.0, 60, 82, 50, 5.0, 8.0, 3.0, *COLLISION_SOUND_ATTR_PUNCH);
    }
    frame(lua, 31.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
    }
}
unsafe extern "C" fn e_hi(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 1.0);
    if is_excute(agent) {
        FLASH(agent, 1.0, 1.0, 1.0, 0.7);
    }
    fx(agent, "sys_smash_flash_s", "top", (0.0, 5.0, 0.0), 1.3, BLANC);
    frame(lua, 5.0);
    if is_excute(agent) {
        LANDING_EFFECT(agent, Hash40::new("sys_jump_smoke"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 1.0, 0, 0, 0, 0, 0, 0, false);
    }
    frame(lua, 7.0);
    if is_excute(agent) {
        COL_NORMAL(agent);
        // gerbe blanche au sol au départ (showcase)
        EFFECT(agent, Hash40::new("sys_bomb_b"), Hash40::new("top"), 0, 2, 0, 0, 0, 0, 1.1, 0, 0, 0, 0, 0, 0, true);
        LAST_EFFECT_SET_COLOR(agent, 1.0, 1.0, 1.0);
        EFFECT(agent, Hash40::new("sys_crown"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.8, 0, 0, 0, 0, 0, 0, false);
    }
    frame(lua, 8.0);
    fx(agent, "sys_smash_flash", "handr", (0.0, 0.0, 0.0), 1.0, BLANC);
    fx(agent, "sys_attack_arc_d", "top", (0.0, 10.0, 3.0), 1.0, BLANC);
    let mut f = 8.0;
    while f <= 30.0 {
        frame(lua, f);
        if is_excute(agent) {
            EFFECT(agent, Hash40::new("sys_smash_flash_s"), Hash40::new("handr"), 0, 0, 0, 0, 0, 0, 0.7, 0, 0, 0, 0, 0, 0, true);
            LAST_EFFECT_SET_COLOR(agent, 0.75, 0.9, 1.0);
            // traînée bleu-blanc le long du corps en montant
            EFFECT(agent, Hash40::new("sys_smash_flash_s"), Hash40::new("hip"), 0, 0, 0, 0, 0, 0, 1.1, 0, 0, 0, 0, 0, 0, true);
            LAST_EFFECT_SET_COLOR(agent, 0.6, 0.85, 1.0);
        }
        f += 2.0;
    }
}
unsafe extern "C" fn s_hi(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 8.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_common_punch_kick_swing_l"));
    }
}

// ================================================================== bas B (saut puis plongée en coup de pied)
static mut LW_PHASE: [u8; 8] = [0; 8];
static mut LW_PREC: [f32; 8] = [0.0; 8];
static mut LW_SOL: [bool; 8] = [false; 8];
static mut LW_T: [i32; 8] = [0; 8];
/// plongée de REX (0E080400 : 0,5 vers l'avant, -5 vers le bas), à l'échelle du modèle
const LW_VX: f32 = 0.5;
const LW_VY: f32 = -5.0;

unsafe extern "C" fn lw_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    LW_PHASE[e] = 0;
    LW_PREC[e] = 0.0;
    LW_T[e] = 0;
    LW_SOL[e] = sol(boma);
    en_l_air(fighter);
    vitesse(fighter, 0.0, 0.0);
    if LW_SOL[e] {
        mouvement(boma, "special_lw_start", 1.1);
    } else {
        mouvement(boma, "special_air_lw_start", 1.3);
    }
    fighter.sub_shift_status_main(L2CValue::Ptr(lw_loop as *const () as _))
}

unsafe extern "C" fn lw_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    let f = MotionModule::frame(boma);
    match LW_PHASE[e] {
        0 => {
            // départ : au sol il saute (trajet de REX), en l'air il s'arrête presque
            if LW_SOL[e] {
                suivre(fighter, trajet("lw_start"), LW_PREC[e], f);
            } else {
                vitesse(fighter, 0.0, 0.15);
            }
            LW_PREC[e] = f;
            if fini(boma) {
                LW_PHASE[e] = 1;
                LW_T[e] = 0;
                mouvement(boma, "special_air_lw_hold", 1.0);
            }
        }
        1 => {
            // plongée
            LW_T[e] += 1;
            vitesse(fighter, LW_VX * ECHELLE, LW_VY * ECHELLE);
            if AttackModule::is_infliction_status(boma, *COLLISION_KIND_MASK_HIT) {
                LW_PHASE[e] = 3;
                LW_PREC[e] = 0.0;
                AttackModule::clear_all(boma);
                mouvement(boma, "special_air_lw_hit", 1.0);
            } else if GroundModule::is_touch(boma, *GROUND_TOUCH_FLAG_DOWN as u32) && LW_T[e] > 1 {
                LW_PHASE[e] = 2;
                AttackModule::clear_all(boma);
                au_sol(fighter);
                vitesse(fighter, 0.0, 0.0);
                mouvement(boma, "special_lw_end", 0.9);
            } else if LW_T[e] > 90 {
                finir(fighter, true);
                return 1.into();
            }
        }
        2 => {
            vitesse(fighter, 0.0, 0.0);
            if fini(boma) {
                finir(fighter, false);
                return 1.into();
            }
        }
        _ => {
            // rebond en arrière (SpecialLwHit)
            suivre(fighter, trajet("lw_hit"), LW_PREC[e], f);
            LW_PREC[e] = f;
            if fini(boma) {
                finir(fighter, false);
                return 1.into();
            }
        }
    }
    0.into()
}

unsafe extern "C" fn lw_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    KineticModule::enable_energy(fighter.module_accessor, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    0.into()
}

/// SpecialLwStart (sol) : 6 % (80°) de 5 à 10
unsafe extern "C" fn g_lw_depart(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 5.0);
    if is_excute(agent) {
        coup(agent, 0, "top", 6.0, 80, 100, 40, 6.25, 4.0, 5.0, *COLLISION_SOUND_ATTR_KICK);
    }
    frame(lua, 10.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
    }
}
/// SpecialLw : 10 % vers le bas (270°, 90 / 35) de 1 à 6, puis 10 % (88°, 70 / 35) de 6 à 21
unsafe extern "C" fn g_lw_plongee(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 1.0);
    if is_excute(agent) {
        coup(agent, 0, "footr", 10.0, 270, 90, 35, 4.0, 0.0, 0.0, *COLLISION_SOUND_ATTR_KICK);
        coup(agent, 1, "top", 10.0, 270, 90, 35, 4.0, 3.0, 2.0, *COLLISION_SOUND_ATTR_KICK);
    }
    frame(lua, 6.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
        coup(agent, 0, "footr", 10.0, 88, 70, 35, 4.0, 0.0, 0.0, *COLLISION_SOUND_ATTR_KICK);
        coup(agent, 1, "top", 10.0, 88, 70, 35, 4.0, 3.0, 2.0, *COLLISION_SOUND_ATTR_KICK);
    }
    frame(lua, 21.0);
    if is_excute(agent) {
        AttackModule::clear_all(agent.module_accessor);
    }
}
unsafe extern "C" fn e_lw_plongee(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    fx(agent, "sys_attack_speedline", "top", (0.0, 4.0, 2.0), 1.0, BLANC);
    let mut f = 1.0;
    while f <= 40.0 {
        frame(lua, f);
        if is_excute(agent) {
            EFFECT(agent, Hash40::new("sys_smash_flash_s"), Hash40::new("footr"), 0, 0, 0, 0, 0, 0, 0.6, 0, 0, 0, 0, 0, 0, true);
            LAST_EFFECT_SET_COLOR(agent, BLEU.0, BLEU.1, BLEU.2);
        }
        f += 2.0;
    }
}
unsafe extern "C" fn e_lw_depart(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 4.0);
    if is_excute(agent) {
        LANDING_EFFECT(agent, Hash40::new("sys_down_smoke"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.9, 0, 0, 0, 0, 0, 0, false);
    }
}
unsafe extern "C" fn e_lw_atterrit(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        LANDING_EFFECT(agent, Hash40::new("sys_down_smoke"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.8, 0, 0, 0, 0, 0, 0, false);
    }
}
unsafe extern "C" fn e_lw_rebond(agent: &mut L2CAgentBase) {
    fx(agent, "sys_smash_flash", "footr", (0.0, 0.0, 0.0), 0.9, BLANC);
}

pub fn install(a: &mut Agent) {
    for (k, m, e) in [(*FIGHTER_STATUS_KIND_SPECIAL_N, n_main as unsafe extern "C" fn(&mut L2CFighterCommon) -> L2CValue, n_end as unsafe extern "C" fn(&mut L2CFighterCommon) -> L2CValue),
                      (*FIGHTER_STATUS_KIND_SPECIAL_S, s_main, s_end),
                      (*FIGHTER_STATUS_KIND_SPECIAL_HI, hi_main, hi_end),
                      (*FIGHTER_STATUS_KIND_SPECIAL_LW, lw_main, lw_end)] {
        a.status(Init, k, rien);
        a.status(Exec, k, rien);
        a.status(Main, k, m);
        a.status(End, k, e);
    }
    // scripts de Ness (PK Flash, PK Feu, PK Tonnerre, PSI Magnet) vidés, ceux de REX à la place (chaque nom n'est
    // enregistré qu'une fois)
    let pris: &[&str] = &["game_specialnfire", "effect_specialnfire", "sound_specialnfire", "game_specialnairfire",
        "effect_specialnairfire", "sound_specialnairfire", "game_specialnend", "effect_specialnend", "sound_specialnend",
        "game_specialairnend", "effect_specialairnend", "sound_specialairnend", "game_specials", "effect_specials",
        "sound_specials", "game_specialairs", "effect_specialairs", "sound_specialairs", "game_specialhistart",
        "effect_specialhistart", "sound_specialhistart", "game_specialairhistart", "effect_specialairhistart",
        "sound_specialairhistart", "game_speciallwstart", "game_speciallwhold", "effect_speciallwhold",
        "game_specialairlwhold", "effect_specialairlwhold", "effect_speciallwend", "effect_specialairlwend",
        "effect_speciallwhit", "effect_specialairlwhit", "effect_speciallwstart"];
    for m in ["specialnstart", "specialairnstart", "specialnhold", "specialairnhold", "specialnfire", "specialnairfire",
              "specialnend", "specialairnend", "specials", "specialairs", "specialhistart", "specialairhistart",
              "speciallwstart", "specialairlwstart", "speciallwhold", "specialairlwhold", "speciallwend",
              "specialairlwend", "speciallwhit", "specialairlwhit"] {
        for p in ["game_", "effect_", "sound_", "expression_"] {
            let n: &'static str = Box::leak(format!("{}{}", p, m).into_boxed_str());
            if !pris.contains(&n) {
                a.acmd(n, rien_acmd, Priority::Low);
            }
        }
    }
    for m in ["specialnfire", "specialnairfire", "specialnend", "specialairnend"] {
        a.acmd(Box::leak(format!("game_{}", m).into_boxed_str()), g_n_tir, Priority::Low);
        a.acmd(Box::leak(format!("effect_{}", m).into_boxed_str()), e_n_tir, Priority::Low);
        a.acmd(Box::leak(format!("sound_{}", m).into_boxed_str()), s_n_tir, Priority::Low);
    }
    for m in ["specials", "specialairs"] {
        a.acmd(Box::leak(format!("game_{}", m).into_boxed_str()), g_s, Priority::Low);
        a.acmd(Box::leak(format!("effect_{}", m).into_boxed_str()), e_s, Priority::Low);
        a.acmd(Box::leak(format!("sound_{}", m).into_boxed_str()), s_s, Priority::Low);
    }
    for m in ["specialhistart", "specialairhistart"] {
        a.acmd(Box::leak(format!("game_{}", m).into_boxed_str()), g_hi, Priority::Low);
        a.acmd(Box::leak(format!("effect_{}", m).into_boxed_str()), e_hi, Priority::Low);
        a.acmd(Box::leak(format!("sound_{}", m).into_boxed_str()), s_hi, Priority::Low);
    }
    a.acmd("game_speciallwstart", g_lw_depart, Priority::Low);
    a.acmd("effect_speciallwstart", e_lw_depart, Priority::Low);
    for m in ["speciallwhold", "specialairlwhold"] {
        a.acmd(Box::leak(format!("game_{}", m).into_boxed_str()), g_lw_plongee, Priority::Low);
        a.acmd(Box::leak(format!("effect_{}", m).into_boxed_str()), e_lw_plongee, Priority::Low);
    }
    for m in ["speciallwend", "specialairlwend"] {
        a.acmd(Box::leak(format!("effect_{}", m).into_boxed_str()), e_lw_atterrit, Priority::Low);
    }
    for m in ["speciallwhit", "specialairlwhit"] {
        a.acmd(Box::leak(format!("effect_{}", m).into_boxed_str()), e_lw_rebond, Priority::Low);
    }
}
