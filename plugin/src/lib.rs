#![feature(proc_macro_hygiene)]
#![allow(unused, non_snake_case, non_upper_case_globals, static_mut_refs, unused_unsafe, unused_must_use)]

//! Super Shadow (Super Smash Bros REX) sur Sonic, costume c00 : usage perso.
//! Étape 1 : ses coups normaux, smashs et aériens de REX (image de sortie, dégâts, angle, recul, taille et position des
//!           hitbox) sur les animations de Sonic, accélérées ou ralenties pour durer autant que les siennes ;
//!           son modèle (dossier fighter/sonic/model/body/c00 du mod, construit à part, rattaché aux os de Sonic).
//! Étapes suivantes : animations, spéciaux, prises, Final Smash, sons, interface, sa propre case.
//! Les données ne sont PAS dans le dépôt : src/generated.rs est créé pendant la compilation (rex/tools/gen_rex.py).

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
    /// corps : -1 rien, 0 normal, 1 invincible, 2 intangible (PSA 06050100)
    pub body: i32,
    /// vitesse de REX à partir de cette image (Frame Speed Modifier), 0 = rien
    pub rate: f32,
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

pub const E: Ev = Ev { f: 0.0, clear: false, id: 0, bone: "top", dmg: 0.0, angle: 361, kbg: 100, fkb: 0, bkb: 0, size: 3.0, x: 0.0, y: 0.0, z: 0.0, hitlag: 1.0, sdi: 1.0, trip: 0.0, attr: "collision_attr_normal", special: false, body: -1, rate: 0.0 };

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

mod effets;
mod speciaux;
mod retouches;

/// perso de base (nom interne) : Sonic
pub const BASE: &str = "sonic";

/// demandé : sa propre case à la sélection (après Super Sonic) : costume c140 de Sonic
pub const SS_COSTUMES: [usize; 1] = [140];

pub(crate) fn find(script: &str) -> Option<&'static Mv> {
    MOVES.iter().find(|m| m.script == script)
}

/// corps rendu invincible / intangible par un coup (smash avant) : remis normal s'il quitte le coup avant la fin
/// (bord du terrain, annulation), sinon il le resterait
static mut CORPS_CHANGE: [bool; 8] = [false; 8];

unsafe extern "C" fn ss_frame(fighter: &mut L2CFighterCommon) {
    effets::aura(fighter);
    speciaux::frame_b(fighter);
    speciaux::frame_throw(fighter);
    speciaux::frame_esquive(fighter);
    speciaux::frame_dair(fighter);
    effets::boost_side_b(fighter);
    let boma = fighter.module_accessor;
    let id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;
    if id < 8 && CORPS_CHANGE[id] && StatusModule::status_kind(boma) != *FIGHTER_STATUS_KIND_ATTACK_S4 {
        HitModule::set_whole(boma, HitStatus(*HIT_STATUS_NORMAL), 0);
        CORPS_CHANGE[id] = false;
    }
}

/// son du coup selon son élément (sinon un coup électrique sonnait comme un coup de poing)
fn son(attr: &str) -> i32 {
    unsafe {
        match attr {
            "collision_attr_elec" => *COLLISION_SOUND_ATTR_ELEC,
            "collision_attr_fire" => *COLLISION_SOUND_ATTR_FIRE,
            "collision_attr_cutup" => *COLLISION_SOUND_ATTR_CUTUP,
            _ => *COLLISION_SOUND_ATTR_PUNCH,
        }
    }
}

