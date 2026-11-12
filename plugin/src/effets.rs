//! Effets et sons des coups de Master Chief (les scripts de Snake : RPG, mortier, fumées… sont remplacés).
//! Méthode des scripts du jeu (SSBU-Dumped-Scripts, HewDraw-Remix) : effets communs (sys_*) teintés.
//! Tirs : éclair au bout du canon + traçantes (sys_attack_line) le long de la capsule du tir, comme les effets n° 7
//! (« Shots ») que REX pose tous les 15 le long du canon jusqu'à la portée. Images = images de ses animations de REX.

use smash::app::lua_bind::*;
use smash::app::sv_animcmd::frame;
use smash::app::*;
use smash::lib::lua_const::*;
use smash::lua2cpp::*;
use smash::phx::Hash40;
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

use crate::{coups, find, Ev};

type C = (f32, f32, f32);
const BLANC: C = (1.0, 1.0, 0.92);
const JAUNE: C = (1.0, 0.82, 0.25);
const ORANGE: C = (1.0, 0.6, 0.15);
const VERT: C = (0.35, 1.0, 0.35);
const BLEU: C = (0.4, 0.75, 1.0);
const CYAN: C = (0.3, 0.95, 1.0);
const ROUGE: C = (1.0, 0.15, 0.1);

#[derive(Clone, Copy)]
enum Fx {
    /// effet posé (reste sur place)
    Pose(&'static str, &'static str, (f32, f32, f32), (f32, f32, f32), f32, C),
    /// effet accroché à l'os
    Suit(&'static str, &'static str, (f32, f32, f32), (f32, f32, f32), f32, C),
    /// effet au sol
    Sol(&'static str, f32),
    Quake,
    Son(&'static str),
}

/// couleur du tir / de l'impact selon le coup
fn couleur(script: &str, e: &Ev) -> C {
    match script {
        "game_attackhi3" => JAUNE,
        "game_attacklw3" => VERT,
        "game_attacks4" => ORANGE,
        "game_attackhi4" => ROUGE,
        "game_attackairhi" => BLEU,
        "game_attackairf" => CYAN,
        _ => match e.attr {
            "collision_attr_elec" | "collision_attr_paralyze" => BLEU,
            _ => BLANC,
        },
    }
}

/// coups à l'arme (pas d'éclat d'impact électrique : feu et fumée au canon, voir canon())
fn arme(script: &str) -> bool {
    matches!(script, "game_attacks3" | "game_attacks4" | "game_attacklw4" | "game_attackhi3" | "game_attacklw3")
}

/// tir d'une arme à balles : éclair de bouche du jeu (sys_muzzleflash) au bout du canon (vidéo v10 : « j'aime bien les
/// effets ») + la flamme / explosion de la version d'avant (demandé : « rajoute des explosions quand même ») ; un petit
/// filet de fumée ensuite
fn canon(out: &mut Vec<(f32, Fx)>, f: f32, os: &'static str, p: (f32, f32, f32), sc: f32) {
    out.push((f, Fx::Pose("sys_muzzleflash", os, p, (0.0, 0.0, 0.0), 0.8 * sc, (1.0, 1.0, 1.0))));
    out.push((f, Fx::Pose("sys_hit_fire", os, p, (0.0, 0.0, 0.0), 0.35 * sc, (1.0, 1.0, 1.0))));
    out.push((f, Fx::Pose("sys_smash_flash_s", os, p, (0.0, 0.0, 0.0), 0.25 * sc, (1.0, 0.9, 0.55))));
    out.push((f + 2.0, Fx::Pose("sys_dash_smoke", os, (p.0, p.1 + 0.5, p.2 + 0.5), (0.0, 0.0, 0.0), 0.22 * sc, (0.85, 0.85, 0.85))));
}

/// effets d'une image de coups (une seule fois par image)
fn effets_coups(script: &str, evs: &[Ev], out: &mut Vec<(f32, Fx)>) {
    let mut vu: Vec<f32> = Vec::new();
    for e in evs {
        if e.clear || e.rate > 0.0 || e.body >= 0 {
            continue;
        }
        if vu.contains(&e.f) {
            continue;
        }
        if e.cap && script == "game_attackhi4" {
            continue;                           // rayon du laser : effets dans extras()
        }
        if e.cap {
            vu.push(e.f);
            let p = (e.x, e.y, e.z);
            match script {
                // fusil à pompe : éclair + grosse flamme en cône devant le canon (comme avant la v10)
                "game_attacks4" => {
                    canon(out, e.f, "top", p, 2.0);
                    for k in 1..4 {
                        let d = 4.0 * k as f32;
                        out.push((e.f, Fx::Pose("sys_hit_fire", "top", (e.x, e.y, e.z + d), (0.0, 0.0, 0.0), 0.4 + 0.15 * k as f32, (1.0, 1.0, 1.0))));
                    }
                }
                "game_attacklw4" => canon(out, e.f, "top", p, 1.6),
                _ => canon(out, e.f, "top", p, 1.0),
            }
        } else if !arme(script) {
            vu.push(e.f);
            let c = couleur(script, e);
            let eff = match e.attr {
                "collision_attr_elec" | "collision_attr_paralyze" => "sys_hit_elec",
                _ => "sys_attack_impact",
            };
            out.push((e.f, Fx::Pose(eff, e.bone, (e.x, e.y, e.z), (0.0, 0.0, 0.0), 0.75, c)));
        }
    }
}

fn trainee(out: &mut Vec<(f32, Fx)>, os: &'static [&'static str], f0: f32, f1: f32, sc: f32, c: C) {
    let mut f = f0;
    while f <= f1 {
        for b in os {
            out.push((f, Fx::Pose("sys_smash_flash_s", b, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), sc, c)));
        }
        f += 1.0;
    }
}

