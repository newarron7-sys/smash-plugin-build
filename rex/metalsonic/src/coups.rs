//! Metal Sonic : effets de ses coups normaux (demandé le 15/11, vidéos du showcase : « il faut que tu rajoutes les
//! effets de ses attaques »). Dans REX ses griffes laissent des traits blancs-bleus (effets n° 07, 13, 18… de son
//! propre fichier d'effets, introuvables dans Ultimate) : refaits comme pour Super Sonic avec les effets du jeu
//! (arcs sys_attack_arc*, traits, éclats) teintés, plus une traînée posée image par image sur la griffe / le pied qui
//! frappe. Images = images de ses coups de REX (generated.rs), positions des effets de REX quand elles existent.

use smash::app::sv_animcmd::frame;
use smash::lua2cpp::*;
use smash::phx::Hash40;
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

type C = (f32, f32, f32);
/// trait de griffe de REX : blanc légèrement bleuté
const GRIFFE: C = (0.75, 0.9, 1.0);
/// tuyère / pied : bleu acier
const ACIER: C = (0.45, 0.65, 1.0);

unsafe fn fx(agent: &mut L2CAgentBase, eff: &str, bone: &str, p: (f32, f32, f32), r: (f32, f32, f32), sc: f32, c: C) {
    if is_excute(agent) {
        EFFECT_FOLLOW(agent, Hash40::new(eff), Hash40::new(bone), p.0, p.1, p.2, r.0, r.1, r.2, sc, true);
        LAST_EFFECT_SET_COLOR(agent, c.0, c.1, c.2);
    }
}
unsafe fn sol(agent: &mut L2CAgentBase, eff: &str, z: f32, sc: f32) {
    if is_excute(agent) {
        LANDING_EFFECT(agent, Hash40::new(eff), Hash40::new("top"), 0, 0, z, 0, 0, 0, sc, 0, 0, 0, 0, 0, 0, false);
    }
}
/// traînée : un éclat par image sur chaque os, de f0 à f1 (pas accroché : il dessine le passage de la griffe)
unsafe fn trainee(agent: &mut L2CAgentBase, os: &[&str], f0: f32, f1: f32, sc: f32, c: C) {
    let lua = agent.lua_state_agent;
    let mut f = f0;
    while f <= f1 {
        frame(lua, f);
        if is_excute(agent) {
            for b in os {
                EFFECT(agent, Hash40::new("sys_smash_flash_s"), Hash40::new(b), 0, 0, 0, 0, 0, 0, sc, 0, 0, 0, 0, 0, 0, true);
                LAST_EFFECT_SET_COLOR(agent, c.0, c.1, c.2);
            }
        }
        f += 1.0;
    }
}

// ------------------------------------------------------------------ jabs « Claw Scratch » (griffe droite puis gauche)
unsafe extern "C" fn e_attack11(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 4.0);
    fx(agent, "sys_attack_arc_b", "top", (0.0, 8.0, 6.0), (0.0, 40.0, 150.0), 0.75, GRIFFE);
    trainee(agent, &["handr"], 4.0, 6.0, 0.55, GRIFFE);
}
unsafe extern "C" fn e_attack12(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 4.0);
    fx(agent, "sys_attack_arc_b", "top", (0.0, 8.0, 6.0), (0.0, 0.0, -20.0), 0.75, GRIFFE);
    fx(agent, "sys_attack_line", "top", (0.0, 8.0, 4.0), (0.0, 0.0, 0.0), 0.8, GRIFFE);
    trainee(agent, &["handl"], 4.0, 6.0, 0.55, GRIFFE);
}

