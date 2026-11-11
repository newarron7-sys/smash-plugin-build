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

/// effets d'une image de coups (une seule fois par image)
fn effets_coups(script: &str, evs: &[Ev], out: &mut Vec<(f32, Fx)>) {
    let mut vu: Vec<f32> = Vec::new();
    for e in evs {
        if e.clear || e.rate > 0.0 || e.body >= 0 {
            continue;
        }
        let c = couleur(script, e);
        if e.cap {
            if vu.contains(&e.f) {
                continue;
            }
            vu.push(e.f);
            // éclair au canon
            out.push((e.f, Fx::Pose("sys_smash_flash_s", "top", (e.x, e.y, e.z), (0.0, 0.0, 0.0), 0.8, c)));
            // traçantes le long du tir (sys_attack_line : trait le long de Z ; incliné de -pente autour de X)
            let (dy, dz) = (e.y2 - e.y, e.z2 - e.z);
            let l = (dy * dy + dz * dz).sqrt().max(0.001);
            let pente = dy.atan2(dz).to_degrees();
            let pas = 15.0;
            let n = ((l / pas) as i32).clamp(1, 14);
            let angles: &[f32] = if script == "game_attacks4" { &[-9.0, 0.0, 9.0] } else { &[0.0] };
            for a in angles {
                for k in 0..n {
                    let t = (k as f32 + 0.5) * pas / l;
                    let p = (e.x, e.y + dy * t, e.z + dz * t);
                    out.push((e.f, Fx::Pose("sys_attack_line", "top", p, (-(pente + a), 0.0, 0.0),
                                            if script == "game_attacks4" { 0.9 } else { 0.55 }, c)));
                }
            }
            if script == "game_attacks4" {
                out.push((e.f, Fx::Pose("sys_bomb_a", "top", (e.x, e.y, e.z + 6.0), (0.0, 0.0, 0.0), 0.45, ORANGE)));
            }
        } else {
            if vu.contains(&e.f) {
                continue;
            }
            vu.push(e.f);
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

/// effets propres à chaque coup (REX : arcs n° 0xC, impacts n° 0xD, laser n° 3, jets du Ground Pound…)
fn extras(script: &str, out: &mut Vec<(f32, Fx)>) {
    match script {
        "game_attack11" => {
            out.push((2.0, Fx::Son("se_common_punch_kick_swing_m")));
            trainee(out, &["handr"], 2.0, 4.0, 0.45, BLANC);
        }
        "game_attack12" => {
            out.push((2.0, Fx::Son("se_common_punch_kick_swing_m")));
            trainee(out, &["handl"], 2.0, 4.0, 0.45, BLANC);
        }
        "game_attack13" => {
            out.push((3.0, Fx::Son("se_common_punch_kick_swing_l")));
            out.push((3.0, Fx::Suit("sys_attack_arc_b", "top", (0.0, 12.0, 5.0), (-80.0, 0.0, 90.0), 0.8, BLANC)));
            trainee(out, &["handr"], 3.0, 6.0, 0.5, BLANC);
        }
        "game_attackdash" => {
            // « Spartan Charge » : traînée de vitesse blanche
            out.push((7.0, Fx::Suit("sys_attack_speedline", "top", (0.0, 9.0, -3.0), (0.0, 0.0, 0.0), 1.2, BLANC)));
            out.push((7.0, Fx::Sol("sys_dash_smoke", 0.9)));
            out.push((7.0, Fx::Son("se_common_punch_kick_swing_l")));
        }
        "game_swingdash" => {
            // « Slide Kick » : glissade au sol
            for f in [9.0, 13.0, 17.0, 21.0] {
                out.push((f, Fx::Sol("sys_sliding_smoke", 0.9)));
            }
            out.push((9.0, Fx::Son("se_common_slip_01")));
        }
        "game_attackhi3" => {
            // mitraillettes : éclairs aux deux mains à chaque rafale
            for f in [25.0, 28.0, 30.0, 32.0, 36.0] {
                out.push((f, Fx::Suit("sys_smash_flash_s", "haver", (0.0, 0.0, 4.0), (0.0, 0.0, 0.0), 0.5, JAUNE)));
                out.push((f, Fx::Suit("sys_smash_flash_s", "havel", (0.0, 0.0, 4.0), (0.0, 0.0, 0.0), 0.5, JAUNE)));
            }
        }
        "game_attacklw3" => {
            // pistolet plasma : lueur verte au canon
            for f in [2.0, 26.0] {
                out.push((f, Fx::Suit("sys_smash_flash_s", "haver", (0.0, 0.0, 3.0), (0.0, 0.0, 0.0), 0.6, VERT)));
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
            out.push((4.0, Fx::Son("se_common_punch_kick_swing_m")));
            trainee(out, &["footl"], 4.0, 8.0, 0.5, BLANC);
        }
        "game_attackairb" => {
            out.push((16.0, Fx::Son("se_common_punch_kick_swing_l")));
            trainee(out, &["footr"], 15.0, 19.0, 0.55, BLANC);
        }
        "game_attackairf" => {
            // épée à énergie : grand arc cyan devant lui
            out.push((3.0, Fx::Son("se_common_sword_swing_l")));
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
            out.push((0.0, Fx::Son("se_common_down_l_01")));
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
    l.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal));
    let boma = agent.module_accessor;
    let end = MotionModule::end_frame(boma);
    let r = if end > 1.0 && mv.total > 1.0 { (mv.total / end).clamp(0.25, 4.0) } else { 1.0 };
    let lua = agent.lua_state_agent;
    let mut cur = 0.0f32;
    for (f, fx) in l {
        let af = (f / r).max(1.0);
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

macro_rules! effets {
    ($($f:ident => $g:literal : $e:literal, $s:literal),* $(,)?) => {
        $( unsafe extern "C" fn $f(agent: &mut L2CAgentBase) { jouer(agent, $g); } )*
        pub fn install(a: &mut Agent) {
            $(
                a.acmd($e, $f, Priority::Low);
                // sons de Snake (voix, RPG, mortier…) retirés ; les sons de Chief passent par l'effet (Fx::Son)
                a.acmd($s, vide, Priority::Low);
            )*
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
