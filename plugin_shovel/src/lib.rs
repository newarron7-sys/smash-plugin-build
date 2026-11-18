#![feature(proc_macro_hygiene)]
#![allow(unused, non_snake_case, non_upper_case_globals, static_mut_refs, unused_unsafe, unused_must_use)]

//! Shovel Knight (Super Smash Bros REX) sur Link enfant, couleur 1 (c00) : usage perso.
//! Même procédure que les autres persos de REX (rex/PROCEDURE.md) : ses coups de REX (generated.rs, créé à partir de
//! ses fichiers de REX, jamais dans le dépôt) joués sur ses animations de REX ; modèle = skin GameBanana (SneakyKNG).
//! Étape 1 : coups normaux, smashs, aériens. Spéciaux, prises / lancers, provocations : ceux de Link enfant pour
//! l'instant.

use smash::app::sv_animcmd::{frame, wait};
use smash::app::lua_bind::*;
use smash::app::*;
use smash::lib::lua_const::*;
use smash::lua2cpp::*;
use smash::phx::{Hash40, Vector3f};
use smash::hash40;
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

#[derive(Clone, Copy)]
pub struct Ev {
    pub f: f32,
    pub clear: bool,
    pub id: u64,
    pub bone: &'static str,
    pub dmg: f32,
    pub angle: u64,
    pub kbg: i32,
    pub fkb: i32,
    pub bkb: i32,
    pub size: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub hitlag: f32,
    pub sdi: f32,
    pub trip: f32,
    pub attr: &'static str,
    pub special: bool,
    pub body: i32,
    pub rate: f32,
    pub cap: bool,
    pub x2: f32,
    pub y2: f32,
    pub z2: f32,
}

pub struct Catch {
    pub f: f32,
    pub id: i32,
    pub size: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub ground_only: bool,
}

pub struct Throw {
    pub id: u64,
    pub dmg: f32,
    pub angle: u64,
    pub kbg: i32,
    pub fkb: i32,
    pub bkb: i32,
    pub attr: &'static str,
}

pub struct Sub {
    pub name: &'static str,
    pub evs: &'static [Ev],
    pub catches: &'static [Catch],
    pub throws: &'static [Throw],
    pub release: f32,
    pub total: f32,
}

pub const E: Ev = Ev { f: 0.0, clear: false, id: 0, bone: "top", dmg: 0.0, angle: 361, kbg: 100, fkb: 0, bkb: 0, size: 3.0, x: 0.0, y: 0.0, z: 0.0, hitlag: 1.0, sdi: 1.0, trip: 0.0, attr: "collision_attr_normal", special: false, body: -1, rate: 0.0, cap: false, x2: 0.0, y2: 0.0, z2: 0.0 };

pub struct Mv {
    pub script: &'static str,
    pub total: f32,
    pub hold: f32,
    pub kind: u8,
    pub evs: &'static [Ev],
}

include!("generated.rs");

/// perso de base : Link enfant
pub const BASE: &str = "younglink";
/// le skin de Shovel Knight est sur la couleur 1 (c00) ; les animations de Link enfant (motion/body/c00) sont
/// partagées par toutes ses couleurs : le moveset vaut donc pour toutes (sinon coups de Link enfant sur les
/// animations de Shovel Knight). Une couleur seule demanderait sa propre case (costume ajouté, reslotter)
pub const SK_COSTUMES: [usize; 8] = [0, 1, 2, 3, 4, 5, 6, 7];

/// le 2e coup du smash avant de Link enfant rejoue le coup du smash avant de REX
fn alias(script: &str) -> &str {
    match script {
        "game_attacks4s2" => "game_attacks4",
        _ => script,
    }
}

pub(crate) fn find(script: &str) -> Option<&'static Mv> {
    let s = alias(script);
    MOVES.iter().find(|m| m.script == s)
}