// ------------------------------------------------------------------ attaque en courant « Dual Claw Strike »
// REX (image 5) : deux traits de griffe croisés devant lui (n° 13, ±4,5 de côté, 8,5 de haut, 9 devant, ±20°),
// showcase (vidéo du 15/11) : traînée blanche derrière lui et poussière au sol pendant qu'il glisse
unsafe extern "C" fn e_attackdash(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 1.0);
    sol(agent, "sys_dash_smoke", -4.0, 1.0);
    fx(agent, "sys_attack_speedline", "top", (0.0, 7.0, -3.0), (0.0, 0.0, 0.0), 1.1, GRIFFE);
    frame(lua, 5.0);
    fx(agent, "sys_attack_arc_b", "top", (0.0, 8.5, 8.0), (0.0, -20.0, -35.0), 1.0, GRIFFE);
    fx(agent, "sys_attack_arc_b", "top", (0.0, 8.5, 8.0), (0.0, 20.0, 35.0), 1.0, GRIFFE);
    sol(agent, "sys_atk_smoke", 2.0, 0.8);
    trainee(agent, &["handr", "handl"], 5.0, 9.0, 0.5, GRIFFE);
}

// ------------------------------------------------------------------ tilts
/// avant « Claw Swipe » (REX : coup de pied droit, trait n° 13 à 8 de haut, 2,5 devant)
unsafe extern "C" fn e_attacks3(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 3.0);
    fx(agent, "sys_attack_arc_b", "top", (0.0, 7.0, 4.0), (-10.0, 0.0, 15.0), 0.85, GRIFFE);
    trainee(agent, &["footr"], 3.0, 7.0, 0.55, ACIER);
}
/// haut « Upper Claw » (REX : arc n° 07 à 18 de haut, rotation 80)
unsafe extern "C" fn e_attackhi3(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 3.0);
    fx(agent, "sys_attack_arc_d", "top", (-1.0, 13.0, 4.0), (0.0, 6.0, 80.0), 0.9, GRIFFE);
    trainee(agent, &["handr"], 3.0, 8.0, 0.55, GRIFFE);
}
/// bas « Sweep Claw » (REX : arc n° 07 au ras du sol, 13,5 devant)
unsafe extern "C" fn e_attacklw3(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 4.0);
    fx(agent, "sys_attack_arc_b", "top", (0.0, 3.0, 9.0), (0.0, 0.0, -10.0), 0.85, GRIFFE);
    sol(agent, "sys_atk_smoke", 6.0, 0.6);
    trainee(agent, &["handr"], 4.0, 6.0, 0.55, GRIFFE);
}

