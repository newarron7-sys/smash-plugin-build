#![feature(proc_macro_hygiene)]
#![allow(unused, non_snake_case, non_upper_case_globals, static_mut_refs, unused_unsafe, unused_must_use)]

//! Master Chief (Super Smash Bros REX) sur Snake, sa propre case : usage perso.
//! Même procédure que Super Shadow / Super Sonic (rex/PROCEDURE.md) : ses coups de REX (generated.rs, créé à la
//! compilation par rex/tools/gen_rex.py, jamais dans le dépôt) sur ses animations de REX ; modèle = skin GameBanana
//! (Demonslayerx8) + ses armes de REX (maillages chief_*, affichés selon le coup).
//! Étape 1 : coups normaux, smashs, aériens (tirs instantanés comme dans REX : capsule du canon jusqu'à la portée).

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
    /// tir instantané de REX (décalage variable le long du canon) : capsule de (x, y, z) à (x2, y2, z2) sur « top »
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

/// perso de base : Snake
pub const BASE: &str = "snake";
/// sa propre case : costume c140 de Snake (c120-127 : Jonesy)
pub const MC_COSTUMES: [usize; 1] = [140];

/// le 2e coup du tilt avant de Snake rejoue la rafale du fusil de combat
fn alias(script: &str) -> &str {
    match script {
        "game_attacks3s2" | "game_attacks3s" => "game_attacks3",
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

/// coups du script (REX) avec les réglages propres à Chief
fn coups(script: &str, mv: &Mv) -> Vec<Ev> {
    mv.evs.iter().map(|e| {
        let mut e = *e;
        if script == "game_attackhi4" && !e.clear && e.rate <= 0.0 && e.body < 0 {
            // laser spartan (demandé : « agrandis beaucoup beaucoup la hitbox du up smash ») : dans REX ses coups
            // sont 3 petites boules espacées (y 34 / 57 / 137) ; ici une grosse colonne continue du sol jusqu'en
            // haut, mêmes dégâts par hauteur (bas 16 / 13 %, milieu 12 %, haut 9 %)
            let (y0, y1) = match e.id { 0 => (3.0, 48.0), 1 => (48.0, 92.0), _ => (92.0, 150.0) };
            e.cap = true;
            e.x = 0.0;
            e.x2 = 0.0;
            e.z = 3.0;
            e.z2 = 3.0;
            e.y = y0;
            e.y2 = y1;
            e.size = 11.0;
            return e;
        }
        if e.cap {
            // capsules très fines dans REX (0,6) : un peu épaissies pour toucher comme son tir (rayon du jeu)
            e.size = e.size.max(2.0);
            if script == "game_attacklw4" && e.id == 0 {
                // sniper : le gros coup (15 %) seulement à bout portant ; plus loin la balle fait ~8 % (vidéo REX)
                let (dx, dy, dz) = (e.x2 - e.x, e.y2 - e.y, e.z2 - e.z);
                let l = (dx * dx + dy * dy + dz * dz).sqrt().max(0.001);
                let k = 25.0 / l;
                e.x2 = e.x + dx * k;
                e.y2 = e.y + dy * k;
                e.z2 = e.z + dz * k;
            }
        }
        e
    }).collect()
}

/// joue le coup de Master Chief (REX) sur son animation de REX
pub(crate) unsafe fn run(agent: &mut L2CAgentBase, script: &str) {
    let mv: &Mv = match find(script) { Some(m) => m, None => return };
    let evs_v = coups(alias(script), mv);
    let evs: &[Ev] = &evs_v;
    let lua = agent.lua_state_agent;
    let boma = agent.module_accessor;
    let end = MotionModule::end_frame(boma);
    let r = if end > 1.0 && mv.total > 1.0 { (mv.total / end).clamp(0.25, 4.0) } else { 1.0 };
    FT_MOTION_RATE(agent, r);
    let first_hit = evs.iter().find(|e| !e.clear && e.body < 0 && e.rate <= 0.0).map(|e| e.f).unwrap_or(1.0);
    let mut cur = 0.0f32;
    let mut smash_hold_done = false;
    let mut landing_on = false;
    let lr_check = if script == "game_attackairb" { *ATTACK_LR_CHECK_B } else { *ATTACK_LR_CHECK_F };
    // charge du smash : le jeu passe à la charge une image APRÈS le drapeau (l'animation avance encore d'une image) :
    // tout ce qui est à moins de 2 images après le drapeau part donc avant la charge (vidéos : tir, flamme et son du
    // smash avant dès le début de la charge ; REX : charge à 28, son à 29, tir à 30). Drapeau posé 2 images plus tôt
    // que dans REX : le son, les effets et le coup tombent tous après le relâchement
    let hold_f = if mv.hold > 0.0 { ((mv.hold - 2.0) / r).max(1.0) } else { ((first_hit - 4.0) / r).max(1.0) };
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
            landing_on = true;
        }
        let (x2, y2, z2) = if e.cap { (Some(e.x2), Some(e.y2), Some(e.z2)) } else { (None, None, None) };
        ATTACK(agent, e.id, 0, Hash40::new(e.bone), e.dmg, e.angle, e.kbg, e.fkb, e.bkb, e.size, e.x, e.y, e.z,
               x2, y2, z2, e.hitlag, e.sdi, *ATTACK_SETOFF_KIND_ON, lr_check, false, 0, e.trip, 0,
               false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL,
               *COLLISION_PART_MASK_ALL, false, Hash40::new(e.attr), *ATTACK_SOUND_LEVEL_M, son(e.attr),
               *ATTACK_REGION_PUNCH);
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

mod effets;
mod armes;
mod speciaux;
mod galerie;
mod prises;
mod neutre;

unsafe extern "C" fn mc_frame(fighter: &mut L2CFighterCommon) {
    armes::visibilite(fighter);
    armes::ground_pound(fighter);
    armes::suivre_animation(fighter);
    speciaux::chaque_image(fighter);
    neutre::chaque_image(fighter);
}

macro_rules! scripts {
    ($($f:ident => $s:literal),* $(,)?) => {
        $( unsafe extern "C" fn $f(agent: &mut L2CAgentBase) { run(agent, $s); } )*
        fn install_scripts() {
            let mut a = Agent::new(BASE);
            a.set_costume(MC_COSTUMES.to_vec());
            $( if find($s).is_some() { a.acmd($s, $f, Priority::Low); } )*
            effets::install(&mut a);
            speciaux::install(&mut a);
            neutre::install(&mut a);
            prises::install(&mut a);
            a.on_line(Main, mc_frame);
            a.install();
        }
    };
}

scripts! {
    g_attack11 => "game_attack11",
    g_attack12 => "game_attack12",
    g_attack13 => "game_attack13",
    g_attackdash => "game_attackdash",
    g_attacks3 => "game_attacks3",
    g_attacks3s2 => "game_attacks3s2",
    g_attacks3s => "game_attacks3s",
    g_attackhi3 => "game_attackhi3",
    g_attacklw3 => "game_attacklw3",
    g_attacks4 => "game_attacks4",
    g_attackhi4 => "game_attackhi4",
    g_attacklw4 => "game_attacklw4",
    g_attackairn => "game_attackairn",
    g_attackairf => "game_attackairf",
    g_attackairb => "game_attackairb",
    g_attackairhi => "game_attackairhi",
    g_attackairlw => "game_attackairlw",
    g_landingairlw => "game_landingairlw",
}

#[skyline::main(name = "master_chief")]
pub fn main() {
    install_scripts();
}