fn son(attr: &str) -> i32 {
    unsafe {
        match attr {
            "collision_attr_elec" | "collision_attr_paralyze" => *COLLISION_SOUND_ATTR_ELEC,
            "collision_attr_fire" => *COLLISION_SOUND_ATTR_FIRE,
            "collision_attr_cutup" => *COLLISION_SOUND_ATTR_CUTUP,
            _ => *COLLISION_SOUND_ATTR_PUNCH,
        }
    }
}

/// image où le smash se met en charge (images de REX) : fin de la sous-action « Start ». Smash bas : sa charge est
/// avant le saut (AttackLw4Start, 4 images), pas avant la retombée (le script de REX commence à la retombée)
fn charge(script: &str, mv: &Mv) -> f32 {
    match script {
        "game_attacklw4" => 4.0,
        _ => mv.hold,
    }
}

/// joue le coup de Shovel Knight (REX) sur son animation de REX
pub(crate) unsafe fn run(agent: &mut L2CAgentBase, script: &str) {
    let mv: &Mv = match find(script) { Some(m) => m, None => return };
    let evs = mv.evs;
    let lua = agent.lua_state_agent;
    let boma = agent.module_accessor;
    // les animations sont celles de REX, même longueur que ses scripts (Shovel Drop : boucle répétée, vitesse 1)
    let end = MotionModule::end_frame(boma);
    let r = if script == "game_attackairlw" { 1.0 } else if end > 1.0 && mv.total > 1.0 { (mv.total / end).clamp(0.25, 4.0) } else { 1.0 };
    FT_MOTION_RATE(agent, r);
    let first_hit = evs.iter().find(|e| !e.clear && e.body < 0 && e.rate <= 0.0).map(|e| e.f).unwrap_or(1.0);
    let mut cur = 0.0f32;
    let mut smash_hold_done = false;
    let mut landing_on = false;
    let lr_check = if script == "game_attackairb" { *ATTACK_LR_CHECK_B } else { *ATTACK_LR_CHECK_F };
    let hold = charge(alias(script), mv);
    // le jeu passe à la charge une image après le drapeau : posé 2 images plus tôt que dans REX
    let hold_f = if hold > 0.0 { ((hold - 2.0) / r).max(1.0) } else { ((first_hit - 4.0) / r).max(1.0) };
    let mut last_hit = 0.0f32;
    for e in evs {
        let af = if e.rate > 0.0 { e.f / r } else { (e.f / r).max(1.0) };
        if mv.kind == 1 && !smash_hold_done && af >= hold_f {
            if hold_f > cur {
                frame(lua, hold_f);
                cur = hold_f;
            }
            if is_excute(agent) {
                WorkModule::on_flag(boma, *FIGHTER_STATUS_ATTACK_FLAG_START_SMASH_HOLD);
            }
            smash_hold_done = true;
        }
        if af > cur {
            frame(lua, af);
            cur = af;
        }
        if e.rate > 0.0 {
            FT_MOTION_RATE(agent, r / e.rate);
            continue;
        }
        if !is_excute(agent) {
            continue;
        }
        if e.clear {
            AttackModule::clear_all(boma);
            continue;
        }
        if e.body >= 0 {
            let st = match e.body { 1 => *HIT_STATUS_INVINCIBLE, 2 => *HIT_STATUS_XLU, _ => *HIT_STATUS_NORMAL };
            WHOLE_HIT(agent, st);
            continue;
        }
        if mv.kind == 2 && !landing_on {
            WorkModule::on_flag(boma, *FIGHTER_STATUS_ATTACK_AIR_FLAG_ENABLE_LANDING);
            if script == "game_attackairlw" {
                // Shovel Drop : rebond sur l'adversaire (mécanique du bas aérien de Link enfant)
                WorkModule::on_flag(boma, *FIGHTER_LINK_INSTANCE_WORK_ID_FLAG_ATTACK_AIR_LW_SET_ATTACK);
            }
            landing_on = true;
        }
        let (x2, y2, z2) = if e.cap { (Some(e.x2), Some(e.y2), Some(e.z2)) } else { (None, None, None) };
        let lr = if script == "game_attackairlw" || script == "game_attacklw4" { *ATTACK_LR_CHECK_POS } else { lr_check };
        ATTACK(agent, e.id, 0, Hash40::new(e.bone), e.dmg, e.angle, e.kbg, e.fkb, e.bkb, e.size, e.x, e.y, e.z,
               x2, y2, z2, e.hitlag, e.sdi, *ATTACK_SETOFF_KIND_ON, lr, false, 0, e.trip, 0,
               false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL,
               *COLLISION_PART_MASK_ALL, false, Hash40::new(e.attr), *ATTACK_SOUND_LEVEL_M, son(e.attr),
               *ATTACK_REGION_SWORD);
        last_hit = af;
    }
    // jabs : enchaînement vers le coup suivant après le dernier coup (comme les scripts de Link enfant)
    if script == "game_attack11" || script == "game_attack12" {
        let f = (evs.iter().filter(|e| e.clear).last().map(|e| e.f).unwrap_or(last_hit * r) / r).max(cur);
        if f > cur {
            frame(lua, f);
            cur = f;
        }
        if is_excute(agent) {
            WorkModule::on_flag(boma, *FIGHTER_STATUS_ATTACK_FLAG_ENABLE_COMBO);
        }
    }
    if mv.kind == 2 && script != "game_attackairlw" {
        let after = (evs.last().map(|e| e.f).unwrap_or(1.0) + 5.0) / r;
        if after > cur {
            frame(lua, after);
        }
        if is_excute(agent) {
            WorkModule::off_flag(boma, *FIGHTER_STATUS_ATTACK_AIR_FLAG_ENABLE_LANDING);
        }
    }
}