/// sons de Chief (banque de REX 155, mis dans la banque de sons de Snake du costume c140 : rex/tools/sons_chief.py)
/// aux images de REX
fn sons(script: &str, out: &mut Vec<(f32, Fx)>) {
    let l: &[(f32, &'static str)] = match script {
        "game_attack11" => &[(1.0, "se_snake_swing_m")],
        "game_attack12" => &[(1.0, "se_snake_swing_s")],
        "game_attack13" => &[(1.0, "se_snake_swing_l")],
        "game_attackdash" => &[(7.0, "se_snake_attackdash_01")],
        "game_swingdash" => &[(7.0, "se_snake_attackdash_02"), (9.0, "se_snake_attackair_n01")],
        "game_attacks3" => &[(1.0, "se_snake_attackhard_s02"), (12.0, "se_snake_attackhard_s01")],
        "game_attackhi3" => &[(25.0, "se_snake_attackhard_h01"), (32.0, "se_snake_attackhard_h01")],
        "game_attacklw3" => &[(4.0, "se_snake_attackhard_l01"), (30.0, "se_snake_attackhard_l01"), (78.0, "se_snake_attackair_n03")],
        "game_attacks4" => &[(1.0, "se_snake_smash_s01"), (29.0, "se_snake_smash_s02")],
        "game_attackhi4" => &[(1.0, "se_snake_smash_h01"), (31.0, "se_snake_smash_h02")],
        "game_attacklw4" => &[(1.0, "se_snake_smash_l01"), (32.0, "se_snake_smash_l02")],
        "game_attackairn" => &[(3.0, "se_snake_attackair_n01")],
        "game_attackairf" => &[(1.0, "se_snake_attackair_f01"), (5.0, "se_snake_attackair_n02")],
        "game_attackairb" => &[(16.0, "se_snake_attackair_b01")],
        "game_attackairhi" => &[(19.0, "se_snake_attackair_h01")],
        "game_attackairlw" => &[(1.0, "se_snake_attackair_l01"), (67.0, "se_snake_attackair_l01"), (87.0, "se_snake_attackair_l02")],
        _ => &[],
    };
    for (f, se) in l {
        out.push((*f, Fx::Son(se)));
    }
}

/// effets propres à chaque coup (REX : arcs n° 0xC, impacts n° 0xD, laser n° 3, jets du Ground Pound…)
fn extras(script: &str, out: &mut Vec<(f32, Fx)>) {
    match script {
        "game_attack11" => {
            trainee(out, &["handr"], 2.0, 4.0, 0.45, BLANC);
        }
        "game_attack12" => {
            trainee(out, &["handl"], 2.0, 4.0, 0.45, BLANC);
        }
        "game_attack13" => {
            out.push((3.0, Fx::Suit("sys_attack_arc_b", "top", (0.0, 12.0, 5.0), (-80.0, 0.0, 90.0), 0.8, BLANC)));
            trainee(out, &["handr"], 3.0, 6.0, 0.5, BLANC);
        }
        "game_attackdash" => {
            // « Spartan Charge » : traînée de vitesse blanche
            out.push((7.0, Fx::Suit("sys_attack_speedline", "top", (0.0, 9.0, -3.0), (0.0, 0.0, 0.0), 1.2, BLANC)));
            out.push((7.0, Fx::Sol("sys_dash_smoke", 0.9)));
        }
        "game_swingdash" => {
            // « Slide Kick » : glissade au sol
            for f in [9.0, 13.0, 17.0, 21.0] {
                out.push((f, Fx::Sol("sys_sliding_smoke", 0.9)));
            }
        }
        "game_attackhi3" => {
            // mitraillettes : éclairs aux deux mains à chaque rafale
            for f in [25.0, 28.0, 30.0, 32.0, 36.0] {
                canon(out, f, "haver", (0.0, 0.0, 4.0), 0.8);
                canon(out, f, "havel", (0.0, 0.0, 4.0), 0.8);
            }
        }
        "game_attacklw3" => {
            // pistolet plasma : lueur verte au canon, tir vert devant
            for f in [2.0, 26.0] {
                out.push((f, Fx::Suit("sys_smash_flash_s", "haver", (0.0, 0.0, 3.0), (0.0, 0.0, 0.0), 0.6, VERT)));
            }
            // demandé : explosions au point d'impact (coups à (0, 2, 11,6)) à la place de la fumée verte
            for f in [4.0, 30.0] {
                out.push((f, Fx::Pose("sys_smash_flash", "top", (0.0, 3.0, 12.0), (0.0, 0.0, 0.0), 0.5, VERT)));
                out.push((f, Fx::Pose("sys_bomb_a", "top", (0.0, 3.0, 11.6), (0.0, 0.0, 0.0), 0.55, (1.0, 1.0, 1.0))));
            }
        }
        "game_attackhi4" => {
            // laser spartan : gros rayon rouge vertical
            out.push((30.0, Fx::Suit("sys_smash_flash", "top", (-2.0, 17.0, 0.0), (0.0, 0.0, 0.0), 1.0, ROUGE)));
            for k in 0..9 {
                let y = 20.0 + 13.0 * k as f32;
                out.push((32.0, Fx::Pose("sys_attack_line", "top", (-2.0, y, 0.0), (-90.0, 0.0, 0.0), 1.6, ROUGE)));
            }
            out.push((32.0, Fx::Quake));
        }
        "game_attacklw4" => {
            out.push((32.0, Fx::Quake));
        }
        "game_attackairn" => {
            trainee(out, &["footl"], 4.0, 8.0, 0.5, BLANC);
        }
        "game_attackairb" => {
            trainee(out, &["footr"], 15.0, 19.0, 0.55, BLANC);
        }
        "game_attackairf" => {
            // épée à énergie : grand arc cyan devant lui
            out.push((3.0, Fx::Suit("sys_attack_arc_b", "top", (0.0, 11.0, 6.0), (0.0, -30.0, 70.0), 1.2, CYAN)));
            trainee(out, &["haver"], 3.0, 10.0, 0.6, CYAN);
        }
        "game_attackairhi" => {
            for f in [19.0, 23.0] {
                out.push((f, Fx::Suit("sys_smash_flash", "top", (0.0, 26.0, 1.0), (0.0, 0.0, 0.0), 0.8, BLEU)));
            }
        }
        "game_attackairlw" => {
            // charge : jets bleus des propulseurs sous lui ; plongée : traînée
            let mut f = 12.0;
            while f < 67.0 {
                out.push((f, Fx::Pose("sys_smash_flash_s", "footl", (0.0, -1.5, 0.0), (0.0, 0.0, 0.0), 0.6, BLEU)));
                out.push((f, Fx::Pose("sys_smash_flash_s", "footr", (0.0, -1.5, 0.0), (0.0, 0.0, 0.0), 0.6, BLEU)));
                f += 3.0;
            }
            out.push((87.0, Fx::Suit("sys_attack_speedline", "top", (0.0, 10.0, 0.0), (-70.0, 0.0, 0.0), 1.2, BLANC)));
        }
        "game_landingairlw" => {
            // onde de choc au sol
            out.push((0.0, Fx::Sol("sys_crown", 1.0)));
            out.push((0.0, Fx::Sol("sys_landing_smoke", 1.3)));
            out.push((0.0, Fx::Quake));
            out.push((0.0, Fx::Son("se_common_bomb_m")));
        }
        _ => {}
    }
}

unsafe fn jouer(agent: &mut L2CAgentBase, script: &str) {
    let mv = match find(script) { Some(m) => m, None => return };
    let evs = coups(crate::alias(script), mv);
    let s = crate::alias(script);
    let mut l: Vec<(f32, Fx)> = Vec::new();
    effets_coups(s, &evs, &mut l);
    extras(s, &mut l);
    sons(s, &mut l);
    l.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal));
    let boma = agent.module_accessor;
    let end = MotionModule::end_frame(boma);
    let r = if end > 1.0 && mv.total > 1.0 { (mv.total / end).clamp(0.25, 4.0) } else { 1.0 };
    let lua = agent.lua_state_agent;
    let mut cur = 0.0f32;
    for (f, fx) in l {
        // une image avant celle du coup (comme les scripts du jeu) : à l'image du coup, l'arrêt sur l'impact
        // (hitlag) fige les scripts d'effets / de sons, le son du tir partait après le coup (vidéo down smash)
        let mut af = ((f - 1.0) / r).max(1.0);
        // smashs : ce qui vient après la charge ne doit pas tomber sur l'image de la charge (sinon il part dès le
        // début de la charge : tir du smash avant à l'image 29 de REX -> 28 = image de la charge, vidéo) : au plus
        // tôt l'image juste après, quand le smash est relâché
        // (le drapeau de charge est posé 2 images avant celle de REX, voir run() : au plus tôt l'image de charge de
        // REX, soit 2 images après le drapeau)
        if mv.kind == 1 && mv.hold > 0.0 && f > mv.hold {
            af = af.max(mv.hold / r);
        }
        if af > cur {
            frame(lua, af);
            cur = af;
        }
        if !is_excute(agent) {
            continue;
        }
        match fx {
            Fx::Pose(e, b, p, rt, sc, c) => {
                EFFECT(agent, Hash40::new(e), Hash40::new(b), p.0, p.1, p.2, rt.0, rt.1, rt.2, sc, 0, 0, 0, 0, 0, 0, true);
                LAST_EFFECT_SET_COLOR(agent, c.0, c.1, c.2);
            }
            Fx::Suit(e, b, p, rt, sc, c) => {
                EFFECT_FOLLOW(agent, Hash40::new(e), Hash40::new(b), p.0, p.1, p.2, rt.0, rt.1, rt.2, sc, true);
                LAST_EFFECT_SET_COLOR(agent, c.0, c.1, c.2);
            }
            Fx::Sol(e, sc) => {
                LANDING_EFFECT(agent, Hash40::new(e), Hash40::new("top"), 0, 0, 0, 0, 0, 0, sc, 0, 0, 0, 0, 0, 0, false);
            }
            Fx::Quake => {
                QUAKE(agent, *CAMERA_QUAKE_KIND_M);
            }
            Fx::Son(se) => {
                PLAY_SE(agent, Hash40::new(se));
            }
        }
    }
}