/// joue le coup de Super Shadow (REX) sur l'animation de Sonic
pub(crate) unsafe fn run(agent: &mut L2CAgentBase, script: &str) {
    // B neutre : le combo n'a pas d'entrée à lui (construit dans retouches.rs à partir de ses 3 coups de REX)
    let combo = Mv { script: "game_specialnhit", total: speciaux::SEG[5], hold: 0.0, kind: 0, evs: &[] };
    let mv: &Mv = if script == "game_specialnhit" { &combo } else { match find(script) { Some(m) => m, None => return } };
    // retouches demandées (jab 3 et up smash en plusieurs coups, down smash plus tard) : construites à partir des
    // coups de REX eux-mêmes
    let evs_v = retouches::evs(script, mv.evs);
    let evs: &[Ev] = &evs_v;
    let lua = agent.lua_state_agent;
    let boma = agent.module_accessor;
    // FT_MOTION_RATE = images de jeu par image d'animation : l'animation de Sonic dure alors autant que la sienne
    let end = MotionModule::end_frame(boma);
    let r = if end > 1.0 && mv.total > 1.0 { (mv.total / end).clamp(0.25, 4.0) } else { 1.0 };
    FT_MOTION_RATE(agent, r);
    let first_hit = evs.iter().find(|e| !e.clear && e.body < 0 && e.rate <= 0.0).map(|e| e.f).unwrap_or(1.0);
    let mut cur = 0.0f32;
    let mut smash_hold_done = false;
    let mut landing_on = false;
    let delay = 0.0;
    // l'arrière aérien envoie derrière lui
    let lr_check = if script == "game_attackairb" { *ATTACK_LR_CHECK_B } else { *ATTACK_LR_CHECK_F };
    // smash : image où la charge commence (juste avant le coup si REX n'a pas de sous-action « Start »)
    let hold_f = if mv.hold > 0.0 { mv.hold / r } else { ((first_hit - 2.0) / r).max(1.0) };
    for e in evs {
        let af = if e.rate > 0.0 { (e.f + delay) / r } else { ((e.f + delay) / r).max(1.0) };
        if mv.kind == 1 && !smash_hold_done && af >= hold_f {
            // le smash se charge à son image (avant les événements qui viennent après)
            if hold_f > cur {
                frame(lua, hold_f);
                cur = hold_f;
            }
            if is_excute(agent) {
                WorkModule::on_flag(boma, *FIGHTER_STATUS_ATTACK_FLAG_START_SMASH_HOLD);
            }
            smash_hold_done = true;
        }
        if e.rate > 0.0 {
            // vitesse de REX (Frame Speed Modifier) : à l'image 0, tout de suite ; sinon à son image
            if af > cur {
                frame(lua, af);
                cur = af;
            }
            FT_MOTION_RATE(agent, r / e.rate);
            continue;
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
        if e.body >= 0 {
            let st = match e.body { 1 => *HIT_STATUS_INVINCIBLE, 2 => *HIT_STATUS_XLU, _ => *HIT_STATUS_NORMAL };
            WHOLE_HIT(agent, st);
            let id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;
            if id < 8 {
                CORPS_CHANGE[id] = e.body != 0;
            }
            continue;
        }
        if mv.kind == 2 && !landing_on {
            WorkModule::on_flag(boma, *FIGHTER_STATUS_ATTACK_AIR_FLAG_ENABLE_LANDING);
            landing_on = true;
        }
        // demandé : hitbox du nair, du jab, du back air et du up air agrandies
        let size = match script {
            "game_attackairn" | "game_attack11" | "game_attack12" | "game_attack13" | "game_attackairb"
            | "game_attackairhi" => e.size * 1.4,
            _ => e.size,
        };
        // demandé : le back air éjecte vraiment derrière lui. REX finit sur des coups à l'angle 363 (lien
        // automatique : l'adversaire suivait Shadow au lieu de partir), dont deux avec seulement 25 de croissance
        // prioritaires : le coup final part à 45° vers l'arrière (ATTACK_LR_CHECK_B) avec la croissance forte.
        let (angle, kbg) = if script == "game_attackairb" && e.f >= 9.0 && e.angle >= 360 {
            (45u64, e.kbg.max(120))
        } else {
            (e.angle, e.kbg)
        };
        ATTACK(agent, e.id, 0, Hash40::new(e.bone), e.dmg, angle, kbg, e.fkb, e.bkb, size, e.x, e.y, e.z,
               None, None, None, e.hitlag, e.sdi, *ATTACK_SETOFF_KIND_ON, lr_check, false, 0, e.trip, 0,
               false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL,
               *COLLISION_PART_MASK_ALL, false, Hash40::new(e.attr), *ATTACK_SOUND_LEVEL_M, son(e.attr),
               *ATTACK_REGION_PUNCH);
    }
    if mv.kind == 2 {
        // fin du coup en l'air : l'atterrissage redevient normal un peu après
        let after = (evs.last().map(|e| e.f).unwrap_or(1.0) + 5.0) / r;
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
            a.set_costume(SS_COSTUMES.to_vec());
            $( if find($s).is_some() { a.acmd($s, $f, Priority::Low); } )*
            effets::install(&mut a);
            speciaux::install(&mut a);
            a.on_line(Main, ss_frame);
            a.install();
        }
    };
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
}


#[skyline::main(name = "super_shadow")]
pub fn main() {
    install_scripts();
}