/// effets de Link enfant (traînées de l'épée, à son rythme à lui) retirés des coups remplacés ; le coup de pelle
/// garde un éclat au moment du coup (sys_attack_arc teinté, posé sur la main)
unsafe extern "C" fn effet_vide(agent: &mut L2CAgentBase) {}

unsafe fn effet_coup(agent: &mut L2CAgentBase, script: &str) {
    let mv: &Mv = match find(script) { Some(m) => m, None => return };
    let lua = agent.lua_state_agent;
    let boma = agent.module_accessor;
    let end = MotionModule::end_frame(boma);
    let r = if script == "game_attackairlw" { 1.0 } else if end > 1.0 && mv.total > 1.0 { (mv.total / end).clamp(0.25, 4.0) } else { 1.0 };
    let mut cur = 0.0f32;
    let mut prev = -10.0f32;
    for e in mv.evs.iter().filter(|e| !e.clear && e.body < 0 && e.rate <= 0.0) {
        if e.f - prev < 3.0 {
            continue;
        }
        prev = e.f;
        let af = ((e.f - 1.0) / r).max(1.0);
        if af > cur {
            frame(lua, af);
            cur = af;
        }
        if is_excute(agent) {
            EFFECT_FOLLOW(agent, Hash40::new("sys_attack_impact"), Hash40::new(e.bone), e.x, e.y, e.z, 0, 0, 0, 0.6, true);
        }
    }
}

/// bruit d'élan de Link enfant au premier coup de chaque série (images de REX)
unsafe fn son_coup(agent: &mut L2CAgentBase, script: &str, se: &str) {
    let mv: &Mv = match find(script) { Some(m) => m, None => return };
    let lua = agent.lua_state_agent;
    let boma = agent.module_accessor;
    let end = MotionModule::end_frame(boma);
    let r = if script == "game_attackairlw" { 1.0 } else if end > 1.0 && mv.total > 1.0 { (mv.total / end).clamp(0.25, 4.0) } else { 1.0 };
    let mut cur = 0.0f32;
    let mut prev = -10.0f32;
    for e in mv.evs.iter().filter(|e| !e.clear && e.body < 0 && e.rate <= 0.0) {
        if e.f - prev < 6.0 {
            prev = e.f;
            continue;
        }
        prev = e.f;
        let af = ((e.f - 2.0) / r).max(1.0);
        if af > cur {
            frame(lua, af);
            cur = af;
        }
        if is_excute(agent) {
            PLAY_SE(agent, Hash40::new(se));
        }
    }
}