// ------------------------------------------------------------------ smashs
// showcase : au début de la charge, une bulle brillante l'entoure un instant (éclat de charge de Brawl)
unsafe fn bulle_charge(agent: &mut L2CAgentBase) {
    fx(agent, "sys_shield", "top", (0.0, 7.0, 0.0), (0.0, 0.0, 0.0), 1.0, (1.0, 1.0, 1.0));
    fx(agent, "sys_smash_flash", "top", (0.0, 8.0, 0.0), (0.0, 0.0, 0.0), 1.4, (1.0, 1.0, 1.0));
}
unsafe fn sans_bulle(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        EFFECT_OFF_KIND(agent, Hash40::new("sys_shield"), false, false);
    }
}
/// bulle bleue transparente de REX (effet n° 04 de son fichier, 2,2) autour de lui pendant qu'il tourne en boule
unsafe fn boule(agent: &mut L2CAgentBase, y: f32, sc: f32) {
    fx(agent, "sys_shield", "top", (0.0, y, 0.0), (0.0, 0.0, 0.0), sc, (0.35, 0.55, 1.0));
}
unsafe extern "C" fn e_charge(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 1.0);
    bulle_charge(agent);
    frame(lua, 6.0);
    sans_bulle(agent);
    let mut f = 6.0;
    while f < 60.0 {
        frame(lua, f);
        if is_excute(agent) {
            EFFECT(agent, Hash40::new("sys_smash_flash_s"), Hash40::new("handr"), 0, 0, 0, 0, 0, 0, 0.6, 0, 0, 0, 0, 0, 0, true);
            LAST_EFFECT_SET_COLOR(agent, GRIFFE.0, GRIFFE.1, GRIFFE.2);
        }
        f += 5.0;
    }
}
/// avant « Heavy Claw » (coup à 16 : 8 du début + REX AttackS4S 8 ; REX : poussière + éclat sur la main à 8, arc n° 07
/// à 11 devant à 11 ; showcase : bulle de charge, puis grand trait blanc bas devant lui)
unsafe extern "C" fn e_attacks4(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 4.0);
    bulle_charge(agent);
    frame(lua, 9.0);
    sans_bulle(agent);
    frame(lua, 12.0);
    fx(agent, "sys_smash_flash", "handr", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 0.8, GRIFFE);
    frame(lua, 15.0);
    sol(agent, "sys_dash_smoke", -4.0, 0.9);
    fx(agent, "sys_attack_arc", "top", (0.0, 7.0, 9.0), (-10.0, 0.0, -30.0), 1.2, GRIFFE);
    fx(agent, "sys_attack_line", "top", (0.0, 7.0, 3.0), (0.0, 0.0, 0.0), 1.1, GRIFFE);
    trainee(agent, &["handr"], 15.0, 18.0, 0.7, GRIFFE);
}
/// haut « Short Spin Jump » (REX AttackHi4, + 14 images de début : coup à 18 quand il bondit, bulle n° 04 à 21 et 26,
/// atterrissage à 58 ; showcase : bulle de charge, gros nuage de poussière blanche au bond, il tourne dans une bulle
/// bleue transparente en l'air, poussière à l'atterrissage)
unsafe extern "C" fn e_attackhi4(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 4.0);
    bulle_charge(agent);
    frame(lua, 9.0);
    sans_bulle(agent);
    frame(lua, 17.0);
    if is_excute(agent) {
        LANDING_EFFECT(agent, Hash40::new("sys_down_smoke"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 1.3, 0, 0, 0, 0, 0, 0, false);
        LANDING_EFFECT(agent, Hash40::new("sys_crown"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.8, 0, 0, 0, 0, 0, 0, false);
    }
    fx(agent, "sys_smash_flash", "top", (0.0, 18.0, 0.0), (0.0, 0.0, 0.0), 1.0, GRIFFE);
    frame(lua, 19.0);
    boule(agent, 13.0, 1.4);
    fx(agent, "sys_spin_wind", "top", (0.0, 13.0, 0.0), (0.0, 0.0, 90.0), 1.1, GRIFFE);
    frame(lua, 26.0);
    sans_bulle(agent);
    boule(agent, 13.0, 1.4);
    frame(lua, 34.0);
    sans_bulle(agent);
    if is_excute(agent) {
        EFFECT_OFF_KIND(agent, Hash40::new("sys_spin_wind"), false, false);
    }
    frame(lua, 56.0);
    if is_excute(agent) {
        LANDING_EFFECT(agent, Hash40::new("sys_landing_smoke"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.9, 0, 0, 0, 0, 0, 0, false);
    }
}
/// bas « Spin Around » (REX AttackLw4, + 10 images de début : bulle n° 04 au ras du sol de 10 à 38, poussière à gauche
/// puis à droite à 15, 22, 28, 34 ; showcase : il tourne en boule dans une bulle bleue, traits de vent et poussière)
unsafe extern "C" fn e_attacklw4(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 3.0);
    bulle_charge(agent);
    frame(lua, 8.0);
    sans_bulle(agent);
    frame(lua, 10.0);
    boule(agent, 5.0, 1.2);
    fx(agent, "sys_spin_wind", "top", (0.0, 4.0, 0.0), (0.0, 0.0, 0.0), 1.2, GRIFFE);
    for (f, cote) in [(15.0, 1.0f32), (22.0, -1.0), (28.0, 1.0), (34.0, -1.0)] {
        frame(lua, f);
        if is_excute(agent) {
            FOOT_EFFECT(agent, Hash40::new("sys_dash_smoke"), Hash40::new("top"), 0, 0, 8.0 * cote, 0, if cote > 0.0 { 180 } else { 0 }, 0, 0.8, 0, 0, 0, 0, 0, 0, false);
        }
        fx(agent, "sys_attack_speedline", "top", (0.0, 4.0, 6.0 * cote), (0.0, if cote > 0.0 { 180.0 } else { 0.0 }, 0.0), 0.8, GRIFFE);
        if f == 22.0 {
            sans_bulle(agent);
            boule(agent, 5.0, 1.2);
        }
    }
    frame(lua, 38.0);
    sans_bulle(agent);
    if is_excute(agent) {
        EFFECT_OFF_KIND(agent, Hash40::new("sys_spin_wind"), false, false);
        LANDING_EFFECT(agent, Hash40::new("sys_landing_smoke_s"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.8, 0, 0, 0, 0, 0, 0, false);
    }
}

// ------------------------------------------------------------------ aériens
/// avant (REX : arc n° 07 à 7,3 de haut, 11 devant, rotation 40 / 20)
unsafe extern "C" fn e_attackairf(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 9.0);
    fx(agent, "sys_attack_arc", "top", (0.7, 7.3, 9.0), (-10.0, 40.0, 20.0), 0.9, GRIFFE);
    trainee(agent, &["handr"], 9.0, 12.0, 0.6, GRIFFE);
}
/// haut (REX : coup de pied vers le haut à 21, trait n° 10 à 6 de haut)
unsafe extern "C" fn e_attackairhi(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 20.0);
    fx(agent, "sys_attack_arc_d", "top", (0.0, 12.0, 1.0), (0.0, 0.0, 90.0), 1.0, ACIER);
    trainee(agent, &["footr"], 20.0, 27.0, 0.55, ACIER);
}
/// bas (REX : à 10, éclats sous lui et devant)
unsafe extern "C" fn e_attackairlw(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 10.0);
    fx(agent, "sys_attack_speedline", "top", (0.0, 9.0, 0.5), (90.0, 0.0, 0.0), 0.8, GRIFFE);
    fx(agent, "sys_smash_flash", "top", (0.0, -1.0, 2.0), (0.0, 0.0, 0.0), 0.9, GRIFFE);
    fx(agent, "sys_attack_arc_d", "top", (0.0, 2.0, 2.0), (0.0, 0.0, -90.0), 0.9, GRIFFE);
    trainee(agent, &["footr", "footl"], 10.0, 13.0, 0.55, ACIER);
}

