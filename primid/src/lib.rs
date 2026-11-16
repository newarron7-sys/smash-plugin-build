#![feature(proc_macro_hygiene)]
#![allow(unused, non_snake_case, non_upper_case_globals, static_mut_refs, unused_unsafe, unused_must_use)]

//! Primid (Super Smash Bros REX) sur Mario : usage perso. Même méthode que Rock (sa version validée sur Terry).
//! v0.1 : étapes 1 + 2 ensemble : ses coups de REX (normaux, smashs, aériens, prises et lancers) + son modèle de REX
//!        rattaché aux os de Mario + ses animations de REX ; provocations à lui (sans les scripts de Mario).
//!        Spéciaux : ceux de Mario pour l'instant (son B bas, les invocations, vient ensuite).
//! v0.2 : son bas B de REX : portail de Subspace et invocations (Mite au stick haut, Botron au stick avant) : invoc.rs
//! Les données ne sont PAS dans le dépôt : src/generated.rs est créé pendant la compilation (rex/tools/gen_rex.py)
//! à partir du build REX du propriétaire du dépôt.

use smash::app::sv_animcmd::{frame, wait};
use smash::app::lua_bind::*;
use smash::app::*;
use smash::lib::lua_const::*;
use smash::lua2cpp::*;
use smash::phx::Hash40;
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
    /// v0.18 : hitbox spéciale du PSA : pendant un lancer, elle touche l'adversaire tenu
    pub special: bool,
}

/// v0.18 : zone de chope de REX (images de son animation)
pub struct Catch {
    pub f: f32,
    pub id: i32,
    pub size: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    /// true = n'attrape que les adversaires au sol
    pub ground_only: bool,
}

/// réglages de projection (id 0 = la projection, id 1 = si la prise est lâchée)
pub struct Throw {
    pub id: u64,
    pub dmg: f32,
    pub angle: u64,
    pub kbg: i32,
    pub fkb: i32,
    pub bkb: i32,
    pub attr: &'static str,
}

/// sous-action de prise : coups, chopes, projections (images de l'animation de REX)
pub struct Sub {
    pub name: &'static str,
    pub evs: &'static [Ev],
    pub catches: &'static [Catch],
    pub throws: &'static [Throw],
    /// image où l'adversaire est lâché (lancers), -1 si aucune
    pub release: f32,
    /// durée de l'animation de REX (images)
    pub total: f32,
}

pub const E: Ev = Ev { f: 0.0, clear: false, id: 0, bone: "top", dmg: 0.0, angle: 361, kbg: 100, fkb: 0, bkb: 0, size: 3.0, x: 0.0, y: 0.0, z: 0.0, hitlag: 1.0, sdi: 1.0, trip: 0.0, attr: "collision_attr_normal", special: false };

pub struct Mv {
    pub script: &'static str,
    /// durée de son animation dans REX (images)
    pub total: f32,
    /// smash : image où la charge commence (fin de l'animation de début), 0 = automatique
    pub hold: f32,
    /// 0 = normal, 1 = smash, 2 = aérien
    pub kind: u8,
    pub evs: &'static [Ev],
}

include!("generated.rs");

mod prises;
mod invoc;
mod coups;
mod speciaux;

/// perso de base (nom interne) : Mario
pub const BASE: &str = "mario";

/// costume de Mario remplacé par Primid
pub const PR_COSTUMES: [usize; 1] = [0];

fn find(script: &str) -> Option<&'static Mv> {
    MOVES.iter().find(|m| m.script == script)
}