mod avance;

unsafe extern "C" fn sk_frame(fighter: &mut L2CFighterCommon) {
    avance::suivre_animation(fighter);
}

macro_rules! scripts {
    ($($f:ident, $ef:ident, $sf:ident => $s:literal, $e:literal, $snd:literal, $se:literal),* $(,)?) => {
        $( unsafe extern "C" fn $f(agent: &mut L2CAgentBase) { run(agent, $s); }
           unsafe extern "C" fn $ef(agent: &mut L2CAgentBase) { effet_coup(agent, $s); }
           unsafe extern "C" fn $sf(agent: &mut L2CAgentBase) { son_coup(agent, $s, $se); } )*
        fn install_scripts() {
            let mut a = Agent::new(BASE);
            a.set_costume(SK_COSTUMES.to_vec());
            $( if find($s).is_some() {
                a.acmd($s, $f, Priority::Low);
                a.acmd($e, $ef, Priority::Low);
                a.acmd($snd, $sf, Priority::Low);
            } )*
            a.on_line(Main, sk_frame);
            a.install();
        }
    };
}

scripts! {
    g_attack11, e_attack11, s_attack11 => "game_attack11", "effect_attack11", "sound_attack11", "se_younglink_swing_s",
    g_attack12, e_attack12, s_attack12 => "game_attack12", "effect_attack12", "sound_attack12", "se_younglink_swing_s",
    g_attack13, e_attack13, s_attack13 => "game_attack13", "effect_attack13", "sound_attack13", "se_younglink_swing_m",
    g_attackdash, e_attackdash, s_attackdash => "game_attackdash", "effect_attackdash", "sound_attackdash", "se_younglink_swing_l",
    g_attacks3, e_attacks3, s_attacks3 => "game_attacks3", "effect_attacks3", "sound_attacks3", "se_younglink_swing_m",
    g_attackhi3, e_attackhi3, s_attackhi3 => "game_attackhi3", "effect_attackhi3", "sound_attackhi3", "se_younglink_swing_m",
    g_attacklw3, e_attacklw3, s_attacklw3 => "game_attacklw3", "effect_attacklw3", "sound_attacklw3", "se_younglink_swing_s",
    g_attacks4, e_attacks4, s_attacks4 => "game_attacks4", "effect_attacks4", "sound_attacks4", "se_younglink_swing_l",
    g_attacks4s2, e_attacks4s2, s_attacks4s2 => "game_attacks4s2", "effect_attacks4s2", "sound_attacks4s2", "se_younglink_swing_l",
    g_attackhi4, e_attackhi4, s_attackhi4 => "game_attackhi4", "effect_attackhi4", "sound_attackhi4", "se_younglink_swing_l",
    g_attacklw4, e_attacklw4, s_attacklw4 => "game_attacklw4", "effect_attacklw4", "sound_attacklw4", "se_younglink_swing_l",
    g_attackairn, e_attackairn, s_attackairn => "game_attackairn", "effect_attackairn", "sound_attackairn", "se_younglink_swing_m",
    g_attackairf, e_attackairf, s_attackairf => "game_attackairf", "effect_attackairf", "sound_attackairf", "se_younglink_swing_m",
    g_attackairb, e_attackairb, s_attackairb => "game_attackairb", "effect_attackairb", "sound_attackairb", "se_younglink_swing_m",
    g_attackairhi, e_attackairhi, s_attackairhi => "game_attackairhi", "effect_attackairhi", "sound_attackairhi", "se_younglink_swing_m",
    g_attackairlw, e_attackairlw, s_attackairlw => "game_attackairlw", "effect_attackairlw", "sound_attackairlw", "se_younglink_swing_l",
}

#[skyline::main(name = "shovel_knight")]
pub fn main() {
    install_scripts();
}