pub fn install(a: &mut Agent) {
    a.acmd("effect_attack11", e_attack11, Priority::Low);
    a.acmd("effect_attack12", e_attack12, Priority::Low);
    a.acmd("effect_attackdash", e_attackdash, Priority::Low);
    a.acmd("effect_attacks3", e_attacks3, Priority::Low);
    a.acmd("effect_attacks3hi", e_attacks3, Priority::Low);
    a.acmd("effect_attacks3lw", e_attacks3, Priority::Low);
    a.acmd("effect_attackhi3", e_attackhi3, Priority::Low);
    a.acmd("effect_attacklw3", e_attacklw3, Priority::Low);
    for n in ["effect_attacks4", "effect_attacks4hi", "effect_attacks4lw"] {
        a.acmd(n, e_attacks4, Priority::Low);
    }
    for n in ["effect_attacks4charge", "effect_attackhi4charge", "effect_attacklw4charge"] {
        a.acmd(n, e_charge, Priority::Low);
    }
    a.acmd("effect_attackhi4", e_attackhi4, Priority::Low);
    a.acmd("effect_attacklw4", e_attacklw4, Priority::Low);
    a.acmd("effect_attackairf", e_attackairf, Priority::Low);
    a.acmd("effect_attackairhi", e_attackairhi, Priority::Low);
    a.acmd("effect_attackairlw", e_attackairlw, Priority::Low);
}