/// joue le coup de Primid (images de REX) sur son animation de REX convertie
unsafe fn run(agent: &mut L2CAgentBase, script: &str) {
    let Some(mv) = find(script) else { return };
    let lua = agent.lua_state_agent;
    let boma = agent.module_accessor;
    // FT_MOTION_RATE = images de jeu par image d'animation : l'animation de Mario dure alors autant que la sienne
    let end = MotionModule::end_frame(boma);
    let r = if end > 1.0 && mv.total > 1.0 { (mv.total / end).clamp(0.25, 4.0) } else { 1.0 };
    FT_MOTION_RATE(agent, r);
    let first_hit = mv.evs.iter().find(|e| !e.clear).map(|e| e.f).unwrap_or(1.0);
    let mut cur = 0.0f32;
    let mut smash_hold_done = false;
    let mut landing_on = false;
    let delay = 1.0;
    // arrière aérien : envoie derrière lui (comme Rock v0.22)
    let lr_check = if script == "game_attackairb" { *ATTACK_LR_CHECK_B } else { *ATTACK_LR_CHECK_F };
    for e in mv.evs {
        // image de REX -> image de l'animation jouée (1 image plus tard, comme Rock v0.22 validé : le membre est déplié)
        let af = ((e.f + delay) / r).max(1.0);
        if mv.kind == 1 && !smash_hold_done {
            // le smash se charge juste avant le coup
            let hold = if mv.hold > 0.0 { mv.hold / r } else { ((first_hit - 2.0) / r).max(1.0) };
            if hold > cur {
                frame(lua, hold);
                cur = hold;
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
        if !is_excute(agent) {
            continue;
        }
        if e.clear {
            AttackModule::clear_all(boma);
            continue;
        }
        if mv.kind == 2 && !landing_on {
            WorkModule::on_flag(boma, *FIGHTER_STATUS_ATTACK_AIR_FLAG_ENABLE_LANDING);
            landing_on = true;
        }
        // v0.27 : coups d'épée sur l'os de l'épée ; tourné à gauche, l'épée est dans l'autre main (copie en miroir)
        let os_coup = if e.bone == "havel" && PostureModule::lr(boma) < 0.0 { "haver" } else { e.bone };
        // v0.34 (le joueur : « le tilt haut, c'est un coup avec des effets de poison ») : coups violets (ombre)
        let attr = if script == "game_attackhi3" { "collision_attr_purple" } else { e.attr };
        ATTACK(agent, e.id, 0, Hash40::new(os_coup), e.dmg, e.angle, e.kbg, e.fkb, e.bkb, e.size, e.x, e.y, e.z,
               None, None, None, e.hitlag, e.sdi, *ATTACK_SETOFF_KIND_ON, lr_check, false, 0, e.trip, 0,
               false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL,
               *COLLISION_PART_MASK_ALL, false, Hash40::new(attr), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_PUNCH,
               *ATTACK_REGION_PUNCH);
    }
    if mv.kind == 2 {
        // fin du coup en l'air : l'atterrissage redevient normal un peu après
        let after = (mv.evs.last().map(|e| e.f).unwrap_or(1.0) + 5.0) / r;
        if after > cur {
            frame(lua, after);
        }
        if is_excute(agent) {
            WorkModule::off_flag(boma, *FIGHTER_STATUS_ATTACK_AIR_FLAG_ENABLE_LANDING);
        }
    }
}

macro_rules! scripts {
    ($($f:ident => $s:literal),* $(,)?) => {
        $( unsafe extern "C" fn $f(agent: &mut L2CAgentBase) { run(agent, $s); } )*
        fn install_scripts() {
            let mut a = Agent::new(BASE);
            a.set_costume(PR_COSTUMES.to_vec());
            $( if find($s).is_some() { a.acmd($s, $f, Priority::Low); } )*
            prises::install(&mut a);
            invoc::install(&mut a);
            coups::install(&mut a);
            speciaux::install(&mut a);
            a.on_line(Main, primid_frame);
            a.install();
        }
    };
}

/// chaque image : recharges des invocations du bas B
unsafe extern "C" fn primid_frame(fighter: &mut L2CFighterCommon) {
    invoc::update(fighter);
    coups::update(fighter);
    speciaux::update(fighter);
}

scripts! {
    g_attack11 => "game_attack11",
    g_attack12 => "game_attack12",
    g_attack13 => "game_attack13",
    g_attackdash => "game_attackdash",
    g_attacks3hi => "game_attacks3hi",
    g_attacks3 => "game_attacks3",
    g_attacks3lw => "game_attacks3lw",
    g_attackhi3 => "game_attackhi3",
    g_attacklw3 => "game_attacklw3",
    g_attacks4hi => "game_attacks4hi",
    g_attacks4 => "game_attacks4",
    g_attacks4lw => "game_attacks4lw",
    g_attackhi4 => "game_attackhi4",
    g_attacklw4 => "game_attacklw4",
    g_attackairn => "game_attackairn",
    g_attackairf => "game_attackairf",
    g_attackairb => "game_attackairb",
    g_attackairhi => "game_attackairhi",
    g_attackairlw => "game_attackairlw",
    g_specialhi => "game_specialhi",
    g_specialairhi => "game_specialairhi",
    g_specials => "game_specials",
    g_specialairs => "game_specialairs",
}

/// effets, sons et expressions des scripts de Mario (placés pour ses animations à lui) : retirés sur celles de Primid
unsafe extern "C" fn rien(_agent: &mut L2CAgentBase) {}

/// provocations : ses animations de REX, sans les scripts de Mario (Cappy, voix, effets, grossissement)
/// + effets des coups normaux de Mario (feu de son smash avant, etc.)
fn install_vides() {
    let mut a = Agent::new(BASE);
    a.set_costume(PR_COSTUMES.to_vec());
    for k in ["game", "effect", "sound", "expression"] {
        for m in ["appealhil", "appealhir", "appealsl", "appealsr", "appeallwl", "appeallwr"] {
            a.acmd(&format!("{}_{}", k, m), rien, Priority::Low);
        }
    }
    for m in ["attack11", "attack12", "attack13", "attackdash", "attacks3hi", "attacks3", "attacks3lw", "attackhi3",
              "attacklw3", "attacks4hi", "attacks4", "attacks4lw", "attackhi4", "attacklw4", "attackairn", "attackairf",
              "attackairb", "attackairhi", "attackairlw"] {
        a.acmd(&format!("effect_{}", m), rien, Priority::Low);
    }
    a.install();
}

#[skyline::main(name = "primid")]
pub fn main() {
    install_scripts();
    install_vides();
}