unsafe extern "C" fn vide(_agent: &mut L2CAgentBase) {}

unsafe extern "C" fn son_charge_laser(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_snake_smash_h01"));
    }
}

macro_rules! effets {
    ($($f:ident => $g:literal : $e:literal, $s:literal),* $(,)?) => {
        $( unsafe extern "C" fn $f(agent: &mut L2CAgentBase) { jouer(agent, $g); } )*
        pub fn install(a: &mut Agent) {
            $(
                a.acmd($e, $f, Priority::Low);
                // sons de Snake (voix, RPG, mortier…) retirés ; les sons de Chief passent par l'effet (Fx::Son)
                a.acmd($s, vide, Priority::Low);
            )*
            // charge des smashs : les scripts de son de Snake jouent ses sons de smash, qui sont maintenant ceux de
            // Chief (tir du fusil à pompe pendant la charge : vidéo) ; REX : seul le laser a un son de charge (n° 5)
            a.acmd("sound_attacks4charge", vide, Priority::Low);
            a.acmd("sound_attacklw4charge", vide, Priority::Low);
            a.acmd("sound_attackhi4charge", son_charge_laser, Priority::Low);
        }
    };
}

effets! {
    e_attack11 => "game_attack11" : "effect_attack11", "sound_attack11",
    e_attack12 => "game_attack12" : "effect_attack12", "sound_attack12",
    e_attack13 => "game_attack13" : "effect_attack13", "sound_attack13",
    e_attackdash => "game_attackdash" : "effect_attackdash", "sound_attackdash",
    e_swingdash => "game_swingdash" : "effect_swingdash", "sound_swingdash",
    e_attacks3 => "game_attacks3" : "effect_attacks3", "sound_attacks3",
    e_attacks3s2 => "game_attacks3s2" : "effect_attacks3s2", "sound_attacks3s2",
    e_attacks3s => "game_attacks3s" : "effect_attacks3s", "sound_attacks3s",
    e_attackhi3 => "game_attackhi3" : "effect_attackhi3", "sound_attackhi3",
    e_attacklw3 => "game_attacklw3" : "effect_attacklw3", "sound_attacklw3",
    e_attacks4 => "game_attacks4" : "effect_attacks4", "sound_attacks4",
    e_attackhi4 => "game_attackhi4" : "effect_attackhi4", "sound_attackhi4",
    e_attacklw4 => "game_attacklw4" : "effect_attacklw4", "sound_attacklw4",
    e_attackairn => "game_attackairn" : "effect_attackairn", "sound_attackairn",
    e_attackairf => "game_attackairf" : "effect_attackairf", "sound_attackairf",
    e_attackairb => "game_attackairb" : "effect_attackairb", "sound_attackairb",
    e_attackairhi => "game_attackairhi" : "effect_attackairhi", "sound_attackairhi",
    e_attackairlw => "game_attackairlw" : "effect_attackairlw", "sound_attackairlw",
    e_landingairlw => "game_landingairlw" : "effect_landingairlw", "sound_landingairlw",
}
